use flate2::bufread::ZlibDecoder;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    io::{BufReader, Cursor, Read},
};

const MAX_SECTIONS: u64 = 256;
const MAX_NAME_BYTES: u64 = 256;
const MAX_PAYLOAD_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_COMPRESSED_BYTES: usize = 16 * 1024 * 1024;
const MAX_DECODED_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug)]
pub struct Vault {
    sections: BTreeMap<String, Value>,
}

impl Vault {
    pub fn section(&self, name: &str) -> Option<&Value> {
        self.sections.get(name)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    #[error("could not read vault: {0}")]
    Io(#[from] std::io::Error),
    #[error("vault exceeds parser limit: {0}")]
    LimitExceeded(&'static str),
    #[error("vault is truncated")]
    Truncated,
    #[error("vault has invalid UTF-8 {0}")]
    InvalidUtf8(&'static str),
    #[error("vault has invalid JSON in section {0}")]
    InvalidJson(String),
    #[error("vault repeats section {0}")]
    DuplicateSection(String),
    #[error("vault has unexpected bytes after its last section")]
    TrailingData,
}

pub struct VaultReader;

impl VaultReader {
    pub fn read<R: Read>(mut input: R) -> Result<Vault, VaultError> {
        let mut compressed = Vec::new();
        input
            .by_ref()
            .take((MAX_COMPRESSED_BYTES + 1) as u64)
            .read_to_end(&mut compressed)?;
        if compressed.len() > MAX_COMPRESSED_BYTES {
            return Err(VaultError::LimitExceeded("compressed vault size"));
        }
        let bytes = decode_zlib(&compressed, MAX_DECODED_BYTES)?;
        parse_sections(&bytes)
    }
}

fn decode_zlib(compressed: &[u8], maximum_size: usize) -> Result<Vec<u8>, VaultError> {
    let cursor = Cursor::new(compressed);
    let reader = BufReader::new(cursor);
    let mut decoder = ZlibDecoder::new(reader);
    let mut decoded = Vec::new();
    let mut buffer = [0_u8; 8192];

    loop {
        let count = decoder.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        if decoded.len().saturating_add(count) > maximum_size {
            return Err(VaultError::LimitExceeded("decoded vault size"));
        }
        decoded.extend_from_slice(&buffer[..count]);
    }

    let reader = decoder.into_inner();
    if !reader.buffer().is_empty() || reader.get_ref().position() != compressed.len() as u64 {
        return Err(VaultError::TrailingData);
    }
    Ok(decoded)
}

fn parse_sections(bytes: &[u8]) -> Result<Vault, VaultError> {
    let mut offset = 0;
    let section_count = read_u64(bytes, &mut offset)?;
    if section_count > MAX_SECTIONS {
        return Err(VaultError::LimitExceeded("section count"));
    }

    let mut sections = BTreeMap::new();
    for _ in 0..section_count {
        let name_length = read_u64(bytes, &mut offset)?;
        if name_length > MAX_NAME_BYTES {
            return Err(VaultError::LimitExceeded("section name"));
        }
        let name = String::from_utf8(read_bytes(bytes, &mut offset, name_length)?.to_vec())
            .map_err(|_| VaultError::InvalidUtf8("section name"))?;
        let payload_length = read_u64(bytes, &mut offset)?;
        if payload_length > MAX_PAYLOAD_BYTES {
            return Err(VaultError::LimitExceeded("section payload"));
        }
        let payload = read_bytes(bytes, &mut offset, payload_length)?;
        let value =
            serde_json::from_slice(payload).map_err(|_| VaultError::InvalidJson(name.clone()))?;
        if sections.insert(name.clone(), value).is_some() {
            return Err(VaultError::DuplicateSection(name));
        }
    }

    // Current game saves append opaque state after the declared JSON section
    // table. It is deliberately neither parsed nor exposed; downstream
    // evidence extraction remains version-gated before it uses any section.
    Ok(Vault { sections })
}

fn read_u64(bytes: &[u8], offset: &mut usize) -> Result<u64, VaultError> {
    let value = read_bytes(bytes, offset, 8)?;
    Ok(u64::from_le_bytes(
        value.try_into().expect("eight bytes were requested"),
    ))
}

fn read_bytes<'a>(
    bytes: &'a [u8],
    offset: &mut usize,
    length: u64,
) -> Result<&'a [u8], VaultError> {
    let length = usize::try_from(length).map_err(|_| VaultError::LimitExceeded("length"))?;
    let end = offset.checked_add(length).ok_or(VaultError::Truncated)?;
    let value = bytes.get(*offset..end).ok_or(VaultError::Truncated)?;
    *offset = end;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::vault::{compress, sections, vault_bytes};
    use std::io::Read;

    #[test]
    fn reads_named_sections_and_rejects_duplicate_names() {
        let bytes = vault_bytes(&sections());
        assert_eq!(
            VaultReader::read(bytes.as_slice())
                .unwrap()
                .section("header")
                .unwrap()["name"],
            "Ari"
        );
        let duplicate = vault_bytes(&[
            ("header", serde_json::json!({})),
            ("header", serde_json::json!({})),
        ]);
        assert!(matches!(
            VaultReader::read(duplicate.as_slice()),
            Err(VaultError::DuplicateSection(_))
        ));
    }

    #[test]
    fn reads_declared_sections_before_an_opaque_game_trailer() {
        let payload = serde_json::to_vec(&serde_json::json!({"version": "synthetic-1"})).unwrap();
        let mut raw = 1_u64.to_le_bytes().to_vec();
        raw.extend_from_slice(&(4_u64).to_le_bytes());
        raw.extend_from_slice(b"info");
        raw.extend_from_slice(&(payload.len() as u64).to_le_bytes());
        raw.extend_from_slice(&payload);
        raw.extend_from_slice(b"opaque-game-data");

        let vault = VaultReader::read(compress(&raw).as_slice()).unwrap();
        assert_eq!(vault.section("info").unwrap()["version"], "synthetic-1");
    }

    #[test]
    fn rejects_excessive_lengths_before_allocating_payloads() {
        for raw in [
            257_u64.to_le_bytes().to_vec(),
            [1_u64.to_le_bytes(), 257_u64.to_le_bytes()].concat(),
            [
                1_u64.to_le_bytes().as_slice(),
                1_u64.to_le_bytes().as_slice(),
                b"a",
                (64_u64 * 1024 * 1024 + 1).to_le_bytes().as_slice(),
            ]
            .concat(),
        ] {
            assert!(matches!(
                VaultReader::read(compress(&raw).as_slice()),
                Err(VaultError::LimitExceeded(_))
            ));
        }
    }

    #[test]
    fn rejects_invalid_utf8_json_and_truncated_sections() {
        for (name, payload) in [
            (b"a".as_slice(), b"{".as_slice()),
            (b"\xff", b"{}"),
            (b"a", b"\xff"),
        ] {
            let raw = [
                1_u64.to_le_bytes().as_slice(),
                (name.len() as u64).to_le_bytes().as_slice(),
                name,
                (payload.len() as u64).to_le_bytes().as_slice(),
                payload,
            ]
            .concat();
            assert!(VaultReader::read(compress(&raw).as_slice()).is_err());
        }
        let raw = [1_u64.to_le_bytes(), 3_u64.to_le_bytes()].concat();
        assert!(VaultReader::read(compress(&raw).as_slice()).is_err());
    }

    #[test]
    fn requires_complete_zlib_stream_and_no_trailing_bytes() {
        let valid = vault_bytes(&sections());
        for end in 0..valid.len() {
            assert!(
                VaultReader::read(&valid[..end]).is_err(),
                "accepted truncation at {end}"
            );
        }
        let mut trailing = valid.clone();
        trailing.push(0);
        assert!(VaultReader::read(trailing.as_slice()).is_err());
        let mut corrupt = valid;
        let last = corrupt.len() - 1;
        corrupt[last] ^= 1;
        assert!(VaultReader::read(corrupt.as_slice()).is_err());
    }

    #[test]
    fn decompression_budget_rejects_expansion() {
        assert!(matches!(
            decode_zlib(&compress(&[0; 1024]), 32),
            Err(VaultError::LimitExceeded(_))
        ));
    }

    #[test]
    fn rejects_compressed_input_larger_than_the_safe_limit() {
        assert!(matches!(
            VaultReader::read(std::io::repeat(0).take(16 * 1024 * 1024 + 1)),
            Err(VaultError::LimitExceeded("compressed vault size"))
        ));
    }

    #[test]
    fn rejects_decoded_vault_larger_than_the_safe_limit() {
        let oversized = vec![0; 16 * 1024 * 1024 + 1];
        assert!(matches!(
            VaultReader::read(compress(&oversized).as_slice()),
            Err(VaultError::LimitExceeded("decoded vault size"))
        ));
    }
}
