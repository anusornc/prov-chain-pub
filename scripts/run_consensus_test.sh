#!/bin/bash

# Build the project
echo "Building project..."
cargo build

# Create data directories
mkdir -p data/node1
mkdir -p data/node2
mkdir -p data/node3

# Generate keys
echo "Generating authority keys..."
mkdir -p data/node1
mkdir -p data/node2
mkdir -p data/node3

# Generate Node 1 key
OUTPUT1=$(cargo run --bin provchain-org -- generate-key --out data/node1/authority.key)
KEY1=$(echo "$OUTPUT1" | grep "Public key (hex):" | cut -d ' ' -f 4)
echo "Node 1 Public Key: $KEY1"

# Generate Node 2 key
OUTPUT2=$(cargo run --bin provchain-org -- generate-key --out data/node2/authority.key)
KEY2=$(echo "$OUTPUT2" | grep "Public key (hex):" | cut -d ' ' -f 4)
echo "Node 2 Public Key: $KEY2"

# Node 1 (Authority)
cat > data/node1/config.toml <<EOF
node_id = "$(uuidgen)"
[network]
network_id = "provchain-test"
listen_port = 8080
bind_address = "127.0.0.1"
known_peers = []
max_peers = 50
connection_timeout = 30
ping_interval = 30

[consensus]
is_authority = true
authority_key_file = "data/node1/authority.key"
authority_keys = ["$KEY1", "$KEY2"]
block_interval = 5
max_block_size = 1048576

[storage]
data_dir = "data/node1/storage"
persistent = true
store_type = "oxigraph"
cache_size_mb = 100

[logging]
level = "info"
format = "pretty"
EOF

# Node 2 (Authority)
cat > data/node2/config.toml <<EOF
node_id = "$(uuidgen)"
[network]
network_id = "provchain-test"
listen_port = 8081
bind_address = "127.0.0.1"
known_peers = ["127.0.0.1:8080"]
max_peers = 50
connection_timeout = 30
ping_interval = 30

[consensus]
is_authority = true
authority_key_file = "data/node2/authority.key"
authority_keys = ["$KEY1", "$KEY2"]
block_interval = 5
max_block_size = 1048576

[storage]
data_dir = "data/node2/storage"
persistent = true
store_type = "oxigraph"
cache_size_mb = 100

[logging]
level = "info"
format = "pretty"
EOF

# Node 3 (Regular Peer)
cat > data/node3/config.toml <<EOF
node_id = "$(uuidgen)"
[network]
network_id = "provchain-test"
listen_port = 8082
bind_address = "127.0.0.1"
known_peers = ["127.0.0.1:8080", "127.0.0.1:8081"]
max_peers = 50
connection_timeout = 30
ping_interval = 30

[consensus]
is_authority = false
authority_key_file = "data/node3/authority.key"
authority_keys = ["$KEY1", "$KEY2"]
block_interval = 5
max_block_size = 1048576

[storage]
data_dir = "data/node3/storage"
persistent = true
store_type = "oxigraph"
cache_size_mb = 100

[logging]
level = "info"
format = "pretty"
EOF

# Start nodes
echo "Starting Node 1..."
cargo run --bin provchain-org -- start-node --config data/node1/config.toml > data/node1/node.log 2>&1 &
PID1=$!
echo "Node 1 PID: $PID1"

sleep 2

echo "Starting Node 2..."
cargo run --bin provchain-org -- start-node --config data/node2/config.toml > data/node2/node.log 2>&1 &
PID2=$!
echo "Node 2 PID: $PID2"

sleep 2

echo "Starting Node 3..."
cargo run --bin provchain-org -- start-node --config data/node3/config.toml > data/node3/node.log 2>&1 &
PID3=$!
echo "Node 3 PID: $PID3"

echo "Nodes running. Logs are in data/node*/node.log"
echo "Press Enter to stop all nodes..."
read

kill $PID1 $PID2 $PID3
echo "Nodes stopped."
