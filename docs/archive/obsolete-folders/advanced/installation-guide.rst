Advanced Installation Guide
==========================

This guide provides detailed instructions for advanced installation and deployment of ProvChainOrg in various environments, including production systems, development environments, and specialized configurations.

.. raw:: html

   <div class="hero-section">
     <div class="hero-content">
       <h1>ProvChainOrg Advanced Installation Guide</h1>
       <p class="hero-subtitle">Comprehensive deployment and configuration for enterprise environments</p>
       <div class="hero-badges">
         <span class="badge badge-advanced">Advanced</span>
         <span class="badge badge-deployment">Deployment</span>
         <span class="badge badge-enterprise">Enterprise</span>
         <span class="badge badge-configuration">Configuration</span>
       </div>
     </div>
   </div>

Overview
--------

This advanced installation guide covers complex deployment scenarios for ProvChainOrg, including:

- **Production Deployment**: Enterprise-grade installation
- **Development Setup**: Advanced development environments
- **Container Deployment**: Docker and Kubernetes configurations
- **Cloud Deployment**: AWS, Azure, and Google Cloud setups
- **High Availability**: Multi-node cluster configurations
- **Security Hardening**: Advanced security configurations
- **Performance Tuning**: Optimization for high-load environments

**Target Audiences:**
- **System Administrators**: Deploying and maintaining systems
- **DevOps Engineers**: CI/CD and infrastructure automation
- **Enterprise Architects**: Large-scale system design
- **Security Specialists**: Security-hardened deployments
- **Performance Engineers**: High-performance configurations

**Prerequisites:**
- **System Administration**: Linux/Unix system administration skills
- **Networking**: Understanding of TCP/IP, DNS, and firewalls
- **Security**: Knowledge of encryption, certificates, and access control
- **Containerization**: Experience with Docker and Kubernetes
- **Cloud Platforms**: Familiarity with AWS, Azure, or GCP

Production Deployment
--------------------

**System Requirements**
For production deployments, ensure your systems meet these requirements:

**Minimum Hardware:**
- **CPU**: 8 cores (Intel Xeon or AMD EPYC recommended)
- **Memory**: 32GB RAM
- **Storage**: 1TB SSD (NVMe recommended)
- **Network**: 1Gbps dedicated network interface

**Recommended Hardware:**
- **CPU**: 16 cores
- **Memory**: 64GB RAM
- **Storage**: 2TB NVMe SSD
- **Network**: 10Gbps network interface

**Operating Systems:**
- **Linux**: Ubuntu 20.04+, CentOS 8+, RHEL 8+
- **Container**: Docker, Kubernetes
- **Cloud**: AWS EC2, Azure VM, Google Compute Engine

**Software Dependencies:**
.. code-block:: bash
   # Ubuntu/Debian
   sudo apt-get update
   sudo apt-get install -y \
       build-essential \
       curl \
       git \
       libssl-dev \
       pkg-config \
       clang \
       cmake

   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env

**Installation Steps:**
1. **Create Service User**
   .. code-block:: bash
      sudo useradd -r -s /bin/false provchain
      sudo mkdir -p /opt/provchain
      sudo chown provchain:provchain /opt/provchain

2. **Download and Compile**
   .. code-block:: bash
      sudo -u provchain git clone https://github.com/anusornc/provchain-org.git /opt/provchain
      cd /opt/provchain
      sudo -u provchain cargo build --release

3. **Configuration**
   Create a production configuration file:
   .. code-block:: toml
      # /etc/provchain/config.toml
      [network]
      network_id = "provchain-production"
      listen_port = 8080
      known_peers = ["peer1.example.com:8080", "peer2.example.com:8080"]
      
      [consensus]
      is_authority = true
      authority_nodes = ["node1.example.com", "node2.example.com", "node3.example.com"]
      
      [storage]
      data_dir = "/var/lib/provchain/data"
      persistent = true
      store_type = "oxigraph"
      
      [ontology]
      path = "/opt/provchain/ontology/traceability.owl.ttl"
      graph_name = "http://provchain.org/ontology"
      auto_load = true
      validate_data = true
      
      [security]
      tls_enabled = true
      tls_cert_path = "/etc/ssl/certs/provchain.crt"
      tls_key_path = "/etc/ssl/private/provchain.key"
      api_key_required = true

