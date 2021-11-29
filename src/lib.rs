use sha2::{Digest, Sha256};

pub mod block;
pub mod chain;

pub const NETWORK_DIFFICULY: &str = "000";

pub fn hash_to_binary(hash: &[u8]) -> String {
    let mut res = String::default();

    for character in hash {
        res.push_str(&format!("{:b}", character));
    }

    res
}

pub fn create_hash(
    id: u64,
    timestamp: i64,
    previous_hash: &str,
    data: &str,
    nonce: u64,
) -> Vec<u8> {
    let data = serde_json::json!({
        "id": id,
        "previous_hash": previous_hash,
        "data": data,
        "timestamp": timestamp,
        "nonce": nonce,
    });

    let mut hasher = Sha256::new();
    hasher.update(data.to_string().as_bytes());
    hasher.finalize().as_slice().to_owned()
}
