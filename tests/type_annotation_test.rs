use std::io::Cursor;

#[path = "../src/main.rs"]
mod main_module;

use main_module::{Annotation, ByteOrder, DataType, Hexdump};

// Import the build function - we need to expose it or test it indirectly
// For now, we'll test through the Hexdump interface

#[test]
fn test_u8_annotation() {
    let data = vec![42u8, 100, 255];
    let mut hexdump = Hexdump::new();

    // Manually create annotations like the build function would
    hexdump.add_annotation(Annotation::new(
        0,
        1,
        format!("u8: {}", DataType::U8.decode(&data[0..1], ByteOrder::Little).unwrap()),
    ));
    hexdump.add_annotation(Annotation::new(
        1,
        1,
        format!("u8: {}", DataType::U8.decode(&data[1..2], ByteOrder::Little).unwrap()),
    ));
    hexdump.add_annotation(Annotation::new(
        2,
        1,
        format!("u8: {}", DataType::U8.decode(&data[2..3], ByteOrder::Little).unwrap()),
    ));

    let mut output = Vec::new();
    hexdump.dump(&mut Cursor::new(&data), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("u8: 42"));
    assert!(output_str.contains("u8: 100"));
    assert!(output_str.contains("u8: 255"));
}

#[test]
fn test_u16_little_endian() {
    let data = vec![0x34, 0x12]; // Little-endian 0x1234 = 4660
    let mut hexdump = Hexdump::new();

    hexdump.add_annotation(Annotation::new(
        0,
        2,
        format!(
            "u16: {}",
            DataType::U16.decode(&data, ByteOrder::Little).unwrap()
        ),
    ));

    let mut output = Vec::new();
    hexdump.dump(&mut Cursor::new(&data), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("u16: 4660"));
}

#[test]
fn test_u16_big_endian() {
    let data = vec![0x12, 0x34]; // Big-endian 0x1234 = 4660
    let mut hexdump = Hexdump::new();

    hexdump.add_annotation(Annotation::new(
        0,
        2,
        format!(
            "u16: {}",
            DataType::U16.decode(&data, ByteOrder::Big).unwrap()
        ),
    ));

    let mut output = Vec::new();
    hexdump.dump(&mut Cursor::new(&data), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("u16: 4660"));
}

#[test]
fn test_u32_little_endian() {
    let data = vec![0x78, 0x56, 0x34, 0x12]; // Little-endian 0x12345678
    let mut hexdump = Hexdump::new();

    hexdump.add_annotation(Annotation::new(
        0,
        4,
        format!(
            "u32: {}",
            DataType::U32.decode(&data, ByteOrder::Little).unwrap()
        ),
    ));

    let mut output = Vec::new();
    hexdump.dump(&mut Cursor::new(&data), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("u32: 305419896"));
}

#[test]
fn test_u32_big_endian() {
    let data = vec![0x12, 0x34, 0x56, 0x78]; // Big-endian 0x12345678
    let mut hexdump = Hexdump::new();

    hexdump.add_annotation(Annotation::new(
        0,
        4,
        format!(
            "u32: {}",
            DataType::U32.decode(&data, ByteOrder::Big).unwrap()
        ),
    ));

    let mut output = Vec::new();
    hexdump.dump(&mut Cursor::new(&data), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("u32: 305419896"));
}

#[test]
fn test_i8_negative() {
    let data = vec![255u8]; // -1 as i8
    let mut hexdump = Hexdump::new();

    hexdump.add_annotation(Annotation::new(
        0,
        1,
        format!("i8: {}", DataType::I8.decode(&data, ByteOrder::Little).unwrap()),
    ));

    let mut output = Vec::new();
    hexdump.dump(&mut Cursor::new(&data), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("i8: -1"));
}

#[test]
fn test_i16_negative() {
    let data = vec![0xFF, 0xFF]; // -1 as i16
    let mut hexdump = Hexdump::new();

    hexdump.add_annotation(Annotation::new(
        0,
        2,
        format!(
            "i16: {}",
            DataType::I16
                .decode(&data, ByteOrder::Little)
                .unwrap()
        ),
    ));

    let mut output = Vec::new();
    hexdump.dump(&mut Cursor::new(&data), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("i16: -1"));
}

