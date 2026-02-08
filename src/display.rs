use anyhow::Result;
use std::io::{Read, Write};

use super::color::ColorScheme;

/// Annotation kind - determines color and rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotationKind {
    Normal,
    Error,
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
    /// Annotation kind (normal or error)
    pub kind: AnnotationKind,
}

impl Annotation {
    pub fn new(offset: usize, length: usize, label: impl Into<String>) -> Self {
        Self {
            offset,
            length,
            label: label.into(),
            kind: AnnotationKind::Normal,
        }
    }

    pub fn error(offset: usize, length: usize, label: impl Into<String>) -> Self {
        Self {
            offset,
            length,
            label: label.into(),
            kind: AnnotationKind::Error,
        }
    }
}

pub struct Hexdump {
    annotations: Vec<Annotation>,
    colors: ColorScheme,
}

impl Hexdump {
    pub fn new() -> Self {
        Self {
            annotations: Vec::new(),
            colors: ColorScheme::new(),
        }
    }

    #[allow(dead_code)]
    pub fn add_annotation(&mut self, annotation: Annotation) {
        self.annotations.push(annotation);
    }

    fn get_byte_annotation_kind(&self, offset: usize) -> Option<AnnotationKind> {
        self.annotations
            .iter()
            .find(|a| {
                let ann_end = a.offset + a.length;
                offset >= a.offset && offset < ann_end
            })
            .map(|a| a.kind)
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
            write!(writer, "{}  ", self.colors.addr(&format!("{:08x}", offset)))?;

            // Print hex bytes
            for i in 0..16 {
                if i < bytes_read {
                    let byte_offset = offset + i;
                    let hex_str = format!("{:02x}", buffer[i]);
                    match self.get_byte_annotation_kind(byte_offset) {
                        Some(AnnotationKind::Normal) => {
                            write!(writer, "{} ", self.colors.annotation(&hex_str))?;
                        }
                        Some(AnnotationKind::Error) => {
                            write!(writer, "{} ", self.colors.error(&hex_str))?;
                        }
                        None => {
                            write!(writer, "{} ", hex_str)?;
                        }
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

        writeln!(writer, "{}", self.colors.addr(&format!("{:08x}", offset)))?;
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
                    // Ends on this line - add full width
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

        // For position 16 annotations, we need to add the "┘" that would normally be added
        // at position end_in_line in the loop (but the loop only goes 0-15)
        if has_closing_at_16 {
            underline.push('┘');
        }

        // Count display width (not bytes)
        let display_width: usize = underline.chars().count();

        write!(writer, "{}", underline)?;

        // Pad to align labels at column 59
        // Labels start after the underline + padding + space (in writeln)
        const LABEL_START_COLUMN: usize = 59;
        let current_pos = display_width;
        // We want the space in writeln to put us at LABEL_START_COLUMN
        // So we need to be at LABEL_START_COLUMN - 1 before the space
        // But for position 16, we need one extra space to match the "┘ " format
        let target_pos = if has_closing_at_16 {
            LABEL_START_COLUMN  // Need to be at 59 before writeln adds its space
        } else {
            LABEL_START_COLUMN - 1  // Need to be at 58 before writeln adds its space
        };
        let padding = if current_pos < target_pos {
            target_pos - current_pos
        } else {
            0
        };
        for _ in 0..padding {
            write!(writer, " ")?;
        }

        // Only show label on the first line of the annotation
        if ann_start >= line_offset && ann_start < line_end {
            let colored_label = match annotation.kind {
                AnnotationKind::Normal => self.colors.label(&annotation.label),
                AnnotationKind::Error => self.colors.error_label(&annotation.label),
            };
            writeln!(writer, " {}", colored_label)?;
        } else {
            writeln!(writer)?;
        }

        Ok(())
    }
}
