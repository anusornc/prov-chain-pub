Command Line Interface (CLI)
============================

Complete reference for the ProvChainOrg command line interface.

.. raw:: html

   <div class="hero-section">
     <div class="hero-content">
       <h1>Command Line Interface</h1>
       <p class="hero-subtitle">Complete reference for the ProvChainOrg CLI</p>
       <div class="hero-badges">
         <span class="badge badge-cli">CLI</span>
         <span class="badge badge-reference">Reference</span>
         <span class="badge badge-advanced">Advanced</span>
       </div>
     </div>
   </div>

.. note::
   The ProvChainOrg CLI provides a powerful interface for managing the blockchain, querying data, and performing administrative tasks. This guide covers all available commands and their usage.

Getting Started
---------------

The ProvChainOrg CLI is accessible through the `provchain-org` command (or `cargo run --` when running from source).

Basic Usage
-----------

.. code-block:: bash

   # Show help
   provchain-org --help
   
   # Show version
   provchain-org --version
   
   # Run a specific command
   provchain-org [COMMAND] [OPTIONS]

Core Commands
-------------

### Initialization and Setup

.. code-block:: bash

   # Initialize a new blockchain
   provchain-org init
   
   # Initialize with custom configuration
   provchain-org init --config /path/to/config.toml
   
   # Reset the blockchain (WARNING: This will delete all data)
   provchain-org reset

### Running the Server

.. code-block:: bash

   # Start the server
   provchain-org server start
   
   # Start with custom host and port
   provchain-org server start --host 0.0.0.0 --port 8080
   
   # Start in background
   provchain-org server start --daemon
   
   # Stop the server
   provchain-org server stop

### Demo Mode

.. code-block:: bash

   # Run the interactive demo
   provchain-org demo
   
   # Run a specific demo scenario
   provchain-org demo --scenario supply-chain

Data Management Commands
------------------------

### Adding Data

.. code-block:: bash

   # Add RDF data from a file
   provchain-org add-file /path/to/data.ttl
   
   # Add RDF data from stdin
   cat data.ttl | provchain-org add-file -
   
   # Add data with a specific format
   provchain-org add-file data.rdf --format rdfxml

### Querying Data

.. code-block:: bash

   # Execute a SPARQL query from a file
   provchain-org query /path/to/query.sparql
   
   # Execute a SPARQL query from stdin
   echo "SELECT * WHERE { ?s ?p ?o } LIMIT 10" | provchain-org query -
   
   # Execute a query with JSON output
   provchain-org query query.sparql --format json

### Exporting Data

.. code-block:: bash

   # Export all blockchain data
   provchain-org export --output blockchain-data.ttl
   
   # Export data in a specific format
   provchain-org export --output data.rdf --format rdfxml
   
   # Export data from a specific block range
   provchain-org export --start 100 --end 200 --output range.ttl

Blockchain Commands
-------------------

### Block Operations

.. code-block:: bash

   # Get blockchain status
   provchain-org blockchain status
   
   # Get information about a specific block
   provchain-org blockchain block 12345
   
   # Get blocks in a range
   provchain-org blockchain blocks --start 1000 --end 1100
   
   # Validate blockchain integrity
   provchain-org blockchain validate

### Transaction Operations

.. code-block:: bash

   # Submit a transaction
   provchain-org transaction submit --data transaction-data.ttl
   
   # Get transaction status
   provchain-org transaction status tx_1234567890
   
   # List transactions
   provchain-org transaction list --limit 50

Network Commands
----------------

### Peer Management

.. code-block:: bash

   # Join a network
   provchain-org network join --peer http://peer.example.com:8080
   
   # List connected peers
   provchain-org network peers
   
   # Add a peer manually
   provchain-org network add-peer --address http://peer2.example.com:8080
   
   # Remove a peer
   provchain-org network remove-peer --address http://peer2.example.com:8080

### Synchronization

.. code-block:: bash

   # Sync with the network
   provchain-org network sync
   
   # Force synchronization
   provchain-org network sync --force
   
   # Check sync status
   provchain-org network sync-status

User Management Commands
------------------------

### User Operations

.. code-block:: bash

   # Create a new user
   provchain-org user create --username john --password securepassword
   
   # List users
   provchain-org user list
   
   # Delete a user
   provchain-org user delete --username john
   
   # Change user password
   provchain-org user change-password --username john

### Authentication

.. code-block:: bash

   # Generate an API key
   provchain-org auth generate-api-key --name "my-application"
   
   # List API keys
   provchain-org auth list-keys
   
   # Revoke an API key
   provchain-org auth revoke-key --key pk_1234567890abcdef

