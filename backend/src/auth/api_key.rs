use rand::RngCore;

/// Generate a 32-byte random API key encoded as 64 hex chars.
pub fn generate() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}
