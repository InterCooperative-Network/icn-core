#!/bin/bash

# ==============================================================================
# ICN Environment Utilities
# ==============================================================================
# Comprehensive environment management for ICN development
# Handles configuration loading, validation, and deployment setup

# ==============================================================================
# Environment Configuration Loading
# ==============================================================================

# Load environment configuration from multiple sources
load_environment_config() {
    local profile="$1"
    local project_root="${PROJECT_ROOT:-$(pwd)}"
    
    log "INFO" "Loading environment configuration for profile: $profile" "ENV"
    
    # Configuration hierarchy (later files override earlier ones)
    local env_files=(
        "$project_root/.env"
        "$project_root/.env.local"
        "$project_root/.env.$profile"
        "$project_root/.env.$profile.local"
        "$project_root/.icn/$profile/.env"
        "$HOME/.icn/global.env"
    )
    
    # Load each configuration file
    for env_file in "${env_files[@]}"; do
        if [[ -f "$env_file" ]]; then
            log "DEBUG" "Loading config: $env_file" "ENV"
            set -a  # Automatically export all variables
            source "$env_file"
            set +a
        else
            log "DEBUG" "Config file not found: $env_file" "ENV"
        fi
    done
    
    # Set profile-specific defaults
    set_profile_defaults "$profile"
    
    log "SUCCESS" "Environment configuration loaded" "ENV"
}

# Set profile-specific default values
set_profile_defaults() {
    local profile="$1"
    
    # Common defaults
    export ICN_PROFILE="${ICN_PROFILE:-$profile}"
    export ICN_LOG_LEVEL="${ICN_LOG_LEVEL:-info}"
    export ICN_LOG_FORMAT="${ICN_LOG_FORMAT:-json}"
    export ICN_RUST_LOG="${ICN_RUST_LOG:-icn=debug}"
    
    # Network configuration
    export ICN_NETWORK_PORT="${ICN_NETWORK_PORT:-8080}"
    export ICN_NETWORK_BIND="${ICN_NETWORK_BIND:-0.0.0.0}"
    export ICN_NETWORK_EXTERNAL_URL="${ICN_NETWORK_EXTERNAL_URL:-http://localhost:8080}"
    
    # DAG storage configuration
    export ICN_DAG_STORE_TYPE="${ICN_DAG_STORE_TYPE:-file}"
    export ICN_DAG_STORE_PATH="${ICN_DAG_STORE_PATH:-$project_root/.icn/$profile/data/dag}"
    
    # Mana system configuration
    export ICN_MANA_REGENERATION_RATE="${ICN_MANA_REGENERATION_RATE:-100}"
    export ICN_MANA_MAX_CAPACITY="${ICN_MANA_MAX_CAPACITY:-10000}"
    
    # Profile-specific overrides
    case "$profile" in
        "dev")
            export ICN_LOG_LEVEL="${ICN_LOG_LEVEL:-debug}"
            export ICN_RUST_LOG="${ICN_RUST_LOG:-icn=trace}"
            export ICN_ENABLE_HOT_RELOAD="${ICN_ENABLE_HOT_RELOAD:-true}"
            export ICN_ENABLE_DEBUG_API="${ICN_ENABLE_DEBUG_API:-true}"
            export ICN_NETWORK_PORT="${ICN_NETWORK_PORT:-8080}"
            ;;
        "test")
            export ICN_LOG_LEVEL="${ICN_LOG_LEVEL:-warn}"
            export ICN_RUST_LOG="${ICN_RUST_LOG:-icn=info}"
            export ICN_ENABLE_MOCK_SERVICES="${ICN_ENABLE_MOCK_SERVICES:-true}"
            export ICN_ENABLE_TEST_HELPERS="${ICN_ENABLE_TEST_HELPERS:-true}"
            export ICN_NETWORK_PORT="${ICN_NETWORK_PORT:-8081}"
            ;;
        "prod")
            export ICN_LOG_LEVEL="${ICN_LOG_LEVEL:-info}"
            export ICN_RUST_LOG="${ICN_RUST_LOG:-icn=info}"
            export ICN_ENABLE_SECURITY_HARDENING="${ICN_ENABLE_SECURITY_HARDENING:-true}"
            export ICN_ENABLE_MONITORING="${ICN_ENABLE_MONITORING:-true}"
            export ICN_NETWORK_PORT="${ICN_NETWORK_PORT:-80}"
            ;;
        "infra")
            export ICN_LOG_LEVEL="${ICN_LOG_LEVEL:-warn}"
            export ICN_ENABLE_DEPLOYMENT_TOOLS="${ICN_ENABLE_DEPLOYMENT_TOOLS:-true}"
            export ICN_ENABLE_REMOTE_MANAGEMENT="${ICN_ENABLE_REMOTE_MANAGEMENT:-true}"
            ;;
    esac
    
    log "DEBUG" "Profile defaults set for: $profile" "ENV"
}

