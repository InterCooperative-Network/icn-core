#!/bin/bash

# ==============================================================================
# ICN Frontend Utilities
# ==============================================================================
# Comprehensive frontend management for ICN development
# Handles multiple repositories, development servers, and build automation

# ==============================================================================
# Frontend Configuration
# ==============================================================================

# Define frontend repository configurations
declare -A FRONTEND_REPOS=(
    ["icn-web"]="icn-web:3000:next"
    ["icn-explorer"]="icn-explorer:3001:next"
    ["icn-wallet"]="icn-wallet:3002:next"
    ["icn-docs"]="icn-docs:3003:astro"
    ["icn-agoranet"]="icn-agoranet:3004:next"
    ["icn-website"]="icn-website:3005:astro"
    ["icn.zone"]="icn.zone:3006:astro"
)

# Framework configurations
declare -A FRAMEWORK_CONFIGS=(
    ["next"]="package.json:next:npm:dev"
    ["astro"]="package.json:astro:pnpm:dev"
    ["react"]="package.json:react-scripts:npm:start"
    ["vue"]="package.json:vue-cli:npm:serve"
)

# ==============================================================================
# Node.js Environment Setup
# ==============================================================================

# Setup Node.js environment
setup_node_environment() {
    local profile="$1"
    
    log "INFO" "Setting up Node.js environment for profile: $profile" "FRONTEND"
    
    # Check if Node.js is installed
    if ! command -v node &> /dev/null; then
        log "ERROR" "Node.js not found. Please install Node.js first." "FRONTEND"
        return 1
    fi
    
    # Check Node.js version
    local node_version=$(node --version | cut -d'v' -f2)
    log "INFO" "Node.js version: $node_version" "FRONTEND"
    
    # Ensure minimum Node.js version (18.0.0)
    if ! node -e "process.exit(process.version.slice(1).split('.')[0] >= 18 ? 0 : 1)" 2>/dev/null; then
        log "ERROR" "Node.js version 18 or higher is required. Current: $node_version" "FRONTEND"
        return 1
    fi
    
    # Setup pnpm (preferred package manager)
    setup_pnpm
    
    # Setup global development tools
    setup_global_tools
    
    log "SUCCESS" "Node.js environment setup completed" "FRONTEND"
}

# Setup pnpm package manager
setup_pnpm() {
    log "INFO" "Setting up pnpm package manager" "FRONTEND"
    
    if ! command -v pnpm &> /dev/null; then
        log "INFO" "Installing pnpm..." "FRONTEND"
        npm install -g pnpm
        
        if ! command -v pnpm &> /dev/null; then
            log "ERROR" "Failed to install pnpm" "FRONTEND"
            return 1
        fi
    fi
    
    local pnpm_version=$(pnpm --version)
    log "INFO" "pnpm version: $pnpm_version" "FRONTEND"
    
    # Configure pnpm for monorepo if needed
    if [[ -f "$PROJECT_ROOT/pnpm-workspace.yaml" ]]; then
        log "INFO" "Configuring pnpm workspace" "FRONTEND"
        cd "$PROJECT_ROOT"
        pnpm install
    fi
    
    log "SUCCESS" "pnpm setup completed" "FRONTEND"
}

# Setup global development tools
setup_global_tools() {
    log "INFO" "Setting up global development tools" "FRONTEND"
    
    local global_tools=(
        "@next/cli"
        "@astro/cli"
        "typescript"
        "eslint"
        "prettier"
        "serve"
        "concurrently"
        "nodemon"
    )
    
    for tool in "${global_tools[@]}"; do
        if ! npm list -g "$tool" &> /dev/null; then
            log "INFO" "Installing global tool: $tool" "FRONTEND"
            npm install -g "$tool"
        else
            log "DEBUG" "Global tool already installed: $tool" "FRONTEND"
        fi
    done
    
    log "SUCCESS" "Global development tools setup completed" "FRONTEND"
}

# ==============================================================================
# Frontend Repository Management
# ==============================================================================

# Setup frontend repositories
setup_frontend_repositories() {
    local profile="$1"
    
    log "INFO" "Setting up frontend repositories for profile: $profile" "FRONTEND"
    
    # Create frontend directory structure
    mkdir -p "$PROJECT_ROOT/../frontend"
    
    # Setup each repository
    for repo_name in "${!FRONTEND_REPOS[@]}"; do
        setup_frontend_repository "$repo_name" "$profile"
    done
    
    log "SUCCESS" "Frontend repositories setup completed" "FRONTEND"
}

