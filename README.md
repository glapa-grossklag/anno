# anno

A minimal hexdump utility that annotates binary data with decoded types and field names.

## Installation

```bash
cargo build --release
```

Binary: `./target/release/anno`

## Examples

### Basic hexdump

```bash
echo "Hello, World!" | anno
```

Output:
```
00000000  48 65 6c 6c 6f 2c 20 57  6f 72 6c 64 21 0a
0000000e
```

### Decode types

```bash
printf '\x2A\x34\x12\x78\x56\x34\x12' | anno u8 u16 u32
```

Output:
```
00000000  2a 34 12 78 56 34 12
         └──┘                                              u8: 42
            └─────┘                                        u16: 4660
                  └────────────┘                           u32: 305419896
00000007
```

### Use field names

```bash
printf '\x01\x00\x64\x00\x00\x00\xFF' | anno u8:version u16:id u32:count i8:delta
```

Output:
```
00000000  01 00 64 00 00 00 ff
         └──┘                                              version: 1
            └─────┘                                        id: 100
                  └────────────┘                           count: 100
                                 └──┘                      delta: -1
00000007
```

### Skip bytes

Use `.N` syntax to skip over bytes (where N is number of bits):

```bash
printf '\x12\x34\xAA\xBB\xCC\xDD\x56\x78' | anno u16:magic .32 u16:data
```

Output:
```
00000000  12 34 aa bb cc dd 56 78
         └─────┘                                           magic: 13330
                           └──────┘                        data: 30806
00000008
```

Common skip sizes: `.8` (1 byte), `.16` (2 bytes), `.32` (4 bytes), `.64` (8 bytes)

### Network packet

```bash
printf '\x12\x34\x00\x01\x00\x00\x00\x64\x00\x00\x00\xC8' | \
  anno u16:packet_id u16:version u32:timestamp u32:sequence
```

Output:
```
00000000  12 34 00 01 00 00 00 64  00 00 00 c8
         └─────┘                                           packet_id: 13330
               └─────┘                                     version: 256
                     └────────────┘                        timestamp: 100
                                  └────────────┘           sequence: 200
0000000c
```

### Byte order

```bash
# Native endianness (default)
printf '\x12\x34\x56\x78' | anno u32
# Output: u32: 2018915346 (on little-endian systems)

# Explicit little-endian
printf '\x12\x34\x56\x78' | anno u32 --byte-order little
# Output: u32: 2018915346

# Big-endian
printf '\x12\x34\x56\x78' | anno u32 --byte-order big
# Output: u32: 305419896
```

### From file

```bash
anno u32:magic u32:version u64:timestamp -f data.bin
```

## Supported types

`u8` `u16` `u32` `u64` `i8` `i16` `i32` `i64` `f32` `f64`

## Options

```
anno [types...] [-f <file>] [--byte-order <native|little|big>]
```

Default byte order is native endianness (determined at compile time).