#[test]
fn test_mixed_types_sequential() {
    // u8 (1 byte) + u16 (2 bytes) + u32 (4 bytes) = 7 bytes total
    let data = vec![0x2A, 0x34, 0x12, 0x78, 0x56, 0x34, 0x12];
    let mut hexdump = Hexdump::new();

    // u8 at offset 0
    hexdump.add_annotation(Annotation::new(
        0,
        1,
        format!("u8: {}", DataType::U8.decode(&data[0..1], ByteOrder::Little).unwrap()),
    ));

    // u16 at offset 1
    hexdump.add_annotation(Annotation::new(
        1,
        2,
        format!(
            "u16: {}",
            DataType::U16.decode(&data[1..3], ByteOrder::Little).unwrap()
        ),
    ));

    // u32 at offset 3
    hexdump.add_annotation(Annotation::new(
        3,
        4,
        format!(
            "u32: {}",
            DataType::U32.decode(&data[3..7], ByteOrder::Little).unwrap()
        ),
    ));

    let mut output = Vec::new();
    hexdump.dump(&mut Cursor::new(&data), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Output:\n{}", output_str);

    assert!(output_str.contains("u8: 42"));
    assert!(output_str.contains("u16: 4660")); // 0x1234 little-endian
    assert!(output_str.contains("u32: 305419896")); // 0x12345678 little-endian
}

#[test]
fn test_u64_little_endian() {
    let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    let mut hexdump = Hexdump::new();

    hexdump.add_annotation(Annotation::new(
        0,
        8,
        format!(
            "u64: {}",
            DataType::U64.decode(&data, ByteOrder::Little).unwrap()
        ),
    ));

    let mut output = Vec::new();
    hexdump.dump(&mut Cursor::new(&data), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    // Little-endian: 0x0807060504030201
    assert!(output_str.contains("u64: 578437695752307201"));
}

#[test]
fn test_f32_decode() {
    let value = 3.14159f32;
    let data = value.to_le_bytes().to_vec();
    let mut hexdump = Hexdump::new();

    hexdump.add_annotation(Annotation::new(
        0,
        4,
        format!(
            "f32: {}",
            DataType::F32.decode(&data, ByteOrder::Little).unwrap()
        ),
    ));

    let mut output = Vec::new();
    hexdump.dump(&mut Cursor::new(&data), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("f32: 3.14159"));
}

#[test]
fn test_type_spanning_line_boundary() {
    // Create data that has a u32 spanning the 16-byte line boundary
    let mut data = vec![0u8; 20];
    // Put a recognizable u32 at offset 14 (spans bytes 14, 15, 16, 17)
    data[14] = 0x78;
    data[15] = 0x56;
    data[16] = 0x34;
    data[17] = 0x12;

    let mut hexdump = Hexdump::new();

    hexdump.add_annotation(Annotation::new(
        14,
        4,
        format!(
            "u32: {}",
            DataType::U32
                .decode(&data[14..18], ByteOrder::Little)
                .unwrap()
        ),
    ));

    let mut output = Vec::new();
    hexdump.dump(&mut Cursor::new(&data), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Output:\n{}", output_str);

    // Should have continuation across line boundary
    assert!(output_str.contains("u32: 305419896"));

    let lines: Vec<&str> = output_str.lines().collect();
    // First line should have start of annotation (no closing)
    assert!(lines.iter().any(|l| l.contains("└───") && !l.contains("┘")));
}

#[test]
fn test_types_at_gap_position() {
    // Test u16 starting at position 7 (gap position)
    let mut data = vec![0u8; 10];
    data[7] = 0x34;
    data[8] = 0x12;

    let mut hexdump = Hexdump::new();

    hexdump.add_annotation(Annotation::new(
        7,
        2,
        format!(
            "u16: {}",
            DataType::U16.decode(&data[7..9], ByteOrder::Little).unwrap()
        ),
    ));

    let mut output = Vec::new();
    hexdump.dump(&mut Cursor::new(&data), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("u16: 4660"));
}

#[test]
fn test_all_integer_types() {
    // Test all integer types in sequence
    let data = vec![
        42u8,       // u8
        255u8,      // i8 (-1)
        0x34, 0x12, // u16 (little-endian 0x1234)
        0xFF, 0xFF, // i16 (-1)
        0x78, 0x56, 0x34, 0x12, // u32 (little-endian)
        0xFF, 0xFF, 0xFF, 0xFF, // i32 (-1)
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, // u64
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // i64 (-1)
    ];

    let mut hexdump = Hexdump::new();
    let mut offset = 0;

    let types = vec![
        DataType::U8,
        DataType::I8,
        DataType::U16,
        DataType::I16,
        DataType::U32,
        DataType::I32,
        DataType::U64,
        DataType::I64,
    ];

    for data_type in types {
        let size = data_type.size();
        let value = data_type
            .decode(&data[offset..offset + size], ByteOrder::Little)
            .unwrap();
        hexdump.add_annotation(Annotation::new(
            offset,
            size,
            format!("{}: {}", data_type.name(), value),
        ));
        offset += size;
    }

    let mut output = Vec::new();
    hexdump.dump(&mut Cursor::new(&data), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Output:\n{}", output_str);

    assert!(output_str.contains("u8: 42"));
    assert!(output_str.contains("i8: -1"));
    assert!(output_str.contains("u16: 4660"));
    assert!(output_str.contains("i16: -1"));
    assert!(output_str.contains("u32: 305419896"));
    assert!(output_str.contains("i32: -1"));
    assert!(output_str.contains("i64: -1"));
}
