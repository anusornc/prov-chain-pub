use chrono::Utc;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use crate::storage::rdf_store::{RDFStore, StorageConfig};
use crate::trace_optimization::{EnhancedTraceabilitySystem, EnhancedTraceResult};
use crate::core::atomic_operations::AtomicOperationContext;
use crate::error::{ProvChainError, Result, BlockchainError};
use crate::ontology::{OntologyManager, OntologyConfig, ShaclValidator};
use oxigraph::model::NamedNode;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: String,
    pub data: String, // RDF in Turtle format
    pub previous_hash: String,
    pub hash: String,
    pub state_root: String, // State root hash for atomic consistency
}

impl Block {
    pub fn new(index: u64, data: String, previous_hash: String, state_root: String) -> Self {
        let timestamp = Utc::now().to_rfc3339();
        let mut block = Block {
            index,
            timestamp,
            data,
            previous_hash,
            hash: String::new(),
            state_root,
        };
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        self.calculate_hash_with_store(None)
    }

    pub fn calculate_hash_with_store(&self, rdf_store: Option<&RDFStore>) -> String {
        let rdf_hash = if let Some(store) = rdf_store {
            // Use RDF canonicalization for the data
            match NamedNode::new(format!("http://provchain.org/block/{}", self.index)) {
                Ok(graph_name) => store.canonicalize_graph(&graph_name),
                Err(_) => {
                    // Fallback to simple hash if graph name creation fails
                    let mut hasher = Sha256::new();
                    hasher.update(self.data.as_bytes());
                    format!("{:x}", hasher.finalize())
                }
            }
        } else {
            // Fallback to simple hash if no store provided (for genesis block)
            let mut hasher = Sha256::new();
            hasher.update(self.data.as_bytes());
            format!("{:x}", hasher.finalize())
        };

        // Combine block metadata with canonicalized RDF hash
        let record = format!(
            "{0}{1}{2}{3}",
            self.index,
            self.timestamp,
            rdf_hash,
            self.previous_hash
        );
        let mut hasher = Sha256::new();
        hasher.update(record.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[derive(Debug, Clone)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub rdf_store: RDFStore,
    pub ontology_manager: Option<OntologyManager>,
    pub shacl_validator: Option<ShaclValidator>,
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}

impl Blockchain {
    /// Create a new in-memory blockchain (for testing and development)
    pub fn new() -> Self {
        let mut bc = Blockchain {
            chain: Vec::new(),
            rdf_store: RDFStore::new(),
            ontology_manager: None,
            shacl_validator: None,
        };
        
        // Load the traceability ontology
        bc.load_ontology();
        
        let genesis_block = bc.create_genesis_block();
        
        // Add genesis block data to RDF store
        if let Ok(graph_name) = NamedNode::new("http://provchain.org/block/0") {
            bc.rdf_store.add_rdf_to_graph(&genesis_block.data, &graph_name);
            bc.rdf_store.add_block_metadata(&genesis_block);
        } else {
            eprintln!("Warning: Could not create graph name for genesis block");
        }
        
        bc.chain.push(genesis_block);
        bc
    }

    /// Create a new persistent blockchain with RocksDB backend
    pub fn new_persistent<P: AsRef<Path>>(data_dir: P) -> Result<Self> {
        let rdf_store = RDFStore::new_persistent(data_dir)?;
        
        let mut bc = Blockchain {
            chain: Vec::new(),
            rdf_store,
            ontology_manager: None,
            shacl_validator: None,
        };
        
        // Load the traceability ontology
        bc.load_ontology();
        
        // Check if we need to create genesis block or load existing chain
        let store_len = bc.rdf_store.store.len().unwrap_or(0);
        if store_len == 0 {
            // Create genesis block for new blockchain
            let genesis_block = bc.create_genesis_block();
            
            // Add genesis block data to RDF store
            if let Ok(graph_name) = NamedNode::new("http://provchain.org/block/0") {
                bc.rdf_store.add_rdf_to_graph(&genesis_block.data, &graph_name);
                bc.rdf_store.add_block_metadata(&genesis_block);
            } else {
                eprintln!("Warning: Could not create graph name for genesis block in persistent store");
            }
            
            bc.chain.push(genesis_block);
            
            // Save to disk
            if let Err(e) = bc.rdf_store.save_to_disk() {
                eprintln!("Warning: Could not save to disk: {e}");
            }
        } else {
            // Load existing blockchain from persistent storage
            bc.load_chain_from_store()?;
            
            // If no blocks were loaded but store has data, something is wrong
            // Create genesis block as fallback
            if bc.chain.is_empty() {
                eprintln!("Warning: Store has data but no blocks loaded, creating genesis block");
                let genesis_block = bc.create_genesis_block();
                
                if let Ok(graph_name) = NamedNode::new("http://provchain.org/block/0") {
                    bc.rdf_store.add_rdf_to_graph(&genesis_block.data, &graph_name);
                    bc.rdf_store.add_block_metadata(&genesis_block);
                } else {
                    eprintln!("Warning: Could not create graph name for fallback genesis block");
                }
                
                bc.chain.push(genesis_block);
            }
        }
        
        Ok(bc)
    }

    /// Create a persistent blockchain with custom storage configuration
    pub fn new_persistent_with_config(config: StorageConfig) -> Result<Self> {
        let rdf_store = RDFStore::new_persistent_with_config(config)?;
        
        let mut bc = Blockchain {
            chain: Vec::new(),
            rdf_store,
            ontology_manager: None,
            shacl_validator: None,
        };
        
        // Load the traceability ontology
        bc.load_ontology();
        
        // Check if we need to create genesis block or load existing chain
        if bc.rdf_store.store.len().unwrap_or(0) == 0 {
            // Create genesis block for new blockchain
            let genesis_block = bc.create_genesis_block();
            
            // Add genesis block data to RDF store
            if let Ok(graph_name) = NamedNode::new("http://provchain.org/block/0") {
                bc.rdf_store.add_rdf_to_graph(&genesis_block.data, &graph_name);
                bc.rdf_store.add_block_metadata(&genesis_block);
            } else {
                eprintln!("Warning: Could not create graph name for genesis block in config store");
            }
            
            bc.chain.push(genesis_block);
        } else {
            // Load existing blockchain from persistent storage
            bc.load_chain_from_store()?;
        }
        
        Ok(bc)
    }

    /// Load blockchain from persistent RDF store
    fn load_chain_from_store(&mut self) -> Result<()> {
        use oxigraph::sparql::QueryResults;
        
        // Query to get all blocks ordered by index
        let query = r#"
            PREFIX prov: <http://provchain.org/>
            SELECT ?block ?index ?timestamp ?hash ?prevHash ?dataGraph WHERE {
                GRAPH <http://provchain.org/blockchain> {
                    ?block a prov:Block ;
                           prov:hasIndex ?index ;
                           prov:hasTimestamp ?timestamp ;
                           prov:hasHash ?hash ;
                           prov:hasPreviousHash ?prevHash ;
                           prov:hasDataGraphIRI ?dataGraph .
                }
            }
            ORDER BY ?index
        "#;
        
        if let QueryResults::Solutions(solutions) = self.rdf_store.query(query) {
            for sol in solutions.flatten() {
                if let (Some(index_term), Some(timestamp_term), Some(hash_term), Some(prev_hash_term), Some(data_graph_term)) = (
                    sol.get("index"),
                    sol.get("timestamp"), 
                    sol.get("hash"),
                    sol.get("prevHash"),
                    sol.get("dataGraph")
                ) {
                    // Parse block data
                    let index: u64 = index_term.to_string().parse().unwrap_or(0);
                    let timestamp = timestamp_term.to_string().trim_matches('"').to_string();
                    let hash = hash_term.to_string().trim_matches('"').to_string();
                    let previous_hash = prev_hash_term.to_string().trim_matches('"').to_string();
                    
                    // Extract RDF data from the block's graph
                    let data_graph_string = data_graph_term.to_string();
                    println!("Raw data graph string: '{}'", data_graph_string);
                    // Handle typed literals - extract the URI part before the type annotation
                    let data_graph_uri = if let Some(uri_part) = data_graph_string.split("^^").next() {
                        uri_part.trim_matches('"').trim_matches('<').trim_matches('>')
                    } else {
                        data_graph_string.trim_matches('"').trim_matches('<').trim_matches('>')
                    };
                    println!("Processed data graph URI: '{}'", data_graph_uri);
                    let data = self.extract_rdf_data_from_graph(data_graph_uri)?;
                    
                    // For existing blocks, we'll use a placeholder state_root
                    // In a real implementation, this would be loaded from the blockchain metadata
                    let state_root = "0000000000000000000000000000000000000000000000000000000000000000".to_string();
                    
                    let block = Block {
                        index,
                        timestamp,
                        data,
                        previous_hash,
                        hash,
                        state_root,
                    };
                    
                    self.chain.push(block);
                }
            }
        }
        
        println!("Loaded {} blocks from persistent storage", self.chain.len());
        Ok(())
    }

    /// Extract RDF data from a specific graph
    fn extract_rdf_data_from_graph(&self, graph_uri: &str) -> Result<String> {
        
        // Debug output
        println!("Attempting to extract RDF data from graph: '{}'", graph_uri);
        
        let graph_name = NamedNode::new(graph_uri)?;
        
        // Collect all triples from the specific graph
        let mut triples = Vec::new();
        let graph_name_ref = oxigraph::model::GraphNameRef::NamedNode((&graph_name).into());
        for quad_result in self.rdf_store.store.quads_for_pattern(None, None, None, Some(graph_name_ref)) {
            if let Ok(quad) = quad_result {
                // Create a triple from the quad (without the graph component)
                let triple = oxigraph::model::Triple::new(
                    quad.subject,
                    quad.predicate,
                    quad.object,
                );
                triples.push(triple);
            }
        }
        
        println!("Found {} triples in graph '{}'", triples.len(), graph_uri);
        
        // If no triples, return empty string
        if triples.is_empty() {
            return Ok(String::new());
        }
        
        // Manually serialize triples to Turtle format
        let mut turtle_data = String::new();
        
        // Add prefixes (simplified - in a real implementation we'd extract actual prefixes)
        turtle_data.push_str("@prefix ex: <http://example.org/> .\n");
        turtle_data.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
        turtle_data.push_str("@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n");
        turtle_data.push_str("@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\n");
        
        // Serialize each triple
        for triple in triples {
            let subject_str = match &triple.subject {
                oxigraph::model::Subject::NamedNode(node) => format!("<{}>", node.as_str()),
                oxigraph::model::Subject::BlankNode(node) => format!("_:{}", node.as_str()),
                oxigraph::model::Subject::Triple(_) => "<http://example.org/quoted-triple>".to_string(), // Simplified
            };
            
            let predicate_str = match &triple.predicate {
                node => format!("<{}>", node.as_str()),
            };
            
            let object_str = match &triple.object {
                oxigraph::model::Term::NamedNode(node) => format!("<{}>", node.as_str()),
                oxigraph::model::Term::BlankNode(node) => format!("_:{}", node.as_str()),
                oxigraph::model::Term::Literal(lit) => format!("{}", lit),
                oxigraph::model::Term::Triple(_) => "<< >>".to_string(), // Simplified
            };
            
            turtle_data.push_str(&format!("{} {} {} .\n", subject_str, predicate_str, object_str));
        }
        
        println!("Generated Turtle data: {}", turtle_data);
        Ok(turtle_data)
    }

    /// Get storage statistics
    pub fn get_storage_stats(&self) -> Result<crate::storage::rdf_store::StorageStats> {
        self.rdf_store.get_storage_stats().map_err(|e| e.into())
    }

    /// Create a backup of the blockchain
    pub fn create_backup(&self, _backup_id: String) -> Result<crate::storage::rdf_store::BackupInfo> {
        self.rdf_store.create_backup().map_err(|e| e.into())
    }

    /// List available backups
    pub fn list_backups(&self) -> Result<Vec<crate::storage::rdf_store::BackupInfo>> {
        self.rdf_store.list_backups().map_err(|e| e.into())
    }

    /// Create a new in-memory blockchain with ontology configuration
    pub fn new_with_ontology(ontology_config: OntologyConfig) -> Result<Self> {
        let mut bc = Blockchain {
            chain: Vec::new(),
            rdf_store: RDFStore::new(),
            ontology_manager: None,
            shacl_validator: None,
        };
        
        // Initialize ontology manager and SHACL validator
        bc.initialize_ontology_system(ontology_config)?;
        
        let genesis_block = bc.create_genesis_block();
        
        // Add genesis block data to RDF store
        if let Ok(graph_name) = NamedNode::new("http://provchain.org/block/0") {
            bc.rdf_store.add_rdf_to_graph(&genesis_block.data, &graph_name);
            bc.rdf_store.add_block_metadata(&genesis_block);
        } else {
            eprintln!("Warning: Could not create graph name for genesis block");
        }
        
        bc.chain.push(genesis_block);
        Ok(bc)
    }

    /// Create a new persistent blockchain with ontology configuration
    pub fn new_persistent_with_ontology<P: AsRef<Path>>(data_dir: P, ontology_config: OntologyConfig) -> Result<Self> {
        let rdf_store = RDFStore::new_persistent(data_dir)?;
        
        let mut bc = Blockchain {
            chain: Vec::new(),
            rdf_store,
            ontology_manager: None,
            shacl_validator: None,
        };
        
        // Initialize ontology manager and SHACL validator
        bc.initialize_ontology_system(ontology_config)?;
        
        // Check if we need to create genesis block or load existing chain
        let store_len = bc.rdf_store.store.len().unwrap_or(0);
        if store_len == 0 {
            // Create genesis block for new blockchain
            let genesis_block = bc.create_genesis_block();
            
            // Add genesis block data to RDF store
            if let Ok(graph_name) = NamedNode::new("http://provchain.org/block/0") {
                bc.rdf_store.add_rdf_to_graph(&genesis_block.data, &graph_name);
                bc.rdf_store.add_block_metadata(&genesis_block);
            } else {
                eprintln!("Warning: Could not create graph name for genesis block in persistent store");
            }
            
            bc.chain.push(genesis_block);
            
            // Save to disk
            if let Err(e) = bc.rdf_store.save_to_disk() {
                eprintln!("Warning: Could not save to disk: {e}");
            }
        } else {
            // Load existing blockchain from persistent storage
            bc.load_chain_from_store()?;
            
            // If no blocks were loaded but store has data, something is wrong
            // Create genesis block as fallback
            if bc.chain.is_empty() {
                eprintln!("Warning: Store has data but no blocks loaded, creating genesis block");
                let genesis_block = bc.create_genesis_block();
                
                if let Ok(graph_name) = NamedNode::new("http://provchain.org/block/0") {
                    bc.rdf_store.add_rdf_to_graph(&genesis_block.data, &graph_name);
                    bc.rdf_store.add_block_metadata(&genesis_block);
                } else {
                    eprintln!("Warning: Could not create graph name for fallback genesis block");
                }
                
                bc.chain.push(genesis_block);
            }
        }
        
        Ok(bc)
    }

    /// Initialize the ontology management system
    fn initialize_ontology_system(&mut self, ontology_config: OntologyConfig) -> std::result::Result<(), ProvChainError> {
        // Create ontology manager
        let ontology_manager = OntologyManager::new(ontology_config.clone())
            .map_err(|e| ProvChainError::Blockchain(BlockchainError::OntologyInitializationFailed(
                format!("Failed to initialize ontology manager: {}", e)
            )))?;

        // Create SHACL validator
        let shacl_validator = ShaclValidator::new(
            &ontology_config.core_shacl_path,
            &ontology_config.domain_shacl_path,
            ontology_config.ontology_hash.clone(),
        ).map_err(|e| ProvChainError::Blockchain(BlockchainError::OntologyInitializationFailed(
            format!("Failed to initialize SHACL validator: {}", e)
        )))?;

        self.ontology_manager = Some(ontology_manager);
        self.shacl_validator = Some(shacl_validator);

        println!("Initialized ontology system for domain: {}", 
                 self.ontology_manager.as_ref().unwrap().get_domain_name());

        Ok(())
    }

    /// Restore blockchain from backup
    pub fn restore_from_backup<P: AsRef<Path>>(backup_path: P, target_dir: P) -> Result<Self> {
        let rdf_store = RDFStore::restore_from_backup(backup_path, target_dir).map_err(|e| ProvChainError::Anyhow(e))?;
        
        let mut bc = Blockchain {
            chain: Vec::new(),
            rdf_store,
            ontology_manager: None,
            shacl_validator: None,
        };
        
        // Load the chain from the restored store
        bc.load_chain_from_store()?;
        
        Ok(bc)
    }

    /// Flush any pending writes to disk
    pub fn flush(&self) -> Result<()> {
        self.rdf_store.flush().map_err(|e| e.into())
    }

    /// Optimize the underlying database
    pub fn optimize(&self) -> Result<()> {
        self.rdf_store.optimize().map_err(|e| e.into())
    }

    /// Check database integrity
    pub fn check_integrity(&self) -> Result<crate::storage::rdf_store::IntegrityReport> {
        self.rdf_store.check_integrity().map_err(|e| e.into())
    }

    fn create_genesis_block(&self) -> Block {
        // For genesis block, we calculate the initial state root
        let initial_state_root = self.rdf_store.calculate_state_root();
        Block::new(
            0,
            "@prefix ex: <http://example.org/> . ex:genesis ex:type \"Genesis Block\".".into(),
            "0".into(),
            initial_state_root
        )
    }

    /// Add a new block with SHACL validation and ontology consistency checking
    pub fn add_block(&mut self, data: String) -> Result<()> {
        // Ensure we have at least a genesis block
        if self.chain.is_empty() {
            let genesis_block = self.create_genesis_block();
            if let Ok(graph_name) = NamedNode::new("http://provchain.org/block/0") {
                self.rdf_store.add_rdf_to_graph(&genesis_block.data, &graph_name);
                self.rdf_store.add_block_metadata(&genesis_block);
            } else {
                return Err(ProvChainError::Blockchain(BlockchainError::GenesisCreationFailed(
                    "Could not create graph name for genesis block".to_string()
                )));
            }
            self.chain.push(genesis_block);
        }

        // STEP 8: SHACL VALIDATION - Validate transaction data before adding to blockchain
        if let Some(ref shacl_validator) = self.shacl_validator {
            // Validate the RDF data against SHACL shapes
            match shacl_validator.validate_transaction(&data) {
                Ok(validation_result) => {
                    if !validation_result.is_valid {
                        // STRICT ENFORCEMENT: Block invalid transactions
                        let error_msg = format!(
                            "Transaction validation failed: {} violations found. Details:\n{}",
                            validation_result.violations.len(),
                            validation_result.violations.iter()
                                .map(|v| format!("- {}: {} ({})", v.constraint_type, v.message, v.severity))
                                .collect::<Vec<_>>()
                                .join("\n")
                        );
                        
                        return Err(ProvChainError::Blockchain(BlockchainError::ValidationFailed(error_msg)));
                    }
                    
                    // Log successful validation
                    println!("✓ Transaction validation passed: {} constraints validated successfully", 
                             validation_result.constraints_checked);
                }
                Err(validation_error) => {
                    // STRICT ENFORCEMENT: Block transactions that fail validation process
                    let error_msg = format!("SHACL validation process failed: {}", validation_error);
                    return Err(ProvChainError::Blockchain(BlockchainError::ValidationFailed(error_msg)));
                }
            }
        } else {
            // If no SHACL validator is configured, warn but allow (for backward compatibility with existing tests)
            eprintln!("Warning: No SHACL validator configured - transaction added without domain-specific validation");
        }

        let prev_block = self.chain.last()
            .ok_or_else(|| ProvChainError::Blockchain(BlockchainError::InvalidChainState(
                "Chain is empty after genesis block creation".to_string()
            )))?;
        
        // Calculate the state root before creating the new block
        let state_root = self.rdf_store.calculate_state_root();
        let mut new_block = Block::new(prev_block.index + 1, data.clone(), prev_block.hash.clone(), state_root);

        // Use atomic operations to ensure consistency
        let mut atomic_context = AtomicOperationContext::new(&mut self.rdf_store);
        
        // Add RDF data and block metadata atomically
        atomic_context.add_block_atomically(&new_block)?;
        
        // Recalculate hash using RDF canonicalization after successful atomic operation
        new_block.hash = new_block.calculate_hash_with_store(Some(&self.rdf_store));
        
        // Update the block metadata with the new hash
        self.rdf_store.add_block_metadata(&new_block);

        self.chain.push(new_block);
        
        Ok(())
    }

    pub fn is_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let prev = &self.chain[i - 1];
            
            // Check if the block's data matches what's stored in the RDF store
            if !self.validate_block_data_integrity(current) {
                return false;
            }
            
            // Use RDF canonicalization for validation
            let expected_hash = current.calculate_hash_with_store(Some(&self.rdf_store));
            
            if current.hash != expected_hash || current.previous_hash != prev.hash {
                return false;
            }
        }
        true
    }

