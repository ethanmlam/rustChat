# rust-chat

A simple TCP chat application for the terminal

![Demo](showcase.gif)

## Build

```bash
cargo build --release
```

## Usage

Start a server:
```bash
./target/release/rust-chat Alice start 127.0.0.1 8080
```

Connect as a client:
```bash
./target/release/rust-chat Bob connect 127.0.0.1 8080
```

Type messages and press Enter. Type `exit` to quit.

## Features

- Color-coded usernames
- Timestamps on messages
- Real-time TLS-encrypted communication
- Raw terminal mode for clean input
- Self-signed certificate generation
- Private and secure peer-to-peer chat

## Security

All communications are encrypted using TLS 1.3. On first run, the server automatically generates a self-signed certificate (`server-cert.pem` and `server-key.pem`) for encryption. While these self-signed certificates provide strong encryption, they are not verified by a certificate authority - this is perfect for private, peer-to-peer chat between trusted parties.
