use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use prettytable::{Cell, Row, Table};
use reqwest::header::{CONTENT_RANGE, CONTENT_TYPE, RANGE, USER_AGENT};
use reqwest::{StatusCode, Url};
use serde::Serialize;

use crate::api::client::PodcastIndexClient;
use crate::api::endpoints::episodes::get_episode_by_id;
use crate::api::types::Episode;
use crate::cli::DownloadArgs;
use crate::config::ConfigManager;
use crate::error::{PodcastCliError, Result};
use crate::output::json::to_pretty_json;
use crate::output::OutputFormat;

const DOWNLOAD_USER_AGENT: &str = "podcast-cli/0.1 download";
const PROGRESS_UPDATE_INTERVAL_MS: u64 = 250;
const MAX_FILENAME_LEN: usize = 200;
const MAX_DOWNLOAD_SIZE_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const ALLOWED_CONTENT_TYPES: &[&str] = &[
    "audio/mpeg",
    "audio/mp4",
    "audio/x-m4a",
    "audio/aac",
    "audio/ogg",
    "audio/opus",
    "audio/wav",
    "audio/x-wav",
    "audio/flac",
    "video/mp4",
    "application/octet-stream",
];

#[derive(Debug, Serialize)]
pub struct DownloadResult {
    pub episode_id: u64,
    pub path: String,
    pub filename: String,
    pub enclosure_url: String,
    pub content_type: Option<String>,
    pub size: u64,
    pub resumed: bool,
}

#[derive(Debug, Serialize)]
pub struct DryRunResult {
    pub episode_id: u64,
    pub path: String,
    pub filename: String,
    pub enclosure_url: String,
    pub content_type: Option<String>,
    pub dry_run: bool,
}

#[derive(Debug, Serialize)]
struct MinimalDownloadResult {
    path: String,
    size: u64,
}

#[derive(Debug, Serialize)]
struct MinimalDryRunResult {
    path: String,
    filename: String,
    enclosure_url: String,
    content_type: Option<String>,
    dry_run: bool,
}

#[derive(Debug, Clone, Default)]
struct RemoteMetadata {
    content_type: Option<String>,
}

#[derive(Debug)]
struct DownloadStats {
    bytes_written: u64,
    resumed: bool,
}

#[derive(Debug, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
enum ProgressEvent {
    Start {
        episode_id: u64,
        url: String,
        path: String,
        resumed: bool,
    },
    Progress {
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
        percent: Option<f64>,
        bytes_per_sec: u64,
    },
    Finish {
        path: String,
        size: u64,
        elapsed_ms: u64,
    },
    Error {
        code: String,
        message: String,
    },
}

#[derive(Debug, Clone, Copy)]
enum ProgressMode {
    Off,
    Text,
    Json,
}

#[derive(Debug)]
struct ProgressEmitter {
    mode: ProgressMode,
    start: Option<Instant>,
    last_progress: Option<Instant>,
}

impl ProgressEmitter {
    fn new(no_progress: bool, progress_json: bool) -> Self {
        let mode = if no_progress {
            ProgressMode::Off
        } else if progress_json {
            ProgressMode::Json
        } else {
            ProgressMode::Text
        };

        Self {
            mode,
            start: None,
            last_progress: None,
        }
    }

    fn emit_start(&mut self, episode_id: u64, url: &Url, path: &Path, resumed: bool) {
        self.start = Some(Instant::now());

        match self.mode {
            ProgressMode::Off => {}
            ProgressMode::Text => {
                eprintln!(
                    "Downloading episode {episode_id} -> {}{}",
                    path.display(),
                    if resumed { " (resuming)" } else { "" }
                );
            }
            ProgressMode::Json => self.emit_json(ProgressEvent::Start {
                episode_id,
                url: url.to_string(),
                path: path_to_string(path),
                resumed,
            }),
        }
    }

