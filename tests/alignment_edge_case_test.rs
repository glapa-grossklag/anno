use std::io::Cursor;

#[path = "../src/main.rs"]
mod main_module;

use main_module::{Annotation, ByteOrder, DataType, Hexdump};

#[test]
fn test_alignment_annotation_ending_at_position_16() {
    // This is the exact case from the user: u16 u32 u32 u32 u16
    // The last u16 ends at position 16 (bytes 14-15)
    let data = vec![
        0x00, 0x01, // u16 at 0-1
        0x61, 0x61, 0x61, 0x61, // u32 at 2-5
        0x61, 0x61, 0x61, 0x61, // u32 at 6-9
        0x61, 0x61, 0x61, 0x61, // u32 at 10-13
        0x61, 0x61, // u16 at 14-15 (ends at position 16!)
    ];

    let mut hexdump = Hexdump::new();
    let mut offset = 0;

    // u16 at 0
    hexdump.add_annotation(Annotation::new(
        offset,
        2,
        format!(
            "u16 = {}",
            DataType::U16
                .decode(&data[offset..offset + 2], ByteOrder::Little)
                .unwrap()
        ),
    ));
    offset += 2;

    // u32 at 2
    hexdump.add_annotation(Annotation::new(
        offset,
        4,
        format!(
            "u32 = {}",
            DataType::U32
                .decode(&data[offset..offset + 4], ByteOrder::Little)
                .unwrap()
        ),
    ));
    offset += 4;

    // u32 at 6
    hexdump.add_annotation(Annotation::new(
        offset,
        4,
        format!(
            "u32 = {}",
            DataType::U32
                .decode(&data[offset..offset + 4], ByteOrder::Little)
                .unwrap()
        ),
    ));
    offset += 4;

    // u32 at 10
    hexdump.add_annotation(Annotation::new(
        offset,
        4,
        format!(
            "u32 = {}",
            DataType::U32
                .decode(&data[offset..offset + 4], ByteOrder::Little)
                .unwrap()
        ),
    ));
    offset += 4;

    // u16 at 14 (THIS ONE ENDS AT POSITION 16!)
    hexdump.add_annotation(Annotation::new(
        offset,
        2,
        format!(
            "u16 = {}",
            DataType::U16
                .decode(&data[offset..offset + 2], ByteOrder::Little)
                .unwrap()
        ),
    ));

    let mut output = Vec::new();
    hexdump.dump(&mut Cursor::new(&data), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Output:\n{}", output_str);

    let lines: Vec<&str> = output_str.lines().collect();

    // Find all label positions
    let mut label_positions = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if line.contains("u16 = ") || line.contains("u32 = ") {
            // Find the position where the label starts
            let label_start = if let Some(pos) = line.find("u16 = ") {
                pos
            } else if let Some(pos) = line.find("u32 = ") {
                pos
            } else {
                continue;
            };

            // Count character position (not byte position)
            let char_pos = line[..label_start].chars().count();
            label_positions.push((i, char_pos));
            println!("Line {}: label at char column {}", i, char_pos);
        }
    }

    // All labels should be at the same character position
    assert!(
        !label_positions.is_empty(),
        "No labels found in output"
    );

    let first_pos = label_positions[0].1;
    for (line_num, pos) in &label_positions {
        assert_eq!(
            *pos, first_pos,
            "Label at line {} at column {} doesn't match first label at column {}",
            line_num, pos, first_pos
        );
    }

    println!("\n✓ All {} labels aligned at column {}", label_positions.len(), first_pos);
}
