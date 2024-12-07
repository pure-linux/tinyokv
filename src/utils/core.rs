// ================================================================================================
// This file contains the core implementation of the distributed Key-Value store. It includes the
// following components:
//
// - Storage: Encapsulation of the sled embedded database for persistent Key-Value storage.
// - RaftNode: Implementation of the Raft consensus protocol for distributed state synchronization.
// - KVService: A gRPC API implementation for client interactions with the Key-Value store.
//
// Key features of this implementation:
// - Strong consistency and fault tolerance using Raft consensus.
// - Persistent storage of Key-Value pairs with snapshot support using sled.
// - Seamless client interactions via a gRPC API based on tonic.
// - Modular design for scalability and ease of integration.
//
// Dependencies:
// - sled: Embedded database for storage.
// - raft-rs: Raft consensus implementation.
// - tonic: gRPC framework for network communication.
//
// ================================================================================================

use sled::{Db, IVec};
use raft::{prelude::*, storage::MemStorage};
use tonic::{Request, Response, Status};
use async_trait::async_trait;
use tokio::{
    net::TcpStream,
    io::{AsyncReadExt, AsyncWriteExt},
};
use tokio::sync::mpsc;
use std::sync::{Arc, Mutex};
use bincode;

pub struct Storage {
    db: Db,
}

impl Storage {
    pub fn new(path: &str) -> Self {
        let db = sled::open(path).expect("Failed to open sled DB");
        Self { db }
    }

    pub fn set(&self, key: &str, value: &[u8]) {
        self.db.insert(key, value).expect("Failed to insert key-value");
    }

    pub fn get(&self, key: &str) -> Option<IVec> {
        self.db.get(key).expect("Failed to get value")
    }

    pub fn delete(&self, key: &str) {
        self.db.remove(key).expect("Failed to delete key");
    }

    pub fn snapshot(&self) -> Vec<u8> {
        self.db.export().expect("Failed to create snapshot").to_vec()
    }

    pub fn load_snapshot(&self, snapshot: &[u8]) {
        self.db.import(snapshot).expect("Failed to load snapshot");
    }
}

pub struct RaftNode {
    raw_node: RawNode<MemStorage>,
    storage: Arc<Mutex<Storage>>,
    peers: Vec<String>,
    sender: mpsc::Sender<Message>,
}

impl RaftNode {
    pub fn new(
        id: u64,
        peers: Vec<String>,
        sender: mpsc::Sender<Message>,
        storage: Arc<Mutex<Storage>>,
    ) -> Self {
        let cfg = Config {
            id,
            ..Default::default()
        };
        let raft_storage = MemStorage::new();
        let raw_node = RawNode::new(&cfg, raft_storage, vec![]).unwrap();

        Self {
            raw_node,
            storage,
            peers,
            sender,
        }
    }

    pub async fn run(&mut self) {
        loop {
            if self.raw_node.has_ready() {
                let mut ready = self.raw_node.ready();

                for msg in ready.take_messages() {
                    self.send_to_peer(msg).await;
                }

                for entry in ready.take_entries() {
                    if let EntryType::EntryNormal = entry.get_entry_type() {
                        if !entry.data.is_empty() {
                            if let Ok(cmd) = String::from_utf8(entry.data.to_vec()) {
                                self.handle_command(cmd);
                            }
                        }
                    }
                }

                self.raw_node.advance(ready);
            }
        }
    }

    pub async fn send_to_peer(&self, msg: Message) {
        if let Some(peer) = self.peers.get((msg.to as usize) - 1) {
            if let Ok(mut stream) = TcpStream::connect(peer).await {
                if let Ok(data) = bincode::serialize(&msg) {
                    if let Err(e) = stream.write_all(&data).await {
                        eprintln!("Failed to send message to peer {}: {}", peer, e);
                    }
                }
            }
        }
    }

    pub fn handle_command(&mut self, cmd: String) {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.len() < 2 {
            return;
        }
        match parts[0] {
            "SET" => {
                if parts.len() == 3 {
                    self.storage
                        .lock()
                        .unwrap()
                        .set(parts[1], parts[2].as_bytes());
                }
            }
            "DELETE" => {
                self.storage.lock().unwrap().delete(parts[1]);
            }
            _ => {}
        }
    }

    pub fn propose(&mut self, data: Vec<u8>) {
        if let Err(e) = self.raw_node.propose(vec![], data) {
            eprintln!("Failed to propose data: {}", e);
        }
    }

    pub fn step(&mut self, msg: Message) {
        if let Err(e) = self.raw_node.step(msg) {
            eprintln!("Failed to step Raft message: {}", e);
        }
    }
}

pub mod kv_proto {
    tonic::include_proto!("kv");
}

use kv_proto::{kv_server::Kv, GetRequest, GetResponse, SetRequest, SetResponse};

#[derive(Default)]
pub struct KVService {
    pub raft_node: Arc<Mutex<RaftNode>>,
}

#[async_trait]
impl Kv for KVService {
    async fn set(
        &self,
        request: Request<SetRequest>,
    ) -> Result<Response<SetResponse>, Status> {
        let req = request.into_inner();
        let key = req.key;
        let value = req.value;

        let cmd = format!("SET {} {}", key, value);
        self.raft_node
            .lock()
            .unwrap()
            .propose(cmd.into_bytes());

        Ok(Response::new(SetResponse { success: true }))
    }

    async fn get(
        &self,
        request: Request<GetRequest>,
    ) -> Result<Response<GetResponse>, Status> {
        let req = request.into_inner();
        let key = req.key;

        if let Some(value) = self
            .raft_node
            .lock()
            .unwrap()
            .storage
            .lock()
            .unwrap()
            .get(&key)
        {
            Ok(Response::new(GetResponse {
                value: Some(String::from_utf8(value.to_vec()).unwrap()),
            }))
        } else {
            Ok(Response::new(GetResponse { value: None }))
        }
    }
}
