pub fn crc16_modbus(bytes: &[u8]) -> u16 {
    crc16_modbus_with_skips(bytes, &[])
}

pub fn crc16_modbus_with_skips(bytes: &[u8], skips: &[(usize, usize)]) -> u16 {
    let mut merged = skips
        .iter()
        .copied()
        .filter(|&(_start, len)| len > 0)
        .map(|(start, len)| (start, start.saturating_add(len)))
        .collect::<Vec<_>>();
    merged.sort_by_key(|(start, _end)| *start);

    // Merge overlaps
    let mut merged2: Vec<(usize, usize)> = Vec::new();
    for (start, end) in merged {
        if let Some(last) = merged2.last_mut() {
            if start <= last.1 {
                last.1 = last.1.max(end);
                continue;
            }
        }
        merged2.push((start, end));
    }

    let mut crc: u16 = 0xffff;

    let mut pos = 0usize;
    for (start, end) in merged2 {
        if start > bytes.len() {
            break;
        }
        let start = start.min(bytes.len());
        let end = end.min(bytes.len());

        if pos < start {
            crc = crc16_modbus_update(crc, &bytes[pos..start]);
        }
        pos = end.max(pos);
    }

    if pos < bytes.len() {
        crc = crc16_modbus_update(crc, &bytes[pos..]);
    }

    crc
}

fn crc16_modbus_update(mut crc: u16, bytes: &[u8]) -> u16 {
    for &b in bytes {
        crc ^= b as u16;
        for _ in 0..8 {
            if (crc & 1) == 1 {
                crc = (crc >> 1) ^ 0xa001;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc16_modbus_known_vector() {
        // Standard MODBUS test vector: "123456789" -> 0x4B37
        let crc = crc16_modbus(b"123456789");
        assert_eq!(crc, 0x4b37);
    }
}