# ==============================================================================
# Configuration Validation
# ==============================================================================

# Validate environment configuration
validate_environment() {
    local profile="$1"
    local validation_passed=true
    
    log "INFO" "Validating environment configuration" "ENV"
    
    # Required variables for all profiles
    local required_vars=(
        "ICN_PROFILE"
        "ICN_LOG_LEVEL"
        "ICN_NETWORK_PORT"
        "ICN_DAG_STORE_PATH"
    )
    
    # Profile-specific required variables
    case "$profile" in
        "prod")
            required_vars+=(
                "ICN_SECURITY_KEY_PATH"
                "ICN_MONITORING_ENDPOINT"
            )
            ;;
        "infra")
            required_vars+=(
                "ICN_SSH_KEY_PATH"
                "ICN_DEPLOYMENT_TARGETS"
            )
            ;;
    esac
    
    # Check required variables
    for var in "${required_vars[@]}"; do
        if [[ -z "${!var:-}" ]]; then
            log "ERROR" "Required environment variable not set: $var" "ENV"
            validation_passed=false
        else
            log "DEBUG" "Environment variable set: $var=${!var}" "ENV"
        fi
    done
    
    # Validate specific configurations
    validate_network_config "$profile"
    validate_storage_config "$profile"
    validate_security_config "$profile"
    
    if [[ "$validation_passed" == "false" ]]; then
        handle_error 10 "Environment validation failed" "
        Please check your environment configuration files:
        - .env.$profile
        - .icn/$profile/.env
        
        Required variables: ${required_vars[*]}
        "
    fi
    
    log "SUCCESS" "Environment validation passed" "ENV"
}

# Validate network configuration
validate_network_config() {
    local profile="$1"
    
    # Check port availability
    if command -v netstat &> /dev/null; then
        if netstat -an | grep -q ":${ICN_NETWORK_PORT}"; then
            log "WARN" "Port ${ICN_NETWORK_PORT} appears to be in use" "ENV"
        fi
    fi
    
    # Validate bind address
    if [[ "$ICN_NETWORK_BIND" != "0.0.0.0" && "$ICN_NETWORK_BIND" != "127.0.0.1" ]]; then
        if ! [[ "$ICN_NETWORK_BIND" =~ ^[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}$ ]]; then
            log "ERROR" "Invalid bind address: $ICN_NETWORK_BIND" "ENV"
            return 1
        fi
    fi
    
    log "DEBUG" "Network configuration validated" "ENV"
}

# Validate storage configuration
validate_storage_config() {
    local profile="$1"
    
    # Create storage directory if it doesn't exist
    if [[ ! -d "$ICN_DAG_STORE_PATH" ]]; then
        mkdir -p "$ICN_DAG_STORE_PATH"
        log "INFO" "Created storage directory: $ICN_DAG_STORE_PATH" "ENV"
    fi
    
    # Check write permissions
    if [[ ! -w "$ICN_DAG_STORE_PATH" ]]; then
        log "ERROR" "No write permission for storage path: $ICN_DAG_STORE_PATH" "ENV"
        return 1
    fi
    
    log "DEBUG" "Storage configuration validated" "ENV"
}

