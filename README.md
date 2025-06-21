# cc-hook

A terminal wrapper that monitors output and triggers commands based on regex patterns.

## Usage

```bash
# Show config file path
cc-hook config-path

# Run a command with default config
cc-hook run <command> [args...]

# Run with custom config
cc-hook -c <config.toml> <command> [args...]

# Implicit run (shorthand)
cc-hook <command> [args...]
```

## Configuration

Default config location: `~/.config/cc-hook/config.toml`

Example config:
```toml
[[hooks]]
name = "permission_prompt"
regex = "Do you want to proceed\\?"
command = "notify-send '[cc-hook] Permission required'"

[[hooks]]
name = "task_finished"
regex = "^●.*"
command = "notify-send '[cc-hook] Task completed'"
```

## Build

```bash
cargo build --release
```

## Example

```bash
# Create a test config
cat > test-config.toml << 'EOF'
[[hooks]]
name = "hello_detection"
regex = "Hello"
command = "echo 'Detected Hello!'"
EOF

# Run with the test config
./target/release/cc-hook -c test-config.toml echo "Hello World"
```