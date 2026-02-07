# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`anno` is a minimal hexdump utility written in Rust with type annotation support. It displays hex dumps with colored output and automatically decodes binary data into typed values (integers, floats) with configurable byte order. Annotations appear as labeled underlines using Unicode box-drawing characters.

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

**Hexdump struct** (`src/main.rs`): Main engine that processes input and renders output
- Maintains a list of `Annotation` objects
- Auto-detects color support (respects `NO_COLOR`, `TERM=dumb`, TTY detection)
- Renders 16 bytes per line with 8+8 grouping

**Annotation system**: Associates byte ranges with labels
- `Annotation { offset, length, label }` - marks byte ranges
- Annotations can span multiple lines
- Renders with Unicode box-drawing: `└──────┘` under annotated bytes
- All annotation labels align vertically at column 59 (character position, not byte position)
- Label format: `"value (type)"` e.g., `"42 (u8)"` or `"3.141590 (f32)"`

**Type annotation system** (`src/types.rs`): Automatic decoding of binary data
- `DataType` enum: U8, U16, U32, U64, I8, I16, I32, I64, F32, F64
- `ByteOrder` enum: Little (default) or Big
- `build_annotations_from_types()`: Sequential type decoding from command-line args
- Example: `anno u8 u32 f64 --byte-order little -f data.bin`

### Color Scheme
- Addresses (left column): Green
- Annotated bytes and labels: Blue
- Regular hex bytes: No color

### Critical Implementation Details

**Label alignment**: The annotation rendering uses Unicode box-drawing characters which are multi-byte in UTF-8 but single display characters. Label alignment is calculated by counting `chars()`, not bytes. The alignment logic builds the entire underline string first, counts display width with `.chars().count()`, then pads to align all labels at column 59.

**Position 16 edge case**: Annotations ending exactly at position 16 (byte 15, the last byte of a line) require special handling. The last annotated byte adds only "──" (2 chars) instead of "───" (3 chars) to the underline, ensuring correct alignment when the closing "┘" is added after the loop. This fix is critical for vertical alignment. See `tests/alignment_edge_case_test.rs` for regression testing.

**Multi-line annotations**: Annotations spanning line boundaries show continuation with no closing corner on first line, and no opening corner on subsequent lines. Labels only appear on the first line.

**Test architecture**: Integration tests import from `main.rs` using `#[path = "../src/main.rs"]` to test the public API (`Hexdump`, `Annotation`, `DataType`, `ByteOrder`). Tests verify:
- Vertical alignment of all annotation labels (including position 16 edge case)
- Type decoding for all integer and float types with both byte orders
- Multi-line continuation characters
- Edge cases: empty input, boundary conditions, overlapping annotations

## Size Optimization

The release profile is configured for minimal binary size:
- `opt-level = "z"` - optimize for size
- `lto = true` - link-time optimization
- `strip = true` - strip symbols
- `panic = "abort"` - smaller panic handler
- `codegen-units = 1` - better optimization

Binary size target: ~340KB

## Test Coverage

106 tests passing across 6 test files:
- `src/types.rs`: 11 unit tests for type parsing, sizes, and decoding
- `tests/alignment_test.rs`: 12 tests verifying vertical label alignment
- `tests/alignment_edge_case_test.rs`: 1 test for position 16 edge case (u16 u32 u32 u32 u16 pattern)
- `tests/comprehensive_test.rs`: 22 tests covering edge cases, boundaries, multi-line, overlapping
- `tests/continuation_test.rs`: 14 tests for multi-line annotation continuation
- `tests/type_annotation_test.rs`: 24 tests for all data types with both byte orders
- `src/main.rs`: 11 module-level tests

## Usage Examples

```bash
# Basic hexdump
echo "Hello" | ./target/release/anno

# Decode types: u16 followed by four u32s and another u16
printf '\x00\x01aaaaaaaaaaaaaaaa' | ./target/release/anno u16 u32 u32 u32 u16

# Big-endian decoding
./target/release/anno u32 u64 --byte-order big -f data.bin

# Floats and doubles
python3 -c "import struct; print(struct.pack('fd', 3.14159, 2.71828), end='')" | \
  ./target/release/anno f32 f64
```