    fn emit_progress(&mut self, downloaded_bytes: u64, total_bytes: Option<u64>) {
        match self.mode {
            ProgressMode::Off => {}
            ProgressMode::Text => {
                let now = Instant::now();
                if let Some(last) = self.last_progress {
                    if now.duration_since(last) < Duration::from_millis(PROGRESS_UPDATE_INTERVAL_MS)
                    {
                        return;
                    }
                }
                self.last_progress = Some(now);

                if let Some(total) = total_bytes {
                    let percent = if total > 0 {
                        (downloaded_bytes as f64 / total as f64) * 100.0
                    } else {
                        0.0
                    };
                    eprintln!("Progress: {downloaded_bytes}/{total} bytes ({percent:.1}%)");
                } else {
                    eprintln!("Progress: {downloaded_bytes} bytes");
                }
            }
            ProgressMode::Json => {
                let elapsed = self
                    .start
                    .map(|started| started.elapsed().as_secs_f64())
                    .unwrap_or(0.0)
                    .max(0.001);
                let bytes_per_sec = (downloaded_bytes as f64 / elapsed).round() as u64;
                let percent = total_bytes.and_then(|total| {
                    if total == 0 {
                        None
                    } else {
                        Some((downloaded_bytes as f64 / total as f64) * 100.0)
                    }
                });

                self.emit_json(ProgressEvent::Progress {
                    downloaded_bytes,
                    total_bytes,
                    percent,
                    bytes_per_sec,
                });
            }
        }
    }

    fn emit_finish(&mut self, path: &Path, size: u64) {
        let elapsed_ms = self
            .start
            .map(|started| started.elapsed().as_millis() as u64)
            .unwrap_or(0);

        match self.mode {
            ProgressMode::Off => {}
            ProgressMode::Text => {
                eprintln!(
                    "Download finished: {} ({} bytes, {} ms)",
                    path.display(),
                    size,
                    elapsed_ms
                );
            }
            ProgressMode::Json => self.emit_json(ProgressEvent::Finish {
                path: path_to_string(path),
                size,
                elapsed_ms,
            }),
        }
    }

    fn emit_error(&mut self, error: &PodcastCliError) {
        match self.mode {
            ProgressMode::Off => {}
            ProgressMode::Text => eprintln!("Download failed: {error}"),
            ProgressMode::Json => self.emit_json(ProgressEvent::Error {
                code: error.progress_code().to_string(),
                message: error.to_string(),
            }),
        }
    }

    fn emit_json(&self, event: ProgressEvent) {
        if let Ok(line) = serde_json::to_string(&event) {
            eprintln!("{line}");
        }
    }
}

pub async fn run(args: DownloadArgs, manager: &ConfigManager) -> Result<()> {
    let cfg = manager.load()?;
    let (api_key, api_secret) = cfg.require_credentials()?;
    let output = args.output.map(Into::into).unwrap_or(cfg.default_output);

    let api_client = PodcastIndexClient::new(api_key, api_secret);
    let episode_response = get_episode_by_id(&api_client, args.episode_id).await?;
    let episode = episode_response
        .first_episode()
        .ok_or_else(|| PodcastCliError::Metadata("episode not found".to_string()))?;

    let enclosure_url = parse_enclosure_url(episode)?;
    let download_client = build_download_client(args.timeout)?;
    let remote_metadata = probe_remote_metadata(&download_client, &enclosure_url).await?;
    validate_content_type(remote_metadata.content_type.as_deref())?;

    let ext = infer_extension(
        remote_metadata.content_type.as_deref(),
        episode.enclosure_type.as_deref(),
        &enclosure_url,
    );
    let suggested_filename = derive_filename(episode, args.episode_id, &enclosure_url, &ext);
    let target_path = resolve_target_path(
        args.dest.as_deref(),
        args.filename.as_deref(),
        &suggested_filename,
    )?;
    let output_path = normalize_output_path(&target_path)?;
    let filename = output_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("download.bin")
        .to_string();

    if args.dry_run {
        let result = DryRunResult {
            episode_id: args.episode_id,
            path: path_to_string(&output_path),
            filename,
            enclosure_url: enclosure_url.to_string(),
            content_type: remote_metadata.content_type,
            dry_run: true,
        };

        return emit_dry_run_output(&result, args.path_only, args.minimal, output);
    }

    let mut progress = ProgressEmitter::new(args.no_progress, args.progress_json);
    let download_result = execute_download(
        &download_client,
        &enclosure_url,
        &output_path,
        args.overwrite,
        args.resume,
        args.episode_id,
        &mut progress,
    )
    .await;

    let stats = match download_result {
        Ok(stats) => stats,
        Err(err) => {
            progress.emit_error(&err);
            return Err(err);
        }
    };

    let result = DownloadResult {
        episode_id: args.episode_id,
        path: path_to_string(&output_path),
        filename,
        enclosure_url: enclosure_url.to_string(),
        content_type: remote_metadata.content_type,
        size: stats.bytes_written,
        resumed: stats.resumed,
    };

    emit_download_output(&result, args.path_only, args.minimal, output)
}