    /// Validate that the block's data field matches what's stored in the RDF store
    pub fn validate_block_data_integrity(&self, block: &Block) -> bool {
        // Create a temporary RDF store to parse the block's data
        let mut temp_rdf_store = crate::storage::RDFStore::new();
        
        // Use the same graph name structure as the main store for proper comparison
        let graph_name = match NamedNode::new(format!("http://provchain.org/block/{}", block.index)) {
            Ok(name) => name,
            Err(_) => {
                eprintln!("Warning: Could not create graph name for block validation");
                return false;
            }
        };
        
        // Add the block's data to the temporary store using the same graph name
        temp_rdf_store.add_rdf_to_graph(&block.data, &graph_name);
        
        // Get canonical hash from temporary store
        let temp_canonical_hash = temp_rdf_store.canonicalize_graph(&graph_name);
        
        // Get canonical hash from main store for this block's graph
        let main_canonical_hash = self.rdf_store.canonicalize_graph(&graph_name);
        
        // Compare canonical hashes - this handles blank node differences correctly
        temp_canonical_hash == main_canonical_hash
    }

    fn load_ontology(&mut self) {
        // For now, use hardcoded ontology loading
        // TODO: Integrate with CLI-based ontology selection in Step 7
        self.load_ontology_hardcoded();
    }

