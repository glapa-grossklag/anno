use std::io::Cursor;

#[path = "../src/main.rs"]
mod main_module;

use main_module::Hexdump;

#[test]
fn test_skip_8_bits() {
    // Test .8 skips 1 byte
    let data = vec![0xAA, 0xBB, 0xCC];
    let type_specs = vec![".8".to_string(), "u16".to_string()];
    let byte_order = main_module::ByteOrder::Little;

    let annotations = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();

    // Should have 1 annotation (u16), skip doesn't create annotation
    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].offset, 1); // Starts after skipped byte
    assert_eq!(annotations[0].length, 2);
    assert!(annotations[0].label.contains("52411")); // 0xBBCC in little endian = 0xCC*256 + 0xBB
}

#[test]
fn test_skip_16_bits() {
    // Test .16 skips 2 bytes
    let data = vec![0xAA, 0xBB, 0xCC, 0xDD];
    let type_specs = vec![".16".to_string(), "u16".to_string()];
    let byte_order = main_module::ByteOrder::Little;

    let annotations = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();

    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].offset, 2); // Starts after 2 skipped bytes
    assert_eq!(annotations[0].length, 2);
}

#[test]
fn test_skip_32_bits() {
    // Test .32 skips 4 bytes
    let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
    let type_specs = vec![".32".to_string(), "u16".to_string()];
    let byte_order = main_module::ByteOrder::Little;

    let annotations = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();

    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].offset, 4); // Starts after 4 skipped bytes
}

#[test]
fn test_skip_64_bits() {
    // Test .64 skips 8 bytes
    let data = vec![0x00; 10]; // 10 bytes of zeros
    let type_specs = vec![".64".to_string(), "u16".to_string()];
    let byte_order = main_module::ByteOrder::Little;

    let annotations = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();

    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].offset, 8); // Starts after 8 skipped bytes
}

#[test]
fn test_multiple_skips() {
    // Test multiple skip directives
    let data = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
    let type_specs = vec![".8".to_string(), ".16".to_string(), "u8".to_string()];
    let byte_order = main_module::ByteOrder::Little;

    let annotations = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();

    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].offset, 3); // After 1 + 2 skipped bytes
    assert_eq!(annotations[0].label, "u8: 4");
}

#[test]
fn test_skip_between_types() {
    // Test skip in the middle of type sequence
    let data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let type_specs = vec!["u8".to_string(), ".16".to_string(), "u16".to_string()];
    let byte_order = main_module::ByteOrder::Little;

    let annotations = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();

    assert_eq!(annotations.len(), 2);
    assert_eq!(annotations[0].offset, 0);
    assert_eq!(annotations[0].label, "u8: 1");
    assert_eq!(annotations[1].offset, 3); // After u8 (1 byte) + skip (2 bytes)
}

#[test]
fn test_skip_with_field_names() {
    // Test skip works with field names
    let data = vec![0x12, 0x34, 0x00, 0x00, 0x00, 0x00, 0x56, 0x78];
    let type_specs = vec![
        "u16:magic".to_string(),
        ".32".to_string(),
        "u16:data".to_string(),
    ];
    let byte_order = main_module::ByteOrder::Little;

    let annotations = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();

    assert_eq!(annotations.len(), 2);
    assert_eq!(annotations[0].label, "magic: 13330");
    assert_eq!(annotations[1].offset, 6);
    assert_eq!(annotations[1].label, "data: 30806");
}

#[test]
fn test_skip_insufficient_data() {
    // Test error when skip goes beyond data length
    let data = vec![0x01, 0x02];
    let type_specs = vec![".32".to_string()]; // Need 4 bytes but only have 2
    let byte_order = main_module::ByteOrder::Little;

    let result = main_module::build_annotations_from_types(&type_specs, byte_order, &data);

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Not enough data"));
    assert!(err_msg.contains("skip 4 bytes"));
}

#[test]
fn test_skip_invalid_syntax_not_number() {
    // Test error for invalid skip syntax
    let data = vec![0x01];
    let type_specs = vec![".abc".to_string()];
    let byte_order = main_module::ByteOrder::Little;

    let result = main_module::build_annotations_from_types(&type_specs, byte_order, &data);

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Invalid skip syntax"));
}