# Setup individual frontend repository
setup_frontend_repository() {
    local repo_name="$1"
    local profile="$2"
    
    log "INFO" "Setting up repository: $repo_name" "FRONTEND"
    
    # Parse repository configuration
    local repo_config="${FRONTEND_REPOS[$repo_name]}"
    IFS=':' read -r repo_dir port framework <<< "$repo_config"
    
    local repo_path="$PROJECT_ROOT/../$repo_dir"
    
    # Clone or update repository
    if [[ ! -d "$repo_path" ]]; then
        log "INFO" "Repository not found, creating template: $repo_name" "FRONTEND"
        create_frontend_template "$repo_name" "$repo_path" "$framework" "$port"
    else
        log "INFO" "Repository exists, updating: $repo_name" "FRONTEND"
        update_frontend_repository "$repo_path" "$framework"
    fi
    
    # Setup development configuration
    setup_frontend_config "$repo_path" "$profile" "$port"
    
    log "SUCCESS" "Repository setup completed: $repo_name" "FRONTEND"
}

# Create frontend template
create_frontend_template() {
    local repo_name="$1"
    local repo_path="$2"
    local framework="$3"
    local port="$4"
    
    log "INFO" "Creating template for: $repo_name ($framework)" "FRONTEND"
    
    case "$framework" in
        "next")
            create_nextjs_template "$repo_name" "$repo_path" "$port"
            ;;
        "astro")
            create_astro_template "$repo_name" "$repo_path" "$port"
            ;;
        "react")
            create_react_template "$repo_name" "$repo_path" "$port"
            ;;
        *)
            log "ERROR" "Unsupported framework: $framework" "FRONTEND"
            return 1
            ;;
    esac
    
    log "SUCCESS" "Template created: $repo_name" "FRONTEND"
}

