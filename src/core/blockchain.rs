use crate::error::{BlockchainError, ProvChainError, Result};
use crate::governance::Governance;
use crate::ontology::{OntologyConfig, OntologyManager, ShaclValidator};
use crate::storage::rdf_store::{RDFStore, StorageConfig};
use crate::trace_optimization::{EnhancedTraceResult, EnhancedTraceabilitySystem};
use chrono::Utc;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use hex;
use oxigraph::model::NamedNode;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use tracing::{debug, info};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: String,
    pub data: String, // RDF in Turtle format
    pub encrypted_data: Option<String>, // JSON-serialized EncryptedData
    pub previous_hash: String,
    pub hash: String,
    pub state_root: String, // State root hash for atomic consistency
    pub validator: String,  // Public key of the validator
    pub signature: String,  // Signature of the block hash
}

impl Block {
    pub fn new(
        index: u64,
        data: String,
        previous_hash: String,
        state_root: String,
        validator: String,
    ) -> Self {
        let timestamp = Utc::now().to_rfc3339();
        let mut block = Block {
            index,
            timestamp,
            data,
            encrypted_data: None,
            previous_hash,
            hash: String::new(),
            state_root,
            validator,
            signature: String::new(),
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
        // Note: Validator is part of the hash, but signature is NOT (signature signs the hash)
        let encrypted_part = self.encrypted_data.as_deref().unwrap_or("");
        let record = format!(
            "{0}{1}{2}{3}{4}{5}",
            self.index, self.timestamp, rdf_hash, self.previous_hash, self.validator, encrypted_part
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
    pub governance: Governance,
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
            governance: Governance::new(),
        };

        // Load the traceability ontology
        bc.load_ontology();

        let mut genesis_block = bc.create_genesis_block();

        // Add genesis block data to RDF store BEFORE calculating final hash
        if let Ok(graph_name) = NamedNode::new("http://provchain.org/block/0") {
            bc.rdf_store
                .add_rdf_to_graph(&genesis_block.data, &graph_name);
            bc.rdf_store.add_block_metadata(&genesis_block);
        } else {
            eprintln!("Warning: Could not create graph name for genesis block");
        }

        // Recalculate hash after all data is in the store
        genesis_block.hash = genesis_block.calculate_hash_with_store(Some(&bc.rdf_store));

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
            governance: Governance::new(),
        };

        // Load the traceability ontology
        bc.load_ontology();

        // Check if we need to create genesis block or load existing chain
        let store_len = bc.rdf_store.store.len().unwrap_or(0);
        if store_len == 0 {
            // Create genesis block for new blockchain
            let mut genesis_block = bc.create_genesis_block();

            // Add genesis block data to RDF store
            if let Ok(graph_name) = NamedNode::new("http://provchain.org/block/0") {
                bc.rdf_store
                    .add_rdf_to_graph(&genesis_block.data, &graph_name);
                bc.rdf_store.add_block_metadata(&genesis_block);
            } else {
                eprintln!(
                    "Warning: Could not create graph name for genesis block in persistent store"
                );
            }

            // Recalculate hash after adding data to RDF store (consistent with new() method)
            genesis_block.hash = genesis_block.calculate_hash_with_store(Some(&bc.rdf_store));
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
                let mut genesis_block = bc.create_genesis_block();

                if let Ok(graph_name) = NamedNode::new("http://provchain.org/block/0") {
                    bc.rdf_store
                        .add_rdf_to_graph(&genesis_block.data, &graph_name);
                    bc.rdf_store.add_block_metadata(&genesis_block);
                } else {
                    eprintln!("Warning: Could not create graph name for fallback genesis block");
                }

                // Recalculate hash after adding data to RDF store (consistent with new() method)
                genesis_block.hash = genesis_block.calculate_hash_with_store(Some(&bc.rdf_store));
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
            governance: Governance::new(),
        };

        // Load the traceability ontology
        bc.load_ontology();

        // Check if we need to create genesis block or load existing chain
        if bc.rdf_store.store.len().unwrap_or(0) == 0 {
            // Create genesis block for new blockchain
            let mut genesis_block = bc.create_genesis_block();

            // Add genesis block data to RDF store
            if let Ok(graph_name) = NamedNode::new("http://provchain.org/block/0") {
                bc.rdf_store
                    .add_rdf_to_graph(&genesis_block.data, &graph_name);
                bc.rdf_store.add_block_metadata(&genesis_block);
            } else {
                eprintln!("Warning: Could not create graph name for genesis block in config store");
            }

            // Recalculate hash after adding data to RDF store (consistent with new() method)
            genesis_block.hash = genesis_block.calculate_hash_with_store(Some(&bc.rdf_store));
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
            SELECT ?block ?index ?timestamp ?hash ?prevHash ?dataGraph ?validator ?signature ?encryptedData WHERE {
                GRAPH <http://provchain.org/blockchain> {
                    ?block a prov:Block ;
                           prov:hasIndex ?index ;
                           prov:hasTimestamp ?timestamp ;
                           prov:hasHash ?hash ;
                           prov:hasPreviousHash ?prevHash ;
                           prov:hasDataGraphIRI ?dataGraph ;
                           prov:hasValidator ?validator ;
                           prov:hasSignature ?signature .
                    OPTIONAL { ?block prov:hasEncryptedData ?encryptedData }
                }
            }
            ORDER BY ?index
        "#;

        if let QueryResults::Solutions(solutions) = self.rdf_store.query(query) {
            for sol in solutions.flatten() {
                if let (
                    Some(index_term),
                    Some(timestamp_term),
                    Some(hash_term),
                    Some(prev_hash_term),
                    Some(data_graph_term),
                    Some(validator_term),
                    Some(signature_term),
                ) = (
                    sol.get("index"),
                    sol.get("timestamp"),
                    sol.get("hash"),
                    sol.get("prevHash"),
                    sol.get("dataGraph"),
                    sol.get("validator"),
                    sol.get("signature"),
                ) {
                    // Parse block data
                    let index: u64 = index_term.to_string().parse().unwrap_or(0);
                    let timestamp = timestamp_term.to_string().trim_matches('"').to_string();
                    let hash = hash_term.to_string().trim_matches('"').to_string();
                    let previous_hash = prev_hash_term.to_string().trim_matches('"').to_string();
                    let validator = validator_term.to_string().trim_matches('"').to_string();
                    let signature = signature_term.to_string().trim_matches('"').to_string();
                    
                    let encrypted_data = sol.get("encryptedData")
                        .map(|t| t.to_string().trim_matches('"').to_string());

                    // Extract RDF data from the block's graph
                    let data_graph_string = data_graph_term.to_string();
                    println!("Raw data graph string: '{}'", data_graph_string);
                    // Handle typed literals - extract the URI part before the type annotation
                    let data_graph_uri =
                        if let Some(uri_part) = data_graph_string.split("^^").next() {
                            uri_part
                                .trim_matches('"')
                                .trim_matches('<')
                                .trim_matches('>')
                        } else {
                            data_graph_string
                                .trim_matches('"')
                                .trim_matches('<')
                                .trim_matches('>')
                        };
                    println!("Processed data graph URI: '{}'", data_graph_uri);
                    let data = self.extract_rdf_data_from_graph(data_graph_uri)?;

                    // For existing blocks, we'll use a placeholder state_root
                    // In a real implementation, this would be loaded from the blockchain metadata
                    let state_root =
                        "0000000000000000000000000000000000000000000000000000000000000000"
                            .to_string();

                    let block = Block {
                        index,
                        timestamp,
                        data,
                        encrypted_data,
                        previous_hash,
                        hash,
                        state_root,
                        validator,
                        signature,
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
        for quad_result in
            self.rdf_store
                .store
                .quads_for_pattern(None, None, None, Some(graph_name_ref))
        {
            if let Ok(quad) = quad_result {
                // Create a triple from the quad (without the graph component)
                let triple =
                    oxigraph::model::Triple::new(quad.subject, quad.predicate, quad.object);
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
                oxigraph::model::Subject::Triple(_) => {
                    "<http://example.org/quoted-triple>".to_string()
                } // Simplified
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

            turtle_data.push_str(&format!(
                "{} {} {} .\n",
                subject_str, predicate_str, object_str
            ));
        }

        println!("Generated Turtle data: {}", turtle_data);
        Ok(turtle_data)
    }

    /// Get storage statistics
    pub fn get_storage_stats(&self) -> Result<crate::storage::rdf_store::StorageStats> {
        self.rdf_store.get_storage_stats().map_err(|e| e.into())
    }

    /// Create a backup of the blockchain
    pub fn create_backup(
        &self,
        _backup_id: String,
    ) -> Result<crate::storage::rdf_store::BackupInfo> {
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
            governance: Governance::new(),
        };

        // Initialize ontology manager and SHACL validator
        bc.initialize_ontology_system(ontology_config)?;

        let genesis_block = bc.create_genesis_block();

        // Add genesis block data to RDF store
        if let Ok(graph_name) = NamedNode::new("http://provchain.org/block/0") {
            bc.rdf_store
                .add_rdf_to_graph(&genesis_block.data, &graph_name);
            bc.rdf_store.add_block_metadata(&genesis_block);
        } else {
            eprintln!("Warning: Could not create graph name for genesis block");
        }

        bc.chain.push(genesis_block);
        Ok(bc)
    }

    /// Create a new persistent blockchain with ontology configuration
    pub fn new_persistent_with_ontology<P: AsRef<Path>>(
        data_dir: P,
        ontology_config: OntologyConfig,
    ) -> Result<Self> {
        let rdf_store = RDFStore::new_persistent(data_dir)?;

        let mut bc = Blockchain {
            chain: Vec::new(),
            rdf_store,
            ontology_manager: None,
            shacl_validator: None,
            governance: Governance::new(),
        };

        // Initialize ontology manager and SHACL validator
        bc.initialize_ontology_system(ontology_config)?;

        // Check if we need to create genesis block or load existing chain
        let store_len = bc.rdf_store.store.len().unwrap_or(0);
        if store_len == 0 {
            // Create genesis block for new blockchain
            let mut genesis_block = bc.create_genesis_block();

            // Add genesis block data to RDF store
            if let Ok(graph_name) = NamedNode::new("http://provchain.org/block/0") {
                bc.rdf_store
                    .add_rdf_to_graph(&genesis_block.data, &graph_name);
                bc.rdf_store.add_block_metadata(&genesis_block);
            } else {
                eprintln!(
                    "Warning: Could not create graph name for genesis block in persistent store"
                );
            }

            // Recalculate hash after adding data to RDF store (consistent with new() method)
            genesis_block.hash = genesis_block.calculate_hash_with_store(Some(&bc.rdf_store));
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
                let mut genesis_block = bc.create_genesis_block();

                if let Ok(graph_name) = NamedNode::new("http://provchain.org/block/0") {
                    bc.rdf_store
                        .add_rdf_to_graph(&genesis_block.data, &graph_name);
                    bc.rdf_store.add_block_metadata(&genesis_block);
                } else {
                    eprintln!("Warning: Could not create graph name for fallback genesis block");
                }

                // Recalculate hash after adding data to RDF store (consistent with new() method)
                genesis_block.hash = genesis_block.calculate_hash_with_store(Some(&bc.rdf_store));
                bc.chain.push(genesis_block);
            }
        }

        Ok(bc)
    }

    /// Initialize the ontology management system
    fn initialize_ontology_system(
        &mut self,
        ontology_config: OntologyConfig,
    ) -> std::result::Result<(), ProvChainError> {
        // Create ontology manager
        let ontology_manager = OntologyManager::new(ontology_config.clone()).map_err(|e| {
            ProvChainError::Blockchain(BlockchainError::OntologyInitializationFailed(format!(
                "Failed to initialize ontology manager: {}",
                e
            )))
        })?;

        // Create SHACL validator
        let shacl_validator = ShaclValidator::new(
            &ontology_config.core_shacl_path,
            &ontology_config.domain_shacl_path,
            ontology_config.ontology_hash.clone(),
            None,
        )
        .map_err(|e| {
            ProvChainError::Blockchain(BlockchainError::OntologyInitializationFailed(format!(
                "Failed to initialize SHACL validator: {}",
                e
            )))
        })?;

        self.ontology_manager = Some(ontology_manager);
        self.shacl_validator = Some(shacl_validator);

        println!(
            "Initialized ontology system for domain: {}",
            self.ontology_manager.as_ref().unwrap().get_domain_name()
        );

        Ok(())
    }

    /// Restore blockchain from backup
    pub fn restore_from_backup<P: AsRef<Path>>(backup_path: P, target_dir: P) -> Result<Self> {
        let rdf_store = RDFStore::restore_from_backup(backup_path, target_dir)
            .map_err(ProvChainError::Anyhow)?;

        let mut bc = Blockchain {
            chain: Vec::new(),
            rdf_store,
            ontology_manager: None,
            shacl_validator: None,
            governance: Governance::new(),
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
        let mut block = Block::new(
            0,
            "@prefix ex: <http://example.org/> . ex:genesis ex:type \"Genesis Block\".".into(),
            "0".into(),
            initial_state_root,
            "GENESIS_VALIDATOR".to_string(),
        );
        block.encrypted_data = None;
        block.signature = "GENESIS_SIGNATURE".to_string();
        block
    }

    /// Create a block proposal that can be signed by a validator
    pub fn create_block_proposal(&mut self, data: String, validator: String) -> Result<Block> {
        // Ensure we have at least a genesis block
        if self.chain.is_empty() {
            let mut genesis_block = self.create_genesis_block();
            if let Ok(graph_name) = NamedNode::new("http://provchain.org/block/0") {
                self.rdf_store
                    .add_rdf_to_graph(&genesis_block.data, &graph_name);
                self.rdf_store.add_block_metadata(&genesis_block);
            } else {
                return Err(ProvChainError::Blockchain(
                    BlockchainError::GenesisCreationFailed(
                        "Could not create graph name for genesis block".to_string(),
                    ),
                ));
            }
            // Recalculate hash after adding data to RDF store
            genesis_block.hash = genesis_block.calculate_hash_with_store(Some(&self.rdf_store));
            self.chain.push(genesis_block);
        }

        let previous_block = self.chain.last().unwrap();
        let index = previous_block.index + 1;
        let previous_hash = previous_block.hash.clone();

        // STEP 8: SHACL VALIDATION - Validate transaction data before adding to blockchain
        if let Some(ref shacl_validator) = self.shacl_validator {
            match shacl_validator.validate_transaction(&data) {
                Ok(validation_result) => {
                    if !validation_result.is_valid {
                        let error_msg = format!(
                            "Transaction validation failed for block {}: {} violations found: {}",
                            index,
                            validation_result.violations.len(),
                            validation_result
                                .violations
                                .iter()
                                .map(|r| r.message.clone())
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                        eprintln!("❌ {}", error_msg);
                        return Err(ProvChainError::Blockchain(
                            BlockchainError::ValidationFailed(error_msg),
                        ));
                    }
                    info!("✅ SHACL validation passed for block {}", index);
                }
                Err(validation_error) => {
                    let error_msg = format!(
                        "SHACL Validation Process Error for block {}: {}",
                        index, validation_error
                    );
                    eprintln!("❌ {}", error_msg);
                    return Err(ProvChainError::Blockchain(
                        BlockchainError::ValidationFailed(error_msg),
                    ));
                }
            }
        } else {
            debug!("No SHACL validator configured - transaction added without domain-specific validation");
        }

        // Calculate state root
        let state_root = self.rdf_store.calculate_state_root();

        let mut block = Block::new(index, data, previous_hash, state_root, validator);
        block.encrypted_data = None;

        Ok(block)
    }

    /// Submit a signed block to the blockchain
    pub fn submit_signed_block(&mut self, block: Block) -> Result<()> {
        // Verify signature
        if !self.governance.validator_set.is_empty() {
            if !self.governance.validator_set.contains(&block.validator) {
                return Err(ProvChainError::Blockchain(
                    BlockchainError::ValidationFailed(format!(
                        "Block validator {} is not authorized",
                        block.validator
                    )),
                ));
            }
            // Perform actual Ed25519 signature verification
            // 1. Get validator's public key (from block.validator)
            // 2. Verify block.signature against block.hash using the public key

            let validator_bytes = hex::decode(&block.validator).map_err(|e| {
                ProvChainError::Blockchain(BlockchainError::InvalidBlock(format!(
                    "Invalid validator public key hex: {}",
                    e
                )))
            })?;

            let public_key =
                VerifyingKey::from_bytes(validator_bytes.as_slice().try_into().map_err(|_| {
                    ProvChainError::Blockchain(BlockchainError::InvalidBlock(
                        "Invalid validator public key length".to_string(),
                    ))
                })?)
                .map_err(|e| {
                    ProvChainError::Blockchain(BlockchainError::InvalidBlock(format!(
                        "Invalid validator public key: {}",
                        e
                    )))
                })?;

            let signature_bytes = hex::decode(&block.signature).map_err(|e| {
                ProvChainError::Blockchain(BlockchainError::InvalidBlock(format!(
                    "Invalid signature hex: {}",
                    e
                )))
            })?;

            let signature =
                Signature::from_bytes(signature_bytes.as_slice().try_into().map_err(|_| {
                    ProvChainError::Blockchain(BlockchainError::InvalidBlock(
                        "Invalid signature length".to_string(),
                    ))
                })?);

            public_key
                .verify(block.hash.as_bytes(), &signature)
                .map_err(|e| {
                    ProvChainError::Blockchain(BlockchainError::InvalidBlock(format!(
                        "Signature verification failed: {}",
                        e
                    )))
                })?;

            info!(
                "✅ Block signature verified for validator {}",
                block.validator
            );
        }

        // Add block data to RDF store
        if let Ok(graph_name) =
            NamedNode::new(format!("http://provchain.org/block/{}", block.index))
        {
            self.rdf_store.add_rdf_to_graph(&block.data, &graph_name);
            self.rdf_store.add_block_metadata(&block);
        } else {
            return Err(ProvChainError::Blockchain(
                BlockchainError::BlockAdditionFailed(
                    "Could not create graph name for block".to_string(),
                ),
            ));
        }

        // Recalculate hash after adding data to RDF store to ensure consistency
        // Note: We modify the block here, which invalidates the signature if the hash changes!
        // Ideally, the hash should be stable.
        let mut final_block = block;
        final_block.hash = final_block.calculate_hash_with_store(Some(&self.rdf_store));

        self.chain.push(final_block);

        // Persist changes to disk if using persistent storage
        if let Err(e) = self.rdf_store.save_to_disk() {
            eprintln!("Warning: Failed to persist blockchain to disk: {}", e);
        }

        Ok(())
    }

    /// Legacy add_block for backward compatibility (uses dummy validator)
    pub fn add_block(&mut self, data: String) -> Result<()> {
        let validator = "LEGACY_VALIDATOR".to_string();
        let mut block = self.create_block_proposal(data, validator)?;
        block.signature = "LEGACY_SIGNATURE".to_string();
        self.submit_signed_block(block)
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
        let graph_name = match NamedNode::new(format!("http://provchain.org/block/{}", block.index))
        {
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
                self.rdf_store
                    .load_ontology(&ontology_data, &ontology_graph);
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
        serde_json::to_string_pretty(&self.chain).map_err(ProvChainError::Json)
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
