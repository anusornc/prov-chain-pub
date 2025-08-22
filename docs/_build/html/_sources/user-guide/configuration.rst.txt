Configuration Guide
===================

Complete guide to configuring ProvChainOrg for your environment.

.. raw:: html

   <div class="hero-section">
     <div class="hero-content">
       <h1>Configuration Guide</h1>
       <p class="hero-subtitle">Complete guide to configuring ProvChainOrg</p>
       <div class="hero-badges">
         <span class="badge badge-config">Configuration</span>
         <span class="badge badge-admin">Administration</span>
         <span class="badge badge-advanced">Advanced</span>
       </div>
     </div>
   </div>

.. note::
   This guide provides comprehensive information about configuring ProvChainOrg for different environments and use cases. Proper configuration is essential for optimal performance and security.

Configuration Methods
---------------------

ProvChainOrg can be configured using several methods, which are applied in the following order of precedence:

1. **Command Line Arguments** - Highest precedence
2. **Environment Variables** - Medium precedence
3. **Configuration File** - Lowest precedence (default values)

Configuration File
------------------

The primary configuration file is in TOML format. By default, ProvChainOrg looks for configuration files in the following locations:

1. `./config.toml` (current directory)
2. `~/.provchain/config.toml` (user home directory)
3. `/etc/provchain/config.toml` (system-wide configuration)

### Basic Configuration File Structure

.. code-block:: toml

   # Server configuration
   [server]
   host = "127.0.0.1"
   port = 8080
   workers = 4
   
   # Database configuration
   [database]
   path = "./data"
   backup_enabled = true
   backup_interval_hours = 24
   
   # Logging configuration
   [logging]
   level = "info"
   file = "./logs/provchain.log"
   format = "json"
   
   # Network configuration
   [network]
   discovery_enabled = true
   max_peers = 50
   listen_address = "/ip4/0.0.0.0/tcp/4001"
   
   # Security configuration
   [security]
   jwt_secret = "your-secret-key-here"
   api_key_ttl_days = 90

### Server Configuration

.. code-block:: toml

   [server]
   # Host to bind to (use 0.0.0.0 for external access)
   host = "127.0.0.1"
   
   # Port to listen on
   port = 8080
   
   # Number of worker threads
   workers = 4
   
   # Enable HTTPS
   https_enabled = false
   
   # SSL certificate and key paths (if HTTPS is enabled)
   ssl_cert_path = "/path/to/cert.pem"
   ssl_key_path = "/path/to/key.pem"
   
   # CORS settings
   cors_allowed_origins = ["*"]
   cors_allowed_methods = ["GET", "POST", "PUT", "DELETE"]
   cors_allowed_headers = ["Content-Type", "Authorization"]

### Database Configuration

.. code-block:: toml

   [database]
   # Path to store blockchain data
   path = "./data"
   
   # Enable automatic backups
   backup_enabled = true
   
   # Backup interval in hours
   backup_interval_hours = 24
   
   # Maximum number of backup files to keep
   max_backup_files = 7
   
   # Enable data compression
   compression_enabled = true
   
   # Enable encryption at rest
   encryption_enabled = false
   
   # Cache size in MB
   cache_size_mb = 100

### Logging Configuration

.. code-block:: toml

   [logging]
   # Log level (trace, debug, info, warn, error)
   level = "info"
   
   # Log file path
   file = "./logs/provchain.log"
   
   # Log format (json, text)
   format = "json"
   
   # Enable console logging
   console_enabled = true
   
   # Enable file logging
   file_enabled = true
   
   # Maximum log file size in MB
   max_file_size_mb = 100
   
   # Maximum number of log files to keep
   max_files = 5

### Network Configuration

.. code-block:: toml

   [network]
   # Enable peer discovery
   discovery_enabled = true
   
   # Maximum number of connected peers
   max_peers = 50
   
   # Listen address for P2P connections
   listen_address = "/ip4/0.0.0.0/tcp/4001"
   
   # Bootstrap peers
   bootstrap_peers = [
     "/ip4/10.0.0.1/tcp/4001",
     "/ip4/10.0.0.2/tcp/4001"
   ]
   
   # Network ID (for network isolation)
   network_id = "provchain-mainnet"
   
   # Enable NAT traversal
   nat_enabled = true

