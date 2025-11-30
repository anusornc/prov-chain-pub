//! Container orchestration and Docker support for production deployment

use crate::production::ProductionError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Container configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    /// Docker image name
    pub image_name: String,
    /// Docker image tag
    pub image_tag: String,
    /// Container port mappings
    pub port_mappings: HashMap<u16, u16>,
    /// Environment variables
    pub environment_vars: HashMap<String, String>,
    /// Volume mounts
    pub volumes: Vec<VolumeMount>,
    /// Resource limits
    pub resources: ResourceLimits,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        let mut port_mappings = HashMap::new();
        port_mappings.insert(8080, 8080);
        port_mappings.insert(9090, 9090); // Prometheus metrics

        let mut environment_vars = HashMap::new();
        environment_vars.insert("RUST_LOG".to_string(), "info".to_string());
        environment_vars.insert("ENVIRONMENT".to_string(), "production".to_string());

        Self {
            image_name: "provchain-org".to_string(),
            image_tag: "latest".to_string(),
            port_mappings,
            environment_vars,
            volumes: vec![
                VolumeMount {
                    host_path: "./data".to_string(),
                    container_path: "/app/data".to_string(),
                    read_only: false,
                },
                VolumeMount {
                    host_path: "./config".to_string(),
                    container_path: "/app/config".to_string(),
                    read_only: true,
                },
            ],
            resources: ResourceLimits::default(),
            health_check: HealthCheckConfig::default(),
        }
    }
}

/// Volume mount configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    pub host_path: String,
    pub container_path: String,
    pub read_only: bool,
}

/// Resource limits for containers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Memory limit in MB
    pub memory_mb: u64,
    /// CPU limit (number of cores)
    pub cpu_cores: f64,
    /// Disk space limit in MB
    pub disk_mb: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            memory_mb: 2048,
            cpu_cores: 2.0,
            disk_mb: 10240,
        }
    }
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check endpoint
    pub endpoint: String,
    /// Check interval in seconds
    pub interval_seconds: u64,
    /// Timeout in seconds
    pub timeout_seconds: u64,
    /// Number of retries before marking unhealthy
    pub retries: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            endpoint: "/health".to_string(),
            interval_seconds: 30,
            timeout_seconds: 10,
            retries: 3,
        }
    }
}

/// Container manager for Docker operations
pub struct ContainerManager {
    config: ContainerConfig,
}

impl ContainerManager {
    /// Create a new container manager
    pub fn new(config: ContainerConfig) -> Self {
        Self { config }
    }

    /// Generate Dockerfile content
    pub fn generate_dockerfile(&self) -> String {
        format!(
            r#"# Multi-stage Docker build for ProvChain
FROM rust:1.75 as builder

# Set working directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {{}}" > src/main.rs
RUN cargo build --release
RUN rm src/main.rs

# Copy source code
COPY src ./src
COPY ontology ./ontology
COPY queries ./queries
COPY test_data ./test_data

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 appuser

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/provchain-org /app/
COPY --from=builder /app/ontology /app/ontology
COPY --from=builder /app/queries /app/queries
COPY --from=builder /app/test_data /app/test_data

# Create data and config directories
RUN mkdir -p /app/data /app/config && \
    chown -R appuser:appuser /app

# Switch to app user
USER appuser

# Expose ports
{}

# Health check
HEALTHCHECK --interval={}s --timeout={}s --retries={} \
    CMD curl -f http://localhost:8080{} || exit 1

# Set environment variables
{}

# Default command
CMD ["./provchain-org", "web-server", "--port", "8080"]
"#,
            self.generate_expose_ports(),
            self.config.health_check.interval_seconds,
            self.config.health_check.timeout_seconds,
            self.config.health_check.retries,
            self.config.health_check.endpoint,
            self.generate_env_vars()
        )
    }

