mod types;

use anyhow::Result;
use argh::FromArgs;
use std::env;
use std::fs::File;
use std::io::{self, IsTerminal, Read, Write};
use std::path::PathBuf;

pub use types::{ByteOrder, DataType};

#[derive(FromArgs)]
/// A simple hexdump utility with type annotations
struct Args {
    /// data types to annotate (e.g., u8 u16 u32)
    #[argh(positional)]
    types: Vec<String>,

    /// file to read (reads from stdin if not provided)
    #[argh(option, short = 'f')]
    file: Option<PathBuf>,

    /// byte order for multi-byte types: little (default) or big
    #[argh(option, default = "String::from(\"little\")")]
    byte_order: String,
}

/// Represents an annotation for a range of bytes
#[derive(Debug, Clone)]
pub struct Annotation {
    /// Starting byte offset
    pub offset: usize,
    /// Number of bytes to annotate
    pub length: usize,
    /// Label for this annotation
    pub label: String,
}

impl Annotation {
    pub fn new(offset: usize, length: usize, label: impl Into<String>) -> Self {
        Self {
            offset,
            length,
            label: label.into(),
        }
    }
}

pub struct Hexdump {
    annotations: Vec<Annotation>,
    use_color: bool,
}

// ANSI color codes
const GREEN: &str = "\x1b[32m";
const BLUE: &str = "\x1b[34m";
const PURPLE: &str = "\x1b[35m";
const RESET: &str = "\x1b[0m";

impl Hexdump {
    pub fn new() -> Self {
        let use_color = should_use_color();
        Self {
            annotations: Vec::new(),
            use_color,
        }
    }

    fn color_addr(&self, text: &str) -> String {
        if self.use_color {
            format!("{}{}{}", GREEN, text, RESET)
        } else {
            text.to_string()
        }
    }

    fn color_annotation(&self, text: &str) -> String {
        if self.use_color {
            format!("{}{}{}", BLUE, text, RESET)
        } else {
            text.to_string()
        }
    }

    fn color_label(&self, label: &str) -> String {
        if !self.use_color {
            return label.to_string();
        }

        // Format is "type: value" - color type purple, colon uncolored, value blue
        if let Some(colon_pos) = label.find(": ") {
            let type_part = &label[..colon_pos];
            let value_part = &label[colon_pos + 2..];
            format!("{}{}{}: {}{}{}", PURPLE, type_part, RESET, BLUE, value_part, RESET)
        } else {
            // Fallback: just color it blue if format doesn't match
            format!("{}{}{}", BLUE, label, RESET)
        }
    }

    fn is_byte_annotated(&self, offset: usize) -> bool {
        self.annotations.iter().any(|a| {
            let ann_end = a.offset + a.length;
            offset >= a.offset && offset < ann_end
        })
    }

    #[allow(dead_code)]
    pub fn add_annotation(&mut self, annotation: Annotation) {
        self.annotations.push(annotation);
    }

    pub fn dump<R: Read, W: Write>(&self, reader: &mut R, writer: &mut W) -> Result<()> {
        let mut offset = 0;
        let mut buffer = [0u8; 16];

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            // Print offset
            write!(writer, "{}  ", self.color_addr(&format!("{:08x}", offset)))?;

            // Print hex bytes
            for i in 0..16 {
                if i < bytes_read {
                    let byte_offset = offset + i;
                    let hex_str = format!("{:02x}", buffer[i]);
                    if self.is_byte_annotated(byte_offset) {
                        write!(writer, "{} ", self.color_annotation(&hex_str))?;
                    } else {
                        write!(writer, "{} ", hex_str)?;
                    }
                } else {
                    write!(writer, "   ")?;
                }
                if i == 7 {
                    write!(writer, " ")?;
                }
            }

            writeln!(writer)?;

            // Print annotations for this line
            let line_end = offset + bytes_read;
            let mut line_annotations: Vec<_> = self
                .annotations
                .iter()
                .filter(|a| {
                    let ann_end = a.offset + a.length;
                    // Annotation overlaps with this line
                    a.offset < line_end && ann_end > offset
                })
                .collect();

            // Sort by offset for consistent rendering
            line_annotations.sort_by_key(|a| a.offset);

            for annotation in line_annotations {
                self.print_annotation(writer, offset, bytes_read, annotation)?;
            }

            offset += bytes_read;
        }