    /// Fallback method for hardcoded ontology loading
    fn load_ontology_hardcoded(&mut self) {
        if let Ok(ontology_data) = std::fs::read_to_string("ontologies/generic_core.owl") {
            if let Ok(ontology_graph) = NamedNode::new("http://provchain.org/ontology") {
                self.rdf_store.load_ontology(&ontology_data, &ontology_graph);
                println!("Loaded traceability ontology from ontologies/generic_core.owl");
            } else {
                eprintln!("Warning: Could not create ontology graph name");
            }
        } else {
            eprintln!("Warning: Could not load ontology file ontologies/generic_core.owl");
        }
    }

    /// Enhanced trace function applying SSSP-inspired optimization concepts
    pub fn enhanced_trace(&self, batch_id: &str, optimization_level: u8) -> EnhancedTraceResult {
        let trace_system = EnhancedTraceabilitySystem::new(self);
        trace_system.enhanced_trace(batch_id, optimization_level)
    }

    pub fn dump(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.chain)
            .map_err(|e| ProvChainError::Json(e))
    }

    /// Get the latest block index (for WebSocket integration)
    pub fn get_latest_block_index(&self) -> u64 {
        self.chain.last().map(|block| block.index).unwrap_or(0)
    }

    /// Get the total number of transactions (placeholder implementation)
    pub fn get_transaction_count(&self) -> usize {
        // This is a simplified implementation
        // In a real system, this would count actual transactions across all blocks
        self.chain.len().saturating_sub(1) // Subtract genesis block
    }

    /// Get the number of active participants (placeholder implementation)
    pub fn get_participant_count(&self) -> usize {
        // This is a simplified implementation
        // In a real system, this would query the RDF store for unique participants
        // For now, return a reasonable default
        5
    }
}
