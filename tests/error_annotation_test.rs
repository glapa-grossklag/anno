use std::io::Cursor;

#[path = "../src/main.rs"]
mod main_module;

use main_module::{Annotation, AnnotationKind, Hexdump};

#[test]
fn test_error_annotation_insufficient_data_u32() {
    // Test that we get an error annotation when we don't have enough data for u32
    let data = vec![0x01, 0x02, 0x03]; // Only 3 bytes, u32 needs 4
    let type_specs = vec!["u32".to_string()];
    let byte_order = main_module::ByteOrder::Little;

    let (annotations, error) = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();

    // Should have 1 error annotation
    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].kind, AnnotationKind::Error);
    assert_eq!(annotations[0].offset, 0);
    assert_eq!(annotations[0].length, 3); // All 3 bytes we have
    assert!(annotations[0].label.contains("expected 4 bytes"));
    assert!(annotations[0].label.contains("only 3 available"));

    // Should also return an error
    assert!(error.is_some());
    let error_msg = error.unwrap().to_string();
    assert!(error_msg.contains("Not enough data"));
}

#[test]
fn test_error_annotation_partial_data_after_successful() {
    // Test that we successfully decode u8, then get error annotation for u32
    let data = vec![0x01, 0x02, 0x03]; // 1 byte for u8, only 2 bytes left for u32
    let type_specs = vec!["u8".to_string(), "u32".to_string()];
    let byte_order = main_module::ByteOrder::Little;

    let (annotations, error) = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();

    // Should have 2 annotations: 1 normal, 1 error
    assert_eq!(annotations.len(), 2);

    // First annotation: successful u8
    assert_eq!(annotations[0].kind, AnnotationKind::Normal);
    assert_eq!(annotations[0].offset, 0);
    assert_eq!(annotations[0].length, 1);
    assert_eq!(annotations[0].label, "u8: 1");

    // Second annotation: error for u32
    assert_eq!(annotations[1].kind, AnnotationKind::Error);
    assert_eq!(annotations[1].offset, 1);
    assert_eq!(annotations[1].length, 2); // Only 2 bytes available
    assert!(annotations[1].label.contains("expected 4 bytes"));
    assert!(annotations[1].label.contains("only 2 available"));

    // Should return an error
    assert!(error.is_some());
}

#[test]
fn test_error_annotation_with_field_name() {
    // Test error annotation shows field name
    let data = vec![0x01, 0x02]; // Only 2 bytes, u32 needs 4
    let type_specs = vec!["u32:timestamp".to_string()];
    let byte_order = main_module::ByteOrder::Little;

    let (annotations, error) = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();

    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].kind, AnnotationKind::Error);
    assert!(annotations[0].label.contains("timestamp:"));
    assert!(annotations[0].label.contains("expected 4 bytes"));
    assert!(error.is_some());
}

#[test]
fn test_error_annotation_zero_bytes_available() {
    // Test when we have no bytes left
    let data = vec![0x01]; // 1 byte for u8, 0 bytes left for u32
    let type_specs = vec!["u8".to_string(), "u32".to_string()];
    let byte_order = main_module::ByteOrder::Little;

    let (annotations, error) = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();

    assert_eq!(annotations.len(), 2);

    // Second annotation should be an error with 0 length
    assert_eq!(annotations[1].kind, AnnotationKind::Error);
    assert_eq!(annotations[1].offset, 1);
    assert_eq!(annotations[1].length, 0);
    assert!(annotations[1].label.contains("expected 4 bytes"));
    assert!(annotations[1].label.contains("only 0 available"));

    assert!(error.is_some());
}

