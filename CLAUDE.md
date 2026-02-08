# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`anno` is a minimal hexdump utility written in Rust with type annotation support. It displays hex dumps with colored output and automatically decodes binary data into typed values (integers, floats) with configurable byte order. Annotations appear as labeled underlines using Unicode box-drawing characters.

## Build & Test Commands

```bash
# Build optimized release binary (size-optimized, ~393KB)
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

The codebase is organized into focused modules:

**`src/main.rs`** (138 lines): CLI parsing and entry point
- `Args` struct: Command-line argument parsing with argh
- `TypeSpec` enum: Parses type specifications, field names, or skip directives
  - `Type { data_type, field_name }`: Data type with optional field name (e.g., `u16:apid`)
  - `Skip { bytes }`: Skip directive (e.g., `.32` to skip 4 bytes)
- `build_annotations_from_types()`: Sequential type decoding from command-line args
- `main()`: Orchestrates reading input, building annotations, and rendering output

**`src/display.rs`** (270 lines): Hexdump rendering engine
- `Annotation` struct: Marks byte ranges with labels `{ offset, length, label }`
- `Hexdump` struct: Main rendering engine
  - Maintains list of annotations
  - Renders 16 bytes per line with 8+8 grouping
  - Handles multi-line annotations with continuation characters
  - All annotation labels align vertically at column 59 (character position, not byte position)

**`src/color.rs`** (75 lines): Color support and TTY detection
- `ColorScheme` struct: Manages colored output
- Auto-detects color support (respects `NO_COLOR`, `TERM=dumb`, TTY detection)
- Methods: `addr()` (green), `annotation()` (blue), `label()` (purple type, blue value)

**`src/types.rs`** (297 lines): Type system and decoding
- `DataType` enum: U8, U16, U32, U64, I8, I16, I32, I64, F32, F64
- `ByteOrder` enum: Native (default), Little, or Big
  - Native uses compile-time detection via `cfg!(target_endian)`
- `decode()`: Converts raw bytes to string representations
- Field names: Use `type:fieldname` syntax to display custom names instead of type names

### Color Scheme
- Addresses (left column): Green
- Annotated bytes (hex): Blue
- Annotation labels:
  - Type name: Purple
  - Colon: Uncolored
  - Value: Blue
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

Current binary size: 393KB (includes type system, field names, and full feature set)

## Test Coverage

141 tests passing across 8 test files:
- `src/types.rs`: 11 unit tests for type parsing, sizes, and decoding
- `tests/alignment_test.rs`: 12 tests verifying vertical label alignment
- `tests/alignment_edge_case_test.rs`: 1 test for position 16 edge case (u16 u32 u32 u32 u16 pattern)
- `tests/comprehensive_test.rs`: 22 tests covering edge cases, boundaries, multi-line, overlapping
- `tests/continuation_test.rs`: 14 tests for multi-line annotation continuation
- `tests/type_annotation_test.rs`: 24 tests for all data types with both byte orders
- `tests/field_name_test.rs`: 8 tests for field name parsing, mixed types, alignment, error handling
- `tests/skip_test.rs`: 16 tests for skip directives (.8, .16, .32, etc.), error cases, complex patterns
- `src/main.rs`: 11 module-level tests

## Usage Examples

```bash
# Basic hexdump
echo "Hello" | ./target/release/anno

# Decode types: u16 followed by four u32s and another u16
printf '\x00\x01aaaaaaaaaaaaaaaa' | ./target/release/anno u16 u32 u32 u32 u16

# Use field names for struct-like data
printf '\x12\x34aaaaaaaaaa\xFF' | ./target/release/anno u16:packet_id u32:timestamp u32:sequence u16:flags u8:version

# Mix field names and plain types
./target/release/anno u16:apid u32 u32:data u8:x -f data.bin

# Explicit byte order (default is native endianness)
./target/release/anno u32 u64 --byte-order big -f data.bin
./target/release/anno u32 u64 --byte-order little -f data.bin

# Floats and doubles
python3 -c "import struct; print(struct.pack('fd', 3.14159, 2.71828), end='')" | \
  ./target/release/anno f32 f64

# Skip bytes with .N syntax (N = bits)
printf '\x12\x34\xAA\xBB\xCC\xDD\x56\x78' | ./target/release/anno u16:magic .32 u16:data
# Skips bytes aa bb cc dd (4 bytes = 32 bits)

# Complex pattern with skips
printf '\x01\x02\x03\x04\x05\x06\x07\x08' | \
  ./target/release/anno u8:version .8 u16:id .16 u16:checksum
```
