use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{create_hash, hash_to_binary, NETWORK_DIFFICULY};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub id: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: i64,
    pub data: String,
    pub nonce: u64,
}

impl Block {
    pub fn new(id: u64, previous_hash: String, data: String) -> Self {
        let now = Utc::now();
        let (nonce, hash) = Block::mine_block(id, now.timestamp(), &previous_hash, &data);

        Self {
            id,
            hash,
            timestamp: now.timestamp(),
            previous_hash,
            data,
            nonce,
        }
    }

    fn mine_block(id: u64, timestamp: i64, previous_hash: &str, data: &str) -> (u64, String) {
        println!("Mining block {}", id);
        let mut nonce = 0;

        loop {
            if nonce % 100000 == 0 {
                println!("nonce: {}", nonce);
            }

            let hash = create_hash(id, timestamp, previous_hash, data, nonce);
            let binary_hash = hash_to_binary(&hash);

            if binary_hash.starts_with(NETWORK_DIFFICULY) {
                println!(
                    "Block mined! nonce: {}, hash: {}",
                    nonce,
                    hex::encode(&hash)
                );

                return (nonce, hex::encode(&hash));
            }

            nonce += 1;
        }
    }
}
