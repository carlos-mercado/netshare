# netshare

A peer-to-peer LAN chat application written in Rust. Discover other users on your local network and engage in real-time text chat.

## Features

- **Automatic Peer Discovery**: Uses UDP broadcast to find other netshare instances on your LAN
- **Real-time Bidirectional Chat**: Multi-threaded design allows simultaneous sending and receiving
- **Cross-Platform**: Works on Linux, macOS?, and Windows

## Requirements

- Rust 1.85 or later (Edition 2024)

## Installation

```bash
git clone https://github.com/carlos-mercado/netshare.git
cd netshare
cargo build --release
```

The compiled binary will be at `./target/release/netshare`.

## Usage

Run the application:

```bash
cargo run
# or
./target/release/netshare
```

### Modes

**Listen Mode**

Waits for incoming chat requests from other users on the network. Select this option if you want others to connect to you.

**Send Mode**

Broadcasts a discovery request to find other listeners on the network. Once discovered, you can select a peer to connect to.

### Chat

Once connected:
- Type your message and press `Enter` to send
- Type `/q` to quit the chat session

## How It Works

1. **Discovery**: The sender broadcasts a UDP packet to the network's broadcast address (port 14953)
2. **Response**: Listeners receive the broadcast and respond with their IP address
3. **Connection**: A TCP connection is established between the two peers on port 14953
4. **Chat**: Messages are exchanged over the TCP connection in real-time

## Technical Details

- **Language**: Rust (Edition 2024)
- **UDP Port**: 14953 (peer discovery)
- **TCP Port**: 14953 (chat connection)

### Dependencies

| Crate | Purpose |
|-------|---------|
| crossterm | Terminal UI manipulation |
| clearscreen | Cross-platform terminal clearing |
| getifaddrs | Network interface information |
| local-ip-address | Local IPv4 address detection |

## Limitations

- No encryption.
- Single chat session at a time
- Requires both peers to be on the same local network

## Development

This project is in its early stages and mostly serves as a learning exercise for Rust's more advanced features.
