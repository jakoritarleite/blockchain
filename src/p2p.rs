use libp2p::{
    floodsub::{Floodsub, FloodsubEvent, Topic},
    identity,
    mdns::{Mdns, MdnsEvent},
    swarm::NetworkBehaviourEventProcess,
    NetworkBehaviour, PeerId,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{block::Block, chain::Blockchain};

pub static KEYS: Lazy<identity::Keypair> = Lazy::new(identity::Keypair::generate_ed25519);
pub static PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(KEYS.public()));
pub static CHAIN_TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("chains"));
pub static BLOCK_TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("blocks"));

#[derive(Serialize, Deserialize, Debug)]
pub struct ChainResponse {
    pub blocks: Vec<Block>,
    pub receiver: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalChainRequest {
    pub from_peer_id: String,
}

pub enum EventType {
    LocalChainResponse(ChainResponse),
    Input(String),
    Init,
}

#[derive(NetworkBehaviour)]
pub struct BlockchainBehaviour {
    pub floodsub: Floodsub,
    pub mdns: Mdns,
    #[behaviour(ignore)]
    pub response_sender: mpsc::UnboundedSender<ChainResponse>,
    #[behaviour(ignore)]
    pub init_sender: mpsc::UnboundedSender<bool>,
    #[behaviour(ignore)]
    pub blockchain: Blockchain,
}

impl BlockchainBehaviour {
    pub async fn new(
        blockchain: Blockchain,
        response_sender: mpsc::UnboundedSender<ChainResponse>,
        init_sender: mpsc::UnboundedSender<bool>,
    ) -> Self {
        let mut behaviour = Self {
            blockchain,
            floodsub: Floodsub::new(*PEER_ID),
            mdns: Mdns::new(Default::default())
                .await
                .expect("can create mdns"),
            response_sender,
            init_sender,
        };

        behaviour.floodsub.subscribe(CHAIN_TOPIC.clone());
        behaviour.floodsub.subscribe(BLOCK_TOPIC.clone());

        behaviour
    }
}

impl NetworkBehaviourEventProcess<MdnsEvent> for BlockchainBehaviour {
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(discovered_list) => {
                for (peer, _addr) in discovered_list {
                    self.floodsub.add_node_to_partial_view(peer);
                }
            }
            MdnsEvent::Expired(expired_list) => {
                for (peer, _addr) in expired_list {
                    if !self.mdns.has_node(&peer) {
                        self.floodsub.remove_node_from_partial_view(&peer);
                    }
                }
            }
        }
    }
}

impl NetworkBehaviourEventProcess<FloodsubEvent> for BlockchainBehaviour {
    fn inject_event(&mut self, event: FloodsubEvent) {
        if let FloodsubEvent::Message(msg) = event {
            if let Ok(resp) = serde_json::from_slice::<ChainResponse>(&msg.data) {
                if resp.receiver == PEER_ID.to_string() {
                    println!("response from {}", msg.source);

                    resp.blocks.iter().for_each(|block| println!("{:?}", block));
                    self.blockchain.blocks = self
                        .blockchain
                        .choose_chain(self.blockchain.blocks.clone(), resp.blocks);
                }
            } else if let Ok(resp) = serde_json::from_slice::<LocalChainRequest>(&msg.data) {
                println!("sending local chain to {}", msg.source);

                let peer_id = resp.from_peer_id;
                if PEER_ID.to_string() == peer_id {
                    if let Err(err) = self.response_sender.send(ChainResponse {
                        blocks: self.blockchain.blocks.clone(),
                        receiver: msg.source.to_string(),
                    }) {
                        println!("error sending response via channel {}", err);
                    }
                }
            } else if let Ok(block) = serde_json::from_slice::<Block>(&msg.data) {
                println!("received new block from {}", msg.source);

                self.blockchain.try_to_add_a_block(block);
            }
        }
    }
}