# Create Next.js template
create_nextjs_template() {
    local repo_name="$1"
    local repo_path="$2"
    local port="$3"
    
    log "INFO" "Creating Next.js template: $repo_name" "FRONTEND"
    
    # Create Next.js app
    npx create-next-app@latest "$repo_path" --typescript --tailwind --eslint --app --src-dir --import-alias "@/*" --yes
    
    cd "$repo_path"
    
    # Install additional dependencies
    pnpm add @tanstack/react-query axios zustand
    pnpm add -D @types/node
    
    # Create ICN-specific configuration
    cat > "next.config.js" << EOF
/** @type {import('next').NextConfig} */
const nextConfig = {
  env: {
    ICN_API_URL: process.env.ICN_API_URL || 'http://localhost:8080',
    ICN_WS_URL: process.env.ICN_WS_URL || 'ws://localhost:8080/ws',
  },
  async rewrites() {
    return [
      {
        source: '/api/:path*',
        destination: '\${process.env.ICN_API_URL}/api/:path*',
      },
    ];
  },
};

module.exports = nextConfig;
EOF
    
    # Create ICN components directory
    mkdir -p "src/components/icn"
    mkdir -p "src/lib/icn"
    mkdir -p "src/hooks/icn"
    
    # Create ICN API client
    cat > "src/lib/icn/api.ts" << 'EOF'
import axios from 'axios';

export const icnApi = axios.create({
  baseURL: process.env.ICN_API_URL || 'http://localhost:8080',
  headers: {
    'Content-Type': 'application/json',
  },
});

// Request interceptor for authentication
icnApi.interceptors.request.use((config) => {
  const token = localStorage.getItem('icn_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Response interceptor for error handling
icnApi.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('icn_token');
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);
EOF
    
    # Create ICN types
    cat > "src/lib/icn/types.ts" << 'EOF'
export interface Did {
  id: string;
  method: string;
  identifier: string;
}

export interface MeshJob {
  id: string;
  submitter: Did;
  specification: JobSpecification;
  status: JobStatus;
  created_at: string;
  updated_at: string;
}

export interface JobSpecification {
  command: string;
  resources: ResourceRequirements;
  timeout: number;
  max_cost: number;
}

export interface ResourceRequirements {
  cpu: number;
  memory: number;
  disk: number;
}

export type JobStatus = 'pending' | 'assigned' | 'running' | 'completed' | 'failed' | 'cancelled';

export interface ManaAccount {
  did: Did;
  balance: number;
  capacity: number;
  regeneration_rate: number;
  last_regeneration: string;
}

export interface ExecutionReceipt {
  id: string;
  job_id: string;
  executor: Did;
  result: any;
  signature: string;
  timestamp: string;
}
EOF
    
    # Update package.json with custom scripts
    cat > "package.json.tmp" << EOF
{
  "name": "$repo_name",
  "version": "0.1.0",
  "scripts": {
    "dev": "next dev -p $port",
    "build": "next build",
    "start": "next start -p $port",
    "lint": "next lint",
    "type-check": "tsc --noEmit",
    "icn:health": "curl -f http://localhost:$port/api/health || exit 1"
  },
  "dependencies": {
    "next": "latest",
    "react": "latest",
    "react-dom": "latest",
    "@tanstack/react-query": "latest",
    "axios": "latest",
    "zustand": "latest",
    "tailwindcss": "latest",
    "autoprefixer": "latest",
    "postcss": "latest"
  },
  "devDependencies": {
    "@types/node": "latest",
    "@types/react": "latest",
    "@types/react-dom": "latest",
    "eslint": "latest",
    "eslint-config-next": "latest",
    "typescript": "latest"
  }
}
EOF
    
    # Merge with existing package.json
    jq -s '.[0] * .[1]' package.json package.json.tmp > package.json.new
    mv package.json.new package.json
    rm package.json.tmp
    
    # Install dependencies
    pnpm install
    
    log "SUCCESS" "Next.js template created: $repo_name" "FRONTEND"
}

# Create Astro template
create_astro_template() {
    local repo_name="$1"
    local repo_path="$2"
    local port="$3"
    
    log "INFO" "Creating Astro template: $repo_name" "FRONTEND"
    
    # Create Astro project
    npm create astro@latest "$repo_path" -- --template minimal --typescript --yes
    
    cd "$repo_path"
    
    # Install additional dependencies
    pnpm add @astrojs/tailwind @astrojs/react @astrojs/mdx
    pnpm add -D @types/node
    
    # Create ICN-specific Astro configuration
    cat > "astro.config.mjs" << EOF
import { defineConfig } from 'astro/config';
import tailwind from '@astrojs/tailwind';
import react from '@astrojs/react';
import mdx from '@astrojs/mdx';

export default defineConfig({
  integrations: [tailwind(), react(), mdx()],
  server: {
    port: $port,
    host: true
  },
  vite: {
    define: {
      'process.env.ICN_API_URL': JSON.stringify(process.env.ICN_API_URL || 'http://localhost:8080'),
    }
  }
});
EOF
    
    # Create ICN components directory
    mkdir -p "src/components/icn"
    mkdir -p "src/lib/icn"
    mkdir -p "src/content/icn"
    
    # Create ICN API client for Astro
    cat > "src/lib/icn/client.ts" << 'EOF'
export class IcnClient {
  private baseUrl: string;
  
  constructor(baseUrl: string = 'http://localhost:8080') {
    this.baseUrl = baseUrl;
  }
  
  async request(endpoint: string, options: RequestInit = {}) {
    const url = `${this.baseUrl}${endpoint}`;
    
    const response = await fetch(url, {
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
      ...options,
    });
    
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    
    return response.json();
  }
  
  async getJobs() {
    return this.request('/api/v1/jobs');
  }
  
  async getJob(id: string) {
    return this.request(`/api/v1/jobs/${id}`);
  }
  
  async getAccount(did: string) {
    return this.request(`/api/v1/accounts/${did}`);
  }
}

export const icnClient = new IcnClient(process.env.ICN_API_URL);
EOF
    
    # Update package.json with custom scripts
    jq '.scripts.dev = "astro dev --port '$port'"' package.json > package.json.tmp
    jq '.scripts.start = "astro dev --port '$port'"' package.json.tmp > package.json.tmp2
    jq '.scripts["icn:health"] = "curl -f http://localhost:'$port'/health || exit 1"' package.json.tmp2 > package.json
    rm package.json.tmp package.json.tmp2
    
    # Install dependencies
    pnpm install
    
    log "SUCCESS" "Astro template created: $repo_name" "FRONTEND"
}

# Create React template
create_react_template() {
    local repo_name="$1"
    local repo_path="$2"
    local port="$3"
    
    log "INFO" "Creating React template: $repo_name" "FRONTEND"
    
    # Create React app
    npx create-react-app "$repo_path" --template typescript
    
    cd "$repo_path"
    
    # Install additional dependencies
    npm install axios @tanstack/react-query zustand
    npm install -D @types/node
    
    # Create ICN-specific configuration
    cat > ".env.local" << EOF
PORT=$port
REACT_APP_ICN_API_URL=http://localhost:8080
REACT_APP_ICN_WS_URL=ws://localhost:8080/ws
EOF
    
    # Create ICN components directory
    mkdir -p "src/components/icn"
    mkdir -p "src/lib/icn"
    mkdir -p "src/hooks/icn"
    
    # Update package.json with custom scripts
    jq '.scripts.start = "PORT='$port' react-scripts start"' package.json > package.json.tmp
    jq '.scripts["icn:health"] = "curl -f http://localhost:'$port'/health || exit 1"' package.json.tmp > package.json
    rm package.json.tmp
    
    log "SUCCESS" "React template created: $repo_name" "FRONTEND"
}

# Update existing frontend repository
update_frontend_repository() {
    local repo_path="$1"
    local framework="$2"
    
    log "INFO" "Updating repository: $repo_path ($framework)" "FRONTEND"
    
    cd "$repo_path"
    
    # Update dependencies based on framework
    case "$framework" in
        "next"|"react")
            if command -v pnpm &> /dev/null; then
                pnpm update
            else
                npm update
            fi
            ;;
        "astro")
            pnpm update
            ;;
    esac
    
    log "SUCCESS" "Repository updated: $repo_path" "FRONTEND"
}