4. **Service Setup**
   Create a systemd service file:
   .. code-block:: ini
      # /etc/systemd/system/provchain.service
      [Unit]
      Description=ProvChainOrg Node
      After=network.target
      
      [Service]
      Type=simple
      User=provchain
      Group=provchain
      WorkingDirectory=/opt/provchain
      ExecStart=/opt/provchain/target/release/provchain-node --config /etc/provchain/config.toml
      Restart=always
      RestartSec=10
      Environment=RUST_LOG=info
      
      [Install]
      WantedBy=multi-user.target

5. **Start Service**
   .. code-block:: bash
      sudo systemctl daemon-reload
      sudo systemctl enable provchain
      sudo systemctl start provchain
      sudo systemctl status provchain

Container Deployment
-------------------

**Docker Deployment**
Deploy ProvChainOrg using Docker containers:

1. **Dockerfile**
   .. code-block:: dockerfile
      FROM rust:1.70 as builder
      
      WORKDIR /app
      COPY . .
      RUN cargo build --release
      
      FROM debian:bullseye-slim
      
      RUN apt-get update && apt-get install -y \
          ca-certificates \
          && rm -rf /var/lib/apt/lists/*
      
      COPY --from=builder /app/target/release/provchain-node /usr/local/bin/provchain-node
      COPY --from=builder /app/ontology /ontology
      
      EXPOSE 8080
      USER 1000
      
      CMD ["provchain-node", "--config", "/config/config.toml"]

2. **Docker Compose**
   .. code-block:: yaml
      version: '3.8'
      
      services:
        provchain-node:
          build: .
          ports:
            - "8080:8080"
          volumes:
            - provchain-data:/var/lib/provchain
            - ./config:/config
            - ./certs:/certs
          environment:
            - RUST_LOG=info
          restart: unless-stopped
          
        provchain-ui:
          image: provchain/provchain-ui:latest
          ports:
            - "3000:3000"
          environment:
            - PROVCHAIN_API_URL=http://provchain-node:8080
          depends_on:
            - provchain-node
          restart: unless-stopped
          
      volumes:
        provchain-data:

3. **Build and Run**
   .. code-block:: bash
      # Build the image
      docker build -t provchain/provchain-node .
      
      # Run with Docker Compose
      docker-compose up -d

**Kubernetes Deployment**
Deploy ProvChainOrg in a Kubernetes cluster:

1. **Deployment Manifest**
   .. code-block:: yaml
      # provchain-deployment.yaml
      apiVersion: apps/v1
      kind: Deployment
      metadata:
        name: provchain-node
        labels:
          app: provchain
      spec:
        replicas: 3
        selector:
          matchLabels:
            app: provchain
        template:
          metadata:
            labels:
              app: provchain
          spec:
            containers:
            - name: provchain-node
              image: provchain/provchain-node:latest
              ports:
              - containerPort: 8080
              env:
              - name: RUST_LOG
                value: "info"
              volumeMounts:
              - name: provchain-data
                mountPath: /var/lib/provchain
              - name: provchain-config
                mountPath: /config
              resources:
                requests:
                  memory: "4Gi"
                  cpu: "2"
                limits:
                  memory: "8Gi"
                  cpu: "4"
            volumes:
            - name: provchain-data
              persistentVolumeClaim:
                claimName: provchain-pvc
            - name: provchain-config
              configMap:
                name: provchain-config
            securityContext:
              runAsNonRoot: true
              runAsUser: 1000
              fsGroup: 2000

2. **Service Manifest**
   .. code-block:: yaml
      # provchain-service.yaml
      apiVersion: v1
      kind: Service
      metadata:
        name: provchain-service
      spec:
        selector:
          app: provchain
        ports:
        - protocol: TCP
          port: 8080
          targetPort: 8080
        type: LoadBalancer

3. **Persistent Volume**
   .. code-block:: yaml
      # provchain-pvc.yaml
      apiVersion: v1
      kind: PersistentVolumeClaim
      metadata:
        name: provchain-pvc
      spec:
        accessModes:
        - ReadWriteOnce
        resources:
          requests:
            storage: 100Gi

4. **ConfigMap**
   .. code-block:: yaml
      # provchain-config.yaml
      apiVersion: v1
      kind: ConfigMap
      metadata:
        name: provchain-config
      data:
        config.toml: |
          [network]
          network_id = "provchain-k8s"
          listen_port = 8080
          
          [storage]
          data_dir = "/var/lib/provchain/data"
          persistent = true
          
          [ontology]
          path = "/ontology/traceability.owl.ttl"
          auto_load = true

5. **Deploy to Kubernetes**
   .. code-block:: bash
      kubectl apply -f provchain-config.yaml
      kubectl apply -f provchain-pvc.yaml
      kubectl apply -f provchain-deployment.yaml
      kubectl apply -f provchain-service.yaml

Cloud Deployment
---------------

**AWS Deployment**
Deploy ProvChainOrg on Amazon Web Services:

1. **EC2 Instance Setup**
   .. code-block:: bash
      # Launch EC2 instance with appropriate AMI
      aws ec2 run-instances \
          --image-id ami-0abcdef1234567890 \
          --count 1 \
          --instance-type m5.2xlarge \
          --key-name my-key-pair \
          --security-group-ids sg-0123456789abcdef0 \
          --subnet-id subnet-0123456789abcdef0

2. **EBS Volume Configuration**
   .. code-block:: bash
      # Create and attach EBS volume
      aws ec2 create-volume \
          --availability-zone us-west-2a \
          --size 100 \
          --volume-type gp3
      
      aws ec2 attach-volume \
          --volume-id vol-0123456789abcdef0 \
          --instance-id i-0123456789abcdef0 \
          --device /dev/sdf

3. **S3 Backup Configuration**
   .. code-block:: bash
      # Create S3 bucket for backups
      aws s3 mb s3://provchain-backups-$(date +%s)
      
      # Configure backup policy
      aws s3api put-bucket-policy \
          --bucket provchain-backups-$(date +%s) \
          --policy file://backup-policy.json

**Azure Deployment**
Deploy ProvChainOrg on Microsoft Azure:

1. **Virtual Machine Setup**
   .. code-block:: bash
      # Create resource group
      az group create \
          --name provchain-rg \
          --location westus2
      
      # Create virtual machine
      az vm create \
          --resource-group provchain-rg \
          --name provchain-vm \
          --image UbuntuLTS \
          --size Standard_D4s_v3 \
          --admin-username provchain \
          --ssh-key-value ~/.ssh/id_rsa.pub

2. **Managed Disk Configuration**
   .. code-block:: bash
      # Create managed disk
      az disk create \
          --resource-group provchain-rg \
          --name provchain-data-disk \
          --size-gb 100 \
          --sku Premium_LRS
      
      # Attach disk to VM
      az vm disk attach \
          --resource-group provchain-rg \
          --vm-name provchain-vm \
          --name provchain-data-disk

**Google Cloud Deployment**
Deploy ProvChainOrg on Google Cloud Platform:

1. **Compute Engine Setup**
   .. code-block:: bash
      # Create instance
      gcloud compute instances create provchain-node \
          --zone=us-west1-a \
          --machine-type=n1-standard-4 \
          --image-family=ubuntu-2004-lts \
          --image-project=ubuntu-os-cloud \
          --boot-disk-size=50GB \
          --boot-disk-type=pd-ssd

2. **Persistent Disk Configuration**
   .. code-block:: bash
      # Create persistent disk
      gcloud compute disks create provchain-data-disk \
          --size=100GB \
          --type=pd-ssd \
          --zone=us-west1-a
      
      # Attach disk to instance
      gcloud compute instances attach-disk provchain-node \
          --disk=provchain-data-disk \
          --zone=us-west1-a

High Availability Configuration
------------------------------

**Multi-Node Cluster Setup**
Configure a high-availability ProvChainOrg cluster:

1. **Authority Node Configuration**
   .. code-block:: toml
      # Authority node config
      [network]
      network_id = "provchain-ha"
      listen_port = 8080
      known_peers = ["node2.example.com:8080", "node3.example.com:8080"]
      
      [consensus]
      is_authority = true
      authority_nodes = ["node1.example.com", "node2.example.com", "node3.example.com"]
      
      [storage]
      data_dir = "/var/lib/provchain/data"
      persistent = true

2. **Load Balancer Configuration**
   .. code-block:: nginx
      # Nginx load balancer configuration
      upstream provchain_backend {
          server node1.example.com:8080;
          server node2.example.com:8080;
          server node3.example.com:8080;
      }
      
      server {
          listen 80;
          server_name provchain.example.com;
          
          location / {
              proxy_pass http://provchain_backend;
              proxy_set_header Host $host;
              proxy_set_header X-Real-IP $remote_addr;
          }
      }

3. **Health Check Configuration**
   .. code-block:: bash
      # Health check script
      #!/bin/bash
      curl -f http://localhost:8080/api/v1/status || exit 1

Security Hardening
-----------------

**Advanced Security Configuration**
Implement enterprise-grade security measures:

1. **TLS Configuration**
   .. code-block:: toml
      [security]
      tls_enabled = true
      tls_cert_path = "/etc/ssl/certs/provchain.crt"
      tls_key_path = "/etc/ssl/private/provchain.key"
      tls_min_version = "TLS1.3"
      
      [authentication]
      jwt_enabled = true
      jwt_secret = "your-jwt-secret-here"
      api_key_required = true

2. **Firewall Configuration**
   .. code-block:: bash
      # UFW firewall rules
      sudo ufw allow 22/tcp    # SSH
      sudo ufw allow 8080/tcp  # ProvChainOrg API
      sudo ufw allow 8081/tcp  # WebSocket
      sudo ufw deny from any to any
      
      # IP whitelisting
      sudo ufw allow from 192.168.1.0/24 to any port 8080

3. **Security Headers**
   .. code-block:: nginx
      # Nginx security headers
      add_header X-Frame-Options "DENY" always;
      add_header X-Content-Type-Options "nosniff" always;
      add_header X-XSS-Protection "1; mode=block" always;
      add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;

Performance Tuning
-----------------

**Optimization for High-Load Environments**
Configure ProvChainOrg for maximum performance:

1. **Database Optimization**
   .. code-block:: toml
      [storage]
      data_dir = "/var/lib/provchain/data"
      persistent = true
      cache_size = "2GB"
      max_open_files = 10000
      block_size = "64KB"
      
      [performance]
      query_cache_size = "512MB"
      query_timeout = 30
      max_concurrent_queries = 100

2. **System-Level Tuning**
   .. code-block:: bash
      # Kernel parameters for high-performance networking
      echo 'net.core.rmem_max = 134217728' >> /etc/sysctl.conf
      echo 'net.core.wmem_max = 134217728' >> /etc/sysctl.conf
      echo 'net.ipv4.tcp_rmem = 4096 87380 134217728' >> /etc/sysctl.conf
      echo 'net.ipv4.tcp_wmem = 4096 65536 134217728' >> /etc/sysctl.conf
      
      # Apply changes
      sysctl -p

3. **Resource Limits**
   .. code-block:: bash
      # /etc/security/limits.conf
      provchain soft nofile 65536
      provchain hard nofile 65536
      provchain soft nproc 32768
      provchain hard nproc 32768

Monitoring and Logging
---------------------

**Production Monitoring Setup**
Implement comprehensive monitoring and logging:

1. **Prometheus Configuration**
   .. code-block:: yaml
      # prometheus.yml
      scrape_configs:
        - job_name: 'provchain'
          static_configs:
            - targets: ['node1.example.com:8080', 'node2.example.com:8080', 'node3.example.com:8080']
          metrics_path: '/metrics'

2. **Logging Configuration**
   .. code-block:: toml
      [logging]
      level = "info"
      format = "json"
      file = "/var/log/provchain/provchain.log"
      max_size = "100MB"
      max_files = 10
      
      [audit_logging]
      enabled = true
      file = "/var/log/provchain/audit.log"

3. **Log Rotation**
   .. code-block:: bash
      # /etc/logrotate.d/provchain
      /var/log/provchain/*.log {
          daily
          rotate 30
          compress
          delaycompress
          missingok
          notifempty
          create 644 provchain provchain
      }

Backup and Recovery
------------------

**Data Protection Strategy**
Implement comprehensive backup and recovery procedures:

1. **Automated Backup Script**
   .. code-block:: bash
      #!/bin/bash
      # provchain-backup.sh
      
      BACKUP_DIR="/backup/provchain"
      DATE=$(date +%Y%m%d_%H%M%S)
      BACKUP_NAME="provchain_backup_$DATE"
      
      # Create backup
      tar -czf "$BACKUP_DIR/$BACKUP_NAME.tar.gz" \
          /var/lib/provchain/data \
          /etc/provchain/config.toml
      
      # Upload to cloud storage
      aws s3 cp "$BACKUP_DIR/$BACKUP_NAME.tar.gz" s3://provchain-backups/
      
      # Clean old backups
      find $BACKUP_DIR -name "provchain_backup_*.tar.gz" -mtime +30 -delete

2. **Backup Schedule**
   .. code-block:: bash
      # Add to crontab
      0 2 * * * /opt/provchain/scripts/provchain-backup.sh

3. **Disaster Recovery Plan**
   .. code-block:: bash
      # Recovery script
      #!/bin/bash
      # provchain-restore.sh
      
      BACKUP_FILE=$1
      tar -xzf $BACKUP_FILE -C /
      systemctl restart provchain

Troubleshooting
--------------

**Common Production Issues**
Solutions to common production deployment problems:

1. **Performance Issues**
   - **Symptom**: Slow query responses
   - **Solution**: Check resource utilization, optimize queries, increase cache size

2. **Network Connectivity**
   - **Symptom**: Nodes unable to communicate
   - **Solution**: Verify firewall rules, check DNS resolution, test network connectivity

3. **Storage Problems**
   - **Symptom**: Insufficient disk space
   - **Solution**: Monitor disk usage, implement log rotation, expand storage volumes

4. **Security Issues**
   - **Symptom**: Unauthorized access attempts
   - **Solution**: Review access logs, update firewall rules, rotate credentials

**Diagnostic Commands**
Useful commands for troubleshooting:

.. code-block:: bash
   # Check service status
   systemctl status provchain
   
   # View logs
   journalctl -u provchain -f
   
   # Check resource usage
   top -p $(pgrep provchain-node)
   
   # Network connectivity
   netstat -tlnp | grep 8080
   ss -tlnp | grep 8080
   
   # Disk usage
   df -h /var/lib/provchain
   du -sh /var/lib/provchain/*

Best Practices
-------------

**Enterprise Deployment Best Practices**
Follow these guidelines for successful production deployments:

1. **Infrastructure**
   - Use dedicated hardware or isolated cloud instances
   - Implement redundant network connections
   - Configure automated monitoring and alerting
   - Establish regular backup and recovery procedures

2. **Security**
   - Implement defense-in-depth security measures
   - Regularly update and patch systems
   - Use strong authentication and authorization
   - Encrypt data at rest and in transit

3. **Performance**
   - Monitor system performance metrics
   - Optimize database and query performance
   - Implement caching strategies
   - Plan for horizontal scaling

4. **Operations**
   - Document all configuration changes
   - Implement change management processes
   - Conduct regular security audits
   - Maintain disaster recovery plans

5. **Compliance**
   - Ensure regulatory compliance
   - Maintain audit trails
   - Implement data retention policies
   - Regular compliance assessments

.. raw:: html

   <div class="footer-note">
     <p><strong>Need help with your deployment?</strong> Contact our enterprise support team at enterprise@provchain-org.com for professional services.</p>
   </div>
