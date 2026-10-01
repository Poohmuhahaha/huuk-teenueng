//! Encrypted-at-rest storage for provider access tokens.
//!
//! The live mirror refreshes on a schedule, so provider tokens must survive
//! restarts — but a leaked snapshot must not hand out working credentials. The
//! tokens are sealed with ChaCha20-Poly1305 under a key derived from
//! `ADMIN_TOKEN` (Argon2id, random per-store salt). Without `ADMIN_TOKEN` (demo
//! mode, tests) tokens stay in memory only, exactly as before this module.
//!
//! Threat model: an attacker who reads the snapshot file learns nothing without
//! the deployment's `ADMIN_TOKEN`; an attacker who also has the env secret can
//! decrypt, which is the same as having server access.
use argon2::Argon2;
use base64::Engine;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use rand::RngCore;

/// Random salt length (bytes) for the key derivation.
pub const SALT_LEN: usize = 16;
/// Nonce length (bytes) prefixed to every sealed payload.
const NONCE_LEN: usize = 12;

/// Derives the sealing key from the deployment secret. Deliberately slow
/// (Argon2id) — this runs once per process start, not per request.
pub fn derive_key(admin_token: &str, salt: &[u8]) -> Option<[u8; 32]> {
    if admin_token.trim().len() < 16 {
        return None;
    }
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(admin_token.as_bytes(), salt, &mut key)
        .ok()?;
    Some(key)
}

/// Fresh random salt for a store that does not have one yet.
pub fn new_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    salt
}

/// Seals `plaintext` as base64(nonce ‖ ciphertext ‖ tag).
pub fn seal(key: &[u8; 32], plaintext: &str) -> Option<String> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let mut nonce = [0u8; NONCE_LEN];
    rand::rngs::OsRng.fill_bytes(&mut nonce);
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext.as_bytes())
        .ok()?;
    let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    Some(base64::engine::general_purpose::STANDARD.encode(out))
}

/// Opens a payload produced by [`seal`]. `None` = wrong key or tampered data.
pub fn open(key: &[u8; 32], payload: &str) -> Option<String> {
    let raw = base64::engine::general_purpose::STANDARD
        .decode(payload)
        .ok()?;
    if raw.len() <= NONCE_LEN {
        return None;
    }
    let (nonce, ciphertext) = raw.split_at(NONCE_LEN);
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let plaintext = cipher.decrypt(Nonce::from_slice(nonce), ciphertext).ok()?;
    String::from_utf8(plaintext).ok()
}

/// Hex helpers for storing the salt in JSON (same alphabet as `random_hex`).
pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn from_hex(hex: &str) -> Option<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seal_and_open_round_trip() {
        let salt = new_salt();
        let key = derive_key("a-long-enough-admin-secret", &salt).unwrap();
        let sealed = seal(&key, "EAAB-page-token").unwrap();
        assert!(
            !sealed.contains("EAAB"),
            "ciphertext must not leak plaintext"
        );
        assert_eq!(open(&key, &sealed).as_deref(), Some("EAAB-page-token"));
    }

    #[test]
    fn wrong_key_and_tampering_fail_closed() {
        let salt = new_salt();
        let key = derive_key("a-long-enough-admin-secret", &salt).unwrap();
        let other = derive_key("another-long-enough-secret", &salt).unwrap();
        let sealed = seal(&key, "token").unwrap();
        assert!(open(&other, &sealed).is_none());
        assert!(open(&key, "not-base64!!").is_none());
        let mut raw = base64::engine::general_purpose::STANDARD
            .decode(&sealed)
            .unwrap();
        let last = raw.len() - 1;
        raw[last] ^= 0xff;
        let tampered = base64::engine::general_purpose::STANDARD.encode(raw);
        assert!(open(&key, &tampered).is_none());
    }

    #[test]
    fn short_secrets_do_not_enable_the_vault() {
        let salt = new_salt();
        assert!(derive_key("too-short", &salt).is_none());
    }

    #[test]
    fn hex_round_trip() {
        let salt = new_salt();
        assert_eq!(from_hex(&to_hex(&salt)).unwrap(), salt);
        assert!(from_hex("abc").is_none());
        assert!(from_hex("zz").is_none());
    }
}