# Setup frontend configuration
setup_frontend_config() {
    local repo_path="$1"
    local profile="$2"
    local port="$3"
    
    log "INFO" "Setting up configuration for: $repo_path" "FRONTEND"
    
    cd "$repo_path"
    
    # Create profile-specific environment file
    cat > ".env.${profile}" << EOF
# ICN Frontend Configuration - Profile: $profile
# Generated on: $(date)

# API Configuration
ICN_API_URL=http://localhost:8080
ICN_WS_URL=ws://localhost:8080/ws

# Development Configuration
PORT=$port
NODE_ENV=$profile

# Profile-specific settings
EOF
    
    case "$profile" in
        "dev")
            cat >> ".env.${profile}" << EOF
ICN_LOG_LEVEL=debug
ICN_ENABLE_DEBUG=true
ICN_ENABLE_HOT_RELOAD=true
EOF
            ;;
        "test")
            cat >> ".env.${profile}" << EOF
ICN_LOG_LEVEL=warn
ICN_ENABLE_DEBUG=false
ICN_TEST_MODE=true
EOF
            ;;
        "prod")
            cat >> ".env.${profile}" << EOF
ICN_LOG_LEVEL=error
ICN_ENABLE_DEBUG=false
ICN_ENABLE_ANALYTICS=true
EOF
            ;;
    esac
    
    # Create ICN-specific configuration
    if [[ ! -f "icn.config.js" ]]; then
        cat > "icn.config.js" << EOF
// ICN Frontend Configuration
module.exports = {
  apiUrl: process.env.ICN_API_URL || 'http://localhost:8080',
  wsUrl: process.env.ICN_WS_URL || 'ws://localhost:8080/ws',
  profile: process.env.NODE_ENV || 'dev',
  features: {
    meshJobs: true,
    governance: true,
    economics: true,
    identity: true,
  },
  ui: {
    theme: 'icn-default',
    brand: {
      name: 'InterCooperative Network',
      logo: '/logo.svg',
    },
  },
};
EOF
    fi
    
    log "SUCCESS" "Configuration setup completed: $repo_path" "FRONTEND"
}

# ==============================================================================
# Development Server Management
# ==============================================================================

