#!/bin/bash

# ==============================================================================
# ICN Docker Utilities
# ==============================================================================
# Comprehensive Docker management for ICN development
# Handles containerization, service orchestration, and development environments

# ==============================================================================
# Docker Health and Setup
# ==============================================================================

# Check Docker availability and health
check_docker_health() {
    local profile="$1"
    
    log "INFO" "Checking Docker environment health" "DOCKER"
    
    # Check if Docker is installed and running
    if ! command -v docker &> /dev/null; then
        log "ERROR" "Docker not found. Please install Docker first." "DOCKER"
        return 1
    fi
    
    # Check if Docker daemon is running
    if ! docker info &> /dev/null; then
        log "ERROR" "Docker daemon not running. Please start Docker." "DOCKER"
        return 1
    fi
    
    # Check Docker version
    local docker_version=$(docker --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)
    log "INFO" "Docker version: $docker_version" "DOCKER"
    
    # Check if docker-compose is available
    if command -v docker-compose &> /dev/null; then
        local compose_version=$(docker-compose --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)
        log "INFO" "Docker Compose version: $compose_version" "DOCKER"
    else
        log "WARN" "Docker Compose not found. Some features may be limited." "DOCKER"
    fi
    
    # Check available resources
    local available_memory=$(docker system info --format '{{.MemTotal}}' 2>/dev/null)
    if [[ -n "$available_memory" ]]; then
        local memory_gb=$((available_memory / 1024 / 1024 / 1024))
        log "INFO" "Available Docker memory: ${memory_gb}GB" "DOCKER"
        
        if [[ "$memory_gb" -lt 4 ]]; then
            log "WARN" "Docker has limited memory. Consider increasing memory allocation." "DOCKER"
        fi
    fi
    
    log "SUCCESS" "Docker health check passed" "DOCKER"
    return 0
}

# Create ICN Docker network
create_icn_network() {
    local network_name="icn-network"
    
    log "INFO" "Creating ICN Docker network" "DOCKER"
    
    # Check if network already exists
    if docker network ls | grep -q "$network_name"; then
        log "INFO" "Network $network_name already exists" "DOCKER"
        return 0
    fi
    
    # Create network with specific subnet
    docker network create \
        --driver bridge \
        --subnet=172.20.0.0/16 \
        --ip-range=172.20.240.0/20 \
        --gateway=172.20.0.1 \
        "$network_name" || {
        log "ERROR" "Failed to create Docker network" "DOCKER"
        return 1
    }
    
    log "SUCCESS" "Created Docker network: $network_name" "DOCKER"
    return 0
}

# ==============================================================================
# Dockerfile Generation
# ==============================================================================

# Generate profile-specific Dockerfiles
generate_dockerfiles() {
    local profile="$1"
    
    log "INFO" "Generating Dockerfiles for profile: $profile" "DOCKER"
    
    # Create docker directory
    mkdir -p "$PROJECT_ROOT/docker"
    
    # Generate base Dockerfile
    generate_base_dockerfile
    
    # Generate profile-specific Dockerfile
    case "$profile" in
        "dev")
            generate_dev_dockerfile
            ;;
        "test")
            generate_test_dockerfile
            ;;
        "prod")
            generate_prod_dockerfile
            ;;
        "infra")
            generate_infra_dockerfile
            ;;
    esac
    
    log "SUCCESS" "Dockerfiles generated for profile: $profile" "DOCKER"
}

# Generate base Dockerfile
generate_base_dockerfile() {
    local dockerfile_path="$PROJECT_ROOT/docker/Dockerfile.base"
    
    cat > "$dockerfile_path" << 'EOF'
# ICN Base Dockerfile
# Multi-stage build for optimal image size

FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build dependencies (cached layer)
RUN cargo build --release --workspace

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false -m -d /app icn

# Set working directory
WORKDIR /app

# Copy binaries from builder
COPY --from=builder /app/target/release/icn-node /usr/local/bin/
COPY --from=builder /app/target/release/icn-cli /usr/local/bin/

# Create necessary directories
RUN mkdir -p /app/config /app/data /app/logs /app/keys \
    && chown -R icn:icn /app

# Switch to app user
USER icn

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Default command
CMD ["icn-node"]
EOF
    
    log "DEBUG" "Base Dockerfile generated: $dockerfile_path" "DOCKER"
}