fn build_download_client(timeout_secs: u64) -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent(DOWNLOAD_USER_AGENT)
        .build()
        .map_err(PodcastCliError::Http)
}

fn parse_enclosure_url(episode: &Episode) -> Result<Url> {
    let raw = episode
        .enclosure_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| PodcastCliError::Metadata("episode has no enclosure URL".to_string()))?;

    let url = Url::parse(raw).map_err(|err| {
        PodcastCliError::Validation(format!("invalid enclosure URL `{raw}`: {err}"))
    })?;

    match url.scheme() {
        "http" | "https" => Ok(url),
        other => Err(PodcastCliError::Validation(format!(
            "unsupported enclosure URL scheme: {other}"
        ))),
    }
}

async fn probe_remote_metadata(client: &reqwest::Client, url: &Url) -> Result<RemoteMetadata> {
    if let Ok(response) = client.head(url.clone()).send().await {
        if response.status().is_success() {
            return Ok(RemoteMetadata {
                content_type: response_content_type(&response),
            });
        }
    }

    let response = client
        .get(url.clone())
        .header(RANGE, "bytes=0-0")
        .send()
        .await?;

    let status = response.status();
    if !(status.is_success() || status == StatusCode::PARTIAL_CONTENT) {
        return Err(PodcastCliError::Api(format!(
            "metadata probe failed with status {status}"
        )));
    }

    Ok(RemoteMetadata {
        content_type: response_content_type(&response),
    })
}

async fn execute_download(
    client: &reqwest::Client,
    url: &Url,
    target_path: &Path,
    overwrite: bool,
    resume: bool,
    episode_id: u64,
    progress: &mut ProgressEmitter,
) -> Result<DownloadStats> {
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let part_path = temporary_part_path(target_path);
    let mut resume_from = 0_u64;
    if resume && part_path.exists() {
        resume_from = fs::metadata(&part_path)?.len();
    }
    ensure_download_size_within_limit(None, resume_from)?;

    let mut request = client
        .get(url.clone())
        .header(USER_AGENT, DOWNLOAD_USER_AGENT);
    if resume_from > 0 {
        request = request.header(RANGE, format!("bytes={resume_from}-"));
    }

    let mut response = request.send().await?;
    let status = response.status();
    if !(status.is_success() || status == StatusCode::PARTIAL_CONTENT) {
        return Err(PodcastCliError::Api(format!(
            "download failed with status {status}"
        )));
    }

    let resumed = resume_from > 0 && status == StatusCode::PARTIAL_CONTENT;
    if !resumed {
        resume_from = 0;
    }

    let total_bytes = response
        .content_length()
        .map(|value| value + if resumed { resume_from } else { 0 })
        .or_else(|| {
            response_content_length(&response)
                .map(|value| value + if resumed { resume_from } else { 0 })
        });
    ensure_download_size_within_limit(total_bytes, resume_from)?;

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(resumed)
        .truncate(!resumed)
        .open(&part_path)?;

    progress.emit_start(episode_id, url, target_path, resumed);

    let mut bytes_written = resume_from;
    while let Some(chunk) = response.chunk().await? {
        let next_bytes_written = bytes_written
            .checked_add(chunk.len() as u64)
            .ok_or_else(|| PodcastCliError::Validation("download size overflow".to_string()))?;
        ensure_download_size_within_limit(Some(next_bytes_written), 0)?;

        file.write_all(&chunk)?;
        bytes_written = next_bytes_written;
        progress.emit_progress(bytes_written, total_bytes);
    }

    file.flush()?;
    file.sync_data()?;
    drop(file);

    // When resuming a download (resume_from > 0), we should allow overwriting the target
    // because we're completing a previously started download that has a .part file
    let allow_overwrite = overwrite || resume_from > 0;
    promote_part_file(&part_path, target_path, allow_overwrite)?;

    progress.emit_finish(target_path, bytes_written);

    Ok(DownloadStats {
        bytes_written,
        resumed,
    })
}

