use anyhow::Result;
use argh::FromArgs;
use std::env;
use std::fs::File;
use std::io::{self, IsTerminal, Read, Write};
use std::path::PathBuf;

#[derive(FromArgs)]
/// A simple hexdump utility
struct Args {
    /// file to read (reads from stdin if not provided)
    #[argh(positional)]
    file: Option<PathBuf>,
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
                } else {
                    // Ends on next position
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

        // Handle closing corner if annotation ends exactly at end of line (position 16)
        if end_in_line == 16 && !continues_to_next {
            underline.push('┘');
        }

        // Count display width (not bytes)
        let display_width: usize = underline.chars().count();

        // Pad to align labels at column 58
        const LABEL_COLUMN: usize = 58;
        write!(writer, "{}", underline)?;
        let padding = if display_width < LABEL_COLUMN {
            LABEL_COLUMN - display_width
        } else {
            0
        };
        for _ in 0..padding {
            write!(writer, " ")?;
        }

        // Only show label on the first line of the annotation
        if ann_start >= line_offset && ann_start < line_end {
            writeln!(writer, " {}", self.color_annotation(&annotation.label))?;
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

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    let mut reader: Box<dyn Read> = match args.file {
        Some(path) => Box::new(File::open(path)?),
        None => Box::new(io::stdin()),
    };

    let mut hexdump = Hexdump::new();

    // Example annotations
    hexdump.add_annotation(Annotation::new(0, 5, "Hello"));
    hexdump.add_annotation(Annotation::new(5, 2, "Comma+Space"));
    hexdump.add_annotation(Annotation::new(7, 6, "World!"));
    hexdump.add_annotation(Annotation::new(14, 4, "This"));

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    hexdump.dump(&mut reader, &mut handle)?;

    Ok(())
}
