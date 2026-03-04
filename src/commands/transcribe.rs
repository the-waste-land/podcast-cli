use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::cli::TranscribeArgs;
use crate::error::{PodcastCliError, Result};

const WHISPER_BINARY: &str = "whisper";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranscriptSegment {
    pub id: usize,
    pub start: f64,
    pub end: f64,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct TranscribeResult {
    pub text: String,
    pub segments: Vec<TranscriptSegment>,
    pub language: String,
    pub model: String,
    pub duration: f64,
}

pub async fn run(args: TranscribeArgs) -> Result<()> {
    let audio_path = if let Some(_episode_id) = args.episode_id {
        // TODO: Download episode first, then transcribe
        return Err(PodcastCliError::NotImplemented(
            "episode_id download not implemented yet. Please use --audio-file instead.".to_string(),
        ));
    } else if let Some(audio_file) = &args.audio_file {
        audio_file.clone()
    } else {
        return Err(PodcastCliError::Validation(
            "either --audio-file or --episode-id must be provided".to_string(),
        ));
    };

    // Validate audio file exists
    if !audio_path.exists() {
        return Err(PodcastCliError::Validation(format!(
            "audio file not found: {}",
            audio_path.display()
        )));
    }

    // Check if whisper is available
    let whisper_available = Command::new(WHISPER_BINARY)
        .arg("--version")
        .output()
        .is_ok();

    if !whisper_available {
        return Err(PodcastCliError::Validation(
            "whisper CLI not found. Please install whisper: pip install openai-whisper".to_string(),
        ));
    }

    // Prepare output path
    let output_path = if let Some(output) = args.output {
        output
    } else {
        let stem = audio_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("transcript");
        audio_path.parent().unwrap_or(&audio_path).join(stem)
    };

    // Run whisper
    let result = run_whisper(&audio_path, &args.model, &args.language, &output_path, args.format)?;

    // Handle output based on format
    match args.format {
        crate::cli::TranscribeFormat::Json => {
            let json = serde_json::to_string_pretty(&result).map_err(|e| {
                PodcastCliError::Validation(format!("failed to serialize result: {}", e))
            })?;
            println!("{}", json);
        }
        crate::cli::TranscribeFormat::Text => {
            println!("{}", result.text);
        }
        crate::cli::TranscribeFormat::Srt => {
            let srt = segments_to_srt(&result.segments);
            println!("{}", srt);
        }
    }

    Ok(())
}

fn run_whisper(
    audio_path: &PathBuf,
    model: &str,
    language: &str,
    output_path: &PathBuf,
    format: crate::cli::TranscribeFormat,
) -> Result<TranscribeResult> {
    // Determine output format for whisper
    let whisper_format = match format {
        crate::cli::TranscribeFormat::Json => "json",
        crate::cli::TranscribeFormat::Text => "txt",
        crate::cli::TranscribeFormat::Srt => "srt",
    };

    // Create temp output path
    let _temp_output = output_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("transcript");

    let mut cmd = Command::new(WHISPER_BINARY);
    cmd.arg("--model")
        .arg(model)
        .arg("--language")
        .arg(language)
        .arg("--output_format")
        .arg(whisper_format)
        .arg("--output_dir")
        .arg(output_path.parent().unwrap_or(output_path))
        .arg(audio_path);

    let output = cmd.output().map_err(|e| {
        PodcastCliError::Validation(format!("failed to run whisper: {}", e))
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PodcastCliError::Validation(format!(
            "whisper failed: {}",
            stderr
        )));
    }

    // Read the output file
    let output_file = output_path.with_extension(match format {
        crate::cli::TranscribeFormat::Json => "json",
        crate::cli::TranscribeFormat::Text => "txt",
        crate::cli::TranscribeFormat::Srt => "srt",
    });

    let content = std::fs::read_to_string(&output_file).map_err(|e| {
        PodcastCliError::Io(e)
    })?;

    // Parse result based on format
    match format {
        crate::cli::TranscribeFormat::Json => {
            // Parse whisper JSON output
            #[derive(Deserialize)]
            struct WhisperJson {
                text: String,
                segments: Vec<WhisperSegment>,
            }

            #[derive(Deserialize)]
            struct WhisperSegment {
                id: usize,
                start: f64,
                end: f64,
                text: String,
            }

            let whisper_result: WhisperJson = serde_json::from_str(&content).map_err(|e| {
                PodcastCliError::Validation(format!("failed to parse whisper output: {}", e))
            })?;

            Ok(TranscribeResult {
                text: whisper_result.text,
                segments: whisper_result
                    .segments
                    .into_iter()
                    .map(|s| TranscriptSegment {
                        id: s.id,
                        start: s.start,
                        end: s.end,
                        text: s.text,
                    })
                    .collect(),
                language: language.to_string(),
                model: model.to_string(),
                duration: 0.0, // Could extract from metadata
            })
        }
        crate::cli::TranscribeFormat::Text => Ok(TranscribeResult {
            text: content,
            segments: vec![],
            language: language.to_string(),
            model: model.to_string(),
            duration: 0.0,
        }),
        crate::cli::TranscribeFormat::Srt => {
            // Parse SRT to segments
            let segments = srt_to_segments(&content)?;
            let text = segments.iter().map(|s| s.text.trim()).collect::<Vec<_>>().join(" ");

            Ok(TranscribeResult {
                text,
                segments,
                language: language.to_string(),
                model: model.to_string(),
                duration: 0.0,
            })
        }
    }
}

fn srt_to_segments(srt: &str) -> Result<Vec<TranscriptSegment>> {
    let mut segments = Vec::new();
    let mut current_id = 0;
    let mut current_start = 0.0;
    let mut current_end = 0.0;
    let mut current_text = String::new();

    for line in srt.lines() {
        let line = line.trim();

        if line.is_empty() {
            if !current_text.is_empty() {
                current_id += 1;
                segments.push(TranscriptSegment {
                    id: current_id,
                    start: current_start,
                    end: current_end,
                    text: current_text.trim().to_string(),
                });
                current_text = String::new();
            }
            continue;
        }

        // Try to parse as index
        if line.parse::<usize>().is_ok() {
            continue;
        }

        // Try to parse time line: 00:00:00,000 --> 00:00:00,000
        if line.contains("-->") {
            let times: Vec<&str> = line.split("-->").collect();
            if times.len() == 2 {
                current_start = parse_srt_time(times[0]);
                current_end = parse_srt_time(times[1]);
            }
            continue;
        }

        // Otherwise it's text
        if !current_text.is_empty() {
            current_text.push(' ');
        }
        current_text.push_str(line);
    }

    // Don't forget the last segment
    if !current_text.is_empty() {
        current_id += 1;
        segments.push(TranscriptSegment {
            id: current_id,
            start: current_start,
            end: current_end,
            text: current_text.trim().to_string(),
        });
    }

    Ok(segments)
}

fn parse_srt_time(time: &str) -> f64 {
    let time = time.trim();
    // Format: 00:00:00,000
    let parts: Vec<&str> = time.split(|c| c == ':' || c == ',').collect();
    if parts.len() >= 3 {
        let hours: f64 = parts[0].parse().unwrap_or(0.0);
        let minutes: f64 = parts[1].parse().unwrap_or(0.0);
        let seconds: f64 = parts[2].parse().unwrap_or(0.0);
        let millis: f64 = parts.get(3).unwrap_or(&"0").parse().unwrap_or(0.0);
        return hours * 3600.0 + minutes * 60.0 + seconds + millis / 1000.0;
    }
    0.0
}

fn segments_to_srt(segments: &[TranscriptSegment]) -> String {
    let mut output = String::new();

    for (i, segment) in segments.iter().enumerate() {
        output.push_str(&format!("{}\n", i + 1));
        output.push_str(&format!(
            "{} --> {}\n{}\n\n",
            format_srt_time(segment.start),
            format_srt_time(segment.end),
            segment.text
        ));
    }

    output
}

fn format_srt_time(seconds: f64) -> String {
    let hours = (seconds / 3600.0).floor() as u64;
    let minutes = ((seconds % 3600.0) / 60.0).floor() as u64;
    let secs = (seconds % 60.0).floor() as u64;
    let millis = ((seconds % 1.0) * 1000.0).round() as u64;

    format!("{:02}:{:02}:{:02},{:03}", hours, minutes, secs, millis)
}
