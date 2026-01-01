# Multi-Node Deployment Guide

This guide explains how to deploy ProvChainOrg on three separate machines (nodes) and set up a central monitoring server.

## Prerequisites

*   3 Machines for the blockchain nodes (Node 1, Node 2, Node 3).
*   (Optional) 1 Machine for Monitoring (or run it on Node 1).
*   Docker and Docker Compose installed on all machines.
*   Network connectivity between all machines (Ports 8080 and 9090 must be open).
*   This repository cloned on all machines.

## Deployment Steps

### 1. Configure Node 1 (Bootstrap Node)

On the first machine:

1.  Navigate to `deploy/`.
2.  Create a `.env` file:
    ```bash
    # Node 1 .env
    PEERS=""  # Empty for the first node
    JWT_SECRET=your_secure_secret_here
    RUST_LOG=info
    # Optional: Point to Monitoring Node IP for Tracing
    # JAEGER_ENDPOINT=http://MONITORING_IP:14268/api/traces
    ```
3.  Start the node:
    ```bash
    docker-compose -f docker-compose.node.yml up -d --build
    ```
4.  Note the IP address of this machine (e.g., `192.168.1.101`).

### 2. Configure Node 2

On the second machine:

1.  Navigate to `deploy/`.
2.  Create a `.env` file:
    ```bash
    # Node 2 .env
    # Point to Node 1
    PEERS="192.168.1.101:8080" 
    JWT_SECRET=your_secure_secret_here
    RUST_LOG=info
    ```
3.  Start the node:
    ```bash
    docker-compose -f docker-compose.node.yml up -d --build
    ```
4.  Note the IP address (e.g., `192.168.1.102`).

### 3. Configure Node 3

On the third machine:

1.  Navigate to `deploy/`.
2.  Create a `.env` file:
    ```bash
    # Node 3 .env
    # Point to Node 1 and Node 2
    PEERS="192.168.1.101:8080,192.168.1.102:8080"
    JWT_SECRET=your_secure_secret_here
    RUST_LOG=info
    ```
3.  Start the node:
    ```bash
    docker-compose -f docker-compose.node.yml up -d --build
    ```

### 4. Configure Monitoring (Optional but Recommended)

On the monitoring machine (can be Node 1):

1.  Navigate to `deploy/`.
2.  Edit `monitoring/prometheus_multi_node.yml`. Replace the placeholders with the real IP addresses of your nodes:
    ```yaml
      - targets: 
        - '192.168.1.101:9090'
        - '192.168.1.102:9090'
        - '192.168.1.103:9090'
    ```
3.  Start the monitoring stack:
    ```bash
    docker-compose -f docker-compose.monitoring.yml up -d
    ```

## Verification

*   **API:** Visit `http://NODE_IP:8080/health`
*   **Grafana:** Visit `http://MONITORING_IP:3000` (Default user/pass: admin/admin)
*   **Prometheus:** Visit `http://MONITORING_IP:9090`
