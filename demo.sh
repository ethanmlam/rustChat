#!/bin/bash

# Automated tmux demo setup for rust-chat with TLS

echo "Building rust-chat..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

echo "Setting up tmux demo session..."

# Kill any existing demo session
tmux kill-session -t rust-chat-demo 2>/dev/null

# Create new tmux session with split panes
tmux new-session -d -s rust-chat-demo -n demo

# Split window vertically (side by side)
tmux split-window -h -t rust-chat-demo:demo

# Set pane titles (if supported)
tmux select-pane -t rust-chat-demo:demo.0 -T "Server (Alice)"
tmux select-pane -t rust-chat-demo:demo.1 -T "Client (Bob)"

# Left pane (0): Run server as Alice
tmux send-keys -t rust-chat-demo:demo.0 './target/release/rust-chat Alice start 127.0.0.1 8080' Enter

# Wait for server to start
sleep 2

# Right pane (1): Run client as Bob
tmux send-keys -t rust-chat-demo:demo.1 './target/release/rust-chat Bob connect 127.0.0.1 8080' Enter

# Wait a moment for connection
sleep 1

echo ""
echo "✓ Demo session ready!"
echo ""
echo "To attach:  tmux attach-session -t rust-chat-demo"
echo "To detach:  Ctrl+B then D"
echo "To exit:    Type 'exit' in either pane"
echo ""

# Attach to the session
tmux attach-session -t rust-chat-demo
