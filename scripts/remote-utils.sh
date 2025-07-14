#!/bin/bash

# ==============================================================================
# ICN Remote Infrastructure Utilities
# ==============================================================================
# Comprehensive remote infrastructure management for ICN deployment
# Handles SSH configuration, multi-node deployment, and federation orchestration

# ==============================================================================
# Remote Infrastructure Configuration
# ==============================================================================

# Define remote node configurations
declare -A REMOTE_NODES=(
    ["bootstrap"]="10.8.10.11:22:bootstrap"
    ["worker1"]="10.8.10.12:22:worker"
    ["worker2"]="10.8.10.13:22:worker"
    ["gateway"]="10.8.10.1:22:gateway"
    ["auth"]="10.8.10.2:22:auth"
)

# Node role configurations
declare -A NODE_ROLES=(
    ["bootstrap"]="primary federation coordinator"
    ["worker"]="mesh computation and storage"
    ["gateway"]="external network interface"
    ["auth"]="identity and authentication"
)

# Deployment configurations
declare -A DEPLOYMENT_CONFIGS=(
    ["bootstrap"]="icn-node:8080:full"
    ["worker"]="icn-node:8080:worker"
    ["gateway"]="icn-gateway:80:proxy"
    ["auth"]="icn-auth:3000:identity"
)

# ==============================================================================
# SSH Configuration and Management
# ==============================================================================

# Setup SSH configuration
setup_ssh_config() {
    local profile="$1"
    
    log "INFO" "Setting up SSH configuration for profile: $profile" "REMOTE"
    
    # Create SSH directory if it doesn't exist
    mkdir -p "$HOME/.ssh"
    chmod 700 "$HOME/.ssh"
    
    # Generate SSH key if it doesn't exist
    local ssh_key_path="$HOME/.ssh/icn_$profile"
    if [[ ! -f "$ssh_key_path" ]]; then
        log "INFO" "Generating SSH key for ICN infrastructure" "REMOTE"
        ssh-keygen -t ed25519 -f "$ssh_key_path" -N "" -C "icn-infrastructure-$profile-$(date +%Y%m%d)"
        chmod 600 "$ssh_key_path"
        chmod 644 "$ssh_key_path.pub"
    fi
    
    # Generate SSH config
    generate_ssh_config "$profile" "$ssh_key_path"
    
    # Setup SSH agent
    setup_ssh_agent "$ssh_key_path"
    
    log "SUCCESS" "SSH configuration completed" "REMOTE"
}

# Generate SSH configuration file
generate_ssh_config() {
    local profile="$1"
    local ssh_key_path="$2"
    
    log "INFO" "Generating SSH configuration" "REMOTE"
    
    local ssh_config_file="$HOME/.ssh/config.icn"
    
    cat > "$ssh_config_file" << EOF
# ICN Infrastructure SSH Configuration
# Profile: $profile
# Generated on: $(date)

# Global SSH settings for ICN infrastructure
Host icn-*
    User icn
    IdentityFile $ssh_key_path
    StrictHostKeyChecking no
    UserKnownHostsFile /dev/null
    LogLevel ERROR
    ConnectTimeout 10
    ServerAliveInterval 60
    ServerAliveCountMax 3

EOF
    
    # Add configuration for each node
    for node_name in "${!REMOTE_NODES[@]}"; do
        local node_config="${REMOTE_NODES[$node_name]}"
        IFS=':' read -r ip port role <<< "$node_config"
        
        cat >> "$ssh_config_file" << EOF
# $node_name node ($role)
Host icn-$node_name
    HostName $ip
    Port $port
    User icn

EOF
    done
    
    # Include ICN config in main SSH config
    if ! grep -q "Include.*config.icn" "$HOME/.ssh/config" 2>/dev/null; then
        echo "Include ~/.ssh/config.icn" | cat - "$HOME/.ssh/config" > /tmp/ssh_config.tmp 2>/dev/null && mv /tmp/ssh_config.tmp "$HOME/.ssh/config" || echo "Include ~/.ssh/config.icn" > "$HOME/.ssh/config"
    fi
    
    log "SUCCESS" "SSH configuration generated: $ssh_config_file" "REMOTE"
}

