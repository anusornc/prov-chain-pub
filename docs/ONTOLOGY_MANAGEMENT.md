# Ontology Management System

The ontology management system provides a flexible and configurable way to load and manage ontologies in the ProvChain system.

## Features

### 1. Flexible Loading Strategies
- **File System**: Load ontologies from local files
- **HTTP/HTTPS**: Load ontologies from remote URLs
- **Embedded**: Load ontologies from embedded resources
- **Auto-detection**: Automatically detect loading strategy

### 2. Configuration-Based Loading
Ontologies can be configured through:
- TOML configuration files
- Environment variables
- Programmatic configuration

### 3. Runtime Flexibility
- Load/unload ontologies at runtime
- Reload ontologies without restarting the application
- Dynamic namespace management

### 4. Domain-Specific Ontologies
Support for loading domain-specific ontologies:
- Supply Chain
- Healthcare
- Pharmaceutical
- Automotive
- Digital Assets

## Configuration

### TOML Configuration File

Create a configuration file (e.g., `config/ontology.toml`):

```toml
[main_ontology]
path = "ontologies/generic_core.owl"
graph_iri = "http://provchain.org/ontology/core"
auto_load = true
validate_data = false

[resolution]
strategy = "FileSystem"

[namespaces]
core = "http://provchain.org/core#"
prov = "http://www.w3.org/ns/prov#"
xsd = "http://www.w3.org/2001/XMLSchema#"
rdfs = "http://www.w3.org/2000/01/rdf-schema#"
owl = "http://www.w3.org/2002/07/owl#"

[domain_ontologies.supply_chain]
path = "ontologies/supply-chain.owl"
graph_iri = "http://provchain.org/ontology/supply-chain"
enabled = true
priority = 100

[domain_ontologies.healthcare]
path = "ontologies/healthcare.owl"
graph_iri = "http://provchain.org/ontology/healthcare"
enabled = true
priority = 90
```

### Environment Variables

Override configuration with environment variables:

```bash
export ONTOLOGY_MAIN_PATH="custom/ontology.owl"
export ONTOLOGY_AUTO_LOAD=true
export ONTOLOGY_VALIDATE_DATA=false
```

## Usage

### Basic Usage

```rust
use provchain_org::ontology::{OntologyManager, OntologyConfig};

// Create manager with default configuration
let mut manager = OntologyManager::new()?;

// Or create with custom configuration
let config = OntologyConfig::default();
let mut manager = OntologyManager::with_config(config)?;

// Load ontologies
manager.load_configured_ontologies()?;

// Get RDF store with loaded ontologies
let rdf_store = manager.get_rdf_store();
```

### Programmatic Configuration

```rust
use provchain_org::ontology::{OntologyManager, OntologyConfig, DomainOntologyConfig};

let mut config = OntologyConfig::default();

// Customize main ontology
config.main_ontology_path = "custom/custom_core.owl".to_string();
config.main_ontology_graph = "http://custom.org/ontology/core".to_string();

// Add domain ontology
let domain_config = DomainOntologyConfig {
    path: "domains/custom_domain.owl".to_string(),
    graph_iri: "http://custom.org/ontology/custom-domain".to_string(),
    enabled: true,
    priority: 100,
};

config.domain_ontologies.insert("custom_domain".to_string(), domain_config);

let mut manager = OntologyManager::with_config(config)?;
manager.load_configured_ontologies()?;
```

### Runtime Management

```rust
// Add namespace mapping
manager.add_namespace("custom", "http://custom.org/ns#");

// Reload specific ontology
manager.reload_ontology("http://provchain.org/ontology/core")?;

// Reload all ontologies
manager.reload_ontologies()?;

// Validate loaded ontologies
manager.validate_required_ontologies()?;
```

## Benefits

### 1. Decoupling
- Ontology loading is separated from core blockchain logic
- Easy to swap ontologies without code changes
- Configuration-driven behavior

### 2. Flexibility
- Support multiple loading strategies
- Runtime ontology management
- Domain-specific ontology loading

### 3. Maintainability
- Centralized ontology management
- Clear configuration interface
- Extensible architecture

### 4. Performance
- Caching of loaded ontologies
- Efficient namespace resolution
- Lazy loading when possible

## Extending the System

### Adding New Resolution Strategies

Implement the `OntologyResolutionStrategy` enum and corresponding loading methods.

### Custom Domain Ontologies

Add new domain configurations to the `domain_ontologies` section in configuration.

### Embedded Ontologies

Implement the `load_embedded_ontology` method to load ontologies from embedded resources.