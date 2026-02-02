use crate::crc16::{crc16_modbus, crc16_modbus_with_skips};
use crate::profile::{ChecksumAlgo, Endianness, Profile};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Candidate {
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub struct VerifyReport {
    pub valid: Vec<Candidate>,
}

#[derive(Debug, Clone)]
pub struct PatchOptions {
    pub ssid: String,
    pub psk: String,
    pub select_index: Option<usize>,
}

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("firmware too small to contain magic")]
    FirmwareTooSmall,
    #[error("too many candidates: found {found} (max_matches={max_matches})")]
    TooManyCandidates { found: usize, max_matches: usize },
}

#[derive(Debug, Error)]
pub enum VerifyError {
    #[error("scan failed: {0}")]
    Scan(#[from] ScanError),
    #[error("no candidates found")]
    NoCandidates,
    #[error("no valid candidates (candidates={candidates})")]
    NoValidCandidates { candidates: usize },
    #[error("multiple valid candidates (valid={valid})")]
    MultipleValidCandidates { valid: usize },
    #[error("select_index out of range: index={index} (valid={valid})")]
    SelectIndexOutOfRange { index: usize, valid: usize },
    #[error("ssid bytes too long: {bytes} (max={max})")]
    SsidTooLong { bytes: usize, max: usize },
    #[error("psk bytes too long: {bytes} (max={max})")]
    PskTooLong { bytes: usize, max: usize },
}

impl Profile {
    pub fn scan_candidates(&self, firmware: &[u8]) -> Result<Vec<Candidate>, ScanError> {
        if firmware.len() < 4 {
            return Err(ScanError::FirmwareTooSmall);
        }

        let magic = self.scan.endianness.u32_to_bytes(self.scan.magic_u32);
        let max_matches = self.scan.max_matches.unwrap_or(1);

        let mut out: Vec<Candidate> = Vec::new();
        let mut pos = 0usize;
        while pos + 4 <= firmware.len() {
            if firmware[pos..pos + 4] == magic && pos + self.scan.struct_size <= firmware.len() {
                out.push(Candidate { offset: pos });
                if out.len() > max_matches {
                    return Err(ScanError::TooManyCandidates {
                        found: out.len(),
                        max_matches,
                    });
                }
            }
            pos += 1;
        }

        Ok(out)
    }

    pub fn verify(&self, firmware: &[u8]) -> Result<VerifyReport, VerifyError> {
        let candidates = self.scan_candidates(firmware)?;
        if candidates.is_empty() {
            return Err(VerifyError::NoCandidates);
        }

        let mut valid: Vec<Candidate> = Vec::new();
        for c in candidates.iter().copied() {
            if self.verify_candidate(firmware, c).is_ok() {
                valid.push(c);
            }
        }

        if valid.is_empty() {
            return Err(VerifyError::NoValidCandidates {
                candidates: candidates.len(),
            });
        }

        Ok(VerifyReport { valid })
    }

    pub fn patch(&self, firmware: &[u8], opts: PatchOptions) -> Result<Vec<u8>, VerifyError> {
        let report = self.verify(firmware)?;
        let selected = match (report.valid.as_slice(), opts.select_index) {
            ([only], None) => *only,
            (_many, Some(index)) => {
                let Some(candidate) = report.valid.get(index).copied() else {
                    return Err(VerifyError::SelectIndexOutOfRange {
                        index,
                        valid: report.valid.len(),
                    });
                };
                candidate
            }
            (_many, None) => {
                return Err(VerifyError::MultipleValidCandidates {
                    valid: report.valid.len(),
                });
            }
        };

        self.patch_at_offset(firmware, selected.offset, &opts.ssid, &opts.psk)
    }

    fn verify_candidate(&self, firmware: &[u8], candidate: Candidate) -> Result<(), ()> {
        let Some(block) = firmware.get(candidate.offset..candidate.offset + self.scan.struct_size)
        else {
            return Err(());
        };

        // version
        let version_len = self.layout.version_len.unwrap_or(2);
        let Some(version_bytes) =
            block.get(self.layout.version_offset..self.layout.version_offset + version_len)
        else {
            return Err(());
        };
        let version_value = read_uint(self.scan.endianness, version_bytes).ok_or(())?;
        if version_value != self.layout.version as u64 {
            return Err(());
        }

        // lengths
        let ssid_len = *block.get(self.layout.ssid_len_offset).ok_or(())? as usize;
        let psk_len = *block.get(self.layout.psk_len_offset).ok_or(())? as usize;
        if ssid_len > self.layout.ssid_max || psk_len > self.layout.psk_max {
            return Err(());
        }

        // checksum
        let Some(stored_checksum_bytes) = block.get(
            self.layout.checksum_offset..self.layout.checksum_offset + self.layout.checksum_len,
        ) else {
            return Err(());
        };
        let stored = self
            .scan
            .endianness
            .read_u16(stored_checksum_bytes)
            .ok_or(())?;

        let skips = self
            .checksum
            .skip
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|s| (s.offset, s.len))
            .collect::<Vec<_>>();

