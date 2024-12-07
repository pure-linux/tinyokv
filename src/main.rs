// ================================================================================================
// This file is the entry point of the distributed Key-Value store. It initializes and starts the
// Raft-based Key-Value service. The file performs the following tasks:
//
// - Parses command-line arguments to identify the node's ID and its peers in the cluster.
// - Sets up the Raft node for consensus and state synchronization.
// - Configures the gRPC server for client interactions using tonic.
// - Launches the main Raft processing loop in a separate asynchronous task.
//
// Key components imported from core.rs:
// - Storage: Persistent storage layer for Key-Value pairs.
// - RaftNode: Consensus and state synchronization based on Raft.
// - KVService: gRPC API implementation for client operations.
//
// ================================================================================================

use tokio::sync::mpsc;
use utils::core::{RaftNode, Storage, KVService};
use tonic::transport::Server;
use std::sync::{Arc, Mutex};

mod utils {
    pub mod core;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments for node ID and peer information.
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <node_id> <peer1,peer2,...>", args[0]);
        std::process::exit(1);
    }

    let id: u64 = args[1].parse().expect("Invalid node ID");
    let peers: Vec<String> = args[2].split(',').map(String::from).collect();

    // Define the gRPC server address and initialize the sled-based storage.
    let addr = format!("127.0.0.1:{}", 50050 + id).parse().unwrap();
    let storage = Arc::new(Mutex::new(Storage::new(&format!("data_{}", id))));
    let (raft_tx, _raft_rx) = mpsc::channel(100);

    // Initialize the Raft node with the given ID and peer configuration.
    let raft_node = Arc::new(Mutex::new(RaftNode::new(
        id,
        peers.clone(),
        raft_tx.clone(),
        storage.clone(),
    )));

    // Launch the main Raft processing loop in a separate task.
    let raft_node_cloned = raft_node.clone();
    tokio::spawn(async move {
        raft_node_cloned.lock().unwrap().run().await;
    });

    // Start the gRPC server for handling client requests.
    let kv_service = KVService {
        raft_node: raft_node.clone(),
    };

    println!("KV Service running on {}", addr);

    Server::builder()
        .add_service(kv_proto::kv_server::KvServer::new(kv_service))
        .serve(addr)
        .await?;

    Ok(())
}
