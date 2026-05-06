use anyhow::{anyhow, Result};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};

pub type DemoKey = [u8; 32];

pub fn hash_bytes(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

pub fn demo_key_from_passphrase(passphrase: &str) -> DemoKey {
    let mut key = [0_u8; 32];
    key.copy_from_slice(
        blake3::derive_key("clearmesh-v2-demo-passphrase-key", passphrase.as_bytes()).as_slice(),
    );
    key
}

pub fn encrypt_chunk(key: &DemoKey, plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let nonce_digest = blake3::keyed_hash(key, plaintext);
    let nonce = Nonce::from_slice(&nonce_digest.as_bytes()[..12]);
    let mut ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|_| anyhow!("chunk encryption failed"))?;
    let mut out = nonce.to_vec();
    out.append(&mut ciphertext);
    Ok(out)
}

pub fn decrypt_chunk(key: &DemoKey, encrypted: &[u8]) -> Result<Vec<u8>> {
    if encrypted.len() < 12 {
        return Err(anyhow!("encrypted chunk is too short"));
    }
    let (nonce_bytes, ciphertext) = encrypted.split_at(12);
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow!("chunk decryption failed"))
}

#[cfg(test)]
mod tests {
    use super::{decrypt_chunk, demo_key_from_passphrase, encrypt_chunk, hash_bytes};

    #[test]
    fn encrypt_decrypt_round_trip() {
        let key = demo_key_from_passphrase("correct horse battery staple");
        let encrypted = encrypt_chunk(&key, b"private bytes").unwrap();
        assert_ne!(encrypted, b"private bytes");
        let decrypted = decrypt_chunk(&key, &encrypted).unwrap();
        assert_eq!(decrypted, b"private bytes");
    }

    #[test]
    fn encryption_is_stable_for_same_key_and_plaintext() {
        let key = demo_key_from_passphrase("correct horse battery staple");
        assert_eq!(
            encrypt_chunk(&key, b"same bytes").unwrap(),
            encrypt_chunk(&key, b"same bytes").unwrap()
        );
    }

    #[test]
    fn hashes_are_blake3_hex() {
        assert_eq!(
            hash_bytes(b"abc"),
            "6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85"
        );
    }
}