        writeln!(writer, "{}", self.color_addr(&format!("{:08x}", offset)))?;
        Ok(())
    }

    fn print_annotation<W: Write>(
        &self,
        writer: &mut W,
        line_offset: usize,
        line_length: usize,
        annotation: &Annotation,
    ) -> Result<()> {
        let ann_start = annotation.offset;
        let ann_end = ann_start + annotation.length;
        let line_end = line_offset + line_length;

        // Calculate which bytes in this line are annotated
        let start_in_line = if ann_start > line_offset {
            ann_start - line_offset
        } else {
            0
        };
        let end_in_line = if ann_end < line_end {
            ann_end - line_offset
        } else {
            line_length
        };

        // Build the underline string first
        let mut underline = String::from("         "); // Offset spacing (9 spaces to extend left by 1)
        let mut in_annotation = false;

        // Check if annotation continues from previous line or to next line
        let continues_from_prev = ann_start < line_offset;
        let continues_to_next = ann_end > line_end;

        // Track if we just started the annotation on this iteration
        let mut just_started = false;

        for i in 0..16 {
            if i == start_in_line {
                // Start of annotation on this line
                in_annotation = true;
                just_started = true;
                if i == end_in_line - 1 {
                    // Single byte annotation (starts and ends here)
                    if continues_from_prev {
                        // Continuation from previous line, ending here
                        underline.push_str("───");
                    } else if continues_to_next {
                        // Single byte continuing (shouldn't happen but handle it)
                        underline.push_str("└──");
                    } else {
                        // Complete single byte annotation
                        underline.push_str("└──");
                    }
                } else {
                    // First byte of multi-byte annotation on this line
                    if continues_from_prev {
                        // Continuation from previous line
                        underline.push_str("───");
                    } else {
                        // Start of annotation
                        underline.push_str("└──");
                    }
                }
            } else if i == end_in_line {
                // Position after last annotated byte - put closing corner here if not continuing
                if !continues_to_next {
                    underline.push_str("┘ ");
                } else {
                    underline.push_str("  ");
                }
                in_annotation = false;
            } else if i == end_in_line - 1 && in_annotation {
                // Last byte of annotation on this line (not the start)
                if continues_to_next {
                    // Continues to next line, no closing corner
                    underline.push_str("───");
                } else if end_in_line == 16 {
                    // Ends at position 16 - closing corner will be added after loop
                    // Only add 2 chars here instead of 3
                    underline.push_str("──");
                } else {
                    // Ends on next position (inside this line)
                    underline.push_str("───");
                }
            } else if in_annotation {
                // Middle of annotation
                underline.push_str("───");
            } else {
                // Not in annotation
                underline.push_str("   ");
            }

            if i == 7 && !just_started {
                // Add extra spacing at byte 7 for the gap in hex output
                // Skip this if we just started at position 7 (gap is implicit in the opening)
                if in_annotation {
                    // In annotation - continue the line
                    underline.push('─');
                } else {
                    // Not in annotation - use space
                    underline.push(' ');
                }
            }

            just_started = false;
        }

        // Check if we need to add closing corner at position 16
        let has_closing_at_16 = end_in_line == 16 && !continues_to_next;

        // Count display width (not bytes)
        let display_width: usize = underline.chars().count();

        // Pad to align labels at column 58 (or 57 if we have closing corner at 16)
        const LABEL_COLUMN: usize = 58;
        let target_column = if has_closing_at_16 {
            LABEL_COLUMN - 1 // Aim for 57 so that after adding ┘ we're at 58
        } else {
            LABEL_COLUMN
        };

        write!(writer, "{}", underline)?;

        // Add closing corner if needed (at position 16)
        if has_closing_at_16 {
            write!(writer, "┘")?;
        }

        // Calculate padding
        let padding = if display_width < target_column {
            target_column - display_width
        } else {
            0
        };
        for _ in 0..padding {
            write!(writer, " ")?;
        }

        // Only show label on the first line of the annotation
        if ann_start >= line_offset && ann_start < line_end {
            writeln!(writer, " {}", self.color_label(&annotation.label))?;
        } else {
            writeln!(writer)?;
        }

        Ok(())
    }
}

