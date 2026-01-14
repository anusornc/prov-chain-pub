Overview

  This project has three deployment strategies designed for different use cases:

  📦 Docker Deployment Strategies
  ├── Single-Node All-in-One (docker-compose.production.yml)
  ├── Single-Machine 3-Node Cluster (docker-compose.3node.yml)
  └── Multi-Machine Distributed Cluster (docker-compose.node.yml + monitoring)

  ---
  ★ Insight ─────────────────────────────────────
  Multi-Stage Builds & Container Optimization
  1. The Dockerfile uses a 3-stage build (backend → frontend → runtime) to minimize the final image size by excluding build tools and compile-time dependencies
  2. The non-root user (provchain) enhances security by following the principle of least privilege—critical for production containers
  3. Layer caching is optimized by copying Cargo.toml and package.json before source code, speeding up rebuilds when dependencies haven't changed
  ─────────────────────────────────────────────────

  ---
  1. Dockerfile: Production Build

  The multi-stage Dockerfile (deploy/Dockerfile.production) builds a production-ready container:

  | Stage                     | Base Image           | Purpose                                      |
  |---------------------------|----------------------|----------------------------------------------|
  | Stage 1: backend-builder  | rust:1.75-slim       | Compiles the Rust backend with optimizations |
  | Stage 2: frontend-builder | node:18-alpine       | Builds the React/Vue frontend                |
  | Stage 3: runtime          | debian:bookworm-slim | Minimal runtime image with compiled binary   |

  Key Features:
  - Ports: 8080 (HTTP API), 9090 (Metrics endpoint)
  - Health Check: Built-in health monitoring via /health endpoint
  - Volumes: Separate data, backups, and logs for persistence
  - Security: Runs as non-root user with minimal runtime dependencies

  ---
  2. Deployment Strategies

  Strategy A: Single-Node All-in-One

  File: docker-compose.production.yml

  Best for: Development, testing, or small production deployments

  ┌─────────────────────────────────────────────────────────────────┐
  │                    All-in-One Deployment                        │
  ├─────────────────────────────────────────────────────────────────┤
  │  ┌──────────────┐  ┌──────────┐  ┌─────────┐  ┌────────────┐  │
  │  │   Nginx      │  │   App    │  │  Redis  │  │  Backup    │  │
  │  │  (80/443)    │─▶│ (8080)   │  │ (6379)  │  │  Service   │  │
  │  └──────────────┘  └────┬─────┘  └─────────┘  └────────────┘  │
  │                          │                                       │
  │              ┌───────────┼───────────┐                          │
  │              │           │           │                          │
  │         ┌────▼───┐ ┌────▼──┐ ┌────▼─────┐ ┌────▼────┐         │
  │         │Prometh │ │Grafana│ │  Jaeger  │ │  Loki   │         │
  │         │(9091)  │ │(3001) │ │ (16686)  │ │ (3100)  │         │
  │         └────────┘ └───────┘ └──────────┘ └─────────┘         │
  └─────────────────────────────────────────────────────────────────┘

  Services (10 containers):
  | Service       | Port         | Purpose                         |
  |---------------|--------------|---------------------------------|
  | provchain-org | 8080, 9090   | Main application                |
  | nginx         | 80, 443      | Reverse proxy + SSL termination |
  | redis         | 6379         | Caching layer                   |
  | prometheus    | 9091         | Metrics collection              |
  | grafana       | 3001         | Metrics visualization           |
  | jaeger        | 16686, 14268 | Distributed tracing             |
  | loki          | 3100         | Log aggregation                 |
  | promtail      | -            | Log shipping agent              |
  | backup        | -            | Scheduled database backups      |

  ---
  Strategy B: 3-Node Cluster (Single Machine)

  File: docker-compose.3node.yml

  Best for: Testing consensus behavior, development clustering

  ┌─────────────────────────────────────────────────────────────────┐
  │                 3-Node Cluster (Single Host)                    │
  ├─────────────────────────────────────────────────────────────────┤
  │                                                                  │
  │   ┌──────────┐      ┌──────────┐      ┌──────────┐             │
  │   │  Node 1  │◀────▶│  Node 2  │◀────▶│  Node 3  │             │
  │   │  (8080)  │      │  (8081)  │      │  (8082)  │             │
  │   └─────┬────┘      └─────┬────┘      └─────┬────┘             │
  │         │                 │                 │                   │
  │         └─────────────────┼─────────────────┘                   │
  │                           │                                     │
  │              ┌────────────┼────────────┐                       │
  │              │            │            │                        │
  │         ┌────▼──┐   ┌────▼───┐   ┌────▼─────┐                  │
  │         │Prometh│   │Grafana │   │  Jaeger  │                  │
  │         │(9091) │   │ (3001) │   │ (16686)  │                  │
  │         └───────┘   └────────┘   └──────────┘                  │
  └─────────────────────────────────────────────────────────────────┘

  Key Differences:
  - Peer Discovery: Nodes discover each other via PROVCHAIN_PEERS environment variable
  - Individual Data: Each node has its own data volume (node1_data, node2_data, node3_data)
  - Metrics Tagging: Each node uses OTEL_SERVICE_NAME to differentiate metrics

  ---
  Strategy C: Multi-Machine Distributed Cluster

  Files: docker-compose.node.yml (per node) + docker-compose.monitoring.yml

  Best for: Production deployments across VMs or cloud infrastructure

  ┌──────────────────────────────────────────────────────────────────────────────┐
  │                      Multi-Machine Distributed Deployment                    │
  ├──────────────────────────────────────────────────────────────────────────────┤
  │                                                                              │
  │  Machine 1 (192.168.1.101)        Machine 2 (192.168.1.102)                 │
  │  ┌─────────────────────┐         ┌─────────────────────┐                     │
  │  │   Node 1            │         │   Node 2            │                     │
  │  │   PEERS=""          │◀───────▶│   PEERS="192.168...│                     │
  │  │   (Bootstrap)       │         │                     │                     │
  │  │   Port: 8080/9090   │         │   Port: 8080/9090   │                     │
  │  └─────────────────────┘         └─────────────────────┘                     │
  │                                                                              │
  │                                    Machine 3 (192.168.1.103)                 │
  │                                    ┌─────────────────────┐                   │
  │                                    │   Node 3            │                   │
  │                                    │   PEERS="192.168...│                   │
  │                                    │   Port: 8080/9090   │                   │
  │                                    └─────────────────────┘                   │
  │                                                                              │
  │  Monitoring Machine (Optional - can be Node 1)                               │
  │  ┌──────────┐  ┌──────────┐  ┌──────────┐                                   │
  │  │Prometheus│  │ Grafana  │  │  Jaeger  │                                   │
  │  │ (9090)   │  │ (3000)   │  │ (16686)  │                                   │
  │  └────┬─────┘  └────┬─────┘  └────┬─────┘                                   │
  │       └──────────────┴──────────────┘                                        │
  │              Scrape metrics from all nodes                                    │
  └──────────────────────────────────────────────────────────────────────────────┘

  Configuration per node (.env file):

  # Node 1 (Bootstrap)
  PEERS=""
  JWT_SECRET=your_secure_secret_here

  # Node 2
  PEERS="192.168.1.101:8080"
  JWT_SECRET=your_secure_secret_here

  # Node 3
  PEERS="192.168.1.101:8080,192.168.1.102:8080"
  JWT_SECRET=your_secure_secret_here

  ---
  ★ Insight ─────────────────────────────────────
  Distributed Systems Patterns
  1. Bootstrap Node Pattern: Node 1 starts with empty PEERS list, serving as the network bootstrap point for other nodes to discover the cluster
  2. Mesh Topology: Each node can connect to multiple peers, creating a resilient P2P network that can handle node failures without network partition
  3. Service Identity: OTEL_SERVICE_NAME environment variable allows observability tools to distinguish metrics from different nodes in the cluster
  ─────────────────────────────────────────────────

  ---
  3. Monitoring Stack

  The monitoring stack is consistent across all strategies:

  | Tool       | Purpose
  | Prometheus | Scrapes metrics from /metrics endpoint | :9090 or :9091 |
  | Grafana    | Visualizes metrics with dashboards     | :3000 or :3001 |
  | Jaeger     | Distributed tracing for request flows  | :16686 (UI)    |

  Prometheus Configuration (monitoring/prometheus.yml):
  scrape_configs:
    - job_name: 'provchain'
      static_configs:
        - targets: ['node1:9090', 'node2:9090', 'node3:9090']

  For multi-machine deployments, the prometheus_multi_node.yml template requires manual IP configuration:
  static_configs:
    - targets:
      - 'NODE_1_IP:9090'  # Replace with actual IPs
      - 'NODE_2_IP:9090'
      - 'NODE_3_IP:9090'

  ---
  4. Key Ports Reference

  | Port      | Service         | Description                  |
  |-----------|-----------------|------------------------------|
  | 80        | Nginx           | HTTP (production)            |
  | 443       | Nginx           | HTTPS (production)           |
  | 8080-8082 | ProvChain Nodes | API & P2P communication      |
  | 9090      | Node Metrics    | Prometheus scraping endpoint |
  | 9091      | Prometheus      | Metrics server (all-in-one)  |
  | 3000/3001 | Grafana         | Dashboard UI                 |
  | 16686     | Jaeger          | Tracing UI                   |
  | 14268     | Jaeger          | Trace collector              |
  | 6379      | Redis           | Caching (optional)           |

  ---
  Summary

  | Deployment      | Use Case          | Complexity | Cost   |
  |-----------------|-------------------|------------|--------|
  | Single-Node     | Dev/testing       | ★☆☆        | Low    |
  | 3-Node (1 host) | Consensus testing | ★★☆        | Medium |
  | Multi-Machine   | Production        | ★★★        | High   |

  The Docker setup is well-architected with:
  - Multi-stage builds for optimized images
  - Health checks for container orchestration
  - Volume persistence for data durability
  - Separation of concerns (compute, monitoring, proxy)
  - Scalable patterns from single-node to distributed clusters