# Validate security configuration
validate_security_config() {
    local profile="$1"
    
    case "$profile" in
        "prod")
            # Check for security key
            if [[ -n "${ICN_SECURITY_KEY_PATH:-}" ]]; then
                if [[ ! -f "$ICN_SECURITY_KEY_PATH" ]]; then
                    log "ERROR" "Security key file not found: $ICN_SECURITY_KEY_PATH" "ENV"
                    return 1
                fi
                
                # Check key file permissions
                local key_perms=$(stat -c "%a" "$ICN_SECURITY_KEY_PATH" 2>/dev/null || stat -f "%A" "$ICN_SECURITY_KEY_PATH" 2>/dev/null)
                if [[ "$key_perms" != "600" ]]; then
                    log "WARN" "Security key file permissions should be 600: $ICN_SECURITY_KEY_PATH" "ENV"
                fi
            fi
            ;;
        "infra")
            # Check SSH key
            if [[ -n "${ICN_SSH_KEY_PATH:-}" ]]; then
                if [[ ! -f "$ICN_SSH_KEY_PATH" ]]; then
                    log "ERROR" "SSH key file not found: $ICN_SSH_KEY_PATH" "ENV"
                    return 1
                fi
            fi
            ;;
    esac
    
    log "DEBUG" "Security configuration validated" "ENV"
}

# ==============================================================================
# Secrets Management
# ==============================================================================

# Generate secure configuration template
generate_secure_config() {
    local profile="$1"
    local config_path="$PROJECT_ROOT/.icn/$profile/.env"
    
    log "INFO" "Generating secure configuration template" "ENV"
    
    # Create secure directory
    mkdir -p "$(dirname "$config_path")"
    
    # Generate configuration file
    cat > "$config_path" << EOF
# ICN Secure Configuration - Profile: $profile
# Generated on: $(date)
# WARNING: This file contains sensitive information. Do not commit to version control.

# Core Configuration
ICN_PROFILE=$profile
ICN_LOG_LEVEL=${ICN_LOG_LEVEL:-info}
ICN_NETWORK_PORT=${ICN_NETWORK_PORT:-8080}

# Security Configuration
ICN_SECURITY_KEY_PATH=$PROJECT_ROOT/.icn/$profile/keys/node.key
ICN_SECURITY_CERT_PATH=$PROJECT_ROOT/.icn/$profile/keys/node.crt

# Database Configuration
ICN_DATABASE_URL=postgres://icn:$(generate_password)@localhost:5432/icn_$profile

# Mana System Configuration
ICN_MANA_REGENERATION_RATE=${ICN_MANA_REGENERATION_RATE:-100}
ICN_MANA_MAX_CAPACITY=${ICN_MANA_MAX_CAPACITY:-10000}

# Network Configuration
ICN_BOOTSTRAP_PEERS=${ICN_BOOTSTRAP_PEERS:-}
ICN_NETWORK_KEY=$(generate_network_key)

EOF
    
    # Set secure permissions
    chmod 600 "$config_path"
    
    log "SUCCESS" "Secure configuration generated: $config_path" "ENV"
}

# Generate random password
generate_password() {
    openssl rand -base64 32 | tr -d "=+/" | cut -c1-25
}

# Generate network key
generate_network_key() {
    openssl rand -hex 32
}

# Setup secure key directory
setup_secure_keys() {
    local profile="$1"
    local keys_dir="$PROJECT_ROOT/.icn/$profile/keys"
    
    log "INFO" "Setting up secure key directory" "ENV"
    
    # Create keys directory
    mkdir -p "$keys_dir"
    chmod 700 "$keys_dir"
    
    # Generate node key if it doesn't exist
    local node_key="$keys_dir/node.key"
    local node_cert="$keys_dir/node.crt"
    
    if [[ ! -f "$node_key" ]]; then
        log "INFO" "Generating node key and certificate" "ENV"
        
        # Generate private key
        openssl genpkey -algorithm Ed25519 -out "$node_key"
        chmod 600 "$node_key"
        
        # Generate self-signed certificate
        openssl req -new -x509 -key "$node_key" -out "$node_cert" -days 365 -subj "/CN=icn-node-$profile"
        chmod 644 "$node_cert"
        
        log "SUCCESS" "Node key and certificate generated" "ENV"
    fi
    
    # Generate additional keys as needed
    case "$profile" in
        "prod")
            setup_production_keys "$keys_dir"
            ;;
        "infra")
            setup_infrastructure_keys "$keys_dir"
            ;;
    esac
}