#[test]
fn test_error_annotation_rendering() {
    // Test that error annotations actually render in the output
    let data = vec![0x01, 0x02, 0x03];
    let type_specs = vec!["u8".to_string(), "u32".to_string()];
    let byte_order = main_module::ByteOrder::Little;

    let (annotations, _) = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();

    let mut hexdump = Hexdump::new();
    for annotation in annotations {
        hexdump.add_annotation(annotation);
    }

    let mut output = Vec::new();
    hexdump.dump(&mut Cursor::new(&data), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();

    // Should show the successful u8 value
    assert!(output_str.contains("1"));

    // Should show the error message about insufficient data
    assert!(output_str.contains("expected 4 bytes"));
    assert!(output_str.contains("only 2 available"));
}

#[test]
fn test_error_annotation_u16_insufficient() {
    // Test with u16 needing 2 bytes but only 1 available
    let data = vec![0x01]; // Only 1 byte
    let type_specs = vec!["u16".to_string()];
    let byte_order = main_module::ByteOrder::Little;

    let (annotations, error) = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();

    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].kind, AnnotationKind::Error);
    assert_eq!(annotations[0].offset, 0);
    assert_eq!(annotations[0].length, 1);
    assert!(annotations[0].label.contains("expected 2 bytes"));
    assert!(annotations[0].label.contains("only 1 available"));
    assert!(error.is_some());
}

#[test]
fn test_error_annotation_u64_partial() {
    // Test with u64 needing 8 bytes but only 5 available
    let data = vec![0x01, 0x02, 0x03, 0x04, 0x05]; // Only 5 bytes
    let type_specs = vec!["u64".to_string()];
    let byte_order = main_module::ByteOrder::Little;

    let (annotations, error) = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();

    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].kind, AnnotationKind::Error);
    assert_eq!(annotations[0].offset, 0);
    assert_eq!(annotations[0].length, 5);
    assert!(annotations[0].label.contains("expected 8 bytes"));
    assert!(annotations[0].label.contains("only 5 available"));
    assert!(error.is_some());
}

#[test]
fn test_error_annotation_multiple_successful_then_error() {
    // Test several successful decodings followed by an error
    let data = vec![
        0x01,                   // u8 = 1
        0x02, 0x03,             // u16 = 770
        0x04, 0x05,             // u16 (incomplete, only 2 bytes but this is complete)
        0x06,                   // u32 (incomplete - only 1 byte)
    ];

    let type_specs = vec![
        "u8".to_string(),
        "u16".to_string(),
        "u16".to_string(),
        "u32".to_string(),
    ];
    let byte_order = main_module::ByteOrder::Little;

    let (annotations, error) = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();

    // Should have 3 normal + 1 error
    assert_eq!(annotations.len(), 4);
    assert_eq!(annotations[0].kind, AnnotationKind::Normal);
    assert_eq!(annotations[1].kind, AnnotationKind::Normal);
    assert_eq!(annotations[2].kind, AnnotationKind::Normal);
    assert_eq!(annotations[3].kind, AnnotationKind::Error);

    assert_eq!(annotations[3].offset, 5);
    assert_eq!(annotations[3].length, 1);
    assert!(annotations[3].label.contains("expected 4 bytes"));
    assert!(annotations[3].label.contains("only 1 available"));

    assert!(error.is_some());
}

#[test]
fn test_successful_decode_no_error() {
    // Verify that successful decoding doesn't produce error annotations
    let data = vec![0x01, 0x02, 0x03, 0x04];
    let type_specs = vec!["u32".to_string()];
    let byte_order = main_module::ByteOrder::Little;

    let (annotations, error) = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();

    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].kind, AnnotationKind::Normal);
    assert!(error.is_none());
}

#[test]
fn test_error_annotation_big_endian() {
    // Test that error annotations work with big endian too
    let data = vec![0x01, 0x02]; // Only 2 bytes, u32 needs 4
    let type_specs = vec!["u32".to_string()];
    let byte_order = main_module::ByteOrder::Big;

    let (annotations, error) = main_module::build_annotations_from_types(&type_specs, byte_order, &data).unwrap();

    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].kind, AnnotationKind::Error);
    assert!(error.is_some());
}
