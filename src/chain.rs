use chrono::Utc;

use crate::block::Block;
use crate::{create_hash, hash_to_binary, NETWORK_DIFFICULY};

pub struct Blockchain {
    pub blocks: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        Self { blocks: vec![] }
    }

    pub fn genesis(&mut self) {
        let genesis_block = Block {
            id: 0,
            timestamp: Utc::now().timestamp(),
            previous_hash: String::from("genesis"),
            data: String::from("two pizzas!"),
            nonce: 0090,
            hash: String::from("d9c05ea826335b6a0d53170e56e2ff241fd222c71439c0c49faffa638edad56c"),
        };
        self.blocks.push(genesis_block);
    }

    pub fn is_block_valid(&self, block: &Block, previous_block: &Block) -> bool {
        if block.previous_hash != previous_block.hash {
            println!("block with id: {} has wrong previous hash", block.id);
            return false;
        } else if !hash_to_binary(&hex::decode(&block.hash).expect("Can decode from hex"))
            .starts_with(NETWORK_DIFFICULY)
        {
            return false;
        } else if block.id != previous_block.id + 1 {
            println!(
                "block with id: {} is not the next block after the latest: {}",
                block.id, previous_block.id
            );
        } else if hex::encode(create_hash(
            block.id,
            block.timestamp,
            &block.previous_hash,
            &block.data,
            block.nonce,
        )) != block.hash
        {
            println!("block with id: {} has invalid hash", block.id);
        }

        true
    }

    pub fn try_to_add_a_block(&mut self, block: Block) {
        let last_block = self
            .blocks
            .last()
            .expect("There should be at least one block");

        if self.is_block_valid(&block, last_block) {
            self.blocks.push(block);
        } else {
            println!("Could not add block");
        }
    }

    pub fn is_chain_valid(&self, chain: &[Block]) -> bool {
        for block_index in 0..chain.len() {
            if block_index == 0 {
                continue;
            }

            let first = chain.get(block_index - 1).expect("has to exist");
            let second = chain.get(block_index).expect("has to exist");

            if !self.is_block_valid(second, first) {
                return false;
            }
        }

        true
    }

    pub fn choose_chain(&mut self, local: Vec<Block>, remote: Vec<Block>) -> Vec<Block> {
        let is_local_valid = self.is_chain_valid(&local);
        let is_remote_valid = self.is_chain_valid(&remote);

        if is_local_valid && is_remote_valid {
            if local.len() >= remote.len() {
                local
            } else {
                remote
            }
        } else if !is_local_valid && is_remote_valid {
            remote
        } else if is_local_valid && !is_remote_valid {
            local
        } else {
            panic!("local and remote chains are both invalid ???");
        }
    }
}