fn validate_content_type(content_type: Option<&str>) -> Result<()> {
    if let Some(value) = content_type {
        if !is_allowed_content_type(value) {
            return Err(PodcastCliError::Validation(format!(
                "unsupported enclosure content-type `{value}`"
            )));
        }
    }

    Ok(())
}

fn is_allowed_content_type(value: &str) -> bool {
    let normalized = value.trim();
    ALLOWED_CONTENT_TYPES
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(normalized))
}

fn ensure_download_size_within_limit(
    expected_total: Option<u64>,
    bytes_written: u64,
) -> Result<()> {
    if bytes_written > MAX_DOWNLOAD_SIZE_BYTES {
        return Err(PodcastCliError::Validation(format!(
            "download exceeds maximum allowed size ({} bytes)",
            MAX_DOWNLOAD_SIZE_BYTES
        )));
    }

    if let Some(total) = expected_total {
        if total > MAX_DOWNLOAD_SIZE_BYTES {
            return Err(PodcastCliError::Validation(format!(
                "download exceeds maximum allowed size ({} bytes)",
                MAX_DOWNLOAD_SIZE_BYTES
            )));
        }
    }

    Ok(())
}

fn promote_part_file(part_path: &Path, target_path: &Path, overwrite: bool) -> Result<()> {
    if overwrite {
        #[cfg(unix)]
        {
            fs::rename(part_path, target_path)?;
            return Ok(());
        }

        #[cfg(not(unix))]
        {
            if let Err(err) = fs::rename(part_path, target_path) {
                if err.kind() != ErrorKind::AlreadyExists {
                    return Err(PodcastCliError::Io(err));
                }

                let backup_path = temporary_backup_path(target_path);
                fs::rename(target_path, &backup_path)?;

                return match fs::rename(part_path, target_path) {
                    Ok(()) => {
                        let _ = fs::remove_file(&backup_path);
                        Ok(())
                    }
                    Err(rename_err) => {
                        let _ = fs::rename(&backup_path, target_path);
                        Err(PodcastCliError::Io(rename_err))
                    }
                };
            }

            return Ok(());
        }
    }

    match fs::hard_link(part_path, target_path) {
        Ok(()) => {
            fs::remove_file(part_path)?;
            Ok(())
        }
        Err(err) if err.kind() == ErrorKind::AlreadyExists => Err(target_exists_error(target_path)),
        Err(_) => copy_part_file_noclobber(part_path, target_path),
    }
}

fn copy_part_file_noclobber(part_path: &Path, target_path: &Path) -> Result<()> {
    let mut source = OpenOptions::new().read(true).open(part_path)?;
    let mut target = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(target_path)
        .map_err(|err| {
            if err.kind() == ErrorKind::AlreadyExists {
                target_exists_error(target_path)
            } else {
                PodcastCliError::Io(err)
            }
        })?;

    if let Err(err) = std::io::copy(&mut source, &mut target) {
        let _ = fs::remove_file(target_path);
        return Err(PodcastCliError::Io(err));
    }

    target.flush()?;
    target.sync_all()?;
    drop(target);
    drop(source);

    fs::remove_file(part_path)?;
    Ok(())
}

fn target_exists_error(target_path: &Path) -> PodcastCliError {
    PodcastCliError::Io(std::io::Error::new(
        ErrorKind::AlreadyExists,
        format!(
            "target file already exists (use --overwrite): {}",
            target_path.display()
        ),
    ))
}

#[cfg(not(unix))]
fn temporary_backup_path(target_path: &Path) -> PathBuf {
    let file_name = target_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("download");

    target_path.with_file_name(format!("{file_name}.bak"))
}

