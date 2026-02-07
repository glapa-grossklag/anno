# anno

A minimal hexdump utility with type annotation support. Display hex dumps with colored output and automatic type decoding.

## Features

- **Compact output**: 16 bytes per line with 8+8 grouping
- **Type annotations**: Automatically decode and annotate binary data as typed values
- **Visual annotations**: Pretty Unicode box-drawing characters (└──┘) highlight annotated bytes
- **Byte order support**: Little-endian (default) or big-endian decoding
- **Smart colors**: Respects `NO_COLOR`, `TERM=dumb`, and TTY detection
- **Minimal size**: ~340KB optimized binary

## Installation

```bash
cargo build --release
```

Binary will be at `./target/release/anno`

## Usage

### Basic hexdump (no annotations)

```bash
# From file
anno -f data.bin

# From stdin
echo "Hello, World!" | anno
cat data.bin | anno
```

### Type annotations

Specify data types to automatically decode and annotate:

```bash
# Annotate as: u16, then u32, then u32, then u32, then u16
printf '\x00\x01aaaaaaaaaaaaaaaa' | anno u16 u32 u32 u32 u16
```

### Field names

Use custom field names instead of type names for better clarity:

```bash
# Use field names for struct-like data
printf '\x12\x34aaaaaaaaaa\xFF' | anno u16:packet_id u32:timestamp u32:sequence u16:flags u8:version
```

Output shows field names instead of types:
```
00000000  12 34 61 61 61 61 61 61  61 61 61 61 ff
         └─────┘                                           packet_id: 13330
               └───────────┘                               timestamp: 1633771873
                           └────────────┘                  sequence: 1633771873
                                        └─────┘            flags: 24929
                                              └──┘         version: 255
```

Mix field names and plain types as needed:

```bash
# Some fields named, others use type names
anno u16:apid u32 u32:data u8:x
```

Output:
```
00000000  00 01 61 61 61 61 61 61  61 61 61 61 61 61 61 61
          └─────┘                                           u16: 256
                └───────────┘                               u32: 1633771873
                            └────────────┘                  u32: 1633771873
                                         └───────────┘      u32: 1633771873
                                                     └────┘ u16: 24929
00000010  61 61
00000012
```

### Byte order

```bash
# Big-endian decoding
anno u32 --byte-order big -f data.bin

# Little-endian (default)
anno u32 --byte-order little -f data.bin
```

## Supported Types

| Type | Size | Description |
|------|------|-------------|
| `u8` | 1 byte | Unsigned 8-bit integer |
| `u16` | 2 bytes | Unsigned 16-bit integer |
| `u32` | 4 bytes | Unsigned 32-bit integer |
| `u64` | 8 bytes | Unsigned 64-bit integer |
| `i8` | 1 byte | Signed 8-bit integer |
| `i16` | 2 bytes | Signed 16-bit integer |
| `i32` | 4 bytes | Signed 32-bit integer |
| `i64` | 8 bytes | Signed 64-bit integer |
| `f32` / `float` | 4 bytes | 32-bit floating point |
| `f64` / `double` | 8 bytes | 64-bit floating point |

## Examples

### Decode a struct

```bash
# Decode: uint8_t version, uint32_t count, double value
echo -ne '\x01\x00\x00\x00\x64\x40\x09\x21\xfb\x54\x44\x2d\x18' | \
  anno u8 u32 f64
```

### Mix signed and unsigned

```bash
# i8 can display negative values
printf '\xFF\x00\x00\x00\x64' | anno i8 u32
```

### Examine floats

```bash
python3 -c "import struct; print(struct.pack('ff', 3.14159, 2.71828), end='')" | \
  anno f32 f32
```

## Color Scheme

- **Addresses** (left column): Green
- **Annotated bytes** (hex): Blue
- **Annotation labels**:
  - Type name (e.g., `u16`, `f32`): Purple
  - Colon: Uncolored
  - Value (e.g., `256`, `3.14`): Blue
- **Regular bytes**: Default terminal color

Colors are automatically disabled when:
- `NO_COLOR` environment variable is set
- `TERM=dumb`
- Output is not a TTY (e.g., piped to file)

## Options

```
Usage: anno [<types...>] [-f <file>] [--byte-order <byte-order>]

Positional Arguments:
  types             data types to annotate (e.g., u8 u16 u32)
                    optionally with field names (e.g., u16:apid u32:count)

Options:
  -f, --file        file to read (reads from stdin if not provided)
  --byte-order      byte order for multi-byte types: little (default) or big
  --help            display usage information
```

## License

MIT (or specify your license)
