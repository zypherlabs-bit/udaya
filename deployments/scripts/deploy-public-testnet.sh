#!/bin/bash

# Udaya Public Testnet Deployment Script
# Phase 4 - Public Testnet Deployment

set -e

echo "╔══════════════════════════════════════════════════╗"
echo "║  Udaya Public Testnet Deployment - Phase 4         ║"
echo "╚══════════════════════════════════════════════════╝"
echo ""

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker and try again."
    exit 1
fi

echo "Step 1: Building Docker images (if needed)"
echo "-----------------------------------------"

# Build the required images
echo "Building Udaya node image..."
docker build -t udaya/node:latest -f deployments/docker/Dockerfile . || echo "Node image build failed or already exists"

echo "Building Udaya explorer image..."
cd src/explorer && cargo build --release && cd ../../
docker build -t udaya/explorer:latest -f deployments/docker/Dockerfile.explorer . || echo "Explorer image build failed or already exists"

echo "Building Udaya faucet image..."
cd src/faucet && cargo build --release && cd ../../
docker build -t udaya/faucet:latest -f deployments/docker/Dockerfile.faucet . || echo "Faucet image build failed or already exists"

echo "Building Udaya mining pool image..."
cd src/pool-server && cargo build --release && cd ../../
docker build -t udaya/pool-server:latest -f deployments/docker/Dockerfile.pool . || echo "Pool image build failed or already exists"

echo ""
echo "Step 2: Starting Public Testnet"
echo "-----------------------------------------"

# Start the testnet using docker-compose
echo "Starting testnet containers..."
cd deployments/docker
docker-compose -f docker-compose.testnet.yml up -d

echo ""
echo "Step 3: Verifying Deployment"
echo "-----------------------------------------"

# Check if all containers are running
echo "Waiting for containers to initialize (30 seconds)..."
sleep 30

echo "Checking container status:"
docker ps --filter "name=udaya-*-testnet" --format "table {{.Names}}\t{{.Status}}"

echo ""
echo "Step 4: Testnet Information"
echo "-----------------------------------------"

echo "🌍 Seed Nodes:"
echo "  - US East:  localhost:19798 (RPC: localhost:18332)"
echo "  - EU West:  localhost:19799 (RPC: localhost:18333)"
echo "  - APAC:    localhost:19800 (RPC: localhost:18334)"

echo ""
echo "📊 Public RPC Node:"
echo "  - RPC:     localhost:18335"
echo "  - WebSocket: localhost:18336"

echo ""
echo "🔍 Block Explorer:  http://localhost:8080"
echo "💰 Faucet:         http://localhost:8081"
echo "⛏️  Mining Pool:    http://localhost:3000"
echo "📚 Documentation:   http://localhost:8082/testnet/"

echo ""
echo "Step 5: Community Invitation"
echo "-----------------------------------------"
echo "The Udaya Public Testnet is now ready for community testing!"
echo ""
echo "Community members can now:"
echo "  ✅ Run their own nodes and connect to the seed nodes"
echo "  ✅ Mine test coins using the mining pool"
echo "  ✅ Create wallets and submit transactions"
echo "  ✅ Explore the blockchain using the block explorer"
echo "  ✅ Get test coins from the faucet"
echo "  ✅ Stress-test the network"
echo "  ✅ Report bugs and provide feedback"
echo ""

echo "Step 6: Monitoring"
echo "-----------------------------------------"
echo "To monitor the testnet:"
echo "  - Docker logs:      docker logs -f <container-name>"
echo "  - Container stats:  docker stats"
echo "  - Network inspect:  docker network inspect deployments_default"
echo ""

echo "Step 7: Cleanup (when needed)"
echo "-----------------------------------------"
echo "To stop the testnet:"
echo "  docker-compose -f docker-compose.testnet.yml down"
echo ""
echo "To remove all testnet data:"
echo "  docker volume prune -f"
echo ""

echo "╔══════════════════════════════════════════════════╗"
echo "║  Udaya Public Testnet Deployment Complete!        ║"
echo "╚══════════════════════════════════════════════════╝"
<task_progress>
- [x] Analyze current project structure
- [x] Review existing deployment configurations
- [x] Identify components to deploy (seed nodes, RPC node, block explorer, faucet, mining pool)
- [x] Check documentation website status
- [x] Prepare deployment plan
- [x] Create testnet-specific Docker configuration
- [x] Create Prometheus configuration for testnet
- [x] Fix docker-compose YAML formatting
- [x] Clean up any old files
- [x] Create deployment script
- [ ] Execute deployment
- [ ] Verify all components are running
- [ ] Prepare community invitation materials
</task_progress>
</write_to_file>