# Setup SSH agent
setup_ssh_agent() {
    local ssh_key_path="$1"
    
    log "INFO" "Setting up SSH agent" "REMOTE"
    
    # Start SSH agent if not running
    if [[ -z "$SSH_AUTH_SOCK" ]]; then
        eval "$(ssh-agent -s)"
    fi
    
    # Add SSH key to agent
    ssh-add "$ssh_key_path" 2>/dev/null || {
        log "WARN" "Failed to add SSH key to agent" "REMOTE"
    }
    
    log "SUCCESS" "SSH agent configured" "REMOTE"
}

# Test SSH connectivity to all nodes
test_ssh_connectivity() {
    local profile="$1"
    
    log "INFO" "Testing SSH connectivity to remote nodes" "REMOTE"
    
    local connected_nodes=0
    local total_nodes=0
    
    for node_name in "${!REMOTE_NODES[@]}"; do
        ((total_nodes++))
        
        log "INFO" "Testing connection to: $node_name" "REMOTE"
        
        if ssh -o ConnectTimeout=5 "icn-$node_name" "echo 'SSH connection successful'" &>/dev/null; then
            log "SUCCESS" "Connection successful: $node_name" "REMOTE"
            ((connected_nodes++))
        else
            log "ERROR" "Connection failed: $node_name" "REMOTE"
        fi
    done
    
    local connectivity_percentage=$((connected_nodes * 100 / total_nodes))
    
    log "INFO" "SSH connectivity: $connected_nodes/$total_nodes nodes ($connectivity_percentage%)" "REMOTE"
    
    if [[ "$connectivity_percentage" -ge 80 ]]; then
        log "SUCCESS" "SSH connectivity check passed" "REMOTE"
        return 0
    else
        log "ERROR" "SSH connectivity check failed" "REMOTE"
        return 1
    fi
}

# ==============================================================================
# Remote Node Management
# ==============================================================================

# Setup remote nodes
setup_remote_nodes() {
    local profile="$1"
    
    log "INFO" "Setting up remote nodes for profile: $profile" "REMOTE"
    
    # Test SSH connectivity first
    if ! test_ssh_connectivity "$profile"; then
        log "ERROR" "SSH connectivity test failed. Cannot proceed with remote setup." "REMOTE"
        return 1
    fi
    
    # Setup each node based on its role
    for node_name in "${!REMOTE_NODES[@]}"; do
        setup_remote_node "$node_name" "$profile"
    done
    
    log "SUCCESS" "Remote nodes setup completed" "REMOTE"
}

# Setup individual remote node
setup_remote_node() {
    local node_name="$1"
    local profile="$2"
    
    log "INFO" "Setting up remote node: $node_name" "REMOTE"
    
    local node_config="${REMOTE_NODES[$node_name]}"
    IFS=':' read -r ip port role <<< "$node_config"
    
    # Install system dependencies
    install_node_dependencies "$node_name"
    
    # Setup ICN user and directories
    setup_node_user "$node_name"
    
    # Transfer ICN binaries
    transfer_icn_binaries "$node_name" "$profile"
    
    # Configure node for its role
    configure_node_role "$node_name" "$role" "$profile"
    
    # Setup systemd service
    setup_node_service "$node_name" "$role" "$profile"
    
    log "SUCCESS" "Remote node setup completed: $node_name" "REMOTE"
}

# Install dependencies on remote node
install_node_dependencies() {
    local node_name="$1"
    
    log "INFO" "Installing dependencies on: $node_name" "REMOTE"
    
    ssh "icn-$node_name" << 'EOF'
# Update system packages
sudo apt-get update

# Install required packages
sudo apt-get install -y \
    curl \
    wget \
    git \
    htop \
    tmux \
    jq \
    prometheus-node-exporter \
    logrotate \
    fail2ban

# Install Docker if not present
if ! command -v docker &> /dev/null; then
    curl -fsSL https://get.docker.com | sh
    sudo usermod -aG docker $USER
fi

# Install Rust if not present
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
fi
EOF
    
    if [[ $? -eq 0 ]]; then
        log "SUCCESS" "Dependencies installed on: $node_name" "REMOTE"
    else
        log "ERROR" "Failed to install dependencies on: $node_name" "REMOTE"
        return 1
    fi
}

