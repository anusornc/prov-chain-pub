Installation Guide
=================

Complete instructions for installing and setting up ProvChainOrg on your system.

.. raw:: html

   <div class="hero-section">
     <div class="hero-content">
       <h1>Installation Guide</h1>
       <p class="hero-subtitle">Step-by-step instructions for installing ProvChainOrg</p>
       <div class="hero-badges">
         <span class="badge badge-install">Installation</span>
         <span class="badge badge-setup">Setup</span>
         <span class="badge badge-beginner">Beginner</span>
       </div>
     </div>
   </div>

.. note::
   This guide provides comprehensive instructions for installing ProvChainOrg on various platforms. Follow the steps appropriate for your operating system and intended use case.

System Requirements
-------------------

Before installing ProvChainOrg, ensure your system meets the following requirements:

**Minimum Requirements:**
- **Operating System**: Linux, macOS, or Windows with WSL
- **Processor**: x86_64 architecture
- **Memory**: 4GB RAM (8GB recommended)
- **Storage**: 100GB SSD storage (500GB recommended)
- **Network**: Internet connection for downloading dependencies

**Recommended Requirements:**
- **Operating System**: Ubuntu 20.04+, macOS 12+, or Windows 11 with WSL2
- **Processor**: Multi-core CPU
- **Memory**: 8GB RAM or more
- **Storage**: 500GB SSD or more
- **Network**: 100Mbps connection or faster

Prerequisites
-------------

Before installing ProvChainOrg, you'll need to install the following dependencies:

**Rust Toolchain**
ProvChainOrg is built with Rust, so you'll need the Rust toolchain:

.. code-block:: bash

   # Install Rust using rustup (recommended)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Follow the on-screen instructions to complete installation
   # Restart your terminal or run:
   source $HOME/.cargo/env

**Git**
You'll need Git to clone the repository:

.. code-block:: bash

   # Ubuntu/Debian
   sudo apt update && sudo apt install git
   
   # macOS (with Homebrew)
   brew install git
   
   # Windows (with Chocolatey)
   choco install git

**Build Tools**
Additional build tools may be required:

.. code-block:: bash

   # Ubuntu/Debian
   sudo apt install build-essential pkg-config libssl-dev
   
   # macOS (with Homebrew)
   brew install openssl pkg-config
   
   # Windows (with Chocolatey)
   choco install visualstudio2022buildtools

Installation Methods
--------------------

ProvChainOrg can be installed in several ways depending on your needs:

### Method 1: From Source (Recommended for Development)

This method is recommended if you plan to contribute to the project or need the latest features:

.. code-block:: bash

   # Clone the repository
   git clone https://github.com/anusornc/provchain-org.git
   cd provchain-org
   
   # Build the project
   cargo build --release
   
   # Run the application
   cargo run --release

### Method 2: Using Cargo Install

If you just want to use ProvChainOrg without modifying the source code:

.. code-block:: bash

   # Install directly from crates.io (when available)
   cargo install provchain-org
   
   # Or install from the local repository
   cd provchain-org
   cargo install --path .

### Method 3: Docker Installation

For containerized deployment:

.. code-block:: bash

   # Pull the latest Docker image
   docker pull provchain/provchain-org:latest
   
   # Run the container
   docker run -p 8080:8080 provchain/provchain-org:latest

### Method 4: Pre-built Binaries

Download pre-built binaries from the GitHub releases page:

.. code-block:: bash

   # Download the appropriate binary for your platform
   wget https://github.com/anusornc/provchain-org/releases/latest/download/provchain-org-linux-amd64.tar.gz
   
   # Extract the archive
   tar -xzf provchain-org-linux-amd64.tar.gz
   
   # Make the binary executable
   chmod +x provchain-org
   
   # Run the application
   ./provchain-org

Verification
------------

After installation, verify that ProvChainOrg is working correctly:

.. code-block:: bash

   # Check the version
   provchain-org --version
   
   # Run the demo
   provchain-org demo
   
   # Check system status
   provchain-org status

Configuration
-------------

ProvChainOrg can be configured using a configuration file or environment variables:

### Configuration File

Create a configuration file at `~/.provchain/config.toml`:

.. code-block:: toml

   [server]
   host = "127.0.0.1"
   port = 8080
   
   [database]
   path = "./data"
   backup_enabled = true
   
   [logging]
   level = "info"
   file = "./logs/provchain.log"

### Environment Variables

You can also configure ProvChainOrg using environment variables:

.. code-block:: bash

   export PROVCHAIN_SERVER_HOST="0.0.0.0"
   export PROVCHAIN_SERVER_PORT=8080
   export PROVCHAIN_DATABASE_PATH="/var/lib/provchain"
   export PROVCHAIN_LOG_LEVEL="debug"

Initial Setup
-------------

After installation, perform the initial setup:

.. code-block:: bash

   # Initialize the blockchain
   provchain-org init
   
   # Create an admin user
   provchain-org user create --username admin --password secure-password
   
   # Start the server
   provchain-org server start

Network Configuration
---------------------

To run ProvChainOrg in a networked environment:

.. code-block:: bash

   # Start with custom network settings
   provchain-org server start --host 0.0.0.0 --port 8080
   
   # Join an existing network
   provchain-org network join --peer http://peer1.example.com:8080

Troubleshooting
---------------

Common installation issues and their solutions:

### Rust Installation Issues

**Problem**: Command not found after installing Rust
**Solution**: Restart your terminal or run `source $HOME/.cargo/env`

**Problem**: Permission denied when running cargo commands
**Solution**: Ensure your user has write permissions to `$HOME/.cargo`

### Build Issues

**Problem**: Missing OpenSSL libraries
**Solution**: Install OpenSSL development packages:
   
.. code-block:: bash

   # Ubuntu/Debian
   sudo apt install libssl-dev
   
   # macOS
   brew install openssl
   export PKG_CONFIG_PATH="/opt/homebrew/opt/openssl@3/lib/pkgconfig"

**Problem**: Compilation fails with memory errors
**Solution**: Increase available memory or use `cargo build --release -j 1` to limit parallel jobs

### Runtime Issues

**Problem**: Port already in use
**Solution**: Change the port or stop the conflicting process:

.. code-block:: bash

   # Find the process using the port
   lsof -i :8080
   
   # Kill the process
   kill -9 <PID>

**Problem**: Database permissions
**Solution**: Ensure the user running ProvChainOrg has read/write permissions to the data directory

Updating
--------

To update ProvChainOrg to the latest version:

.. code-block:: bash

   # If installed from source
   cd provchain-org
   git pull
   cargo build --release
   
   # If installed via cargo
   cargo install provchain-org --force
   
   # If using Docker
   docker pull provchain/provchain-org:latest

Uninstallation
--------------

To uninstall ProvChainOrg:

.. code-block:: bash

   # Remove the binary (if installed via cargo)
   cargo uninstall provchain-org
   
   # Remove the source directory (if cloned)
   rm -rf provchain-org
   
   # Remove Docker image (if using Docker)
   docker rmi provchain/provchain-org:latest
   
   # Remove configuration and data (optional)
   rm -rf ~/.provchain

Support
-------

If you encounter issues not covered in this guide:

1. Check the `issues` section on GitHub
2. Join our community forum
3. Contact support at support@provchain-org.com

.. note::
   For production deployments, consult the Production Deployment Guide for additional security and performance considerations.