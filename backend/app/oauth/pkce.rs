use sha2::{Digest, Sha256};

pub fn verify_s256(code_verifier: &str, code_challenge: &str) -> bool {
    let digest = Sha256::digest(code_verifier.as_bytes());
    let computed = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        digest,
    );
    computed == code_challenge
}

pub fn random_token(len: usize) -> String {
    use rand::RngCore;
    let mut bytes = vec![0u8; len];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}
