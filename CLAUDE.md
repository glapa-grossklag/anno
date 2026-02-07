# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`anno` is a minimal hexdump utility written in Rust with annotation support. It displays hex dumps with colored output and allows annotating byte ranges with labeled underlines using Unicode box-drawing characters.

## Build & Test Commands

```bash
# Build optimized release binary (size-optimized, ~339KB)
cargo build --release

# Run the binary
./target/release/anno <filename>
echo "data" | ./target/release/anno

# Run tests
cargo test

# Run specific integration test
cargo test --test alignment_test
```

## Architecture

### Core Components

**Hexdump struct**: Main engine that processes input and renders output
- Maintains a list of `Annotation` objects
- Auto-detects color support (respects `NO_COLOR`, `TERM=dumb`, TTY detection)
- Renders 16 bytes per line with 8+8 grouping

**Annotation system**: Associates byte ranges with labels
- `Annotation { offset, length, label }` - marks byte ranges
- Annotations can span multiple lines
- Renders with Unicode box-drawing: `└──────┘` under annotated bytes
- All annotation labels align vertically at column 58 (character position, not byte position)

### Color Scheme
- Addresses (left column): Green
- Annotated bytes and labels: Blue
- Regular hex bytes: No color

### Critical Implementation Details

**Label alignment**: The annotation rendering uses Unicode box-drawing characters which are multi-byte in UTF-8 but single display characters. Label alignment is calculated by counting `chars()`, not bytes. The alignment logic builds the entire underline string first, counts display width with `.chars().count()`, then pads to align all labels at column 58.

**Test architecture**: Integration tests import from `main.rs` using `#[path = "../src/main.rs"]` to test the public API (`Hexdump`, `Annotation`). The alignment test verifies that all annotation labels align vertically by checking character positions.

## Size Optimization

The release profile is configured for minimal binary size:
- `opt-level = "z"` - optimize for size
- `lto = true` - link-time optimization
- `strip = true` - strip symbols
- `panic = "abort"` - smaller panic handler
- `codegen-units = 1` - better optimization

Binary size target: ~340KB

## Current State

The `main()` function contains example annotations (lines 255-257) for demonstration. These should be removed or replaced with actual annotation logic when implementing the final application.