    /// Generate Docker Compose configuration
    pub fn generate_docker_compose(&self) -> String {
        format!(
            r#"version: '3.8'

services:
  provchain:
    build:
      context: .
      dockerfile: Dockerfile
    image: {}:{}
    container_name: provchain-app
    restart: unless-stopped
    ports:
{}
    environment:
{}
    volumes:
{}
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080{}"]
      interval: {}s
      timeout: {}s
      retries: {}
    deploy:
      resources:
        limits:
          memory: {}M
          cpus: '{}'
        reservations:
          memory: {}M
          cpus: '{}'
    networks:
      - provchain-network

  prometheus:
    image: prom/prometheus:latest
    container_name: provchain-prometheus
    restart: unless-stopped
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
      - '--storage.tsdb.retention.time=200h'
      - '--web.enable-lifecycle'
    networks:
      - provchain-network

  grafana:
    image: grafana/grafana:latest
    container_name: provchain-grafana
    restart: unless-stopped
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
      - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources:ro
    networks:
      - provchain-network

volumes:
  prometheus_data:
  grafana_data:

networks:
  provchain-network:
    driver: bridge
"#,
            self.config.image_name,
            self.config.image_tag,
            self.generate_port_mappings(),
            self.generate_environment_section(),
            self.generate_volumes_section(),
            self.config.health_check.endpoint,
            self.config.health_check.interval_seconds,
            self.config.health_check.timeout_seconds,
            self.config.health_check.retries,
            self.config.resources.memory_mb,
            self.config.resources.cpu_cores,
            self.config.resources.memory_mb / 2, // Reserve half for minimum
            self.config.resources.cpu_cores / 2.0
        )
    }

    /// Generate Kubernetes deployment manifest
    pub fn generate_kubernetes_deployment(&self) -> String {
        format!(
            r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: provchain-deployment
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
      - name: provchain
        image: {}:{}
        ports:
        - containerPort: 8080
        - containerPort: 9090
        env:
{}
        resources:
          limits:
            memory: "{}Mi"
            cpu: "{}"
          requests:
            memory: "{}Mi"
            cpu: "{}"
        livenessProbe:
          httpGet:
            path: {}
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: {}
        readinessProbe:
          httpGet:
            path: {}
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
        volumeMounts:
{}
      volumes:
{}
---
apiVersion: v1
kind: Service
metadata:
  name: provchain-service
spec:
  selector:
    app: provchain
  ports:
  - name: http
    port: 80
    targetPort: 8080
  - name: metrics
    port: 9090
    targetPort: 9090
  type: LoadBalancer
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: provchain-config
data:
  config.toml: |
    [blockchain]
    max_blocks = 10000
    
    [network]
    port = 8080
    
    [monitoring]
    enabled = true
    prometheus_port = 9090
"#,
            self.config.image_name,
            self.config.image_tag,
            self.generate_k8s_env_vars(),
            self.config.resources.memory_mb,
            self.config.resources.cpu_cores,
            self.config.resources.memory_mb / 2,
            self.config.resources.cpu_cores / 2.0,
            self.config.health_check.endpoint,
            self.config.health_check.interval_seconds,
            self.config.health_check.endpoint,
            self.generate_k8s_volume_mounts(),
            self.generate_k8s_volumes()
        )
    }

    /// Generate Helm chart values
    pub fn generate_helm_values(&self) -> String {
        format!(
            r#"# Helm values for ProvChain deployment
replicaCount: 3

image:
  repository: {}
  tag: {}
  pullPolicy: IfNotPresent

service:
  type: LoadBalancer
  port: 80
  targetPort: 8080
  metricsPort: 9090

ingress:
  enabled: true
  className: "nginx"
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
  hosts:
    - host: provchain.local
      paths:
        - path: /
          pathType: Prefix
  tls: []

resources:
  limits:
    cpu: {}
    memory: {}Mi
  requests:
    cpu: {}
    memory: {}Mi

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 80
  targetMemoryUtilizationPercentage: 80

nodeSelector: {{}}

tolerations: []

affinity: {{}}

monitoring:
  enabled: true
  serviceMonitor:
    enabled: true
    interval: 30s
    path: /metrics

persistence:
  enabled: true
  storageClass: "standard"
  accessMode: ReadWriteOnce
  size: 10Gi

configMap:
  data:
    config.toml: |
      [blockchain]
      max_blocks = 10000
      
      [network]
      port = 8080
      
      [monitoring]
      enabled = true
      prometheus_port = 9090
"#,
            self.config.image_name,
            self.config.image_tag,
            self.config.resources.cpu_cores,
            self.config.resources.memory_mb,
            self.config.resources.cpu_cores / 2.0,
            self.config.resources.memory_mb / 2
        )
    }