# Start all frontend development servers
start_frontend_servers() {
    local profile="$1"
    
    log "INFO" "Starting frontend development servers for profile: $profile" "FRONTEND"
    
    # Check if tmux is available
    if ! command -v tmux &> /dev/null; then
        log "ERROR" "tmux is required for managing multiple servers" "FRONTEND"
        return 1
    fi
    
    # Create tmux session
    local session_name="icn-frontend-$profile"
    
    if tmux has-session -t "$session_name" 2>/dev/null; then
        log "INFO" "Tmux session already exists: $session_name" "FRONTEND"
        tmux attach-session -t "$session_name"
        return 0
    fi
    
    # Create new tmux session
    tmux new-session -d -s "$session_name"
    
    # Start each frontend server in a separate window
    local window_index=0
    for repo_name in "${!FRONTEND_REPOS[@]}"; do
        local repo_config="${FRONTEND_REPOS[$repo_name]}"
        IFS=':' read -r repo_dir port framework <<< "$repo_config"
        
        local repo_path="$PROJECT_ROOT/../$repo_dir"
        
        if [[ -d "$repo_path" ]]; then
            # Create tmux window
            if [[ "$window_index" -eq 0 ]]; then
                tmux rename-window -t "$session_name:0" "$repo_name"
            else
                tmux new-window -t "$session_name" -n "$repo_name"
            fi
            
            # Start development server
            tmux send-keys -t "$session_name:$repo_name" "cd $repo_path" Enter
            tmux send-keys -t "$session_name:$repo_name" "npm run dev" Enter
            
            ((window_index++))
            
            log "INFO" "Started server: $repo_name on port $port" "FRONTEND"
        else
            log "WARN" "Repository not found: $repo_path" "FRONTEND"
        fi
    done
    
    # Attach to tmux session
    tmux attach-session -t "$session_name"
    
    log "SUCCESS" "Frontend development servers started" "FRONTEND"
}

# Stop all frontend development servers
stop_frontend_servers() {
    local profile="$1"
    
    log "INFO" "Stopping frontend development servers for profile: $profile" "FRONTEND"
    
    local session_name="icn-frontend-$profile"
    
    if tmux has-session -t "$session_name" 2>/dev/null; then
        tmux kill-session -t "$session_name"
        log "SUCCESS" "Stopped tmux session: $session_name" "FRONTEND"
    else
        log "INFO" "No tmux session found: $session_name" "FRONTEND"
    fi
    
    # Kill any remaining processes on frontend ports
    for repo_name in "${!FRONTEND_REPOS[@]}"; do
        local repo_config="${FRONTEND_REPOS[$repo_name]}"
        IFS=':' read -r repo_dir port framework <<< "$repo_config"
        
        local pid=$(lsof -ti:$port 2>/dev/null)
        if [[ -n "$pid" ]]; then
            kill -9 "$pid"
            log "INFO" "Killed process on port $port" "FRONTEND"
        fi
    done
    
    log "SUCCESS" "Frontend development servers stopped" "FRONTEND"
}

# Restart frontend development servers
restart_frontend_servers() {
    local profile="$1"
    
    log "INFO" "Restarting frontend development servers for profile: $profile" "FRONTEND"
    
    stop_frontend_servers "$profile"
    sleep 2
    start_frontend_servers "$profile"
    
    log "SUCCESS" "Frontend development servers restarted" "FRONTEND"
}

# Show frontend server status
show_frontend_status() {
    local profile="$1"
    
    log "INFO" "Showing frontend server status for profile: $profile" "FRONTEND"
    
    local session_name="icn-frontend-$profile"
    
    echo "Tmux Session Status:"
    if tmux has-session -t "$session_name" 2>/dev/null; then
        echo "  Session: $session_name (ACTIVE)"
        tmux list-windows -t "$session_name" -F "  Window: #{window_name} (#{window_flags})"
    else
        echo "  Session: $session_name (INACTIVE)"
    fi
    
    echo ""
    echo "Port Status:"
    for repo_name in "${!FRONTEND_REPOS[@]}"; do
        local repo_config="${FRONTEND_REPOS[$repo_name]}"
        IFS=':' read -r repo_dir port framework <<< "$repo_config"
        
        if curl -s -f "http://localhost:$port" &> /dev/null; then
            echo "  $repo_name: Port $port (ACTIVE)"
        else
            echo "  $repo_name: Port $port (INACTIVE)"
        fi
    done
}