### Security Configuration

.. code-block:: toml

   [security]
   # JWT secret for token signing
   jwt_secret = "your-secret-key-here"
   
   # API key time-to-live in days
   api_key_ttl_days = 90
   
   # Enable rate limiting
   rate_limiting_enabled = true
   
   # Maximum requests per minute per IP
   max_requests_per_minute = 1000
   
   # Enable IP whitelisting
   ip_whitelist_enabled = false
   
   # Whitelisted IP addresses
   whitelisted_ips = ["127.0.0.1", "10.0.0.0/8"]

### Performance Configuration

.. code-block:: toml

   [performance]
   # Enable query caching
   query_cache_enabled = true
   
   # Query cache size in MB
   query_cache_size_mb = 50
   
   # Enable canonicalization caching
   canonicalization_cache_enabled = true
   
   # Canonicalization cache size in MB
   canonicalization_cache_size_mb = 100
   
   # Enable concurrent operations
   concurrent_operations_enabled = true
   
   # Maximum concurrent operations
   max_concurrent_operations = 10

Environment Variables
---------------------

All configuration options can also be set using environment variables. Environment variables take precedence over configuration file settings.

### Naming Convention

Environment variables follow the pattern: `PROVCHAIN_{SECTION}_{KEY}` where:
- Section names are capitalized
- Key names are capitalized
- Dots and hyphens are replaced with underscores

### Examples

.. code-block:: bash

   # Set server host
   export PROVCHAIN_SERVER_HOST="0.0.0.0"
   
   # Set server port
   export PROVCHAIN_SERVER_PORT="8080"
   
   # Set log level
   export PROVCHAIN_LOGGING_LEVEL="debug"
   
   # Set database path
   export PROVCHAIN_DATABASE_PATH="/var/lib/provchain"
   
   # Enable HTTPS
   export PROVCHAIN_SERVER_HTTPS_ENABLED="true"
   export PROVCHAIN_SERVER_SSL_CERT_PATH="/path/to/cert.pem"
   export PROVCHAIN_SERVER_SSL_KEY_PATH="/path/to/key.pem"

Command Line Arguments
----------------------

Command line arguments have the highest precedence and override both environment variables and configuration file settings.

### Common Arguments

.. code-block:: bash

   # Set configuration file
   provchain-org --config /path/to/config.toml
   
   # Set log level
   provchain-org --log-level debug
   
   # Set server host and port
   provchain-org server start --host 0.0.0.0 --port 8080
   
   # Enable verbose output
   provchain-org --verbose
   
   # Output in JSON format
   provchain-org --json

Configuration Validation
------------------------

ProvChainOrg validates configuration at startup and reports any issues.

### Validation Commands

.. code-block:: bash

   # Validate configuration file
   provchain-org config validate --config /path/to/config.toml
   
   # Show current configuration
   provchain-org config show
   
   # Check for configuration issues
   provchain-org config check

### Common Validation Errors

1. **Invalid paths**: Ensure all file paths exist and are accessible
2. **Port conflicts**: Ensure ports are not already in use
3. **Missing required values**: Some configuration options are mandatory
4. **Invalid values**: Ensure values are within acceptable ranges

Production Configuration
------------------------

For production deployments, consider these additional configuration recommendations:

### Security Hardening

.. code-block:: toml

   [security]
   # Use a strong JWT secret
   jwt_secret = "a-very-long-random-secret-string"
   
   # Limit API key TTL
   api_key_ttl_days = 30
   
   # Enable rate limiting
   rate_limiting_enabled = true
   max_requests_per_minute = 100
   
   # Enable IP whitelisting for admin endpoints
   ip_whitelist_enabled = true
   whitelisted_ips = ["10.0.0.0/8", "172.16.0.0/12"]