# Setup production-specific keys
setup_production_keys() {
    local keys_dir="$1"
    
    # Generate JWT signing key
    local jwt_key="$keys_dir/jwt.key"
    if [[ ! -f "$jwt_key" ]]; then
        openssl genpkey -algorithm Ed25519 -out "$jwt_key"
        chmod 600 "$jwt_key"
        log "INFO" "JWT signing key generated" "ENV"
    fi
    
    # Generate API key
    local api_key="$keys_dir/api.key"
    if [[ ! -f "$api_key" ]]; then
        generate_password > "$api_key"
        chmod 600 "$api_key"
        log "INFO" "API key generated" "ENV"
    fi
}

# Setup infrastructure-specific keys
setup_infrastructure_keys() {
    local keys_dir="$1"
    
    # Generate SSH key for remote management
    local ssh_key="$keys_dir/infrastructure.key"
    if [[ ! -f "$ssh_key" ]]; then
        ssh-keygen -t ed25519 -f "$ssh_key" -N "" -C "icn-infrastructure-$(date +%Y%m%d)"
        chmod 600 "$ssh_key"
        log "INFO" "Infrastructure SSH key generated" "ENV"
    fi
}

# ==============================================================================
# Deployment Configuration
# ==============================================================================

# Generate deployment configuration
generate_deployment_config() {
    local profile="$1"
    local deployment_type="${2:-local}"
    
    log "INFO" "Generating deployment configuration: $deployment_type" "ENV"
    
    case "$deployment_type" in
        "local")
            generate_local_deployment_config "$profile"
            ;;
        "docker")
            generate_docker_deployment_config "$profile"
            ;;
        "federation")
            generate_federation_deployment_config "$profile"
            ;;
        "kubernetes")
            generate_kubernetes_deployment_config "$profile"
            ;;
        *)
            log "ERROR" "Unknown deployment type: $deployment_type" "ENV"
            return 1
            ;;
    esac
    
    log "SUCCESS" "Deployment configuration generated" "ENV"
}

# Generate local deployment configuration
generate_local_deployment_config() {
    local profile="$1"
    local config_path="$PROJECT_ROOT/.icn/$profile/deploy-local.yaml"
    
    cat > "$config_path" << EOF
# ICN Local Deployment Configuration
# Profile: $profile
# Generated on: $(date)

deployment:
  type: local
  profile: $profile
  
services:
  icn-node:
    binary: ./target/release/icn-node
    config: .icn/$profile/.env
    port: ${ICN_NETWORK_PORT:-8080}
    
  icn-cli:
    binary: ./target/release/icn-cli
    config: .icn/$profile/.env
    
monitoring:
  enabled: ${ICN_ENABLE_MONITORING:-false}
  port: 9090
  
logging:
  level: ${ICN_LOG_LEVEL:-info}
  format: ${ICN_LOG_FORMAT:-json}
  path: .icn/$profile/logs
EOF
    
    log "DEBUG" "Local deployment config: $config_path" "ENV"
}