    // Helper methods for generating configuration sections
    fn generate_expose_ports(&self) -> String {
        self.config
            .port_mappings
            .keys()
            .map(|port| format!("EXPOSE {port}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_env_vars(&self) -> String {
        self.config
            .environment_vars
            .iter()
            .map(|(key, value)| format!("ENV {key}={value}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_port_mappings(&self) -> String {
        self.config
            .port_mappings
            .iter()
            .map(|(container_port, host_port)| format!("      - \"{host_port}:{container_port}\""))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_environment_section(&self) -> String {
        self.config
            .environment_vars
            .iter()
            .map(|(key, value)| format!("      - {key}={value}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_volumes_section(&self) -> String {
        self.config
            .volumes
            .iter()
            .map(|volume| {
                let mode = if volume.read_only { ":ro" } else { "" };
                format!(
                    "      - {}:{}{}",
                    volume.host_path, volume.container_path, mode
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_k8s_env_vars(&self) -> String {
        self.config
            .environment_vars
            .iter()
            .map(|(key, value)| format!("        - name: {key}\n          value: \"{value}\""))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_k8s_volume_mounts(&self) -> String {
        self.config
            .volumes
            .iter()
            .enumerate()
            .map(|(i, volume)| {
                format!(
                    "        - name: volume-{}\n          mountPath: {}\n          readOnly: {}",
                    i, volume.container_path, volume.read_only
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_k8s_volumes(&self) -> String {
        self.config
            .volumes
            .iter()
            .enumerate()
            .map(|(i, volume)| {
                format!(
                    "      - name: volume-{}\n        hostPath:\n          path: {}",
                    i, volume.host_path
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Write all container configuration files to disk
    pub async fn write_container_files(&self, output_dir: &PathBuf) -> Result<(), ProductionError> {
        use tokio::fs;

        // Create output directory
        fs::create_dir_all(output_dir).await.map_err(|e| {
            ProductionError::Configuration(format!("Failed to create output directory: {e}"))
        })?;

        // Write Dockerfile
        let dockerfile_path = output_dir.join("Dockerfile");
        fs::write(&dockerfile_path, self.generate_dockerfile())
            .await
            .map_err(|e| {
                ProductionError::Configuration(format!("Failed to write Dockerfile: {e}"))
            })?;

        // Write Docker Compose
        let compose_path = output_dir.join("docker-compose.yml");
        fs::write(&compose_path, self.generate_docker_compose())
            .await
            .map_err(|e| {
                ProductionError::Configuration(format!("Failed to write docker-compose.yml: {e}"))
            })?;

        // Write Kubernetes deployment
        let k8s_path = output_dir.join("kubernetes-deployment.yaml");
        fs::write(&k8s_path, self.generate_kubernetes_deployment())
            .await
            .map_err(|e| {
                ProductionError::Configuration(format!(
                    "Failed to write Kubernetes deployment: {e}"
                ))
            })?;

        // Write Helm values
        let helm_path = output_dir.join("helm-values.yaml");
        fs::write(&helm_path, self.generate_helm_values())
            .await
            .map_err(|e| {
                ProductionError::Configuration(format!("Failed to write Helm values: {e}"))
            })?;

        tracing::info!(
            "Container configuration files written to: {}",
            output_dir.display()
        );
        Ok(())
    }
}