fn should_use_color() -> bool {
    // Check NO_COLOR environment variable
    if env::var("NO_COLOR").is_ok() {
        return false;
    }

    // Check if TERM is dumb
    if let Ok(term) = env::var("TERM") {
        if term == "dumb" {
            return false;
        }
    }

    // Check if stdout is a terminal
    io::stdout().is_terminal()
}

/// Represents a type specification with optional field name
struct TypeSpec {
    data_type: DataType,
    field_name: Option<String>,
}

impl TypeSpec {
    /// Parse a type specification string (e.g., "u16" or "u16:apid")
    fn from_str(s: &str) -> Result<Self> {
        if let Some(colon_pos) = s.find(':') {
            // Format: "type:fieldname"
            let type_part = &s[..colon_pos];
            let field_part = &s[colon_pos + 1..];

            if field_part.is_empty() {
                return Err(anyhow::anyhow!("Field name cannot be empty in '{}'", s));
            }

            let data_type = DataType::from_str(type_part)?;
            Ok(TypeSpec {
                data_type,
                field_name: Some(field_part.to_string()),
            })
        } else {
            // Format: "type"
            let data_type = DataType::from_str(s)?;
            Ok(TypeSpec {
                data_type,
                field_name: None,
            })
        }
    }

    /// Get the display name (field name if provided, otherwise type name)
    fn display_name(&self) -> &str {
        self.field_name.as_deref().unwrap_or_else(|| self.data_type.name())
    }
}

/// Build annotations from type specifications
pub fn build_annotations_from_types(
    type_specs: &[String],
    byte_order: ByteOrder,
    data: &[u8],
) -> Result<Vec<Annotation>> {
    let mut annotations = Vec::new();
    let mut offset = 0;

    for type_spec_str in type_specs {
        let type_spec = TypeSpec::from_str(type_spec_str)?;
        let size = type_spec.data_type.size();

        // Check if we have enough data
        if offset + size > data.len() {
            return Err(anyhow::anyhow!(
                "Not enough data: type {} at offset {} needs {} bytes, but only {} bytes available",
                type_spec.data_type.name(),
                offset,
                size,
                data.len() - offset
            ));
        }

        // Decode the value
        let value = type_spec.data_type.decode(&data[offset..offset + size], byte_order)?;

        // Create label: "name: value" (using field name if provided, otherwise type name)
        let label = format!("{}: {}", type_spec.display_name(), value);

        annotations.push(Annotation::new(offset, size, label));
        offset += size;
    }

    Ok(annotations)
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    // Read all data into memory (needed for type-based annotation)
    let mut data = Vec::new();
    let mut reader: Box<dyn Read> = match args.file {
        Some(path) => Box::new(File::open(path)?),
        None => Box::new(io::stdin()),
    };
    reader.read_to_end(&mut data)?;

    let mut hexdump = Hexdump::new();

    // If types are specified, build annotations from them
    if !args.types.is_empty() {
        let byte_order = ByteOrder::from_str(&args.byte_order)?;
        let annotations = build_annotations_from_types(&args.types, byte_order, &data)?;
        for annotation in annotations {
            hexdump.add_annotation(annotation);
        }
    }
    // If no types specified, just show plain hexdump without annotations

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    // Dump from the in-memory data
    use std::io::Cursor;
    let mut cursor = Cursor::new(&data);
    hexdump.dump(&mut cursor, &mut handle)?;

    Ok(())
}
