mod color;
mod display;
mod types;

use anyhow::Result;
use argh::FromArgs;
use std::fs::File;
use std::io::{self, Cursor, Read};
use std::path::PathBuf;

pub use display::{Annotation, Hexdump};
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

/// Represents a type specification or skip directive
enum TypeSpec {
    /// A data type with optional field name
    Type {
        data_type: DataType,
        field_name: Option<String>,
    },
    /// Skip directive - number of bytes to skip
    Skip { bytes: usize },
}

impl TypeSpec {
    /// Parse a type specification string (e.g., "u16", "u16:apid", or ".32")
    fn from_str(s: &str) -> Result<Self> {
        // Check for skip directive (.8, .16, .32, etc.)
        if s.starts_with('.') {
            let bits_str = &s[1..];
            let bits: usize = bits_str.parse().map_err(|_| {
                anyhow::anyhow!("Invalid skip syntax '{}': expected .N where N is number of bits", s)
            })?;

            if bits == 0 {
                return Err(anyhow::anyhow!("Skip size cannot be 0"));
            }

            if bits % 8 != 0 {
                return Err(anyhow::anyhow!(
                    "Skip size must be a multiple of 8 bits (got {} bits)",
                    bits
                ));
            }

            let bytes = bits / 8;
            return Ok(TypeSpec::Skip { bytes });
        }

        // Otherwise parse as type with optional field name
        if let Some(colon_pos) = s.find(':') {
            // Format: "type:fieldname"
            let type_part = &s[..colon_pos];
            let field_part = &s[colon_pos + 1..];

            if field_part.is_empty() {
                return Err(anyhow::anyhow!("Field name cannot be empty in '{}'", s));
            }

            let data_type = DataType::from_str(type_part)?;
            Ok(TypeSpec::Type {
                data_type,
                field_name: Some(field_part.to_string()),
            })
        } else {
            // Format: "type"
            let data_type = DataType::from_str(s)?;
            Ok(TypeSpec::Type {
                data_type,
                field_name: None,
            })
        }
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

        match type_spec {
            TypeSpec::Skip { bytes } => {
                // Skip directive - just advance offset
                if offset + bytes > data.len() {
                    return Err(anyhow::anyhow!(
                        "Not enough data: skip {} bytes at offset {} exceeds data length {}",
                        bytes,
                        offset,
                        data.len()
                    ));
                }
                offset += bytes;
            }
            TypeSpec::Type { data_type, field_name } => {
                let size = data_type.size();

                // Check if we have enough data
                if offset + size > data.len() {
                    let display_name = field_name.as_deref().unwrap_or_else(|| data_type.name());
                    return Err(anyhow::anyhow!(
                        "Not enough data: type {} at offset {} needs {} bytes, but only {} bytes available",
                        display_name,
                        offset,
                        size,
                        data.len() - offset
                    ));
                }

                // Decode the value
                let value = data_type.decode(&data[offset..offset + size], byte_order)?;

                // Create label: "name: value" (using field name if provided, otherwise type name)
                let display_name = field_name.as_deref().unwrap_or_else(|| data_type.name());
                let label = format!("{}: {}", display_name, value);

                annotations.push(Annotation::new(offset, size, label));
                offset += size;
            }
        }
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
    let mut cursor = Cursor::new(&data);
    hexdump.dump(&mut cursor, &mut handle)?;

    Ok(())
}
