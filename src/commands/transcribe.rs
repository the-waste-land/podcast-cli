use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::cli::TranscribeArgs;
use crate::error::{PodcastCliError, Result};

const WHISPER_BINARY: &str = "whisper";
const FASTER_WHISPER_SCRIPT: &str = r#"
from faster_whisper import WhisperModel
import json
import sys

model_name = sys.argv[1]
audio_path = sys.argv[2]
language = sys.argv[3]

model = WhisperModel(model_name, device="cpu", compute_type="int8")
segments_gen, info = model.transcribe(audio_path, language=language)

# Collect segments first (generator can only be iterated once)
segments_list = list(segments_gen)

result = {
    "text": " ".join([s.text for s in segments_list]),
    "segments": [{
        "id": s.id,
        "start": s.start,
        "end": s.end,
        "text": s.text
    } for s in segments_list],
    "language": info.language,
    "model": model_name,
    "duration": sum([s.end - s.start for s in segments_list])
}

print(json.dumps(result))
"#;

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

    // Try faster-whisper first, then fall back to whisper CLI
    let result = match try_faster_whisper(&audio_path, &args.model, &args.language) {
        Ok(r) => r,
        Err(_e) => {
            // Fall back to whisper CLI
            run_whisper_cli(&audio_path, &args.model, &args.language, &args.format)?
        }
    };

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

fn try_faster_whisper(
    audio_path: &Path,
    model: &str,
    language: &str,
) -> Result<TranscribeResult> {
    // Check if faster-whisper is available
    let check = Command::new("python3")
        .arg("-c")
        .arg("from faster_whisper import WhisperModel")
        .output();

    if check.is_err() {
        return Err(PodcastCliError::Validation(
            "faster-whisper not available".to_string(),
        ));
    }

    let output = Command::new("python3")
        .arg("-c")
        .arg(FASTER_WHISPER_SCRIPT)
        .arg(model)
        .arg(audio_path)
        .arg(language)
        .output()
        .map_err(|e| {
            PodcastCliError::Validation(format!("failed to run faster-whisper: {}", e))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PodcastCliError::Validation(format!(
            "faster-whisper failed: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    #[derive(Deserialize)]
    struct FasterWhisperResult {
        text: String,
        segments: Vec<TranscriptSegment>,
        language: String,
        model: String,
        duration: f64,
    }

    let result: FasterWhisperResult = serde_json::from_str(&stdout).map_err(|e| {
        PodcastCliError::Validation(format!("failed to parse faster-whisper output: {}", e))
    })?;

    Ok(TranscribeResult {
        text: result.text,
        segments: result.segments,
        language: result.language,
        model: result.model,
        duration: result.duration,
    })
}

fn run_whisper_cli(
    audio_path: &Path,
    model: &str,
    language: &str,
    format: &crate::cli::TranscribeFormat,
) -> Result<TranscribeResult> {
    // Check if whisper CLI is available
    let whisper_available = Command::new(WHISPER_BINARY)
        .arg("--version")
        .output()
        .is_ok();

    if !whisper_available {
        return Err(PodcastCliError::Validation(
            "whisper CLI not found. Please install whisper: pip install openai-whisper".to_string(),
        ));
    }

    // Create temp output path
    let temp_dir = std::env::temp_dir();
    let _temp_output = temp_dir.join("transcript");

    // Determine output format for whisper
    let whisper_format = match format {
        crate::cli::TranscribeFormat::Json => "json",
        crate::cli::TranscribeFormat::Text => "txt",
        crate::cli::TranscribeFormat::Srt => "srt",
    };

    let mut cmd = Command::new(WHISPER_BINARY);
    cmd.arg("--model")
        .arg(model)
        .arg("--language")
        .arg(language)
        .arg("--output_format")
        .arg(whisper_format)
        .arg("--output_dir")
        .arg(&temp_dir)
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
    let output_file = temp_dir.join(format!("transcript.{}", match format {
        crate::cli::TranscribeFormat::Json => "json",
        crate::cli::TranscribeFormat::Text => "txt",
        crate::cli::TranscribeFormat::Srt => "srt",
    }));

    let content = std::fs::read_to_string(&output_file).map_err(PodcastCliError::Io)?;

    // Parse result based on format
    match format {
        crate::cli::TranscribeFormat::Json => {
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
                duration: 0.0,
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
    let parts: Vec<&str> = time.split([':', ',']).collect();
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