        let computed = match self.checksum.algo {
            ChecksumAlgo::Crc16Modbus => {
                if skips.is_empty() {
                    crc16_modbus(block)
                } else {
                    crc16_modbus_with_skips(block, &skips)
                }
            }
        };

        if computed != stored {
            return Err(());
        }

        Ok(())
    }

    fn patch_at_offset(
        &self,
        firmware: &[u8],
        base: usize,
        ssid: &str,
        psk: &str,
    ) -> Result<Vec<u8>, VerifyError> {
        let ssid_bytes = ssid.as_bytes();
        let psk_bytes = psk.as_bytes();
        if ssid_bytes.len() > self.layout.ssid_max {
            return Err(VerifyError::SsidTooLong {
                bytes: ssid_bytes.len(),
                max: self.layout.ssid_max,
            });
        }
        if psk_bytes.len() > self.layout.psk_max {
            return Err(VerifyError::PskTooLong {
                bytes: psk_bytes.len(),
                max: self.layout.psk_max,
            });
        }

        let mut out = firmware.to_vec();
        let Some(block) = out.get_mut(base..base + self.scan.struct_size) else {
            return Err(VerifyError::NoCandidates);
        };

        let ssid_len = u8::try_from(ssid_bytes.len()).map_err(|_| VerifyError::SsidTooLong {
            bytes: ssid_bytes.len(),
            max: u8::MAX as usize,
        })?;
        let psk_len = u8::try_from(psk_bytes.len()).map_err(|_| VerifyError::PskTooLong {
            bytes: psk_bytes.len(),
            max: u8::MAX as usize,
        })?;

        block[self.layout.ssid_len_offset] = ssid_len;
        block[self.layout.psk_len_offset] = psk_len;

        write_padded(
            block,
            self.layout.ssid_offset,
            self.layout.ssid_max,
            ssid_bytes,
        );
        write_padded(
            block,
            self.layout.psk_offset,
            self.layout.psk_max,
            psk_bytes,
        );

        let skips = self
            .checksum
            .skip
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|s| (s.offset, s.len))
            .collect::<Vec<_>>();

        let checksum_value = match self.checksum.algo {
            ChecksumAlgo::Crc16Modbus => {
                if skips.is_empty() {
                    crc16_modbus(block)
                } else {
                    crc16_modbus_with_skips(block, &skips)
                }
            }
        };

        let checksum_range =
            self.layout.checksum_offset..self.layout.checksum_offset + self.layout.checksum_len;
        let Some(dst) = block.get_mut(checksum_range) else {
            return Err(VerifyError::NoCandidates);
        };
        match self.scan.endianness {
            Endianness::Little | Endianness::Big => {
                if !self.scan.endianness.write_u16(dst, checksum_value) {
                    return Err(VerifyError::NoCandidates);
                }
            }
        }

        Ok(out)
    }
}

fn read_uint(endianness: Endianness, bytes: &[u8]) -> Option<u64> {
    match bytes.len() {
        1 => Some(bytes[0] as u64),
        2 => endianness.read_u16(bytes).map(|v| v as u64),
        4 => endianness.read_u32(bytes).map(|v| v as u64),
        _ => None,
    }
}