#[test]
fn test_skip_zero_bits() {
    // Test error for zero-sized skip
    let data = vec![0x01];
    let type_specs = vec![".0".to_string()];
    let byte_order = main_module::ByteOrder::Little;

    let result = main_module::build_annotations_from_types(&type_specs, byte_order, &data);

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Skip size cannot be 0"));
}

#[test]
fn test_skip_not_multiple_of_8() {
    // Test error for non-byte-aligned skip
    let data = vec![0x01];
    let type_specs = vec![".12".to_string()]; // 12 bits = 1.5 bytes
    let byte_order = main_module::ByteOrder::Little;

    let result = main_module::build_annotations_from_types(&type_specs, byte_order, &data);

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("must be a multiple of 8 bits"));
}

#[test]
fn test_skip_rendering() {
    // Test that skipped bytes don't appear in annotations
    let data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let type_specs = vec!["u8".to_string(), ".16".to_string(), "u16".to_string()];
    let byte_order = main_module::ByteOrder::Little;

    let annotations = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();

    let mut hexdump = Hexdump::new();
    for annotation in annotations {
        hexdump.add_annotation(annotation);
    }

    let mut output = Vec::new();
    hexdump.dump(&mut Cursor::new(&data), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();

    // Should see annotations for u8 and u16, but bytes 02 03 (skipped) have no annotation
    assert!(output_str.contains("u8: 1"));
    assert!(output_str.contains("u16: 1284")); // 0x0504 in little endian

    // Count annotation lines (lines with └ character)
    let annotation_count = output_str.lines().filter(|line| line.contains("└")).count();
    assert_eq!(annotation_count, 2, "Should have exactly 2 annotations (skipped section not annotated)");
}

#[test]
fn test_skip_large_value() {
    // Test skipping many bytes
    let data = vec![0xFF; 100]; // 100 bytes
    let type_specs = vec![".8".to_string(), ".800".to_string(), "u8".to_string()]; // Skip 1 + 100 bytes
    let byte_order = main_module::ByteOrder::Little;

    let result = main_module::build_annotations_from_types(&type_specs, byte_order, &data);

    // Should fail because we need 101 bytes but only have 100
    assert!(result.is_err());
}

#[test]
fn test_skip_at_start() {
    // Test skip at the beginning
    let data = vec![0xAA, 0xBB, 0xCC, 0xDD];
    let type_specs = vec![".16".to_string(), "u16".to_string()];
    let byte_order = main_module::ByteOrder::Big;

    let annotations = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();

    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].offset, 2);
    assert!(annotations[0].label.contains("52445")); // 0xCCDD in big endian = 0xCC*256 + 0xDD
}

#[test]
fn test_skip_at_end() {
    // Test skip at the end (should succeed if enough data)
    let data = vec![0x01, 0x02, 0x03, 0x04];
    let type_specs = vec!["u16".to_string(), ".16".to_string()];
    let byte_order = main_module::ByteOrder::Little;

    let annotations = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();

    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].offset, 0);
}

#[test]
fn test_complex_skip_pattern() {
    // Test complex pattern of types and skips
    let data = vec![
        0x01,             // u8
        0x02, 0x03,       // skip 16
        0x04, 0x05,       // u16
        0x06,             // skip 8
        0x07, 0x08,       // u16
    ];

    let type_specs = vec![
        "u8:a".to_string(),
        ".16".to_string(),
        "u16:b".to_string(),
        ".8".to_string(),
        "u16:c".to_string(),
    ];
    let byte_order = main_module::ByteOrder::Little;

    let annotations = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();

    assert_eq!(annotations.len(), 3);
    assert_eq!(annotations[0].offset, 0);
    assert_eq!(annotations[0].label, "a: 1");
    assert_eq!(annotations[1].offset, 3);
    assert_eq!(annotations[1].label, "b: 1284"); // 0x0504
    assert_eq!(annotations[2].offset, 6);
    assert_eq!(annotations[2].label, "c: 2055"); // 0x0807
}
