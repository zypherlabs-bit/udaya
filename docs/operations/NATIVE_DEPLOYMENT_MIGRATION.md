# Native Deployment Migration Guide

This document describes the migration from container-based deployment to native execution for the Udaya Blockchain project.

## Overview

The Udaya Blockchain project has been migrated from Docker/Kubernetes container-based deployment to native binary execution. This migration provides several benefits:

- **Simplified Deployment**: No need for container orchestration tools
- **Better Performance**: Direct execution without container overhead
- **Easier Debugging**: Native process debugging and profiling
- **Reduced Complexity**: Fewer moving parts in production

## Changes Made

### 1. Removed Container Components

The following container-related components have been removed:

- **Docker Files**: `deployments/docker/` directory (Dockerfiles, docker-compose.yml)
- **Kubernetes Manifests**: `deployments/k8s/` directory (YAML manifests)
- **Container Deployment Scripts**: `scripts/deploy/` directory (container-specific scripts)

### 2. Updated Build Process

The build process now focuses on native binary compilation:

```bash
# Build for native execution
cargo build --release

# The binary is now available at:
./target/release/udayad
```

### 3. Simplified Deployment

Native deployment is now straightforward:

```bash
# Start the node directly
./target/release/udayad --config config/mainnet/udaya.conf start

# Or use the deployment script for system integration
sudo cp deployments/scripts/deploy.sh /usr/local/bin/udaya-deploy
sudo udaya-deploy
```

### 4. Updated Documentation

All documentation has been updated to reflect native execution:

- **README.md**: Updated with native execution instructions
- **Deployment Scripts**: Modified to use native binaries
- **Configuration**: Simplified for native environments

## Migration Steps

### For Existing Deployments

If you're currently running Udaya in containers, follow these steps to migrate:

1. **Backup Your Data**:
   ```bash
   # Backup your blockchain data
   tar czf udaya-data-backup-$(date +%Y-%m-%d).tar.gz /data/udaya
   ```

2. **Install Prerequisites**:
   ```bash
   # Install Rust (if not already installed)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

3. **Build Native Binary**:
   ```bash
   git clone https://github.com/UdayaFoundation/Udaya.git
   cd Udaya
   cargo build --release
   ```

4. **Migrate Configuration**:
   ```bash
   # Copy your existing configuration
   cp /path/to/your/udaya.conf config/mainnet/udaya.conf

   # Update paths if needed (remove container-specific paths)
   sed -i 's|/data/udaya|/var/lib/udaya|g' config/mainnet/udaya.conf
   ```

5. **Deploy Native Binary**:
   ```bash
   # Install the binary
   sudo cp target/release/udayad /usr/local/bin/

   # Set up systemd service (optional)
   sudo cp deployments/scripts/deploy.sh /usr/local/bin/udaya-deploy
   sudo udaya-deploy
   ```

6. **Start the Node**:
   ```bash
   # Start manually
   udayad --config config/mainnet/udaya.conf start

   # Or use systemd
   sudo systemctl start udayad
   ```

### For New Deployments

New deployments can now use the simplified native approach:

1. **Install Rust**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Build Udaya**:
   ```bash
   git clone https://github.com/UdayaFoundation/Udaya.git
   cd Udaya
   cargo build --release
   ```

3. **Configure**:
   ```bash
   cp config/bitfury.conf config/udaya.conf
   # Edit config/udaya.conf with your settings
   ```

4. **Run**:
   ```bash
   ./target/release/udayad --config config/udaya.conf start
   ```

## Verification

Verify that your native deployment is working correctly:

```bash
# Check node status
curl http://localhost:8332/health

# Check blockchain info
./target/release/udayad --config config/udaya.conf get-info

# Check logs
tail -f /var/log/udaya/udaya.log
```

## Troubleshooting

### Common Issues

1. **Permission Issues**:
   ```bash
   # Ensure proper permissions
   sudo chown -R udaya:udaya /var/lib/udaya
   sudo chown -R udaya:udaya /var/log/udaya
   ```

2. **Port Conflicts**:
   ```bash
   # Check for port conflicts
   sudo netstat -tulnp | grep 9798
   sudo netstat -tulnp | grep 8332
   ```

3. **Missing Dependencies**:
   ```bash
   # Install required system libraries
   sudo apt-get install -y libssl-dev pkg-config
   ```

## Benefits of Native Deployment

### Performance

- **Lower Latency**: Direct system calls without container overhead
- **Better Resource Utilization**: No container runtime memory overhead
- **Faster Startup**: Immediate execution without container initialization

### Simplicity

- **Fewer Components**: No need for Docker daemon or Kubernetes cluster
- **Easier Debugging**: Native process debugging with gdb/lldb
- **Simpler Monitoring**: Standard system monitoring tools work directly

### Security

- **Smaller Attack Surface**: No container runtime vulnerabilities
- **Better Isolation**: Use standard OS security features
- **Easier Auditing**: Native process auditing

## Backward Compatibility

While container deployment is no longer the primary method, you can still containerize the native binary if needed:

```dockerfile
# Example Dockerfile for reference (not officially supported)
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    libssl1.1 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY target/release/udayad /usr/local/bin/
COPY config/mainnet/udaya.conf /etc/udaya/udaya.conf

USER udaya
WORKDIR /data/udaya

ENTRYPOINT ["udayad"]
CMD ["-c", "/etc/udaya/udaya.conf", "start"]
```

## Support

For migration assistance or issues, please:

- **Open an Issue**: https://github.com/UdayaFoundation/Udaya/issues
- **Join Discord**: https://discord.gg/udaya
- **Check Documentation**: https://docs.udaya.org

## Conclusion

The migration to native execution simplifies Udaya deployment while maintaining all functionality. The project now focuses on direct binary execution, making it easier to deploy, monitor, and maintain Udaya nodes.