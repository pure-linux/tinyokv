<div align="center">
  <img width="100" height="100" src="https://avatars.githubusercontent.com/u/190339082">
  <h2>𐬺 Tinyo Key-Value Store</h2>
  <h5>Distributed High Performance.</h5>
  <p align="center">
    <a href="https://github.com/pure-linux/tinyo#vision"><b>Why</b></a> •
    <a href="https://github.com/pure-linux/tinyo#quickstart"><b>Quickstart</b></a> •
    <a href="https://discord.gg/ERKBk6ArnQ" target="_blank"><b>Discord</b></a> •
    <a href="https://x.com/PureLinux" target="_blank">𝕏</a>
  </p>
</div>

## Overview

This project implements a **distributed Key-Value store** with a focus on strong consistency, fault tolerance, and ease of use. It uses:

- **Raft Consensus Algorithm**: Ensures that all nodes in the cluster agree on the system state, even in the presence of failures.
- **sled Storage Engine**: Provides high-performance, persistent Key-Value storage with minimal overhead.
- **gRPC API**: Enables seamless communication with client applications for CRUD operations.

The system supports multi-node clusters, dynamic communication between peers, and efficient state recovery through snapshotting.

---

## Features

### Core Features
- **Distributed Consensus**: Strong consistency across nodes using Raft.
- **Persistent Storage**: Durable Key-Value storage with sled.
- **Snapshots**: Periodic state saving to reduce the size of Raft logs and enable fast recovery.
- **Fault Tolerance**: Leader election and log replication ensure high availability.
- **gRPC API**: Client-friendly interface for interacting with the Key-Value store.

### Highlights
1. **Lightweight** design for simplicity and scalability.
2. **Modular** architecture, enabling easy customization and extension.
3. **Efficient** network communication using **Raft** and **gRPC** over TCP.

---

## Architecture

The system is divided into three main layers:

### 1. Storage Layer
- **sled** is the backend database, chosen for its speed, reliability, and zero-dependency design.
- Handles:
  - CRUD operations on Key-Value pairs.
  - Snapshots for full-state persistence.
  - Recovery by loading snapshots during initialization.

### 2. Consensus Layer
- Implements **Raft** using `raft-rs`.
- Responsibilities:
  - **Leader Election**: Ensures a single leader coordinates the cluster.
  - **Log Replication**: Distributes commands from the leader to followers.
  - **State Machine Updates**: Applies committed commands to the storage layer.

### 3. Network Layer
- Built with **tonic** for gRPC communication.
- Facilitates:
  - Client interactions for `set` and `get` operations.
  - Peer-to-peer communication for Raft messages, like `AppendEntries` and `RequestVote`.

The layers work together as follows:
1. A client sends a `set` request via the gRPC API.
2. The leader node receives the request, appends it to its Raft log, and proposes the command to followers.
3. Followers replicate the log entry and acknowledge the leader.
4. Once the command is committed, the leader updates the storage layer.
5. The client receives confirmation of success.

---

## Requirements

- **Dependencies**:
  - `sled`: Embedded database for storage.
  - `raft-rs`: Raft implementation in Rust.
  - `tonic`: gRPC framework.
  - `tokio`: Asynchronous runtime.
  - `bincode`: Serialization for Raft messages.

## Usage

### 1. Start a Node
To start a node, provide a unique node ID and a list of peer addresses in the format `<IP:PORT>`.

Example: Start a node with ID `1`, and peers at `127.0.0.1:50052` and `127.0.0.1:50053`:
```bash
cargo run -- 1 127.0.0.1:50052,127.0.0.1:50053
```

The node will:

- Listen for client gRPC requests on 127.0.0.1:<50050 + node ID>.
- Establish communication with the specified peers for Raft consensus.

### 2. Client Operations
You can interact with the Key-Value store through the gRPC API. Common operations include:

#### a. Set a Key
To store a key-value pair, send a `Set` request.

**Request:**
```json
{
  "key": "username",
  "value": "admin"
}
```

**Response:**

```json
{
  "success": true
}
```

**b. Get a Key**

To retrieve a value by its key, send a Get request.

**Request:**

```json
{
  "key": "username"
}
```

**Response:**

```json
{
  "value": "admin"
}
```

**c. Delete a Key**
Currently, deletions are handled internally via Raft commands and are not exposed through the gRPC API.

### 3. Cluster Communication

- Nodes communicate over TCP to share **Raft messages** such as `AppendEntries` and `RequestVote`.
- **Leader Node**: Processes client requests and replicates commands to followers.
- **Follower Nodes**: Replicate logs and forward client requests to the leader.

## Limitations

The current **WIP** implementation has the following limitations:

1. **Dynamic Cluster Reconfiguration**: Nodes cannot be added or removed dynamically during runtime.
2. **High Throughput Optimization**: Performance under heavy workloads or high contention is not fully optimized.
3. **Partial Failure Recovery**: Beyond snapshots, advanced recovery mechanisms like log compaction or leader handoff are not yet implemented.
4. **Monitoring and Observability**: The system lacks built-in tools for monitoring cluster health, performance, and leader election status.
5. **Load Balancing**: Client requests must be manually directed to the current leader, as there is no automated load balancing or redirection mechanism.

**ℹ️ The system is a foundation for further development.**

---

**[PureLinux.org][purelinux.org]** | Delivering to the open-source community what matters most.

###### Linux® is the registered trademark of Linus Torvalds in the U.S. and other countries.

[purelinux.org]: https://purelinux.org