### Performance Optimization

.. code-block:: toml

   [performance]
   # Increase cache sizes
   query_cache_size_mb = 200
   canonicalization_cache_size_mb = 500
   
   # Optimize worker count for your hardware
   [server]
   workers = 8
   
   [database]
   # Enable compression for large datasets
   compression_enabled = true

### Backup and Recovery

.. code-block:: toml

   [database]
   # Enable regular backups
   backup_enabled = true
   backup_interval_hours = 6
   max_backup_files = 30
   
   # Store backups in a separate location
   backup_path = "/backup/provchain"

### Monitoring

.. code-block:: toml

   [logging]
   # Enable detailed logging for monitoring
   level = "info"
   file = "/var/log/provchain/provchain.log"
   
   [performance]
   # Enable metrics collection
   metrics_enabled = true
   metrics_port = 8081

Development Configuration
-------------------------

For development environments, you might want to use these settings:

### Development Settings

.. code-block:: toml

   [server]
   # Enable external access for development
   host = "0.0.0.0"
   port = 8080
   
   [logging]
   # Enable debug logging
   level = "debug"
   console_enabled = true
   
   [database]
   # Disable backups for faster development
   backup_enabled = false
   
   [security]
   # Relax security for development
   rate_limiting_enabled = false

### Environment-Specific Configuration

You can use different configuration files for different environments:

.. code-block:: bash

   # Development
   provchain-org --config config.dev.toml
   
   # Staging
   provchain-org --config config.staging.toml
   
   # Production
   provchain-org --config config.prod.toml

Dynamic Configuration
---------------------

Some configuration options can be changed at runtime:

### Runtime Configuration Commands

.. code-block:: bash

   # View current configuration
   provchain-org config show
   
   # Set a configuration value at runtime
   provchain-org config set logging.level debug
   
   # Reset a configuration value to default
   provchain-org config reset logging.level

### Hot-Reloadable Options

The following options can be changed without restarting the server:

- Log level
- Rate limiting settings
- Cache sizes (with some limitations)

Configuration Best Practices
----------------------------

### Security Best Practices

1. **Never commit secrets to version control**
2. **Use environment variables for sensitive data**
3. **Regularly rotate API keys and secrets**
4. **Enable rate limiting in production**
5. **Use IP whitelisting for administrative endpoints**

### Performance Best Practices

1. **Set appropriate cache sizes for your workload**
2. **Configure worker count based on CPU cores**
3. **Use SSD storage for database files**
4. **Enable compression for large datasets**
5. **Monitor resource usage and adjust accordingly**

### Reliability Best Practices

1. **Enable regular backups**
2. **Store backups in a separate location**
3. **Test backup restoration procedures**
4. **Monitor disk space and system resources**
5. **Set up alerting for critical issues**

### Maintenance Best Practices

1. **Regularly review and update configuration**
2. **Document all configuration changes**
3. **Use version control for configuration files**
4. **Test configuration changes in staging first**
5. **Monitor logs for configuration-related issues**

Troubleshooting
---------------

### Common Issues

**Problem**: Configuration file not found
**Solution**: Ensure the configuration file exists in one of the default locations or specify it explicitly

**Problem**: Invalid configuration value
**Solution**: Check the error message and correct the value according to the documentation

**Problem**: Permission denied when accessing files
**Solution**: Ensure the user running ProvChainOrg has appropriate permissions

**Problem**: Port already in use
**Solution**: Change the port or stop the conflicting process

### Diagnostic Commands

.. code-block:: bash

   # Show effective configuration
   provchain-org config show --effective
   
   # Validate configuration
   provchain-org config validate
   
   # Check for common issues
   provchain-org config check

Support
-------

For additional help with configuration:

1. Refer to the example configuration files in the repository
2. Check the online documentation
3. Join our community forum
4. Contact support at support@provchain-org.com

.. note::
   Proper configuration is crucial for the security, performance, and reliability of your ProvChainOrg deployment. Always test configuration changes in a staging environment before applying them to production.