# ==============================================================================
# Build and Deployment
# ==============================================================================

# Build all frontend applications
build_frontend_apps() {
    local profile="$1"
    local parallel="${2:-true}"
    
    log "INFO" "Building frontend applications for profile: $profile" "FRONTEND"
    
    local build_commands=()
    
    # Prepare build commands
    for repo_name in "${!FRONTEND_REPOS[@]}"; do
        local repo_config="${FRONTEND_REPOS[$repo_name]}"
        IFS=':' read -r repo_dir port framework <<< "$repo_config"
        
        local repo_path="$PROJECT_ROOT/../$repo_dir"
        
        if [[ -d "$repo_path" ]]; then
            local build_cmd="cd $repo_path && npm run build"
            build_commands+=("$build_cmd")
            
            if [[ "$parallel" != "true" ]]; then
                log "INFO" "Building: $repo_name" "FRONTEND"
                eval "$build_cmd"
                
                if [[ $? -eq 0 ]]; then
                    log "SUCCESS" "Build completed: $repo_name" "FRONTEND"
                else
                    log "ERROR" "Build failed: $repo_name" "FRONTEND"
                fi
            fi
        else
            log "WARN" "Repository not found: $repo_path" "FRONTEND"
        fi
    done
    
    # Run parallel builds if requested
    if [[ "$parallel" == "true" ]]; then
        log "INFO" "Running parallel builds" "FRONTEND"
        
        # Use GNU parallel if available, otherwise use background jobs
        if command -v parallel &> /dev/null; then
            printf '%s\n' "${build_commands[@]}" | parallel -j4
        else
            local pids=()
            for cmd in "${build_commands[@]}"; do
                eval "$cmd" &
                pids+=($!)
            done
            
            # Wait for all builds to complete
            for pid in "${pids[@]}"; do
                wait "$pid"
            done
        fi
    fi
    
    log "SUCCESS" "Frontend applications built" "FRONTEND"
}

# Health check for frontend services
health_check_frontend() {
    local profile="$1"
    
    log "INFO" "Performing health check for frontend services" "FRONTEND"
    
    local healthy_services=0
    local total_services=0
    
    for repo_name in "${!FRONTEND_REPOS[@]}"; do
        local repo_config="${FRONTEND_REPOS[$repo_name]}"
        IFS=':' read -r repo_dir port framework <<< "$repo_config"
        
        ((total_services++))
        
        # Check if service is responding
        if curl -s -f "http://localhost:$port" &> /dev/null; then
            log "SUCCESS" "Service healthy: $repo_name (port $port)" "FRONTEND"
            ((healthy_services++))
        else
            log "ERROR" "Service unhealthy: $repo_name (port $port)" "FRONTEND"
        fi
    done
    
    local health_percentage=$((healthy_services * 100 / total_services))
    
    log "INFO" "Health check summary: $healthy_services/$total_services services healthy ($health_percentage%)" "FRONTEND"
    
    if [[ "$health_percentage" -ge 80 ]]; then
        log "SUCCESS" "Overall health check passed" "FRONTEND"
        return 0
    else
        log "ERROR" "Overall health check failed" "FRONTEND"
        return 1
    fi
}

# ==============================================================================
# Utility Functions
# ==============================================================================

# List all frontend repositories
list_frontend_repos() {
    log "INFO" "ICN Frontend Repositories:" "FRONTEND"
    
    for repo_name in "${!FRONTEND_REPOS[@]}"; do
        local repo_config="${FRONTEND_REPOS[$repo_name]}"
        IFS=':' read -r repo_dir port framework <<< "$repo_config"
        
        local repo_path="$PROJECT_ROOT/../$repo_dir"
        local status="NOT_FOUND"
        
        if [[ -d "$repo_path" ]]; then
            if [[ -f "$repo_path/package.json" ]]; then
                status="READY"
            else
                status="INCOMPLETE"
            fi
        fi
        
        echo "  $repo_name:"
        echo "    Directory: $repo_dir"
        echo "    Port: $port"
        echo "    Framework: $framework"
        echo "    Status: $status"
        echo ""
    done
}

