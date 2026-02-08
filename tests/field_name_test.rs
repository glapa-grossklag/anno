use std::io::Cursor;

#[path = "../src/main.rs"]
mod main_module;

use main_module::Hexdump;

#[test]
fn test_field_name_single() {
    // Test single type with field name
    let data = vec![0x34, 0x12]; // u16 = 4660
    let mut hexdump = Hexdump::new();

    // Manually test via main would use: anno u16:apid
    // For now we test via the internal function
    let type_specs = vec!["u16:apid".to_string()];
    let byte_order = main_module::ByteOrder::Little;
    let (annotations, error) = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();
    assert!(error.is_none());

    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].label, "apid: 4660");
}

#[test]
fn test_field_name_mixed() {
    // Test mix of field names and plain types
    let data = vec![
        0x34, 0x12, // u16:apid = 4660
        0x78, 0x56, 0x34, 0x12, // u32 (no field name) = 305419896
        0x2A, // u8:x = 42
    ];

    let type_specs = vec![
        "u16:apid".to_string(),
        "u32".to_string(),
        "u8:x".to_string(),
    ];
    let byte_order = main_module::ByteOrder::Little;
    let (annotations, error) = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();
    assert!(error.is_none());

    assert_eq!(annotations.len(), 3);
    assert_eq!(annotations[0].label, "apid: 4660");
    assert_eq!(annotations[1].label, "u32: 305419896");
    assert_eq!(annotations[2].label, "x: 42");
}

#[test]
fn test_field_name_all_named() {
    // Test all types with field names
    let data = vec![
        0x01, 0x00, // u16:version = 1
        0x64, 0x00, 0x00, 0x00, // u32:count = 100
        0xFF, // u8:flags = 255
    ];

    let type_specs = vec![
        "u16:version".to_string(),
        "u32:count".to_string(),
        "u8:flags".to_string(),
    ];
    let byte_order = main_module::ByteOrder::Little;
    let (annotations, error) = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();
    assert!(error.is_none());

    assert_eq!(annotations.len(), 3);
    assert_eq!(annotations[0].label, "version: 1");
    assert_eq!(annotations[1].label, "count: 100");
    assert_eq!(annotations[2].label, "flags: 255");
}

#[test]
fn test_field_name_output() {
    // Test that field names appear in actual output
    let data = vec![0x34, 0x12, 0x2A];
    let mut hexdump = Hexdump::new();

    let type_specs = vec!["u16:apid".to_string(), "u8:x".to_string()];
    let byte_order = main_module::ByteOrder::Little;
    let (annotations, error) = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();
    assert!(error.is_none());

    for annotation in annotations {
        hexdump.add_annotation(annotation);
    }

    let mut output = Vec::new();
    hexdump.dump(&mut Cursor::new(&data), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("apid: 4660"));
    assert!(output_str.contains("x: 42"));
    // Should NOT contain the type names since field names are provided
    assert!(!output_str.contains("u16: "));
    assert!(!output_str.contains("u8: "));
}

#[test]
fn test_field_name_empty_error() {
    // Test that empty field name returns error
    let data = vec![0x34, 0x12];
    let type_specs = vec!["u16:".to_string()]; // Empty field name
    let byte_order = main_module::ByteOrder::Little;
    let result = main_module::build_annotations_from_types(&type_specs, byte_order, &data);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Field name cannot be empty"));
}

#[test]
fn test_field_name_invalid_type() {
    // Test that invalid type with field name returns error
    let data = vec![0x34, 0x12];
    let type_specs = vec!["invalid:field".to_string()];
    let byte_order = main_module::ByteOrder::Little;
    let result = main_module::build_annotations_from_types(&type_specs, byte_order, &data);

    assert!(result.is_err());
}

#[test]
fn test_field_name_with_underscores() {
    // Test field names with underscores
    let data = vec![0x34, 0x12];
    let type_specs = vec!["u16:packet_id".to_string()];
    let byte_order = main_module::ByteOrder::Little;
    let (annotations, error) = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();
    assert!(error.is_none());

    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].label, "packet_id: 4660");
}

#[test]
fn test_field_name_alignment() {
    // Test that field names still align properly
    let data = vec![
        0x34, 0x12, // u16:a = 4660
        0x78, 0x56, 0x34, 0x12, // u32:very_long_field_name = 305419896
        0x2A, // u8:x = 42
    ];

    let type_specs = vec![
        "u16:a".to_string(),
        "u32:very_long_field_name".to_string(),
        "u8:x".to_string(),
    ];
    let byte_order = main_module::ByteOrder::Little;
    let (annotations, error) = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();
    assert!(error.is_none());

    let mut hexdump = Hexdump::new();
    for annotation in annotations {
        hexdump.add_annotation(annotation);
    }

    let mut output = Vec::new();
    hexdump.dump(&mut Cursor::new(&data), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    let lines: Vec<&str> = output_str.lines().collect();

    // Find label positions - look for the field name at the start of labels
    let mut label_positions = Vec::new();
    for line in &lines {
        // Look for our specific field names
        if let Some(pos) = line.find("a: 4660") {
            label_positions.push(line[..pos].chars().count());
        } else if let Some(pos) = line.find("very_long_field_name: 305419896") {
            label_positions.push(line[..pos].chars().count());
        } else if let Some(pos) = line.find("x: 42") {
            label_positions.push(line[..pos].chars().count());
        }
    }

    // All labels should start at the same position
    assert_eq!(label_positions.len(), 3, "Should find all three labels");
    let first_pos = label_positions[0];
    for (i, pos) in label_positions.iter().enumerate() {
        assert_eq!(*pos, first_pos, "Label {} at position {} doesn't match first label at {}", i, pos, first_pos);
    }
}