# Setup ICN user and directories on remote node
setup_node_user() {
    local node_name="$1"
    
    log "INFO" "Setting up ICN user on: $node_name" "REMOTE"
    
    ssh "icn-$node_name" << 'EOF'
# Create ICN directories
sudo mkdir -p /opt/icn/{bin,config,data,logs,keys}
sudo mkdir -p /var/log/icn
sudo mkdir -p /etc/icn

# Set permissions
sudo chown -R icn:icn /opt/icn
sudo chown -R icn:icn /var/log/icn
sudo chown -R icn:icn /etc/icn

# Create systemd directory
sudo mkdir -p /etc/systemd/system
EOF
    
    if [[ $? -eq 0 ]]; then
        log "SUCCESS" "ICN user setup completed on: $node_name" "REMOTE"
    else
        log "ERROR" "Failed to setup ICN user on: $node_name" "REMOTE"
        return 1
    fi
}

# Transfer ICN binaries to remote node
transfer_icn_binaries() {
    local node_name="$1"
    local profile="$2"
    
    log "INFO" "Transferring ICN binaries to: $node_name" "REMOTE"
    
    # Build binaries if they don't exist
    if [[ ! -f "$PROJECT_ROOT/target/release/icn-node" ]]; then
        log "INFO" "Building ICN binaries" "REMOTE"
        cd "$PROJECT_ROOT"
        cargo build --release --workspace
    fi
    
    # Transfer binaries
    local binaries=("icn-node" "icn-cli")
    for binary in "${binaries[@]}"; do
        if [[ -f "$PROJECT_ROOT/target/release/$binary" ]]; then
            log "INFO" "Transferring binary: $binary" "REMOTE"
            scp "$PROJECT_ROOT/target/release/$binary" "icn-$node_name:/opt/icn/bin/"
            ssh "icn-$node_name" "chmod +x /opt/icn/bin/$binary"
        else
            log "WARN" "Binary not found: $binary" "REMOTE"
        fi
    done
    
    # Transfer configuration files
    transfer_node_configs "$node_name" "$profile"
    
    log "SUCCESS" "ICN binaries transferred to: $node_name" "REMOTE"
}