# Generate development Dockerfile
generate_dev_dockerfile() {
    local dockerfile_path="$PROJECT_ROOT/docker/Dockerfile.dev"
    
    cat > "$dockerfile_path" << 'EOF'
# ICN Development Dockerfile
# Optimized for development with hot reload and debugging

FROM rust:1.75

# Install development dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    build-essential \
    curl \
    vim \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install cargo tools
RUN cargo install cargo-watch cargo-expand cargo-audit

# Create app directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build in development mode
RUN cargo build --workspace

# Expose development ports
EXPOSE 8080 9090 9091

# Set development environment
ENV ICN_PROFILE=dev
ENV ICN_LOG_LEVEL=debug
ENV RUST_LOG=icn=trace
ENV RUST_BACKTRACE=1

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Development command with hot reload
CMD ["cargo", "watch", "-x", "run --bin icn-node"]
EOF
    
    log "DEBUG" "Development Dockerfile generated: $dockerfile_path" "DOCKER"
}

# Generate test Dockerfile
generate_test_dockerfile() {
    local dockerfile_path="$PROJECT_ROOT/docker/Dockerfile.test"
    
    cat > "$dockerfile_path" << 'EOF'
# ICN Test Dockerfile
# Optimized for testing with coverage and validation

FROM rust:1.75

# Install test dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    build-essential \
    curl \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Install testing tools
RUN cargo install cargo-tarpaulin cargo-audit cargo-deny

# Create app directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build test dependencies
RUN cargo build --workspace --tests

# Set test environment
ENV ICN_PROFILE=test
ENV ICN_LOG_LEVEL=warn
ENV RUST_LOG=icn=info
ENV ICN_TEST_MODE=true

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8081/health || exit 1

# Test command
CMD ["cargo", "test", "--workspace", "--all-features"]
EOF
    
    log "DEBUG" "Test Dockerfile generated: $dockerfile_path" "DOCKER"
}

# Generate production Dockerfile
generate_prod_dockerfile() {
    local dockerfile_path="$PROJECT_ROOT/docker/Dockerfile.prod"
    
    cat > "$dockerfile_path" << 'EOF'
# ICN Production Dockerfile
# Optimized for production with security and performance

FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build with optimizations
RUN cargo build --release --workspace

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get autoremove -y \
    && apt-get clean

# Create app user
RUN useradd -r -s /bin/false -m -d /app icn

# Set working directory
WORKDIR /app

# Copy binaries from builder
COPY --from=builder /app/target/release/icn-node /usr/local/bin/
COPY --from=builder /app/target/release/icn-cli /usr/local/bin/

# Create necessary directories
RUN mkdir -p /app/config /app/data /app/logs /app/keys \
    && chown -R icn:icn /app

# Switch to app user
USER icn

# Expose production port
EXPOSE 80

# Set production environment
ENV ICN_PROFILE=prod
ENV ICN_LOG_LEVEL=info
ENV RUST_LOG=icn=info

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:80/health || exit 1

# Production command
CMD ["icn-node"]
EOF
    
    log "DEBUG" "Production Dockerfile generated: $dockerfile_path" "DOCKER"
}