# Open frontend applications in browser
open_frontend_apps() {
    local profile="$1"
    
    log "INFO" "Opening frontend applications in browser" "FRONTEND"
    
    for repo_name in "${!FRONTEND_REPOS[@]}"; do
        local repo_config="${FRONTEND_REPOS[$repo_name]}"
        IFS=':' read -r repo_dir port framework <<< "$repo_config"
        
        local url="http://localhost:$port"
        
        # Check if service is running
        if curl -s -f "$url" &> /dev/null; then
            log "INFO" "Opening: $repo_name ($url)" "FRONTEND"
            
            # Open in browser (cross-platform)
            if command -v xdg-open &> /dev/null; then
                xdg-open "$url" &
            elif command -v open &> /dev/null; then
                open "$url" &
            elif command -v start &> /dev/null; then
                start "$url" &
            else
                log "WARN" "Could not open browser for: $url" "FRONTEND"
            fi
        else
            log "WARN" "Service not running: $repo_name" "FRONTEND"
        fi
    done
}

# Clean frontend build artifacts
clean_frontend_builds() {
    local profile="$1"
    
    log "INFO" "Cleaning frontend build artifacts for profile: $profile" "FRONTEND"
    
    for repo_name in "${!FRONTEND_REPOS[@]}"; do
        local repo_config="${FRONTEND_REPOS[$repo_name]}"
        IFS=':' read -r repo_dir port framework <<< "$repo_config"
        
        local repo_path="$PROJECT_ROOT/../$repo_dir"
        
        if [[ -d "$repo_path" ]]; then
            cd "$repo_path"
            
            # Clean build directories
            local build_dirs=("build" "dist" ".next" ".astro")
            for build_dir in "${build_dirs[@]}"; do
                if [[ -d "$build_dir" ]]; then
                    rm -rf "$build_dir"
                    log "INFO" "Removed build directory: $repo_name/$build_dir" "FRONTEND"
                fi
            done
            
            # Clean cache directories
            local cache_dirs=("node_modules/.cache" ".cache")
            for cache_dir in "${cache_dirs[@]}"; do
                if [[ -d "$cache_dir" ]]; then
                    rm -rf "$cache_dir"
                    log "INFO" "Removed cache directory: $repo_name/$cache_dir" "FRONTEND"
                fi
            done
        fi
    done
    
    log "SUCCESS" "Frontend build artifacts cleaned" "FRONTEND"
}

# ==============================================================================
# Frontend Utility Aliases
# ==============================================================================

# Generate frontend-specific aliases
generate_frontend_aliases() {
    local profile="$1"
    
    cat << EOF

# Frontend Management Aliases
alias icn-frontend-start='start_frontend_servers $profile'
alias icn-frontend-stop='stop_frontend_servers $profile'
alias icn-frontend-restart='restart_frontend_servers $profile'
alias icn-frontend-status='show_frontend_status $profile'
alias icn-frontend-build='build_frontend_apps $profile'
alias icn-frontend-health='health_check_frontend $profile'
alias icn-frontend-open='open_frontend_apps $profile'
alias icn-frontend-clean='clean_frontend_builds $profile'

# Repository Management Aliases
alias icn-frontend-list='list_frontend_repos'
alias icn-frontend-setup='setup_frontend_repositories $profile'

# Individual Repository Aliases
EOF
    
    for repo_name in "${!FRONTEND_REPOS[@]}"; do
        local repo_config="${FRONTEND_REPOS[$repo_name]}"
        IFS=':' read -r repo_dir port framework <<< "$repo_config"
        
        cat << EOF
alias icn-${repo_name%-*}-dev='cd $PROJECT_ROOT/../$repo_dir && npm run dev'
alias icn-${repo_name%-*}-build='cd $PROJECT_ROOT/../$repo_dir && npm run build'
alias icn-${repo_name%-*}-test='cd $PROJECT_ROOT/../$repo_dir && npm run test'
EOF
    done
    
    cat << EOF

# Framework-specific aliases
alias icn-next-create='create_nextjs_template'
alias icn-astro-create='create_astro_template'
alias icn-react-create='create_react_template'

# Package management aliases
alias icn-pnpm-install='pnpm install --recursive'
alias icn-pnpm-update='pnpm update --recursive'
alias icn-pnpm-clean='pnpm store prune'
EOF
}

log "SUCCESS" "Frontend utilities loaded" "FRONTEND" 