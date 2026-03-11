# Transcribe Output Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make `podcast-cli transcribe --output <path>` reliably write the rendered transcript to disk instead of only printing it to stdout.

**Architecture:** Keep the change inside the transcribe command by splitting output handling into two phases: render the final transcript string from `TranscribeResult`, then route that string either to stdout or to the requested file. This isolates the bug fix from backend-specific transcription logic.

**Tech Stack:** Rust, std::fs, existing `PodcastCliError` error handling, Rust unit tests

---

### Task 1: Add a failing output-routing test

**Files:**
- Modify: `src/commands/transcribe.rs`

**Step 1: Write the failing test**

Add a unit test that renders text output with `Some(output_path)` and asserts:
- the file is created
- the file content matches the rendered transcript
- the helper returns no stdout payload

**Step 2: Run test to verify it fails**

Run: `cargo test transcribe_writes_output_file_instead_of_stdout`
Expected: FAIL because the output-routing helper does not exist yet.

### Task 2: Implement minimal output routing

**Files:**
- Modify: `src/commands/transcribe.rs`

**Step 1: Write minimal implementation**

Add:
- a helper that renders `json` / `text` / `srt`
- a helper that either writes to `--output` or returns the string for stdout

Update `run()` to use that helper and only print when stdout output is returned.

**Step 2: Run focused tests**

Run: `cargo test transcribe_writes_output_file_instead_of_stdout`
Expected: PASS

### Task 3: Verify surrounding behavior

**Files:**
- Modify: `src/commands/transcribe.rs`

**Step 1: Add/keep a stdout-path test**

Cover the no-`--output` case so the prior behavior remains intact.

**Step 2: Run targeted verification**

Run: `cargo test transcribe_`
Expected: PASS

**Step 3: Run broader verification**

Run:
- `cargo fmt -- --check`
- `cargo test`

Expected: PASS