# Generate infrastructure Dockerfile
generate_infra_dockerfile() {
    local dockerfile_path="$PROJECT_ROOT/docker/Dockerfile.infra"
    
    cat > "$dockerfile_path" << 'EOF'
# ICN Infrastructure Dockerfile
# Includes deployment and monitoring tools

FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build with release optimizations
RUN cargo build --release --workspace

# Runtime stage
FROM debian:bookworm-slim

# Install runtime and infrastructure dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    wget \
    jq \
    ssh \
    rsync \
    && rm -rf /var/lib/apt/lists/*

# Install Docker CLI for container management
RUN curl -fsSL https://get.docker.com | sh

# Create app user
RUN useradd -r -s /bin/false -m -d /app icn

# Set working directory
WORKDIR /app

# Copy binaries from builder
COPY --from=builder /app/target/release/icn-node /usr/local/bin/
COPY --from=builder /app/target/release/icn-cli /usr/local/bin/

# Copy scripts
COPY scripts/ /app/scripts/

# Create necessary directories
RUN mkdir -p /app/config /app/data /app/logs /app/keys /app/deploy \
    && chown -R icn:icn /app

# Switch to app user
USER icn

# Set infrastructure environment
ENV ICN_PROFILE=infra
ENV ICN_LOG_LEVEL=warn

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Infrastructure command
CMD ["icn-node"]
EOF
    
    log "DEBUG" "Infrastructure Dockerfile generated: $dockerfile_path" "DOCKER"
}

# ==============================================================================
# Docker Compose Configuration
# ==============================================================================

# Generate Docker Compose configuration
generate_docker_compose() {
    local profile="$1"
    
    log "INFO" "Generating Docker Compose configuration for profile: $profile" "DOCKER"
    
    case "$profile" in
        "dev")
            generate_dev_compose
            ;;
        "test")
            generate_test_compose
            ;;
        "prod")
            generate_prod_compose
            ;;
        "infra")
            generate_infra_compose
            ;;
    esac
    
    log "SUCCESS" "Docker Compose configuration generated" "DOCKER"
}

# Generate development Docker Compose
generate_dev_compose() {
    local compose_path="$PROJECT_ROOT/docker-compose.dev.yml"
    
    cat > "$compose_path" << 'EOF'
# ICN Development Docker Compose
# Includes all development services with hot reload

version: '3.8'

services:
  icn-node:
    build:
      context: .
      dockerfile: docker/Dockerfile.dev
    ports:
      - "8080:8080"
      - "9090:9090"
      - "9091:9091"
    environment:
      - ICN_PROFILE=dev
      - ICN_LOG_LEVEL=debug
      - RUST_LOG=icn=trace
      - ICN_DATABASE_URL=postgres://icn:icn_dev@postgres:5432/icn_dev
      - ICN_REDIS_URL=redis://redis:6379/0
    volumes:
      - .:/app
      - cargo_cache:/usr/local/cargo
      - target_cache:/app/target
      - ./.icn/dev/data:/app/data
      - ./.icn/dev/logs:/app/logs
    depends_on:
      - postgres
      - redis
    networks:
      - icn-network

  postgres:
    image: postgres:15
    environment:
      - POSTGRES_DB=icn_dev
      - POSTGRES_USER=icn
      - POSTGRES_PASSWORD=icn_dev
      - POSTGRES_HOST_AUTH_METHOD=trust
    ports:
      - "5432:5432"
    volumes:
      - postgres_dev_data:/var/lib/postgresql/data
      - ./scripts/sql/init.sql:/docker-entrypoint-initdb.d/init.sql
    networks:
      - icn-network

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_dev_data:/data
    networks:
      - icn-network

  # Frontend development services
  icn-web:
    image: node:20-alpine
    working_dir: /app
    ports:
      - "3000:3000"
    volumes:
      - ../icn-web:/app
    command: ["sh", "-c", "npm install && npm run dev"]
    networks:
      - icn-network

  icn-explorer:
    image: node:20-alpine
    working_dir: /app
    ports:
      - "3001:3001"
    volumes:
      - ../icn-explorer:/app
    command: ["sh", "-c", "npm install && npm run dev"]
    networks:
      - icn-network

  # Development tools
  mailhog:
    image: mailhog/mailhog:latest
    ports:
      - "1025:1025"
      - "8025:8025"
    networks:
      - icn-network

volumes:
  postgres_dev_data:
  redis_dev_data:
  cargo_cache:
  target_cache:

networks:
  icn-network:
    external: true
EOF
    
    log "DEBUG" "Development Docker Compose generated: $compose_path" "DOCKER"
}

# Generate test Docker Compose
generate_test_compose() {
    local compose_path="$PROJECT_ROOT/docker-compose.test.yml"
    
    cat > "$compose_path" << 'EOF'
# ICN Test Docker Compose
# Optimized for testing with isolated services

version: '3.8'

services:
  icn-node:
    build:
      context: .
      dockerfile: docker/Dockerfile.test
    ports:
      - "8081:8081"
    environment:
      - ICN_PROFILE=test
      - ICN_LOG_LEVEL=warn
      - RUST_LOG=icn=info
      - ICN_DATABASE_URL=postgres://icn:icn_test@postgres:5432/icn_test
      - ICN_REDIS_URL=redis://redis:6379/1
      - ICN_TEST_MODE=true
    volumes:
      - ./.icn/test/data:/app/data
      - ./.icn/test/logs:/app/logs
    depends_on:
      - postgres
      - redis
    networks:
      - icn-test-network

  postgres:
    image: postgres:15
    environment:
      - POSTGRES_DB=icn_test
      - POSTGRES_USER=icn
      - POSTGRES_PASSWORD=icn_test
      - POSTGRES_HOST_AUTH_METHOD=trust
    volumes:
      - postgres_test_data:/var/lib/postgresql/data
      - ./scripts/sql/test_init.sql:/docker-entrypoint-initdb.d/init.sql
    networks:
      - icn-test-network

  redis:
    image: redis:7-alpine
    volumes:
      - redis_test_data:/data
    networks:
      - icn-test-network

  # Test runner
  test-runner:
    build:
      context: .
      dockerfile: docker/Dockerfile.test
    command: ["cargo", "test", "--workspace", "--all-features", "--", "--nocapture"]
    environment:
      - ICN_PROFILE=test
      - ICN_DATABASE_URL=postgres://icn:icn_test@postgres:5432/icn_test
      - ICN_REDIS_URL=redis://redis:6379/1
    volumes:
      - .:/app
      - ./.icn/test/data:/app/data
      - ./.icn/test/logs:/app/logs
    depends_on:
      - postgres
      - redis
    networks:
      - icn-test-network

volumes:
  postgres_test_data:
  redis_test_data:

networks:
  icn-test-network:
    driver: bridge
EOF
    
    log "DEBUG" "Test Docker Compose generated: $compose_path" "DOCKER"
}

# Generate production Docker Compose
generate_prod_compose() {
    local compose_path="$PROJECT_ROOT/docker-compose.prod.yml"
    
    cat > "$compose_path" << 'EOF'
# ICN Production Docker Compose
# Production-ready with security and monitoring

version: '3.8'

services:
  icn-node:
    build:
      context: .
      dockerfile: docker/Dockerfile.prod
    ports:
      - "80:80"
    environment:
      - ICN_PROFILE=prod
      - ICN_LOG_LEVEL=info
      - RUST_LOG=icn=info
      - ICN_DATABASE_URL=postgres://icn:${ICN_DB_PASSWORD}@postgres:5432/icn_prod
      - ICN_REDIS_URL=redis://redis:6379/0
    volumes:
      - ./.icn/prod/data:/app/data
      - ./.icn/prod/logs:/app/logs
      - ./.icn/prod/keys:/app/keys:ro
    depends_on:
      - postgres
      - redis
    networks:
      - icn-prod-network
    deploy:
      replicas: 3
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
      resources:
        limits:
          cpus: '2'
          memory: 4G
        reservations:
          cpus: '1'
          memory: 2G

  postgres:
    image: postgres:15
    environment:
      - POSTGRES_DB=icn_prod
      - POSTGRES_USER=icn
      - POSTGRES_PASSWORD=${ICN_DB_PASSWORD}
    volumes:
      - postgres_prod_data:/var/lib/postgresql/data
      - ./scripts/sql/prod_init.sql:/docker-entrypoint-initdb.d/init.sql
    networks:
      - icn-prod-network
    deploy:
      replicas: 1
      restart_policy:
        condition: on-failure
      resources:
        limits:
          cpus: '2'
          memory: 4G
        reservations:
          cpus: '1'
          memory: 2G

  redis:
    image: redis:7-alpine
    volumes:
      - redis_prod_data:/data
    networks:
      - icn-prod-network
    deploy:
      replicas: 1
      restart_policy:
        condition: on-failure

  # Production monitoring
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    networks:
      - icn-prod-network

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD}
    volumes:
      - grafana_data:/var/lib/grafana
      - ./monitoring/grafana:/etc/grafana/provisioning
    networks:
      - icn-prod-network

  # Reverse proxy
  nginx:
    image: nginx:alpine
    ports:
      - "443:443"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf
      - ./nginx/ssl:/etc/nginx/ssl
    depends_on:
      - icn-node
    networks:
      - icn-prod-network

volumes:
  postgres_prod_data:
  redis_prod_data:
  prometheus_data:
  grafana_data:

networks:
  icn-prod-network:
    driver: overlay
    attachable: true
EOF
    
    log "DEBUG" "Production Docker Compose generated: $compose_path" "DOCKER"
}

# Generate infrastructure Docker Compose
generate_infra_compose() {
    local compose_path="$PROJECT_ROOT/docker-compose.infra.yml"
    
    cat > "$compose_path" << 'EOF'
# ICN Infrastructure Docker Compose
# Infrastructure management and monitoring stack

version: '3.8'

services:
  icn-node:
    build:
      context: .
      dockerfile: docker/Dockerfile.infra
    ports:
      - "8080:8080"
    environment:
      - ICN_PROFILE=infra
      - ICN_LOG_LEVEL=warn
    volumes:
      - ./.icn/infra/data:/app/data
      - ./.icn/infra/logs:/app/logs
      - ./.icn/infra/keys:/app/keys:ro
      - ./scripts:/app/scripts:ro
      - /var/run/docker.sock:/var/run/docker.sock
    networks:
      - icn-infra-network

  # Full monitoring stack
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    networks:
      - icn-infra-network

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD}
    volumes:
      - grafana_data:/var/lib/grafana
      - ./monitoring/grafana:/etc/grafana/provisioning
    networks:
      - icn-infra-network

  alertmanager:
    image: prom/alertmanager:latest
    ports:
      - "9093:9093"
    volumes:
      - ./monitoring/alertmanager.yml:/etc/alertmanager/alertmanager.yml
      - alertmanager_data:/alertmanager
    networks:
      - icn-infra-network

  # Log aggregation
  loki:
    image: grafana/loki:latest
    ports:
      - "3100:3100"
    volumes:
      - ./monitoring/loki.yml:/etc/loki/local-config.yaml
      - loki_data:/loki
    networks:
      - icn-infra-network

  promtail:
    image: grafana/promtail:latest
    volumes:
      - ./monitoring/promtail.yml:/etc/promtail/config.yml
      - ./.icn/infra/logs:/var/log/icn
    networks:
      - icn-infra-network

  # Infrastructure tools
  portainer:
    image: portainer/portainer-ce:latest
    ports:
      - "9000:9000"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - portainer_data:/data
    networks:
      - icn-infra-network

volumes:
  prometheus_data:
  grafana_data:
  alertmanager_data:
  loki_data:
  portainer_data:

networks:
  icn-infra-network:
    driver: bridge
EOF
    
    log "DEBUG" "Infrastructure Docker Compose generated: $compose_path" "DOCKER"
}

# ==============================================================================
# Container Service Management
# ==============================================================================

# Start Docker services
start_docker_services() {
    local profile="$1"
    local services="${2:-all}"
    
    log "INFO" "Starting Docker services for profile: $profile" "DOCKER"
    
    local compose_file="docker-compose.$profile.yml"
    
    if [[ ! -f "$compose_file" ]]; then
        log "ERROR" "Docker Compose file not found: $compose_file" "DOCKER"
        return 1
    fi
    
    # Create ICN network if it doesn't exist
    create_icn_network
    
    # Start services
    if [[ "$services" == "all" ]]; then
        docker-compose -f "$compose_file" up -d
    else
        docker-compose -f "$compose_file" up -d $services
    fi
    
    log "SUCCESS" "Docker services started" "DOCKER"
}

# Stop Docker services
stop_docker_services() {
    local profile="$1"
    local services="${2:-all}"
    
    log "INFO" "Stopping Docker services for profile: $profile" "DOCKER"
    
    local compose_file="docker-compose.$profile.yml"
    
    if [[ ! -f "$compose_file" ]]; then
        log "ERROR" "Docker Compose file not found: $compose_file" "DOCKER"
        return 1
    fi
    
    # Stop services
    if [[ "$services" == "all" ]]; then
        docker-compose -f "$compose_file" down
    else
        docker-compose -f "$compose_file" stop $services
    fi
    
    log "SUCCESS" "Docker services stopped" "DOCKER"
}

# Restart Docker services
restart_docker_services() {
    local profile="$1"
    local services="${2:-all}"
    
    log "INFO" "Restarting Docker services for profile: $profile" "DOCKER"
    
    stop_docker_services "$profile" "$services"
    start_docker_services "$profile" "$services"
    
    log "SUCCESS" "Docker services restarted" "DOCKER"
}

# Show Docker service logs
show_docker_logs() {
    local profile="$1"
    local service="${2:-icn-node}"
    local follow="${3:-false}"
    
    log "INFO" "Showing logs for service: $service" "DOCKER"
    
    local compose_file="docker-compose.$profile.yml"
    
    if [[ ! -f "$compose_file" ]]; then
        log "ERROR" "Docker Compose file not found: $compose_file" "DOCKER"
        return 1
    fi
    
    # Show logs
    if [[ "$follow" == "true" ]]; then
        docker-compose -f "$compose_file" logs -f "$service"
    else
        docker-compose -f "$compose_file" logs "$service"
    fi
}

# Show Docker service status
show_docker_status() {
    local profile="$1"
    
    log "INFO" "Showing Docker service status for profile: $profile" "DOCKER"
    
    local compose_file="docker-compose.$profile.yml"
    
    if [[ ! -f "$compose_file" ]]; then
        log "ERROR" "Docker Compose file not found: $compose_file" "DOCKER"
        return 1
    fi
    
    # Show status
    docker-compose -f "$compose_file" ps
}

# Build Docker images
build_docker_images() {
    local profile="$1"
    local force_rebuild="${2:-false}"
    
    log "INFO" "Building Docker images for profile: $profile" "DOCKER"
    
    local compose_file="docker-compose.$profile.yml"
    
    if [[ ! -f "$compose_file" ]]; then
        log "ERROR" "Docker Compose file not found: $compose_file" "DOCKER"
        return 1
    fi
    
    # Build images
    if [[ "$force_rebuild" == "true" ]]; then
        docker-compose -f "$compose_file" build --no-cache
    else
        docker-compose -f "$compose_file" build
    fi
    
    log "SUCCESS" "Docker images built" "DOCKER"
}

# ==============================================================================
# Resource Monitoring
# ==============================================================================

# Monitor Docker resources
monitor_docker_resources() {
    local profile="$1"
    local interval="${2:-5}"
    
    log "INFO" "Monitoring Docker resources (interval: ${interval}s)" "DOCKER"
    
    while true; do
        clear
        echo "ICN Docker Resource Monitor - Profile: $profile"
        echo "=================================================="
        echo "Time: $(date)"
        echo
        
        # Show container stats
        echo "Container Resource Usage:"
        docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.MemPerc}}\t{{.NetIO}}\t{{.BlockIO}}"
        
        echo
        echo "System Resource Usage:"
        
        # Show system resources
        echo "Memory: $(free -h | grep Mem | awk '{print $3 "/" $2}')"
        echo "Disk: $(df -h / | tail -1 | awk '{print $3 "/" $2 " (" $5 ")"}')"
        
        # Show Docker system info
        echo
        echo "Docker System Info:"
        docker system df
        
        sleep "$interval"
    done
}

# Clean up Docker resources
cleanup_docker_resources() {
    local profile="$1"
    local aggressive="${2:-false}"
    
    log "INFO" "Cleaning up Docker resources for profile: $profile" "DOCKER"
    
    # Stop services first
    stop_docker_services "$profile"
    
    # Remove containers
    docker-compose -f "docker-compose.$profile.yml" rm -f
    
    # Clean up system resources
    docker system prune -f
    
    if [[ "$aggressive" == "true" ]]; then
        log "WARN" "Performing aggressive cleanup" "DOCKER"
        
        # Remove all unused images
        docker image prune -a -f
        
        # Remove all unused volumes
        docker volume prune -f
        
        # Remove all unused networks
        docker network prune -f
    fi
    
    log "SUCCESS" "Docker resources cleaned up" "DOCKER"
}

# ==============================================================================
# Development Scripts Generation
# ==============================================================================

# Generate development scripts
generate_dev_scripts() {
    local profile="$1"
    
    log "INFO" "Generating development scripts for profile: $profile" "DOCKER"
    
    local scripts_dir="$PROJECT_ROOT/.icn/$profile/scripts"
    mkdir -p "$scripts_dir"
    
    # Generate start script
    cat > "$scripts_dir/start.sh" << EOF
#!/bin/bash
# ICN Docker Start Script - Profile: $profile
# Generated on: $(date)

cd "\$(dirname "\$0")/../../.."
source scripts/docker-utils.sh

echo "Starting ICN Docker services for profile: $profile"
start_docker_services "$profile"
EOF
    
    # Generate stop script
    cat > "$scripts_dir/stop.sh" << EOF
#!/bin/bash
# ICN Docker Stop Script - Profile: $profile
# Generated on: $(date)

cd "\$(dirname "\$0")/../../.."
source scripts/docker-utils.sh

echo "Stopping ICN Docker services for profile: $profile"
stop_docker_services "$profile"
EOF
    
    # Generate logs script
    cat > "$scripts_dir/logs.sh" << EOF
#!/bin/bash
# ICN Docker Logs Script - Profile: $profile
# Generated on: $(date)

cd "\$(dirname "\$0")/../../.."
source scripts/docker-utils.sh

service="\${1:-icn-node}"
follow="\${2:-false}"

echo "Showing logs for service: \$service"
show_docker_logs "$profile" "\$service" "\$follow"
EOF
    
    # Generate status script
    cat > "$scripts_dir/status.sh" << EOF
#!/bin/bash
# ICN Docker Status Script - Profile: $profile
# Generated on: $(date)

cd "\$(dirname "\$0")/../../.."
source scripts/docker-utils.sh

echo "Showing Docker service status for profile: $profile"
show_docker_status "$profile"
EOF
    
    # Generate build script
    cat > "$scripts_dir/build.sh" << EOF
#!/bin/bash
# ICN Docker Build Script - Profile: $profile
# Generated on: $(date)

cd "\$(dirname "\$0")/../../.."
source scripts/docker-utils.sh

force_rebuild="\${1:-false}"

echo "Building Docker images for profile: $profile"
build_docker_images "$profile" "\$force_rebuild"
EOF
    
    # Make scripts executable
    chmod +x "$scripts_dir"/*.sh
    
    log "SUCCESS" "Development scripts generated: $scripts_dir" "DOCKER"
}

# ==============================================================================
# Docker Utility Aliases
# ==============================================================================

# Generate Docker-specific aliases
generate_docker_aliases() {
    local profile="$1"
    
    cat << EOF

# Docker Management Aliases
alias icn-docker-start='start_docker_services $profile'
alias icn-docker-stop='stop_docker_services $profile'
alias icn-docker-restart='restart_docker_services $profile'
alias icn-docker-status='show_docker_status $profile'
alias icn-docker-logs='show_docker_logs $profile'
alias icn-docker-build='build_docker_images $profile'
alias icn-docker-cleanup='cleanup_docker_resources $profile'
alias icn-docker-monitor='monitor_docker_resources $profile'

# Service-specific aliases
alias icn-node-logs='show_docker_logs $profile icn-node'
alias icn-postgres-logs='show_docker_logs $profile postgres'
alias icn-redis-logs='show_docker_logs $profile redis'

# Docker Compose shortcuts
alias icn-compose='docker-compose -f docker-compose.$profile.yml'
alias icn-exec='docker-compose -f docker-compose.$profile.yml exec'
alias icn-run='docker-compose -f docker-compose.$profile.yml run'

# Docker utilities
alias icn-docker-health='check_docker_health $profile'
alias icn-docker-network='create_icn_network'
alias icn-docker-generate='generate_docker_compose $profile'
EOF
}

log "SUCCESS" "Docker utilities loaded" "DOCKER" 