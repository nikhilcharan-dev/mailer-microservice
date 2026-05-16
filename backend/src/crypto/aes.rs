use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{anyhow, Context, Result};
use rand::RngCore;
use std::collections::HashMap;

pub struct EncryptionKeyring {
    keys: HashMap<u32, [u8; 32]>,
    current: u32,
}

impl EncryptionKeyring {
    /// Loads keys from environment.
    /// Reads `ENCRYPTION_KEY_V{n}` (hex-encoded 32 bytes) for n = 1..=current
    /// and `ENCRYPTION_KEY_CURRENT` as the active version.
    /// Falls back to `ENCRYPTION_KEY` as version 1 if no versioned vars exist.
    pub fn from_env() -> Result<Self> {
        let mut keys = HashMap::new();

        if let Ok(current_str) = std::env::var("ENCRYPTION_KEY_CURRENT") {
            let current: u32 = current_str
                .parse()
                .context("ENCRYPTION_KEY_CURRENT must be an integer")?;
            for v in 1..=current {
                let var = format!("ENCRYPTION_KEY_V{v}");
                let hex = std::env::var(&var)
                    .with_context(|| format!("missing {var}"))?;
                let bytes = hex::decode(hex.trim())
                    .with_context(|| format!("{var} must be hex"))?;
                if bytes.len() != 32 {
                    return Err(anyhow!("{var} must decode to 32 bytes"));
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                keys.insert(v, arr);
            }
            Ok(Self { keys, current })
        } else {
            // Single-key fallback
            let hex = std::env::var("ENCRYPTION_KEY")
                .context("ENCRYPTION_KEY (or ENCRYPTION_KEY_V1 + ENCRYPTION_KEY_CURRENT) required")?;
            let bytes = hex::decode(hex.trim()).context("ENCRYPTION_KEY must be hex")?;
            if bytes.len() != 32 {
                return Err(anyhow!("ENCRYPTION_KEY must decode to 32 bytes"));
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            keys.insert(1, arr);
            Ok(Self { keys, current: 1 })
        }
    }

    #[allow(dead_code)] // useful for admin/diagnostic tooling
    pub fn current_version(&self) -> u32 {
        self.current
    }

    /// Encrypt with the current key. Returns (ciphertext, nonce, key_version).
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>, u32)> {
        let key = self
            .keys
            .get(&self.current)
            .ok_or_else(|| anyhow!("current key not loaded"))?;
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| anyhow!("aes encrypt failed: {e}"))?;
        Ok((ciphertext, nonce_bytes.to_vec(), self.current))
    }

    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8], key_version: u32) -> Result<Vec<u8>> {
        let key = self
            .keys
            .get(&key_version)
            .ok_or_else(|| anyhow!("key version {key_version} not loaded"))?;
        if nonce.len() != 12 {
            return Err(anyhow!("nonce must be 12 bytes"));
        }
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let plaintext = cipher
            .decrypt(Nonce::from_slice(nonce), ciphertext)
            .map_err(|e| anyhow!("aes decrypt failed: {e}"))?;
        Ok(plaintext)
    }
}