fn response_content_type(response: &reqwest::Response) -> Option<String> {
    response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            value
                .split(';')
                .next()
                .unwrap_or(value)
                .trim()
                .to_ascii_lowercase()
        })
        .filter(|value| !value.is_empty())
}

fn response_content_length(response: &reqwest::Response) -> Option<u64> {
    response.content_length().or_else(|| {
        response
            .headers()
            .get(CONTENT_RANGE)
            .and_then(|value| value.to_str().ok())
            .and_then(parse_content_range_total)
    })
}

fn parse_content_range_total(value: &str) -> Option<u64> {
    let (_, suffix) = value.split_once('/')?;
    if suffix == "*" {
        return None;
    }

    suffix.parse::<u64>().ok()
}

fn infer_extension(content_type: Option<&str>, enclosure_type: Option<&str>, url: &Url) -> String {
    content_type
        .and_then(content_type_to_extension)
        .or_else(|| enclosure_type.and_then(content_type_to_extension))
        .or_else(|| {
            Path::new(url.path())
                .extension()
                .and_then(|value| value.to_str())
                .filter(|value| !value.trim().is_empty())
                .map(|value| format!(".{}", value.trim().to_ascii_lowercase()))
        })
        .unwrap_or_else(|| ".bin".to_string())
}

fn content_type_to_extension(raw: &str) -> Option<String> {
    let normalized = raw.split(';').next()?.trim().to_ascii_lowercase();
    let extension = match normalized.as_str() {
        "audio/mpeg" => ".mp3",
        "audio/mp4" | "audio/x-m4a" => ".m4a",
        "audio/aac" => ".aac",
        "audio/ogg" => ".ogg",
        "audio/opus" => ".opus",
        "audio/wav" | "audio/x-wav" => ".wav",
        "audio/flac" => ".flac",
        "video/mp4" => ".mp4",
        _ => return None,
    };

    Some(extension.to_string())
}

