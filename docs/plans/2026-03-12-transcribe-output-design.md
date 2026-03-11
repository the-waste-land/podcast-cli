# Transcribe Output Design

## Problem

`podcast-cli transcribe --output <path>` currently exits with code `0` and prints the transcript to stdout, but does not create the requested file. This breaks callers that rely on the output path contract.

## Decision

When `--output <path>` is provided, the command will write the final rendered transcript to that file and will not print the transcript body to stdout. When `--output` is omitted, existing stdout behavior remains unchanged.

## Scope

- Keep the fix local to `src/commands/transcribe.rs`.
- Reuse the same output-routing logic for all `--format` variants.
- Return an `Io` error if the output file cannot be written.

## Non-Goals

- Changing transcription backends or model selection.
- Implementing `--episode-id` download flow.
- Refactoring unrelated `transcribe` parsing behavior.

## Testing

- Add unit coverage for the output-routing logic:
  - writes rendered transcript to `--output`
  - suppresses stdout payload when writing to file
  - keeps stdout behavior unchanged without `--output`
