use anyhow::{anyhow, Result};

/// Byte order for multi-byte types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    Little,
    Big,
}

impl ByteOrder {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "little" | "le" => Ok(ByteOrder::Little),
            "big" | "be" => Ok(ByteOrder::Big),
            _ => Err(anyhow!("Invalid byte order: {}. Use 'little' or 'big'", s)),
        }
    }
}

impl Default for ByteOrder {
    fn default() -> Self {
        // Default to little-endian (most common on modern systems)
        ByteOrder::Little
    }
}

/// Supported data types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
}

impl DataType {
    /// Parse a type from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "u8" => Ok(DataType::U8),
            "u16" => Ok(DataType::U16),
            "u32" => Ok(DataType::U32),
            "u64" => Ok(DataType::U64),
            "i8" => Ok(DataType::I8),
            "i16" => Ok(DataType::I16),
            "i32" => Ok(DataType::I32),
            "i64" => Ok(DataType::I64),
            "f32" | "float" => Ok(DataType::F32),
            "f64" | "double" => Ok(DataType::F64),
            _ => Err(anyhow!("Unknown type: {}", s)),
        }
    }

    /// Get the size in bytes for this type
    pub fn size(&self) -> usize {
        match self {
            DataType::U8 | DataType::I8 => 1,
            DataType::U16 | DataType::I16 => 2,
            DataType::U32 | DataType::I32 | DataType::F32 => 4,
            DataType::U64 | DataType::I64 | DataType::F64 => 8,
        }
    }

    /// Decode value from bytes and return as string
    pub fn decode(&self, bytes: &[u8], byte_order: ByteOrder) -> Result<String> {
        if bytes.len() < self.size() {
            return Err(anyhow!(
                "Not enough bytes: need {}, got {}",
                self.size(),
                bytes.len()
            ));
        }

        let result = match self {
            DataType::U8 => bytes[0].to_string(),
            DataType::I8 => (bytes[0] as i8).to_string(),
            DataType::U16 => {
                let val = match byte_order {
                    ByteOrder::Little => u16::from_le_bytes([bytes[0], bytes[1]]),
                    ByteOrder::Big => u16::from_be_bytes([bytes[0], bytes[1]]),
                };
                val.to_string()
            }
            DataType::I16 => {
                let val = match byte_order {
                    ByteOrder::Little => i16::from_le_bytes([bytes[0], bytes[1]]),
                    ByteOrder::Big => i16::from_be_bytes([bytes[0], bytes[1]]),
                };
                val.to_string()
            }
            DataType::U32 => {
                let val = match byte_order {
                    ByteOrder::Little => {
                        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
                    }
                    ByteOrder::Big => u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                };
                val.to_string()
            }
            DataType::I32 => {
                let val = match byte_order {
                    ByteOrder::Little => {
                        i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
                    }
                    ByteOrder::Big => i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                };
                val.to_string()
            }
            DataType::U64 => {
                let val = match byte_order {
                    ByteOrder::Little => u64::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
                        bytes[7],
                    ]),
                    ByteOrder::Big => u64::from_be_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
                        bytes[7],
                    ]),
                };
                val.to_string()
            }
            DataType::I64 => {
                let val = match byte_order {
                    ByteOrder::Little => i64::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
                        bytes[7],
                    ]),
                    ByteOrder::Big => i64::from_be_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
                        bytes[7],
                    ]),
                };
                val.to_string()
            }
            DataType::F32 => {
                let val = match byte_order {
                    ByteOrder::Little => {
                        f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
                    }
                    ByteOrder::Big => f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                };
                format!("{:.6}", val)
            }
            DataType::F64 => {
                let val = match byte_order {
                    ByteOrder::Little => f64::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
                        bytes[7],
                    ]),
                    ByteOrder::Big => f64::from_be_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
                        bytes[7],
                    ]),
                };
                format!("{:.6}", val)
            }
        };

        Ok(result)
    }

    /// Get a display name for this type
    pub fn name(&self) -> &'static str {
        match self {
            DataType::U8 => "u8",
            DataType::U16 => "u16",
            DataType::U32 => "u32",
            DataType::U64 => "u64",
            DataType::I8 => "i8",
            DataType::I16 => "i16",
            DataType::I32 => "i32",
            DataType::I64 => "i64",
            DataType::F32 => "f32",
            DataType::F64 => "f64",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_data_types() {
        assert_eq!(DataType::from_str("u8").unwrap(), DataType::U8);
        assert_eq!(DataType::from_str("U8").unwrap(), DataType::U8);
        assert_eq!(DataType::from_str("u32").unwrap(), DataType::U32);
        assert_eq!(DataType::from_str("i16").unwrap(), DataType::I16);
        assert_eq!(DataType::from_str("f32").unwrap(), DataType::F32);
        assert_eq!(DataType::from_str("float").unwrap(), DataType::F32);
        assert_eq!(DataType::from_str("double").unwrap(), DataType::F64);
        assert!(DataType::from_str("invalid").is_err());
    }

    #[test]
    fn test_parse_byte_order() {
        assert_eq!(ByteOrder::from_str("little").unwrap(), ByteOrder::Little);
        assert_eq!(ByteOrder::from_str("Little").unwrap(), ByteOrder::Little);
        assert_eq!(ByteOrder::from_str("le").unwrap(), ByteOrder::Little);
        assert_eq!(ByteOrder::from_str("big").unwrap(), ByteOrder::Big);
        assert_eq!(ByteOrder::from_str("BE").unwrap(), ByteOrder::Big);
        assert!(ByteOrder::from_str("invalid").is_err());
    }

    #[test]
    fn test_type_sizes() {
        assert_eq!(DataType::U8.size(), 1);
        assert_eq!(DataType::U16.size(), 2);
        assert_eq!(DataType::U32.size(), 4);
        assert_eq!(DataType::U64.size(), 8);
        assert_eq!(DataType::I8.size(), 1);
        assert_eq!(DataType::F32.size(), 4);
        assert_eq!(DataType::F64.size(), 8);
    }

    #[test]
    fn test_decode_u8() {
        let bytes = [42u8];
        assert_eq!(
            DataType::U8.decode(&bytes, ByteOrder::Little).unwrap(),
            "42"
        );
    }

    #[test]
    fn test_decode_u16_little_endian() {
        let bytes = [0x34, 0x12]; // Little-endian 0x1234
        assert_eq!(
            DataType::U16.decode(&bytes, ByteOrder::Little).unwrap(),
            "4660" // 0x1234 = 4660
        );
    }

    #[test]
    fn test_decode_u16_big_endian() {
        let bytes = [0x12, 0x34]; // Big-endian 0x1234
        assert_eq!(
            DataType::U16.decode(&bytes, ByteOrder::Big).unwrap(),
            "4660"
        );
    }

    #[test]
    fn test_decode_u32_little_endian() {
        let bytes = [0x78, 0x56, 0x34, 0x12]; // Little-endian 0x12345678
        assert_eq!(
            DataType::U32.decode(&bytes, ByteOrder::Little).unwrap(),
            "305419896" // 0x12345678
        );
    }

    #[test]
    fn test_decode_u32_big_endian() {
        let bytes = [0x12, 0x34, 0x56, 0x78]; // Big-endian 0x12345678
        assert_eq!(
            DataType::U32.decode(&bytes, ByteOrder::Big).unwrap(),
            "305419896"
        );
    }

    #[test]
    fn test_decode_i8_negative() {
        let bytes = [255u8]; // -1 as i8
        assert_eq!(
            DataType::I8.decode(&bytes, ByteOrder::Little).unwrap(),
            "-1"
        );
    }

    #[test]
    fn test_decode_not_enough_bytes() {
        let bytes = [0x12]; // Only 1 byte
        assert!(DataType::U16.decode(&bytes, ByteOrder::Little).is_err());
    }

    #[test]
    fn test_decode_f32() {
        let val = 3.14159f32;
        let bytes = val.to_le_bytes();
        let decoded = DataType::F32.decode(&bytes, ByteOrder::Little).unwrap();
        assert!(decoded.starts_with("3.14159"));
    }
}
