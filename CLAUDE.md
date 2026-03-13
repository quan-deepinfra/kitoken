# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Kitoken is a fast, `no_std`-compatible tokenizer library for language models, supporting BPE, Unigram, and WordPiece algorithms. It converts from SentencePiece, HuggingFace Tokenizers, OpenAI Tiktoken, and Mistral Tekken formats into a unified native format (.kit).

## Build & Test Commands

```bash
# Check and lint
cargo check -p kitoken
cargo clippy -p kitoken

# Run all tests (must be single-threaded)
cargo test -- --nocapture --test-threads=1

# Run a single test
cargo test <test_name> -- --nocapture --test-threads=1

# Verify no_std compatibility
cargo check -p kitoken --target wasm32v1-none --no-default-features

# Format check (requires nightly)
cargo +nightly fmt --all -- --check

# Run benchmarks
cargo bench

# Build CLI
cargo build -p kitoken-cli

# JavaScript (wasm-pack + pnpm)
cd packages/javascript && wasm-pack build --target nodejs
cd packages/javascript && pnpm test

# Python (maturin + uv)
cd packages/python && maturin develop
cd packages/python && uv run pytest
```

Set `DUMP_ERRORS=true` when running tests to get detailed error output on failures.

## Architecture

### Core Pipeline

Text flows through: **Normalization** → **Splitting** → **Encoding** → tokens, and reverse: tokens → **Decoding** → bytes.

- `Kitoken` (`src/lib.rs`) — Main API struct. Orchestrates the pipeline: applies configuration (normalize, split via regex), delegates to an `Encoder` impl, handles special tokens.
- `Definition` (`src/definition.rs`) — Serializable tokenizer definition containing `Model`, `Configuration`, `SpecialVocab`, and `Metadata`. The bridge between file formats and runtime `Kitoken`.
- `Encoder` trait (`src/encoder.rs`) — Core encoding interface with three implementations in `src/encoder/`:
  - `BytePair` — Merge-list-free BPE using priority queue
  - `Unigram` — Score-based probabilistic tokenization via dynamic programming
  - `WordPiece` — Greedy left-to-right with `##` continuation prefixes
- `Decoder` (`src/decoder.rs`) — Maps token IDs back to bytes, supports subword prefix handling.
- `Configuration` (`src/config/`) — Composed of normalization (Unicode/charsmap), splitting (regex/Unicode script), processing, and decoding configs.

### Format Conversion

`src/convert/` contains converters from external formats. Each produces a `Definition`. Auto-detection available with `convert-detect` feature.

### Serialization

Native `.kit` format uses `postcard` binary serialization with `b"kitoken"` magic header. See `src/serialization.rs`.

### Workspace Packages

- `packages/cli/` — CLI tool (clap-based): encode, decode, compare, convert, inspect
- `packages/python/` — PyO3 bindings (maturin build, Python 3.10+)
- `packages/javascript/` — WASM bindings (wasm-pack, Node.js ≥18)

## Key Types

- `TokenId` = `u32`, `TokenBytes` = `Vec<u8>`, `TokenScore` = `f32`
- `Vocab` — Ordered token list; `Scores` — parallel score array for Unigram
- `SpecialVocab` — Special tokens with kind (Control, StartSegment, EndSegment) and extraction priority
- `Model` enum — `BytePair`, `Unigram`, or `WordPiece` with their associated data

## Code Conventions

- Rust 2024 edition, MSRV 1.86.0
- Max line width: 100 chars (rustfmt)
- Heavy use of feature flags — most functionality is gated (see `Cargo.toml` `[features]`)
- `#[inline(never)]` on large/cold functions, `#[inline(always)]` on trivial hot paths
- Tests use model files in `tests/models/` and comparison data in `tests/data/`
- Test output compatibility target: identical output to original tokenizer implementations
