mod color;
mod display;
mod types;

use anyhow::Result;
use argh::FromArgs;
use std::fs::File;
use std::io::{self, Cursor, Read};
use std::path::PathBuf;

pub use display::{Annotation, AnnotationKind, Hexdump};
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

    /// byte order for multi-byte types: native (default), little, or big
    #[argh(option, default = "String::from(\"native\")")]
    byte_order: String,
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
/// Returns annotations and optionally an error if we ran out of data
pub fn build_annotations_from_types(
    type_specs: &[String],
    byte_order: ByteOrder,
    data: &[u8],
) -> Result<(Vec<Annotation>, Option<anyhow::Error>)> {
    let mut annotations = Vec::new();
    let mut offset = 0;

    for type_spec_str in type_specs {
        let type_spec = TypeSpec::from_str(type_spec_str)?;
        let size = type_spec.data_type.size();

        // Check if we have enough data
        if offset + size > data.len() {
            let bytes_available = data.len().saturating_sub(offset);

            // Create error annotation showing what we expected vs what we have
            let error_label = format!(
                "{}: expected {} bytes, only {} available",
                type_spec.display_name(),
                size,
                bytes_available
            );

            // Annotate the bytes we DO have (if any)
            annotations.push(Annotation::error(offset, bytes_available, error_label));

            // Return with error
            let error = anyhow::anyhow!(
                "Not enough data: type {} at offset {} needs {} bytes, but only {} bytes available",
                type_spec.data_type.name(),
                offset,
                size,
                bytes_available
            );
            return Ok((annotations, Some(error)));
        }

        // Decode the value
        let value = type_spec.data_type.decode(&data[offset..offset + size], byte_order)?;

        // Create label: "name: value" (using field name if provided, otherwise type name)
        let label = format!("{}: {}", type_spec.display_name(), value);

        annotations.push(Annotation::new(offset, size, label));
        offset += size;
    }

    Ok((annotations, None))
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
    let mut had_error = None;

    // If types are specified, build annotations from them
    if !args.types.is_empty() {
        let byte_order = ByteOrder::from_str(&args.byte_order)?;
        let (annotations, error) = build_annotations_from_types(&args.types, byte_order, &data)?;
        for annotation in annotations {
            hexdump.add_annotation(annotation);
        }
        had_error = error;
    }
    // If no types specified, just show plain hexdump without annotations

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    // Dump from the in-memory data
    let mut cursor = Cursor::new(&data);
    hexdump.dump(&mut cursor, &mut handle)?;

    // If there was an error, return it (after displaying the hexdump with error annotation)
    if let Some(error) = had_error {
        return Err(error);
    }

    Ok(())
}
