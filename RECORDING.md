# Recording Demo for rust-chat

This guide explains how to record an asciinema demo and convert it to a GIF.

## Prerequisites

Install the required tools:

```bash
# Install asciinema (terminal recorder)
brew install asciinema

# Install agg (asciinema to GIF converter)
brew install agg
```

## Recording Workflow

### 1. Start Recording

```bash
asciinema rec rust-chat-demo.cast
```

### 2. Run the Demo Script

Inside the asciinema recording session:

```bash
./demo.sh
```

This will:
- Build the release binary
- Create a tmux session with two panes
- Start Alice (server) in left pane
- Start Bob (client) in right pane
- Show TLS handshake messages

### 3. Type Demo Messages

Switch between panes using `Ctrl+B` then arrow keys.

**Suggested demo dialogue:**

**Left pane (Alice):**
```
Hello Bob! This connection is encrypted with TLS!
```

**Right pane (Bob):**
```
Hi Alice! Love the secure chat!
```

**Left pane (Alice):**
```
All messages use TLS 1.3 encryption
```

**Right pane (Bob):**
```
Perfect for private conversations!
```

**Left pane (Alice):**
```
exit
```

### 4. Stop Recording

After both panes close, press `Ctrl+D` to stop the asciinema recording.

### 5. Convert to GIF

```bash
# Basic conversion
agg rust-chat-demo.cast rust-chat-demo.gif

# With custom settings (recommended)
agg --font-size 16 --speed 1.5 rust-chat-demo.cast showcase.gif
```

#### Optimization Options:

- `--font-size 14-18` - Adjust for better readability
- `--speed 1.0-2.0` - Speed up slow parts
- `--cols 160` - Set terminal width
- `--rows 40` - Set terminal height
- `--theme monokai` - Color theme

### 6. Optimize GIF Size (Optional)

```bash
# Install gifsicle for optimization
brew install gifsicle

# Optimize the GIF
gifsicle -O3 --colors 256 rust-chat-demo.gif -o rust-chat-demo.gif
```

## Recording Tips

1. **Clean Terminal**: Clear your terminal before recording
2. **Timing**: Pause briefly after TLS handshake messages appear
3. **Typing Speed**: Type at a natural, moderate pace
4. **Terminal Size**: Use a standard size (160x40 or 120x30)
5. **Multiple Takes**: You can record multiple times - the script is reusable

## Quick Record Script

For convenience, create a one-liner:

```bash
chmod +x demo.sh
asciinema rec rust-chat-demo.cast
# Then inside: ./demo.sh
# Type your messages
# Exit both panes
# Ctrl+D to stop
agg --font-size 16 --speed 1.5 rust-chat-demo.cast showcase.gif
```

## Troubleshooting

**If tmux session already exists:**
```bash
tmux kill-session -t rust-chat-demo
```

**To preview the recording before converting:**
```bash
asciinema play rust-chat-demo.cast
```

**To adjust GIF speed after creation:**
```bash
agg --speed 2.0 rust-chat-demo.cast rust-chat-demo-fast.gif
```

## Final Steps

Once you have the perfect `showcase.gif`:

1. Copy it to your repo root
2. Commit and push
3. The README.md already references it: `![Demo](showcase.gif)`