fn write_padded(block: &mut [u8], offset: usize, max: usize, data: &[u8]) {
    let Some(dst) = block.get_mut(offset..offset + max) else {
        return;
    };
    dst.fill(0);
    let size = data.len().min(max);
    dst[..size].copy_from_slice(&data[..size]);
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROFILE_TOML: &str = r#"
profile_version = 1
id = "wifi_cfg_v1_crc16_modbus"
name = "Generic WiFi config v1 (CRC16/MODBUS)"

[scan]
magic_u32 = 0x57494649
endianness = "little"
struct_size = 108
max_matches = 4

[layout]
version = 1
version_offset = 4
version_len = 2
checksum_offset = 6
checksum_len = 2
ssid_len_offset = 8
psk_len_offset = 9
ssid_offset = 12
ssid_max = 32
psk_offset = 44
psk_max = 64

[checksum]
algo = "crc16_modbus"
skip = [{ offset = 6, len = 2 }]
"#;

    fn build_valid_block(profile: &Profile, ssid: &str, psk: &str) -> Vec<u8> {
        let mut block = vec![0u8; profile.scan.struct_size];
        let magic = profile.scan.endianness.u32_to_bytes(profile.scan.magic_u32);
        block[0..4].copy_from_slice(&magic);

        let version_len = profile.layout.version_len.unwrap_or(2);
        let version_range =
            profile.layout.version_offset..profile.layout.version_offset + version_len;
        let version_value = profile.layout.version as u16;
        profile
            .scan
            .endianness
            .write_u16(&mut block[version_range], version_value);

        block[profile.layout.ssid_len_offset] = ssid.len() as u8;
        block[profile.layout.psk_len_offset] = psk.len() as u8;
        write_padded(
            &mut block,
            profile.layout.ssid_offset,
            profile.layout.ssid_max,
            ssid.as_bytes(),
        );
        write_padded(
            &mut block,
            profile.layout.psk_offset,
            profile.layout.psk_max,
            psk.as_bytes(),
        );

        let skips = profile
            .checksum
            .skip
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|s| (s.offset, s.len))
            .collect::<Vec<_>>();
        let checksum = crc16_modbus_with_skips(&block, &skips);

        let checksum_range = profile.layout.checksum_offset
            ..profile.layout.checksum_offset + profile.layout.checksum_len;
        profile
            .scan
            .endianness
            .write_u16(&mut block[checksum_range], checksum);

        block
    }

    #[test]
    fn verify_and_patch_roundtrip() {
        let profile = Profile::parse_toml(PROFILE_TOML).unwrap();
        let block = build_valid_block(&profile, "old", "oldpsk");

        let mut firmware = vec![0u8; 16];
        firmware.extend_from_slice(&block);
        firmware.extend_from_slice(&[0u8; 16]);

        let report = profile.verify(&firmware).unwrap();
        assert_eq!(report.valid.len(), 1);

        let patched = profile
            .patch(
                &firmware,
                PatchOptions {
                    ssid: "newssid".to_string(),
                    psk: "newpsk".to_string(),
                    select_index: None,
                },
            )
            .unwrap();

        let report2 = profile.verify(&patched).unwrap();
        assert_eq!(report2.valid.len(), 1);
    }

    #[test]
    fn verify_preserves_scan_error_too_many_candidates() {
        let profile = Profile::parse_toml(PROFILE_TOML).unwrap();
        let block = build_valid_block(&profile, "a", "b");

        let mut firmware = Vec::new();
        for _ in 0..5 {
            firmware.extend_from_slice(&block);
            firmware.extend_from_slice(&[0u8; 4]);
        }

        let err = profile.verify(&firmware).unwrap_err();
        match err {
            VerifyError::Scan(ScanError::TooManyCandidates { found, max_matches }) => {
                assert_eq!(found, 5);
                assert_eq!(max_matches, 4);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn patch_reports_ssid_too_long_in_bytes() {
        let profile = Profile::parse_toml(PROFILE_TOML).unwrap();
        let block = build_valid_block(&profile, "old", "oldpsk");

        let mut firmware = vec![0u8; 16];
        firmware.extend_from_slice(&block);

        let ssid = "你".repeat(11);
        assert_eq!(ssid.len(), 33);

        let err = profile
            .patch(
                &firmware,
                PatchOptions {
                    ssid: ssid.clone(),
                    psk: "newpsk".to_string(),
                    select_index: None,
                },
            )
            .unwrap_err();

        match err {
            VerifyError::SsidTooLong { bytes, max } => {
                assert_eq!(bytes, ssid.len());
                assert_eq!(max, profile.layout.ssid_max);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn patch_reports_select_index_out_of_range() {
        let profile = Profile::parse_toml(PROFILE_TOML).unwrap();
        let block1 = build_valid_block(&profile, "old1", "oldpsk1");
        let block2 = build_valid_block(&profile, "old2", "oldpsk2");

        let mut firmware = vec![0u8; 8];
        firmware.extend_from_slice(&block1);
        firmware.extend_from_slice(&[0u8; 8]);
        firmware.extend_from_slice(&block2);
        firmware.extend_from_slice(&[0u8; 8]);

        let report = profile.verify(&firmware).unwrap();
        assert_eq!(report.valid.len(), 2);

        let err = profile
            .patch(
                &firmware,
                PatchOptions {
                    ssid: "newssid".to_string(),
                    psk: "newpsk".to_string(),
                    select_index: Some(99),
                },
            )
            .unwrap_err();

        match err {
            VerifyError::SelectIndexOutOfRange { index, valid } => {
                assert_eq!(index, 99);
                assert_eq!(valid, 2);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
