use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Endianness {
    Little,
    Big,
}

impl Endianness {
    pub fn u32_to_bytes(self, value: u32) -> [u8; 4] {
        match self {
            Endianness::Little => value.to_le_bytes(),
            Endianness::Big => value.to_be_bytes(),
        }
    }

    pub fn read_u16(self, bytes: &[u8]) -> Option<u16> {
        let raw = bytes.get(0..2)?;
        Some(match self {
            Endianness::Little => u16::from_le_bytes([raw[0], raw[1]]),
            Endianness::Big => u16::from_be_bytes([raw[0], raw[1]]),
        })
    }

    pub fn read_u32(self, bytes: &[u8]) -> Option<u32> {
        let raw = bytes.get(0..4)?;
        Some(match self {
            Endianness::Little => u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]),
            Endianness::Big => u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]),
        })
    }

    pub fn write_u16(self, out: &mut [u8], value: u16) -> bool {
        let raw = match self {
            Endianness::Little => value.to_le_bytes(),
            Endianness::Big => value.to_be_bytes(),
        };
        let Some(dst) = out.get_mut(0..2) else {
            return false;
        };
        dst.copy_from_slice(&raw);
        true
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Profile {
    pub profile_version: u32,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,

    pub scan: Scan,
    pub layout: Layout,
    pub checksum: Checksum,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Scan {
    pub magic_u32: u32,
    pub endianness: Endianness,
    pub struct_size: usize,
    #[serde(default)]
    pub max_matches: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Layout {
    pub version: u32,
    pub version_offset: usize,
    #[serde(default)]
    pub version_len: Option<usize>,

    pub checksum_offset: usize,
    pub checksum_len: usize,

    pub ssid_len_offset: usize,
    pub psk_len_offset: usize,

    pub ssid_offset: usize,
    pub ssid_max: usize,

    pub psk_offset: usize,
    pub psk_max: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Checksum {
    pub algo: ChecksumAlgo,
    #[serde(default)]
    pub skip: Option<Vec<SkipRange>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChecksumAlgo {
    Crc16Modbus,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SkipRange {
    pub offset: usize,
    pub len: usize,
}

#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("invalid TOML: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("profile_version must be 1, got {0}")]
    UnsupportedVersion(u32),
    #[error("scan.struct_size must be > 0")]
    InvalidStructSize,
    #[error("scan.max_matches must be >= 1")]
    InvalidMaxMatches,
    #[error("layout.version_len must be 1/2/4")]
    InvalidVersionLen,
    #[error("layout.checksum_len must be 2 for crc16_modbus")]
    InvalidChecksumLen,
    #[error("layout.ssid_max must be <= 255 (ssid_len is stored as u8)")]
    InvalidSsidMax,
    #[error("layout.psk_max must be <= 255 (psk_len is stored as u8)")]
    InvalidPskMax,
    #[error(
        "layout.{field} is out of bounds: offset={offset}, len={len}, struct_size={struct_size}"
    )]
    LayoutOutOfBounds {
        field: &'static str,
        offset: usize,
        len: usize,
        struct_size: usize,
    },
    #[error(
        "checksum.skip is out of bounds: offset={offset}, len={len}, struct_size={struct_size}"
    )]
    SkipOutOfBounds {
        offset: usize,
        len: usize,
        struct_size: usize,
    },
}

impl Profile {
    pub fn parse_toml(input: &str) -> Result<Self, ProfileError> {
        let profile: Profile = toml::from_str(input)?;
        profile.validate()?;
        Ok(profile)
    }

    pub fn validate(&self) -> Result<(), ProfileError> {
        if self.profile_version != 1 {
            return Err(ProfileError::UnsupportedVersion(self.profile_version));
        }
        if self.scan.struct_size == 0 {
            return Err(ProfileError::InvalidStructSize);
        }
        if let Some(max) = self.scan.max_matches {
            if max < 1 {
                return Err(ProfileError::InvalidMaxMatches);
            }
        }

        let version_len = self.layout.version_len.unwrap_or(2);
        if !matches!(version_len, 1 | 2 | 4) {
            return Err(ProfileError::InvalidVersionLen);
        }
        if matches!(self.checksum.algo, ChecksumAlgo::Crc16Modbus) && self.layout.checksum_len != 2
        {
            return Err(ProfileError::InvalidChecksumLen);
        }

        if self.layout.ssid_max > u8::MAX as usize {
            return Err(ProfileError::InvalidSsidMax);
        }
        if self.layout.psk_max > u8::MAX as usize {
            return Err(ProfileError::InvalidPskMax);
        }

        let struct_size = self.scan.struct_size;
        self.check_bounds(
            "version",
            self.layout.version_offset,
            version_len,
            struct_size,
        )?;
        self.check_bounds(
            "checksum",
            self.layout.checksum_offset,
            self.layout.checksum_len,
            struct_size,
        )?;

        self.check_bounds("ssid_len", self.layout.ssid_len_offset, 1, struct_size)?;
        self.check_bounds("psk_len", self.layout.psk_len_offset, 1, struct_size)?;

        self.check_bounds(
            "ssid",
            self.layout.ssid_offset,
            self.layout.ssid_max,
            struct_size,
        )?;
        self.check_bounds(
            "psk",
            self.layout.psk_offset,
            self.layout.psk_max,
            struct_size,
        )?;

        if let Some(skips) = &self.checksum.skip {
            for skip in skips {
                if skip.offset > struct_size || skip.offset.saturating_add(skip.len) > struct_size {
                    return Err(ProfileError::SkipOutOfBounds {
                        offset: skip.offset,
                        len: skip.len,
                        struct_size,
                    });
                }
            }
        }

        Ok(())
    }

    fn check_bounds(
        &self,
        field: &'static str,
        offset: usize,
        len: usize,
        struct_size: usize,
    ) -> Result<(), ProfileError> {
        if offset > struct_size || offset.saturating_add(len) > struct_size {
            return Err(ProfileError::LayoutOutOfBounds {
                field,
                offset,
                len,
                struct_size,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_ssid_max_over_u8() {
        let toml = r#"
profile_version = 1
id = "bad"
name = "bad"

[scan]
magic_u32 = 0x57494649
endianness = "little"
struct_size = 400

[layout]
version = 1
version_offset = 4
checksum_offset = 6
checksum_len = 2
ssid_len_offset = 8
psk_len_offset = 9
ssid_offset = 12
ssid_max = 256
psk_offset = 300
psk_max = 64

[checksum]
algo = "crc16_modbus"
skip = [{ offset = 6, len = 2 }]
"#;

        let err = Profile::parse_toml(toml).unwrap_err();
        assert!(matches!(err, ProfileError::InvalidSsidMax));
    }

    #[test]
    fn rejects_psk_max_over_u8() {
        let toml = r#"
profile_version = 1
id = "bad"
name = "bad"

[scan]
magic_u32 = 0x57494649
endianness = "little"
struct_size = 400

[layout]
version = 1
version_offset = 4
checksum_offset = 6
checksum_len = 2
ssid_len_offset = 8
psk_len_offset = 9
ssid_offset = 12
ssid_max = 32
psk_offset = 300
psk_max = 256

[checksum]
algo = "crc16_modbus"
skip = [{ offset = 6, len = 2 }]
"#;

        let err = Profile::parse_toml(toml).unwrap_err();
        assert!(matches!(err, ProfileError::InvalidPskMax));
    }
}