# Generate Docker deployment configuration
generate_docker_deployment_config() {
    local profile="$1"
    local config_path="$PROJECT_ROOT/.icn/$profile/deploy-docker.yaml"
    
    cat > "$config_path" << EOF
# ICN Docker Deployment Configuration
# Profile: $profile
# Generated on: $(date)

version: '3.8'

services:
  icn-node:
    build:
      context: .
      dockerfile: docker/Dockerfile.$profile
    ports:
      - "${ICN_NETWORK_PORT:-8080}:8080"
    environment:
      - ICN_PROFILE=$profile
      - ICN_LOG_LEVEL=${ICN_LOG_LEVEL:-info}
    volumes:
      - .icn/$profile/data:/app/data
      - .icn/$profile/keys:/app/keys:ro
    depends_on:
      - postgres
      - redis
      
  postgres:
    image: postgres:15
    environment:
      - POSTGRES_DB=icn_$profile
      - POSTGRES_USER=icn
      - POSTGRES_PASSWORD=\${ICN_DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
      
  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data

volumes:
  postgres_data:
  redis_data:
EOF
    
    log "DEBUG" "Docker deployment config: $config_path" "ENV"
}

# Generate federation deployment configuration
generate_federation_deployment_config() {
    local profile="$1"
    local config_path="$PROJECT_ROOT/.icn/$profile/deploy-federation.yaml"
    
    cat > "$config_path" << EOF
# ICN Federation Deployment Configuration
# Profile: $profile
# Generated on: $(date)

federation:
  name: icn-cooperative
  profile: $profile
  
nodes:
  bootstrap:
    - address: 10.8.10.11
      role: bootstrap
      port: ${ICN_NETWORK_PORT:-8080}
      
  workers:
    - address: 10.8.10.12
      role: worker
      port: ${ICN_NETWORK_PORT:-8080}
    - address: 10.8.10.13
      role: worker
      port: ${ICN_NETWORK_PORT:-8080}
      
governance:
  voting_threshold: 0.67
  proposal_timeout: 168h  # 1 week
  
economics:
  mana_regeneration_rate: ${ICN_MANA_REGENERATION_RATE:-100}
  mana_max_capacity: ${ICN_MANA_MAX_CAPACITY:-10000}
  
security:
  require_signatures: true
  key_rotation_interval: 30d
EOF
    
    log "DEBUG" "Federation deployment config: $config_path" "ENV"
}

# Generate Kubernetes deployment configuration
generate_kubernetes_deployment_config() {
    local profile="$1"
    local config_path="$PROJECT_ROOT/.icn/$profile/deploy-k8s.yaml"
    
    cat > "$config_path" << EOF
# ICN Kubernetes Deployment Configuration
# Profile: $profile
# Generated on: $(date)

apiVersion: apps/v1
kind: Deployment
metadata:
  name: icn-node-$profile
  namespace: icn-system
spec:
  replicas: 3
  selector:
    matchLabels:
      app: icn-node
      profile: $profile
  template:
    metadata:
      labels:
        app: icn-node
        profile: $profile
    spec:
      containers:
      - name: icn-node
        image: icn/node:$profile
        ports:
        - containerPort: 8080
        env:
        - name: ICN_PROFILE
          value: "$profile"
        - name: ICN_LOG_LEVEL
          value: "${ICN_LOG_LEVEL:-info}"
        volumeMounts:
        - name: config
          mountPath: /app/config
        - name: data
          mountPath: /app/data
      volumes:
      - name: config
        configMap:
          name: icn-config-$profile
      - name: data
        persistentVolumeClaim:
          claimName: icn-data-$profile
---
apiVersion: v1
kind: Service
metadata:
  name: icn-node-$profile
  namespace: icn-system
spec:
  selector:
    app: icn-node
    profile: $profile
  ports:
  - port: 80
    targetPort: 8080
  type: LoadBalancer
EOF
    
    log "DEBUG" "Kubernetes deployment config: $config_path" "ENV"
}

# ==============================================================================
# Environment Health Check
# ==============================================================================

# Check environment health
check_environment_health() {
    local profile="$1"
    
    log "INFO" "Checking environment health" "ENV"
    
    local health_checks=(
        "check_configuration_files"
        "check_network_connectivity"
        "check_storage_health"
        "check_security_status"
        "check_resource_availability"
    )
    
    local passed_checks=0
    local total_checks=${#health_checks[@]}
    
    for check in "${health_checks[@]}"; do
        if "$check" "$profile"; then
            ((passed_checks++))
        fi
    done
    
    local health_score=$((passed_checks * 100 / total_checks))
    
    if [[ "$health_score" -ge 80 ]]; then
        log "SUCCESS" "Environment health check passed ($health_score%)" "ENV"
    else
        log "WARN" "Environment health check concerns ($health_score%)" "ENV"
    fi
    
    return 0
}

# Check configuration files
check_configuration_files() {
    local profile="$1"
    
    local config_files=(
        "$PROJECT_ROOT/.icn/$profile/.env"
        "$PROJECT_ROOT/.icn/$profile/deploy-local.yaml"
    )
    
    for config_file in "${config_files[@]}"; do
        if [[ ! -f "$config_file" ]]; then
            log "WARN" "Configuration file missing: $config_file" "ENV"
            return 1
        fi
    done
    
    log "DEBUG" "Configuration files check passed" "ENV"
    return 0
}

# Check network connectivity
check_network_connectivity() {
    local profile="$1"
    
    # Check if port is available
    if command -v nc &> /dev/null; then
        if nc -z localhost "${ICN_NETWORK_PORT}" 2>/dev/null; then
            log "DEBUG" "Port ${ICN_NETWORK_PORT} is accessible" "ENV"
        else
            log "DEBUG" "Port ${ICN_NETWORK_PORT} is available" "ENV"
        fi
    fi
    
    # Check internet connectivity for dependencies
    if curl -s --connect-timeout 5 https://github.com &> /dev/null; then
        log "DEBUG" "Internet connectivity check passed" "ENV"
        return 0
    else
        log "WARN" "Internet connectivity check failed" "ENV"
        return 1
    fi
}

# Check storage health
check_storage_health() {
    local profile="$1"
    
    # Check storage directory
    if [[ -d "$ICN_DAG_STORE_PATH" && -w "$ICN_DAG_STORE_PATH" ]]; then
        log "DEBUG" "Storage health check passed" "ENV"
        return 0
    else
        log "WARN" "Storage health check failed" "ENV"
        return 1
    fi
}

# Check security status
check_security_status() {
    local profile="$1"
    
    local keys_dir="$PROJECT_ROOT/.icn/$profile/keys"
    
    if [[ -d "$keys_dir" ]]; then
        local key_count=$(find "$keys_dir" -name "*.key" -type f | wc -l)
        if [[ "$key_count" -gt 0 ]]; then
            log "DEBUG" "Security keys available ($key_count)" "ENV"
            return 0
        fi
    fi
    
    log "WARN" "Security status check failed" "ENV"
    return 1
}

# Check resource availability
check_resource_availability() {
    local profile="$1"
    
    # Check disk space
    local available_space=$(df -BG "$PROJECT_ROOT" | tail -1 | awk '{print $4}' | sed 's/G//')
    if [[ "$available_space" -gt 1 ]]; then
        log "DEBUG" "Sufficient disk space available (${available_space}GB)" "ENV"
        return 0
    else
        log "WARN" "Low disk space (${available_space}GB)" "ENV"
        return 1
    fi
}

# ==============================================================================
# Utility Aliases for Environment Management
# ==============================================================================

# Generate environment-specific aliases
generate_env_aliases() {
    local profile="$1"
    
    cat << EOF

# Environment Management Aliases
alias icn-env-load='load_environment_config $profile'
alias icn-env-validate='validate_environment $profile'
alias icn-env-health='check_environment_health $profile'
alias icn-env-config='generate_secure_config $profile'
alias icn-env-keys='setup_secure_keys $profile'

# Deployment Aliases
alias icn-deploy-local='generate_deployment_config $profile local'
alias icn-deploy-docker='generate_deployment_config $profile docker'
alias icn-deploy-federation='generate_deployment_config $profile federation'
alias icn-deploy-k8s='generate_deployment_config $profile kubernetes'

# Environment Variables
alias icn-env-show='env | grep ^ICN_ | sort'
alias icn-env-profile='echo "Current profile: \$ICN_PROFILE"'
EOF
}

log "SUCCESS" "Environment utilities loaded" "ENV" 