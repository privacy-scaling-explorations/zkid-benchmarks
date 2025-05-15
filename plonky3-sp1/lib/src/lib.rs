use sha2::{Digest, Sha256};

pub fn sha2(input: &[u8]) -> u32 {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let _ = hasher.finalize();
    1
}