fn derive_filename(episode: &Episode, episode_id: u64, url: &Url, extension: &str) -> String {
    let from_url = url
        .path_segments()
        .and_then(|segments| segments.filter(|value| !value.is_empty()).next_back())
        .map(sanitize_filename)
        .filter(|value| !value.is_empty());

    let base = from_url
        .or_else(|| {
            episode
                .title
                .as_deref()
                .map(sanitize_filename)
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| format!("episode-{episode_id}"));

    if Path::new(&base).extension().is_some() {
        base
    } else {
        format!("{base}{extension}")
    }
}

fn resolve_target_path(
    dest: Option<&Path>,
    filename: Option<&str>,
    fallback_filename: &str,
) -> Result<PathBuf> {
    if let Some(filename) = filename {
        let sanitized = sanitize_filename(filename);
        if sanitized.is_empty() {
            return Err(PodcastCliError::Validation(
                "filename cannot be empty".to_string(),
            ));
        }

        let base_dir = dest.unwrap_or_else(|| Path::new("."));
        if base_dir.exists() && base_dir.is_file() {
            return Err(PodcastCliError::Validation(
                "--filename requires --dest to be a directory path".to_string(),
            ));
        }

        return Ok(base_dir.join(sanitized));
    }

    if let Some(dest) = dest {
        if dest.exists() {
            if dest.is_dir() {
                return Ok(dest.join(fallback_filename));
            }
            return Ok(dest.to_path_buf());
        }

        if looks_like_file_path(dest) {
            return Ok(dest.to_path_buf());
        }

        return Ok(dest.join(fallback_filename));
    }

    Ok(PathBuf::from(fallback_filename))
}

fn looks_like_file_path(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

fn sanitize_filename(value: &str) -> String {
    let mut sanitized = String::with_capacity(value.len());

    for ch in value.chars() {
        let valid =
            !ch.is_control() && !matches!(ch, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|');
        sanitized.push(if valid { ch } else { '_' });
    }

    let trimmed = sanitized.trim().trim_matches('.');
    trimmed
        .chars()
        .take(MAX_FILENAME_LEN)
        .collect::<String>()
        .trim()
        .to_string()
}

fn temporary_part_path(target_path: &Path) -> PathBuf {
    let file_name = target_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("download");

    target_path.with_file_name(format!("{file_name}.part"))
}

fn normalize_output_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }

    Ok(std::env::current_dir()?.join(path))
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn emit_download_output(
    result: &DownloadResult,
    path_only: bool,
    minimal: bool,
    output: OutputFormat,
) -> Result<()> {
    if path_only {
        println!("{}", result.path);
        return Ok(());
    }

    if minimal {
        let payload = MinimalDownloadResult {
            path: result.path.clone(),
            size: result.size,
        };
        println!("{}", serde_json::to_string(&payload)?);
        return Ok(());
    }

    match output {
        OutputFormat::Json => println!("{}", to_pretty_json(result)?),
        OutputFormat::Table => println!("{}", render_download_table(result)),
    }

    Ok(())
}

fn emit_dry_run_output(
    result: &DryRunResult,
    path_only: bool,
    minimal: bool,
    output: OutputFormat,
) -> Result<()> {
    if path_only {
        println!("{}", result.path);
        return Ok(());
    }

    if minimal {
        let payload = MinimalDryRunResult {
            path: result.path.clone(),
            filename: result.filename.clone(),
            enclosure_url: result.enclosure_url.clone(),
            content_type: result.content_type.clone(),
            dry_run: true,
        };
        println!("{}", serde_json::to_string(&payload)?);
        return Ok(());
    }

    match output {
        OutputFormat::Json => println!("{}", to_pretty_json(result)?),
        OutputFormat::Table => println!("{}", render_dry_run_table(result)),
    }

    Ok(())
}

fn render_download_table(result: &DownloadResult) -> String {
    let mut table = Table::new();
    table.add_row(Row::new(vec![Cell::new("Field"), Cell::new("Value")]));
    table.add_row(Row::new(vec![
        Cell::new("Episode ID"),
        Cell::new(&result.episode_id.to_string()),
    ]));
    table.add_row(Row::new(vec![Cell::new("Path"), Cell::new(&result.path)]));
    table.add_row(Row::new(vec![
        Cell::new("Filename"),
        Cell::new(&result.filename),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Size"),
        Cell::new(&result.size.to_string()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Content Type"),
        Cell::new(result.content_type.as_deref().unwrap_or("-")),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Enclosure URL"),
        Cell::new(&result.enclosure_url),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Resumed"),
        Cell::new(if result.resumed { "yes" } else { "no" }),
    ]));

    table.to_string()
}

fn render_dry_run_table(result: &DryRunResult) -> String {
    let mut table = Table::new();
    table.add_row(Row::new(vec![Cell::new("Field"), Cell::new("Value")]));
    table.add_row(Row::new(vec![
        Cell::new("Episode ID"),
        Cell::new(&result.episode_id.to_string()),
    ]));
    table.add_row(Row::new(vec![Cell::new("Path"), Cell::new(&result.path)]));
    table.add_row(Row::new(vec![
        Cell::new("Filename"),
        Cell::new(&result.filename),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Content Type"),
        Cell::new(result.content_type.as_deref().unwrap_or("-")),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Enclosure URL"),
        Cell::new(&result.enclosure_url),
    ]));
    table.add_row(Row::new(vec![Cell::new("Dry Run"), Cell::new("yes")]));

    table.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;
    use tempfile::tempdir;

    #[test]
    fn sanitize_filename_replaces_reserved_characters() {
        let value = sanitize_filename(" hello:/bad\\name?.mp3 ");
        assert_eq!(value, "hello__bad_name_.mp3");
    }

    #[test]
    fn infer_extension_prefers_content_type() {
        let url = Url::parse("https://example.com/audio.ogg").expect("url");
        let ext = infer_extension(Some("audio/mpeg"), Some("audio/ogg"), &url);
        assert_eq!(ext, ".mp3");
    }

    #[test]
    fn infer_extension_falls_back_to_url_extension() {
        let url = Url::parse("https://example.com/audio.opus").expect("url");
        let ext = infer_extension(None, None, &url);
        assert_eq!(ext, ".opus");
    }

    #[test]
    fn resolve_target_path_uses_directory_destination() {
        let dest = Path::new("downloads");
        let path =
            resolve_target_path(Some(dest), None, "episode.mp3").expect("resolve target path");

        assert_eq!(path, PathBuf::from("downloads/episode.mp3"));
    }

    #[test]
    fn resolve_target_path_uses_file_destination() {
        let dest = Path::new("downloads/episode.mp3");
        let path =
            resolve_target_path(Some(dest), None, "ignored.mp3").expect("resolve target path");

        assert_eq!(path, PathBuf::from("downloads/episode.mp3"));
    }

    #[test]
    fn derive_filename_preserves_existing_extension() {
        let episode = Episode {
            title: Some("A title".to_string()),
            ..Episode::default()
        };
        let url = Url::parse("https://cdn.example.com/episode.m4a").expect("url");

        let name = derive_filename(&episode, 42, &url, ".mp3");
        assert_eq!(name, "episode.m4a");
    }

    #[test]
    fn parse_content_range_total_extracts_total_size() {
        assert_eq!(parse_content_range_total("bytes 0-0/123"), Some(123));
        assert_eq!(parse_content_range_total("bytes 0-0/*"), None);
    }

    #[test]
    fn minimal_download_result_is_compact_json() {
        let payload = MinimalDownloadResult {
            path: "/tmp/episode.mp3".to_string(),
            size: 128,
        };

        let encoded = serde_json::to_string(&payload).expect("serialize");
        assert_eq!(encoded, r#"{"path":"/tmp/episode.mp3","size":128}"#);
    }

    #[test]
    fn minimal_dry_run_result_includes_required_fields() {
        let payload = MinimalDryRunResult {
            path: "/tmp/episode.mp3".to_string(),
            filename: "episode.mp3".to_string(),
            enclosure_url: "https://example.com/episode.mp3".to_string(),
            content_type: Some("audio/mpeg".to_string()),
            dry_run: true,
        };

        let encoded = serde_json::to_string(&payload).expect("serialize");
        assert!(encoded.contains("\"path\""));
        assert!(encoded.contains("\"filename\""));
        assert!(encoded.contains("\"enclosure_url\""));
        assert!(encoded.contains("\"dry_run\":true"));
    }

    #[test]
    fn whitelist_allows_known_audio_content_type() {
        assert!(is_allowed_content_type("audio/mpeg"));
    }

    #[test]
    fn whitelist_rejects_unexpected_content_type() {
        assert!(!is_allowed_content_type("text/html"));
    }

    #[test]
    fn size_limit_rejects_oversized_download() {
        let err = ensure_download_size_within_limit(Some(MAX_DOWNLOAD_SIZE_BYTES + 1), 0)
            .expect_err("oversized download should fail");
        assert!(matches!(err, PodcastCliError::Validation(_)));
    }

    #[test]
    fn promote_part_file_without_overwrite_preserves_existing_target() {
        let dir = tempdir().expect("tempdir");
        let target = dir.path().join("episode.mp3");
        let part = dir.path().join("episode.mp3.part");

        fs::write(&target, b"old").expect("write target");
        fs::write(&part, b"new").expect("write part");

        let err = promote_part_file(&part, &target, false).expect_err("must not overwrite");
        assert!(matches!(
            err,
            PodcastCliError::Io(ref io_err) if io_err.kind() == ErrorKind::AlreadyExists
        ));
        assert_eq!(fs::read(&target).expect("read target"), b"old");
        assert_eq!(fs::read(&part).expect("read part"), b"new");
    }

    #[test]
    fn promote_part_file_with_overwrite_replaces_target() {
        let dir = tempdir().expect("tempdir");
        let target = dir.path().join("episode.mp3");
        let part = dir.path().join("episode.mp3.part");

        fs::write(&target, b"old").expect("write target");
        fs::write(&part, b"new").expect("write part");

        promote_part_file(&part, &target, true).expect("overwrite should replace target");
        assert_eq!(fs::read(&target).expect("read target"), b"new");
        assert!(!part.exists());
    }
}