Utility Commands
----------------

### System Information

.. code-block:: bash

   # Check system status
   provchain-org status
   
   # Get system information
   provchain-org info
   
   # Check disk usage
   provchain-org disk-usage

### Backup and Recovery

.. code-block:: bash

   # Create a backup
   provchain-org backup create --output backup-2025-01-15.tar.gz
   
   # List backups
   provchain-org backup list
   
   # Restore from backup
   provchain-org backup restore --input backup-2025-01-15.tar.gz

### Logging

.. code-block:: bash

   # View logs
   provchain-org logs --tail 100
   
   # Follow logs in real-time
   provchain-org logs --follow
   
   # Filter logs by level
   provchain-org logs --level error

### Performance Monitoring

.. code-block:: bash

   # Get performance metrics
   provchain-org metrics
   
   # Monitor in real-time
   provchain-org metrics --watch
   
   # Export metrics
   provchain-org metrics --export metrics.json

Advanced Usage
--------------

### Batch Operations

.. code-block:: bash

   # Submit multiple transactions in a batch
   provchain-org batch submit --file transactions.json
   
   # Process a batch of SPARQL queries
   provchain-org batch query --file queries.json

### Scripting

.. code-block:: bash

   # Run a script file
   provchain-org script run --file setup.sh
   
   # Execute a script with variables
   provchain-org script run --file config.sh --var ENV=production

### Configuration Management

.. code-block:: bash

   # View current configuration
   provchain-org config show
   
   # Set a configuration value
   provchain-org config set server.port 8080
   
   # Reset configuration to defaults
   provchain-org config reset

Command Reference
-----------------

### Global Options

.. code-block:: bash

   # Set log level
   provchain-org --log-level debug
   
   # Set configuration file
   provchain-org --config /path/to/config.toml
   
   # Enable verbose output
   provchain-org --verbose
   
   # Output in JSON format
   provchain-org --json

### Environment Variables

ProvChainOrg CLI can be configured using environment variables:

.. code-block:: bash

   # Set the configuration file path
   export PROVCHAIN_CONFIG_FILE="/path/to/config.toml"
   
   # Set log level
   export PROVCHAIN_LOG_LEVEL="debug"
   
   # Set server host and port
   export PROVCHAIN_SERVER_HOST="0.0.0.0"
   export PROVCHAIN_SERVER_PORT="8080"

### Exit Codes

The CLI returns the following exit codes:

- ``0``: Success
- ``1``: General error
- ``2``: Invalid arguments
- ``3``: Configuration error
- ``4``: Network error
- ``5``: Data validation error

Examples
--------

### Basic Workflow

.. code-block:: bash

   # Initialize the blockchain
   provchain-org init
   
   # Add some supply chain data
   provchain-org add-file supply-chain-data.ttl
   
   # Query the data
   provchain-org query trace-query.sparql
   
   # Check the blockchain status
   provchain-org blockchain status

### Administrative Tasks

.. code-block:: bash

   # Create a backup
   provchain-org backup create --output daily-backup.tar.gz
   
   # Create an API key for an application
   provchain-org auth generate-api-key --name "web-app"
   
   # Monitor system performance
   provchain-org metrics --watch

### Automation Script

.. code-block:: bash

   #!/bin/bash
   
   # Daily maintenance script
   echo "Starting daily maintenance..."
   
   # Backup the blockchain
   provchain-org backup create --output backup-$(date +%Y-%m-%d).tar.gz
   
   # Validate blockchain integrity
   provchain-org blockchain validate
   
   # Clean up old logs
   provchain-org logs --clean --days 30
   
   # Report disk usage
   provchain-org disk-usage
   
   echo "Daily maintenance completed."

Troubleshooting
---------------

### Common Issues

**Problem**: Command not found
**Solution**: Ensure ProvChainOrg is installed and in your PATH

**Problem**: Permission denied
**Solution**: Check file permissions and run with appropriate privileges

**Problem**: Network connection failed
**Solution**: Check network connectivity and firewall settings

### Debugging

.. code-block:: bash

   # Enable debug logging
   provchain-org --log-level debug [COMMAND]
   
   # Get verbose output
   provchain-org --verbose [COMMAND]
   
   # Show detailed error information
   provchain-org --debug [COMMAND]

Support
-------

For additional help with the CLI:

1. Use `provchain-org --help` for general help
2. Use `provchain-org [COMMAND] --help` for command-specific help
3. Check the online documentation
4. Join our community forum
5. Contact support at support@provchain-org.com

.. note::
   The CLI is the most direct way to interact with ProvChainOrg. For production environments, consider using the REST API for programmatic access.