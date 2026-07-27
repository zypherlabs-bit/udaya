#!/usr/bin/env bash
# ============================================================
# Udaya Mainnet Deployment Script
# Production deployment of full Udaya ecosystem
# ============================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info()  { echo -e "${BLUE}[INFO]${NC} $1"; }
log_ok()    { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Environment detection
detect_env() {
    if command -v kubectl &> /dev/null; then
        echo "kubernetes"
    elif command -v docker &> /dev/null; then
        echo "docker"
    else
        echo "unknown"
    fi
}

# Validate prerequisites
check_prereqs() {
    log_info "Checking prerequisites..."
    
    # Check for Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed"
        return 1
    fi
    log_ok "Docker found: $(docker --version)"
    
    # Check for docker-compose
    if ! command -v docker-compose &> /dev/null; then
        log_warn "docker-compose not found, checking compose plugin..."
        if ! docker compose version &> /dev/null; then
            log_error "Docker Compose not available"
            return 1
        fi
    fi
    
    # Check disk space
    local available_space=$(df / | tail -1 | awk '{print $4}')
    if [ "$available_space" -lt 52428800 ]; then  # 50GB
        log_error "Insufficient disk space. Need at least 50GB"
        return 1
    fi
    log_ok "Disk space sufficient: $((available_space / 1024 / 1024))GB"
    
    # Check memory
    local total_mem=$(free -m | awk '/^Mem:/{print $2}')
    if [ "$total_mem" -lt 4096 ]; then
        log_error "Insufficient memory. Need at least 4GB"
        return 1
    fi
    log_ok "Memory sufficient: ${total_mem}MB"
    
    return 0
}

# Build Docker images
build_images() {
    log_info "Building Udaya Docker images..."
    
    docker build \
        -t Udaya/node:latest \
        -f "$PROJECT_ROOT/deployments/docker/Dockerfile" \
        "$PROJECT_ROOT"
    
    log_ok "Udaya node image built"
}

# Initialize blockchain
init_blockchain() {
    log_info "Initializing Udaya blockchain..."
    
    local data_dir="${udaya_DATA_DIR:-/var/lib/Udaya/data}"
    mkdir -p "$data_dir"
    
    # Generate initial configuration
    cat > "$data_dir/udaya.conf" << EOF
network = "mainnet"
[network]
listen_port = 9798
max_peers = 125
[storage]
data_dir = "$data_dir"
db_cache_size_mb = 2048
[rpc]
enable = true
listen_addr = "0.0.0.0"
listen_port = 8332
username = "${RPC_USER:-Udaya}"
password = "$(openssl rand -base64 32)"
EOF
    
    log_ok "Blockchain data directory initialized at $data_dir"
}

# Deploy with Docker Compose
deploy_docker_compose() {
    log_info "Deploying with Docker Compose..."
    
    cd "$PROJECT_ROOT/deployments/docker"
    
    # Pull external images
    docker compose pull
    
    # Build and start services
    docker compose up -d --build
    
    # Wait for services to be healthy
    log_info "Waiting for services to start..."
    sleep 10
    
    # Check service status
    docker compose ps
    
    log_ok "Docker Compose deployment complete"
}

# Deploy with Kubernetes
deploy_kubernetes() {
    log_info "Deploying to Kubernetes..."
    
    # Create namespace
    kubectl apply -f "$PROJECT_ROOT/deployments/k8s/namespace.yaml" 2>/dev/null || true
    
    # Deploy seed node
    kubectl apply -f "$PROJECT_ROOT/deployments/k8s/seed-node-deployment.yaml"
    
    # Wait for seed node
    log_info "Waiting for seed node to be ready..."
    kubectl wait --for=condition=ready pod -l app=Udaya,role=seed-node \
        --namespace=Udaya --timeout=300s
    
    # Deploy explorer
    kubectl apply -f "$PROJECT_ROOT/deployments/k8s/explorer-deployment.yaml" 2>/dev/null || true
    
    # Deploy monitoring
    kubectl apply -f "$PROJECT_ROOT/deployments/k8s/monitoring.yaml" 2>/dev/null || true
    
    log_ok "Kubernetes deployment complete"
}

# Verify deployment
verify_deployment() {
    log_info "Verifying deployment..."
    
    local rpc_port="${udaya_rpc_PORT:-8332}"
    
    # Test RPC endpoint
    if command -v curl &> /dev/null; then
        if curl -s -o /dev/null -w "%{http_code}" \
            --connect-timeout 5 \
            "http://localhost:$rpc_port/health" 2>/dev/null | grep -q 200; then
            log_ok "RPC endpoint healthy"
        else
            log_warn "RPC endpoint not responding"
        fi
    fi
    
    # Check if node is running
    if command -v pgrep &> /dev/null && pgrep -x "udayad" > /dev/null; then
        log_ok "Udaya node process running"
    else
        log_warn "Udaya node not running locally (may be in container)"
    fi
}

# Main deployment flow
main() {
    echo "============================================="
    echo "  Udaya Mainnet Deployment"
    echo "  Version: 1.0.0"
    echo "============================================="
    echo ""
    
    check_prereqs || exit 1
    
    local env=$(detect_env)
    log_info "Detected environment: $env"
    
    case "$env" in
        kubernetes)
            deploy_kubernetes
            ;;
        docker)
            build_images
            deploy_docker_compose
            ;;
        *)
            log_error "No supported deployment environment detected"
            exit 1
            ;;
    esac
    
    verify_deployment
    
    echo ""
    echo "============================================="
    echo -e "${GREEN}Udaya Mainnet Deployment Complete${NC}"
    echo "============================================="
    echo ""
    echo "Next steps:"
    echo "  1. Monitor node: docker compose logs -f"
    echo "  2. Check RPC: curl http://localhost:8332/health"
    echo "  3. View explorer: http://localhost:3000"
    echo "  4. Check metrics: http://localhost:9090"
    echo "  5. View dashboard: http://localhost:3001 (admin/admin)"
    echo ""
}

main "$@"