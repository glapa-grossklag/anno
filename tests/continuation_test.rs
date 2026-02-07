use std::io::Cursor;

#[path = "../src/main.rs"]
mod main_module;

use main_module::{Annotation, Hexdump};

#[test]
fn test_multiline_annotation_continuation() {
    let input = b"Hello, World! This is a test.";
    let mut hexdump = Hexdump::new();

    // Annotation that spans two lines (bytes 14-17, where line break is at 16)
    hexdump.add_annotation(Annotation::new(14, 4, "This"));

    let mut output = Vec::new();
    hexdump.dump(&mut Cursor::new(input), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Output:\n{}", output_str);
    let lines: Vec<&str> = output_str.lines().collect();

    // Line 2: First part of "This" annotation - should have opening corner and no closing
    // Should end with "─────" (continuation, no closing corner)
    assert!(
        lines[1].contains("└─────"),
        "First line of multi-line annotation should have opening corner and continuation (no closing corner)\nGot: {}",
        lines[1]
    );

    // Should have the label on first line only
    assert!(
        lines[1].contains("This"),
        "First line should have the label\nGot: {}",
        lines[1]
    );

    // Line 4: Continuation of "This" annotation - should have no opening corner but have closing
    // Should start with "──────┘" (continuation from previous line with closing)
    assert!(
        lines[3].contains("──────┘"),
        "Continuation line should have no opening corner but have closing corner\nGot: {}",
        lines[3]
    );

    // Should NOT have the label on continuation line
    assert!(
        !lines[3].contains("This"),
        "Continuation line should not repeat the label\nGot: {}",
        lines[3]
    );
}

#[test]
fn test_closing_corner_position() {
    let input = b"Hello, World!";
    let mut hexdump = Hexdump::new();

    hexdump.add_annotation(Annotation::new(0, 5, "Hello"));
    hexdump.add_annotation(Annotation::new(7, 5, "World"));

    let mut output = Vec::new();
    hexdump.dump(&mut Cursor::new(input), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Output:\n{}", output_str);
    let lines: Vec<&str> = output_str.lines().collect();

    // Check that closing corners appear with proper spacing
    // The pattern should be: └── (for each byte) ... ──┘ (closing corner in space after last byte)

    // Count the underline for "Hello" - should be └ + 14 dashes + ┘
    let hello_line = lines[1];
    assert!(
        hello_line.contains("└──────────────┘"),
        "Hello annotation should have opening corner, proper length, and closing corner in space after last byte\nGot: {}",
        hello_line
    );

    // Count the underline for "World" - should be same pattern as Hello (both are 5 bytes)
    let world_line = lines[2];
    assert!(
        world_line.contains("└──────────────┘"),
        "World annotation should have opening corner, proper length, and closing corner in space after last byte\nGot: {}",
        world_line
    );
}

#[test]
fn test_single_byte_annotation() {
    let input = b"ABCDEF";
    let mut hexdump = Hexdump::new();

    hexdump.add_annotation(Annotation::new(2, 1, "C"));

    let mut output = Vec::new();
    hexdump.dump(&mut Cursor::new(input), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Output:\n{}", output_str);
    let lines: Vec<&str> = output_str.lines().collect();

    // Single byte annotation should be └──┘ (3 chars for the byte + 1 char for closing in space)
    assert!(
        lines[1].contains("└──┘"),
        "Single byte annotation should have └──┘ pattern\nGot: {}",
        lines[1]
    );
}
