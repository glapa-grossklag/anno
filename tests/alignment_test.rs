use std::io::Cursor;

// Import from main.rs
#[path = "../src/main.rs"]
mod main_module;

use main_module::{Annotation, Hexdump};

#[test]
fn test_annotation_label_alignment() {
    let input = b"Hello, World! This is a test.";
    let mut hexdump = Hexdump::new();

    hexdump.add_annotation(Annotation::new(0, 5, "Hello"));
    hexdump.add_annotation(Annotation::new(7, 6, "World!"));
    hexdump.add_annotation(Annotation::new(14, 4, "This"));

    let mut output = Vec::new();
    hexdump.dump(&mut Cursor::new(input), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    println!("Output:\n{}", output_str);
    let lines: Vec<&str> = output_str.lines().collect();

    // Find annotation lines (lines with labels)
    let mut label_positions = Vec::new();

    for line in &lines {
        if line.contains("Hello") {
            // Count character position, not byte position
            let char_pos = line.chars().take_while(|&c| c != 'H' || !line[line.char_indices().position(|(_, ch)| ch == c).unwrap()..].starts_with("Hello")).count();
            let char_pos = line.char_indices()
                .find(|(_, _)| line[..].contains("Hello"))
                .map(|(idx, _)| line[..idx].chars().count())
                .unwrap_or(0);
            // Simpler: just count chars before "Hello"
            let char_pos = line.chars().position(|_| line.contains("Hello")).unwrap();
            let before_hello = line.split("Hello").next().unwrap();
            let char_pos = before_hello.chars().count();
            label_positions.push(("Hello", char_pos));
            println!("Hello at char column: {}", char_pos);
        }
        if line.contains("World!") {
            let before_world = line.split("World!").next().unwrap();
            let char_pos = before_world.chars().count();
            label_positions.push(("World!", char_pos));
            println!("World! at char column: {}", char_pos);
        }
        if line.contains("This") {
            let before_this = line.split("This").next().unwrap();
            let char_pos = before_this.chars().count();
            label_positions.push(("This", char_pos));
            println!("This at char column: {}", char_pos);
        }
    }

    // Check all labels are at the same column
    assert!(!label_positions.is_empty(), "No labels found in output");

    let first_pos = label_positions[0].1;
    for (label, pos) in &label_positions {
        assert_eq!(
            *pos, first_pos,
            "Label '{}' at column {} doesn't match first label at column {}",
            label, pos, first_pos
        );
    }

    println!("\n✓ All labels aligned at column {}", first_pos);
}