# Transfer configuration files to remote node
transfer_node_configs() {
    local node_name="$1"
    local profile="$2"
    
    log "INFO" "Transferring configuration files to: $node_name" "REMOTE"
    
    # Create temporary config directory
    local temp_config_dir="/tmp/icn-config-$node_name"
    mkdir -p "$temp_config_dir"
    
    # Generate node-specific configuration
    generate_node_config "$node_name" "$profile" "$temp_config_dir"
    
    # Transfer configuration files
    scp -r "$temp_config_dir"/* "icn-$node_name:/etc/icn/"
    
    # Set proper permissions
    ssh "icn-$node_name" "sudo chown -R icn:icn /etc/icn && sudo chmod 600 /etc/icn/*.toml"
    
    # Cleanup temporary directory
    rm -rf "$temp_config_dir"
    
    log "SUCCESS" "Configuration files transferred to: $node_name" "REMOTE"
}

# Generate node-specific configuration
generate_node_config() {
    local node_name="$1"
    local profile="$2"
    local output_dir="$3"
    
    log "INFO" "Generating configuration for: $node_name" "REMOTE"
    
    local node_config="${REMOTE_NODES[$node_name]}"
    IFS=':' read -r ip port role <<< "$node_config"
    
    # Generate main configuration file
    cat > "$output_dir/icn.toml" << EOF
# ICN Node Configuration
# Node: $node_name
# Role: $role
# Profile: $profile
# Generated on: $(date)

[node]
id = "$node_name"
role = "$role"
profile = "$profile"

[network]
bind_address = "0.0.0.0"
port = 8080
external_address = "$ip"
external_port = 8080

[federation]
name = "icn-cooperative"
discovery_timeout = 30

[database]
url = "sqlite:///opt/icn/data/icn.db"

[logging]
level = "info"
file = "/var/log/icn/icn.log"
format = "json"

[mana]
regeneration_rate = 100
max_capacity = 10000

EOF
    
    # Add role-specific configuration
    case "$role" in
        "bootstrap")
            cat >> "$output_dir/icn.toml" << EOF

[bootstrap]
enabled = true
bootstrap_nodes = []

[governance]
enabled = true
voting_threshold = 0.67
proposal_timeout = "168h"

[economics]
enabled = true
mana_distribution = true
EOF
            ;;
        "worker")
            cat >> "$output_dir/icn.toml" << EOF

[worker]
enabled = true
bootstrap_nodes = ["10.8.10.11:8080"]

[mesh]
enabled = true
max_concurrent_jobs = 10
resource_limits.cpu = 4
resource_limits.memory = "8GB"
resource_limits.disk = "100GB"
EOF
            ;;
        "gateway")
            cat >> "$output_dir/icn.toml" << EOF

[gateway]
enabled = true
upstream_nodes = ["10.8.10.11:8080", "10.8.10.12:8080", "10.8.10.13:8080"]
rate_limiting = true
rate_limit = 1000

[proxy]
enabled = true
timeout = 30
EOF
            ;;
        "auth")
            cat >> "$output_dir/icn.toml" << EOF

[identity]
enabled = true
did_method = "icn"
key_rotation_interval = "30d"

[authentication]
enabled = true
token_lifetime = "24h"
refresh_token_lifetime = "30d"
EOF
            ;;
    esac
    
    # Generate environment file
    cat > "$output_dir/icn.env" << EOF
# ICN Environment Configuration
# Node: $node_name
# Role: $role

ICN_NODE_ID=$node_name
ICN_NODE_ROLE=$role
ICN_PROFILE=$profile
ICN_LOG_LEVEL=info
ICN_CONFIG_FILE=/etc/icn/icn.toml
EOF
    
    log "SUCCESS" "Configuration generated for: $node_name" "REMOTE"
}

# Configure node for its specific role
configure_node_role() {
    local node_name="$1"
    local role="$2"
    local profile="$3"
    
    log "INFO" "Configuring node role: $node_name ($role)" "REMOTE"
    
    case "$role" in
        "bootstrap")
            configure_bootstrap_node "$node_name" "$profile"
            ;;
        "worker")
            configure_worker_node "$node_name" "$profile"
            ;;
        "gateway")
            configure_gateway_node "$node_name" "$profile"
            ;;
        "auth")
            configure_auth_node "$node_name" "$profile"
            ;;
    esac
    
    log "SUCCESS" "Node role configured: $node_name ($role)" "REMOTE"
}

# Configure bootstrap node
configure_bootstrap_node() {
    local node_name="$1"
    local profile="$2"
    
    log "INFO" "Configuring bootstrap node: $node_name" "REMOTE"
    
    ssh "icn-$node_name" << 'EOF'
# Setup federation database
sudo -u icn mkdir -p /opt/icn/data/federation

# Setup governance storage
sudo -u icn mkdir -p /opt/icn/data/governance

# Setup monitoring
sudo systemctl enable prometheus-node-exporter
sudo systemctl start prometheus-node-exporter
EOF
    
    log "SUCCESS" "Bootstrap node configured: $node_name" "REMOTE"
}

# Configure worker node
configure_worker_node() {
    local node_name="$1"
    local profile="$2"
    
    log "INFO" "Configuring worker node: $node_name" "REMOTE"
    
    ssh "icn-$node_name" << 'EOF'
# Setup job execution environment
sudo -u icn mkdir -p /opt/icn/data/jobs
sudo -u icn mkdir -p /opt/icn/data/cache

# Setup resource monitoring
sudo systemctl enable prometheus-node-exporter
sudo systemctl start prometheus-node-exporter

# Configure Docker for job execution
sudo usermod -aG docker icn
EOF
    
    log "SUCCESS" "Worker node configured: $node_name" "REMOTE"
}

# Configure gateway node
configure_gateway_node() {
    local node_name="$1"
    local profile="$2"
    
    log "INFO" "Configuring gateway node: $node_name" "REMOTE"
    
    ssh "icn-$node_name" << 'EOF'
# Install and configure nginx
sudo apt-get install -y nginx

# Setup SSL certificates directory
sudo mkdir -p /etc/nginx/ssl

# Setup rate limiting
sudo mkdir -p /var/cache/nginx/rate_limit

# Configure firewall
sudo ufw allow 80
sudo ufw allow 443
sudo ufw allow 22
EOF
    
    log "SUCCESS" "Gateway node configured: $node_name" "REMOTE"
}

# Configure auth node
configure_auth_node() {
    local node_name="$1"
    local profile="$2"
    
    log "INFO" "Configuring auth node: $node_name" "REMOTE"
    
    ssh "icn-$node_name" << 'EOF'
# Setup identity storage
sudo -u icn mkdir -p /opt/icn/data/identity
sudo -u icn mkdir -p /opt/icn/data/keys

# Setup authentication database
sudo -u icn mkdir -p /opt/icn/data/auth

# Configure secure key storage
sudo chmod 700 /opt/icn/data/keys
EOF
    
    log "SUCCESS" "Auth node configured: $node_name" "REMOTE"
}

# ==============================================================================
# Systemd Service Management
# ==============================================================================

# Setup systemd service for node
setup_node_service() {
    local node_name="$1"
    local role="$2"
    local profile="$3"
    
    log "INFO" "Setting up systemd service for: $node_name" "REMOTE"
    
    # Generate systemd service file
    generate_systemd_service "$node_name" "$role" "$profile"
    
    # Enable and start service
    ssh "icn-$node_name" << 'EOF'
sudo systemctl daemon-reload
sudo systemctl enable icn-node
sudo systemctl start icn-node
EOF
    
    if [[ $? -eq 0 ]]; then
        log "SUCCESS" "Systemd service setup completed for: $node_name" "REMOTE"
    else
        log "ERROR" "Failed to setup systemd service for: $node_name" "REMOTE"
        return 1
    fi
}

# Generate systemd service file
generate_systemd_service() {
    local node_name="$1"
    local role="$2"
    local profile="$3"
    
    log "INFO" "Generating systemd service for: $node_name" "REMOTE"
    
    local service_content=$(cat << EOF
[Unit]
Description=ICN Node ($role)
Documentation=https://github.com/InterCooperative-Network/icn-core
After=network.target
Wants=network.target

[Service]
Type=exec
User=icn
Group=icn
ExecStart=/opt/icn/bin/icn-node --config /etc/icn/icn.toml
ExecReload=/bin/kill -HUP \$MAINPID
KillMode=process
Restart=on-failure
RestartSec=5s
TimeoutStopSec=30s

# Environment
Environment=RUST_LOG=icn=info
Environment=ICN_PROFILE=$profile
EnvironmentFile=-/etc/icn/icn.env

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/icn /var/log/icn
CapabilityBoundingSet=

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=icn-node

[Install]
WantedBy=multi-user.target
EOF
    )
    
    # Transfer service file to remote node
    echo "$service_content" | ssh "icn-$node_name" "sudo tee /etc/systemd/system/icn-node.service > /dev/null"
    
    log "SUCCESS" "Systemd service generated for: $node_name" "REMOTE"
}

# ==============================================================================
# Multi-Node Deployment
# ==============================================================================

# Deploy ICN federation
deploy_federation() {
    local profile="$1"
    local deployment_order=("bootstrap" "worker1" "worker2" "gateway" "auth")
    
    log "INFO" "Deploying ICN federation for profile: $profile" "REMOTE"
    
    # Deploy nodes in order (bootstrap first)
    for node_name in "${deployment_order[@]}"; do
        if [[ -n "${REMOTE_NODES[$node_name]:-}" ]]; then
            deploy_node "$node_name" "$profile"
            
            # Wait for bootstrap node to be ready before deploying others
            if [[ "$node_name" == "bootstrap" ]]; then
                log "INFO" "Waiting for bootstrap node to be ready..." "REMOTE"
                wait_for_node_ready "$node_name" 60
            fi
        fi
    done
    
    # Verify federation health
    verify_federation_health "$profile"
    
    log "SUCCESS" "ICN federation deployment completed" "REMOTE"
}

# Deploy individual node
deploy_node() {
    local node_name="$1"
    local profile="$2"
    
    log "INFO" "Deploying node: $node_name" "REMOTE"
    
    # Stop existing service
    ssh "icn-$node_name" "sudo systemctl stop icn-node" 2>/dev/null || true
    
    # Update binaries and configuration
    transfer_icn_binaries "$node_name" "$profile"
    
    # Start service
    ssh "icn-$node_name" "sudo systemctl start icn-node"
    
    # Verify deployment
    if wait_for_node_ready "$node_name" 30; then
        log "SUCCESS" "Node deployed successfully: $node_name" "REMOTE"
    else
        log "ERROR" "Node deployment failed: $node_name" "REMOTE"
        return 1
    fi
}

# Wait for node to be ready
wait_for_node_ready() {
    local node_name="$1"
    local timeout="${2:-30}"
    
    log "INFO" "Waiting for node to be ready: $node_name" "REMOTE"
    
    local attempts=0
    local max_attempts=$((timeout / 5))
    
    while [[ $attempts -lt $max_attempts ]]; do
        if ssh "icn-$node_name" "curl -s -f http://localhost:8080/health" &>/dev/null; then
            log "SUCCESS" "Node is ready: $node_name" "REMOTE"
            return 0
        fi
        
        ((attempts++))
        sleep 5
        log "DEBUG" "Waiting for node... ($attempts/$max_attempts)" "REMOTE"
    done
    
    log "ERROR" "Node failed to become ready: $node_name" "REMOTE"
    return 1
}

# Verify federation health
verify_federation_health() {
    local profile="$1"
    
    log "INFO" "Verifying federation health" "REMOTE"
    
    local healthy_nodes=0
    local total_nodes=0
    
    for node_name in "${!REMOTE_NODES[@]}"; do
        ((total_nodes++))

        if ssh "icn-$node_name" "systemctl is-active icn-node" &>/dev/null; then
            if ssh "icn-$node_name" "curl -s -f http://localhost:8080/dag/status" &>/dev/null; then
                log "SUCCESS" "Node healthy: $node_name" "REMOTE"
                ((healthy_nodes++))
            else
                log "ERROR" "Node DAG check failed: $node_name" "REMOTE"
            fi
        else
            log "ERROR" "Node unhealthy: $node_name" "REMOTE"
        fi
    done
    
    local health_percentage=$((healthy_nodes * 100 / total_nodes))
    
    log "INFO" "Federation health: $healthy_nodes/$total_nodes nodes ($health_percentage%)" "REMOTE"
    
    if [[ "$health_percentage" -ge 80 ]]; then
        log "SUCCESS" "Federation health check passed" "REMOTE"
        return 0
    else
        log "ERROR" "Federation health check failed" "REMOTE"
        return 1
    fi
}

# ==============================================================================
# Monitoring and Maintenance
# ==============================================================================

# Show federation status
show_federation_status() {
    local profile="$1"
    
    log "INFO" "ICN Federation Status - Profile: $profile" "REMOTE"
    
    echo ""
    echo "Node Status:"
    echo "============"
    
    for node_name in "${!REMOTE_NODES[@]}"; do
        local node_config="${REMOTE_NODES[$node_name]}"
        IFS=':' read -r ip port role <<< "$node_config"
        
        # Check SSH connectivity
        local ssh_status="DISCONNECTED"
        if ssh -o ConnectTimeout=3 "icn-$node_name" "echo" &>/dev/null; then
            ssh_status="CONNECTED"
        fi
        
        # Check service status
        local service_status="STOPPED"
        if [[ "$ssh_status" == "CONNECTED" ]]; then
            if ssh "icn-$node_name" "systemctl is-active icn-node" &>/dev/null; then
                service_status="RUNNING"
            fi
        fi
        
        # Check health endpoint
        local health_status="UNHEALTHY"
        if [[ "$service_status" == "RUNNING" ]]; then
            if ssh "icn-$node_name" "curl -s -f http://localhost:8080/health" &>/dev/null; then
                health_status="HEALTHY"
            fi
        fi
        
        printf "  %-12s %-15s %-10s %-10s %-10s %-10s\n" \
            "$node_name" "$ip" "$role" "$ssh_status" "$service_status" "$health_status"
    done
    
    echo ""
    echo "Federation Metrics:"
    echo "==================="
    
    # Get metrics from bootstrap node if available
    if ssh "icn-bootstrap" "curl -s -f http://localhost:8080/metrics" &>/dev/null; then
        ssh "icn-bootstrap" "curl -s http://localhost:8080/metrics | grep -E '^icn_'"
    else
        echo "  Metrics unavailable (bootstrap node not accessible)"
    fi
}

# Collect logs from all nodes
collect_federation_logs() {
    local profile="$1"
    local output_dir="${2:-./logs-$(date +%Y%m%d-%H%M%S)}"
    
    log "INFO" "Collecting federation logs to: $output_dir" "REMOTE"
    
    mkdir -p "$output_dir"
    
    for node_name in "${!REMOTE_NODES[@]}"; do
        log "INFO" "Collecting logs from: $node_name" "REMOTE"
        
        local node_log_dir="$output_dir/$node_name"
        mkdir -p "$node_log_dir"
        
        # Collect ICN logs
        scp "icn-$node_name:/var/log/icn/*.log" "$node_log_dir/" 2>/dev/null || true
        
        # Collect systemd logs
        ssh "icn-$node_name" "journalctl -u icn-node --since='1 hour ago' --no-pager" > "$node_log_dir/systemd.log" 2>/dev/null || true
        
        # Collect system metrics
        ssh "icn-$node_name" "top -bn1 | head -20" > "$node_log_dir/system.log" 2>/dev/null || true
    done
    
    log "SUCCESS" "Federation logs collected: $output_dir" "REMOTE"
}

# Execute command on all nodes
execute_on_all_nodes() {
    local command="$1"
    local parallel="${2:-true}"
    
    log "INFO" "Executing command on all nodes: $command" "REMOTE"
    
    if [[ "$parallel" == "true" ]]; then
        local pids=()
        
        for node_name in "${!REMOTE_NODES[@]}"; do
            {
                log "INFO" "[$node_name] Executing: $command" "REMOTE"
                ssh "icn-$node_name" "$command"
                log "SUCCESS" "[$node_name] Command completed" "REMOTE"
            } &
            pids+=($!)
        done
        
        # Wait for all commands to complete
        for pid in "${pids[@]}"; do
            wait "$pid"
        done
    else
        for node_name in "${!REMOTE_NODES[@]}"; do
            log "INFO" "[$node_name] Executing: $command" "REMOTE"
            ssh "icn-$node_name" "$command"
            log "SUCCESS" "[$node_name] Command completed" "REMOTE"
        done
    fi
    
    log "SUCCESS" "Command execution completed on all nodes" "REMOTE"
}

# Update all nodes
update_all_nodes() {
    local profile="$1"
    
    log "INFO" "Updating all nodes for profile: $profile" "REMOTE"
    
    # Update system packages
    execute_on_all_nodes "sudo apt-get update && sudo apt-get upgrade -y" false
    
    # Update ICN binaries
    for node_name in "${!REMOTE_NODES[@]}"; do
        transfer_icn_binaries "$node_name" "$profile"
        ssh "icn-$node_name" "sudo systemctl restart icn-node"
    done
    
    # Verify all nodes are healthy
    sleep 10
    verify_federation_health "$profile"
    
    log "SUCCESS" "All nodes updated successfully" "REMOTE"
}

# ==============================================================================
# Remote Utility Aliases
# ==============================================================================

# Generate remote-specific aliases
generate_remote_aliases() {
    local profile="$1"
    
    cat << EOF

# Remote Infrastructure Aliases
alias icn-remote-setup='setup_remote_nodes $profile'
alias icn-remote-deploy='deploy_federation $profile'
alias icn-remote-status='show_federation_status $profile'
alias icn-remote-logs='collect_federation_logs $profile'
alias icn-remote-update='update_all_nodes $profile'

# SSH Aliases
alias icn-ssh-setup='setup_ssh_config $profile'
alias icn-ssh-test='test_ssh_connectivity $profile'

# Node-specific SSH aliases
EOF
    
    for node_name in "${!REMOTE_NODES[@]}"; do
        cat << EOF
alias icn-ssh-$node_name='ssh icn-$node_name'
EOF
    done
    
    cat << EOF

# Service management aliases
alias icn-start-all="execute_on_all_nodes 'sudo systemctl start icn-node'"
alias icn-stop-all="execute_on_all_nodes 'sudo systemctl stop icn-node'"
alias icn-restart-all="execute_on_all_nodes 'sudo systemctl restart icn-node'"
alias icn-status-all="execute_on_all_nodes 'systemctl status icn-node'"

# Monitoring aliases
alias icn-monitor-federation='watch -n 5 "show_federation_status $profile"'
alias icn-tail-logs='execute_on_all_nodes "tail -f /var/log/icn/icn.log" true'
EOF
}

log "SUCCESS" "Remote utilities loaded" "REMOTE" 