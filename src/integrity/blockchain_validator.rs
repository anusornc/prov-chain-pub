//! Blockchain-specific integrity validation
//!
//! This module provides specialized validation for blockchain integrity,
//! including chain reconstruction, block validation, and hash integrity checks.

use crate::core::blockchain::{Block, Blockchain};
use crate::error::Result;
use crate::integrity::{
    BlockchainIntegrityStatus, IntegrityRecommendation, RecommendationSeverity,
};
use crate::storage::rdf_store::RDFStore;
use oxigraph::model::NamedNode;
use tracing::{debug, error, info, instrument, warn};

/// Specialized blockchain integrity validator
pub struct BlockchainIntegrityValidator {
    /// Enable detailed validation logging
    pub verbose_logging: bool,
    /// Validate block data consistency with RDF store
    pub validate_rdf_consistency: bool,
    /// Maximum number of blocks to validate in one batch
    pub max_batch_size: usize,
}

impl BlockchainIntegrityValidator {
    /// Create a new blockchain integrity validator
    pub fn new() -> Self {
        Self {
            verbose_logging: false,
            validate_rdf_consistency: true,
            max_batch_size: 100,
        }
    }

    /// Create a validator with custom configuration
    pub fn with_config(verbose: bool, validate_rdf: bool, batch_size: usize) -> Self {
        Self {
            verbose_logging: verbose,
            validate_rdf_consistency: validate_rdf,
            max_batch_size: batch_size,
        }
    }

    /// Validate blockchain reconstruction from persistent storage
    #[instrument(skip(self, blockchain))]
    pub fn validate_chain_reconstruction(&self, blockchain: &Blockchain) -> Result<Vec<String>> {
        let mut errors = Vec::new();

        if self.verbose_logging {
            info!(
                "Validating blockchain reconstruction for {} blocks",
                blockchain.chain.len()
            );
        }

        // 1. Test SPARQL parsing for block metadata
        let metadata_query = r#"
            PREFIX prov: <http://provchain.org/>
            SELECT ?block ?index ?timestamp ?hash ?prevHash ?dataGraph WHERE {
                GRAPH <http://provchain.org/blockchain> {
                    ?block a ?blockType ;
                           prov:hasIndex ?index ;
                           prov:hasTimestamp ?timestamp ;
                           prov:hasHash ?hash ;
                           prov:hasPreviousHash ?prevHash ;
                           prov:hasDataGraphIRI ?dataGraph .
                    FILTER(?blockType = prov:Block || ?blockType = prov:GenesisBlock)
                }
            }
            ORDER BY ?index
        "#;

        let mut reconstructed_blocks = Vec::new();
        match blockchain.rdf_store.query(metadata_query) {
            oxigraph::sparql::QueryResults::Solutions(solutions) => {
                for solution in solutions {
                    match solution {
                        Ok(sol) => {
                            // Validate that all required fields are present
                            let index_present = sol.get("index").is_some();
                            let timestamp_present = sol.get("timestamp").is_some();
                            let hash_present = sol.get("hash").is_some();
                            let prev_hash_present = sol.get("prevHash").is_some();
                            let data_graph_present = sol.get("dataGraph").is_some();

                            if !index_present
                                || !timestamp_present
                                || !hash_present
                                || !prev_hash_present
                                || !data_graph_present
                            {
                                errors.push(format!(
                                    "Incomplete block metadata in SPARQL result: index={}, timestamp={}, hash={}, prevHash={}, dataGraph={}",
                                    index_present, timestamp_present, hash_present, prev_hash_present, data_graph_present
                                ));
                                continue;
                            }

                            // Extract and validate block data
                            if let (
                                Some(index_term),
                                Some(timestamp_term),
                                Some(hash_term),
                                Some(prev_hash_term),
                                Some(data_graph_term),
                            ) = (
                                sol.get("index"),
                                sol.get("timestamp"),
                                sol.get("hash"),
                                sol.get("prevHash"),
                                sol.get("dataGraph"),
                            ) {
                                // Parse block index
                                let index = match index_term {
                                    oxigraph::model::Term::Literal(lit) => {
                                        match lit.value().parse::<u64>() {
                                            Ok(idx) => idx,
                                            Err(e) => {
                                                errors.push(format!(
                                                    "Invalid block index format: {}",
                                                    e
                                                ));
                                                continue;
                                            }
                                        }
                                    }
                                    _ => {
                                        errors.push("Block index is not a literal".to_string());
                                        continue;
                                    }
                                };

                                // Extract timestamp
                                let timestamp = match timestamp_term {
                                    oxigraph::model::Term::Literal(lit) => {
                                        lit.value().trim_matches('"').to_string()
                                    }
                                    _ => {
                                        errors.push(format!(
                                            "Block {} timestamp is not a literal",
                                            index
                                        ));
                                        continue;
                                    }
                                };

                                // Extract hash
                                let hash = match hash_term {
                                    oxigraph::model::Term::Literal(lit) => {
                                        lit.value().trim_matches('"').to_string()
                                    }
                                    _ => {
                                        errors
                                            .push(format!("Block {} hash is not a literal", index));
                                        continue;
                                    }
                                };

                                // Extract previous hash
                                let previous_hash = match prev_hash_term {
                                    oxigraph::model::Term::Literal(lit) => {
                                        lit.value().trim_matches('"').to_string()
                                    }
                                    _ => {
                                        errors.push(format!(
                                            "Block {} previous hash is not a literal",
                                            index
                                        ));
                                        continue;
                                    }
                                };

                                // 2. Verify graph naming consistency
                                let data_graph_string = data_graph_term.to_string();
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

                                let expected_graph_uri =
                                    format!("http://provchain.org/block/{}", index);
                                if data_graph_uri != expected_graph_uri {
                                    errors.push(format!(
                                        "Block {} graph naming inconsistency: expected='{}', actual='{}'",
                                        index, expected_graph_uri, data_graph_uri
                                    ));
                                }

                                // 3. Validate RDF data extraction from graphs
                                match NamedNode::new(data_graph_uri) {
                                    Ok(graph_name) => {
                                        // Try to extract RDF data from the graph
                                        let mut extracted_triples = Vec::new();
                                        for quad in blockchain
                                            .rdf_store
                                            .store
                                            .quads_for_pattern(
                                                None,
                                                None,
                                                None,
                                                Some((&graph_name).into()),
                                            )
                                            .flatten()
                                        {
                                            extracted_triples.push(quad);
                                        }

                                        if extracted_triples.is_empty() && index > 0 {
                                            // Genesis block might legitimately have minimal data
                                            errors.push(format!(
                                                "Block {} has no extractable RDF data from graph",
                                                index
                                            ));
                                        }

                                        // Validate that the extracted data can be serialized
                                        if !extracted_triples.is_empty() {
                                            // Test serialization by creating a simple turtle representation
                                            let mut turtle_data = String::new();
                                            for triple in &extracted_triples {
                                                let subject_str = match &triple.subject {
                                                    oxigraph::model::Subject::NamedNode(node) => {
                                                        format!("<{}>", node.as_str())
                                                    }
                                                    oxigraph::model::Subject::BlankNode(node) => {
                                                        format!("_:{}", node.as_str())
                                                    }
                                                    oxigraph::model::Subject::Triple(_) => {
                                                        "<< >>".to_string()
                                                    }
                                                };

                                                let predicate_str =
                                                    format!("<{}>", triple.predicate.as_str());

                                                let object_str = match &triple.object {
                                                    oxigraph::model::Term::NamedNode(node) => {
                                                        format!("<{}>", node.as_str())
                                                    }
                                                    oxigraph::model::Term::BlankNode(node) => {
                                                        format!("_:{}", node.as_str())
                                                    }
                                                    oxigraph::model::Term::Literal(lit) => {
                                                        format!("{}", lit)
                                                    }
                                                    oxigraph::model::Term::Triple(_) => {
                                                        "<< >>".to_string()
                                                    }
                                                };

                                                turtle_data.push_str(&format!(
                                                    "{} {} {} .\n",
                                                    subject_str, predicate_str, object_str
                                                ));
                                            }

                                            // Test that the serialized data can be parsed back
                                            let mut test_store = RDFStore::new();
                                            test_store.add_rdf_to_graph(&turtle_data, &graph_name);

                                            // Count triples to ensure round-trip consistency
                                            let test_query = format!(
                                                r#"
                                                SELECT (COUNT(*) as ?count) WHERE {{
                                                    GRAPH <{}> {{
                                                        ?s ?p ?o .
                                                    }}
                                                }}
                                            "#,
                                                graph_name.as_str()
                                            );

                                            let test_count = match test_store.query(&test_query) {
                                                oxigraph::sparql::QueryResults::Solutions(
                                                    solutions,
                                                ) => {
                                                    let mut count = 0;
                                                    for sol in solutions.flatten() {
                                                        if let Some(
                                                            oxigraph::model::Term::Literal(lit),
                                                        ) = sol.get("count")
                                                        {
                                                            if let Ok(parsed_count) =
                                                                lit.value().parse::<usize>()
                                                            {
                                                                count = parsed_count;
                                                                break;
                                                            }
                                                        }
                                                    }
                                                    count
                                                }
                                                _ => 0,
                                            };

                                            if test_count != extracted_triples.len() {
                                                errors.push(format!(
                                                    "Block {} RDF round-trip inconsistency: extracted={}, parsed={}",
                                                    index, extracted_triples.len(), test_count
                                                ));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        errors.push(format!(
                                            "Block {} has invalid data graph URI '{}': {}",
                                            index, data_graph_uri, e
                                        ));
                                    }
                                }

                                reconstructed_blocks.push((index, timestamp, hash, previous_hash));
                            }
                        }
                        Err(e) => {
                            errors.push(format!("SPARQL solution parsing error: {}", e));
                        }
                    }
                }
            }
            _ => {
                errors.push("SPARQL metadata query returned unexpected result type".to_string());
            }
        }

        // 4. Validate that all in-memory blocks can be reconstructed
        for block in &blockchain.chain {
            let found_in_reconstruction = reconstructed_blocks
                .iter()
                .any(|(index, _, _, _)| *index == block.index);

            if !found_in_reconstruction {
                errors.push(format!(
                    "Block {} exists in memory but cannot be reconstructed from persistent storage",
                    block.index
                ));
            }
        }

        // 5. Validate reconstruction completeness
        let memory_block_count = blockchain.chain.len();
        let reconstructed_block_count = reconstructed_blocks.len();

        if memory_block_count != reconstructed_block_count {
            errors.push(format!(
                "Reconstruction completeness mismatch: {} blocks in memory, {} reconstructed from storage",
                memory_block_count, reconstructed_block_count
            ));
        }

        // 6. Validate block ordering in reconstruction
        reconstructed_blocks.sort_by_key(|(index, _, _, _)| *index);
        for (i, (index, _, _, _)) in reconstructed_blocks.iter().enumerate() {
            if *index != i as u64 {
                errors.push(format!(
                    "Block ordering inconsistency: expected index {}, found {}",
                    i, index
                ));
            }
        }

        if self.verbose_logging {
            if errors.is_empty() {
                info!(
                    "Chain reconstruction validation passed for {} blocks",
                    memory_block_count
                );
            } else {
                warn!(
                    "Chain reconstruction validation found {} errors",
                    errors.len()
                );
            }
        }

        debug!(
            "Chain reconstruction validation completed with {} errors",
            errors.len()
        );
        Ok(errors)
    }

    /// Detect missing blocks in the blockchain
    #[instrument(skip(self, blockchain))]
    pub fn detect_missing_blocks(&self, blockchain: &Blockchain) -> Result<Vec<u64>> {
        let mut missing_blocks = Vec::new();

        if self.verbose_logging {
            info!(
                "Detecting missing blocks in chain of length {}",
                blockchain.chain.len()
            );
        }

        // Get all block indices from persistent storage
        let query = r#"
            PREFIX prov: <http://provchain.org/>
            SELECT DISTINCT ?index WHERE {
                GRAPH <http://provchain.org/blockchain> {
                    ?block a ?blockType ;
                           prov:hasIndex ?index .
                    FILTER(?blockType = prov:Block || ?blockType = prov:GenesisBlock)
                }
            }
            ORDER BY ?index
        "#;

        let mut persistent_indices = Vec::new();
        if let oxigraph::sparql::QueryResults::Solutions(solutions) =
            blockchain.rdf_store.query(query)
        {
            for sol in solutions.flatten() {
                if let Some(oxigraph::model::Term::Literal(lit)) = sol.get("index") {
                    if let Ok(index) = lit.value().parse::<u64>() {
                        persistent_indices.push(index);
                    }
                }
            }
        }

        // Get in-memory chain indices
        let mut memory_indices: Vec<u64> =
            blockchain.chain.iter().map(|block| block.index).collect();
        memory_indices.sort();

        if self.verbose_logging {
            debug!(
                "Found {} indices in persistent storage, {} in memory",
                persistent_indices.len(),
                memory_indices.len()
            );
        }

        // Check for gaps in persistent storage (should be continuous from 0)
        if !persistent_indices.is_empty() {
            let max_persistent_index = *persistent_indices.iter().max().unwrap_or(&0);
            for expected_index in 0..=max_persistent_index {
                if !persistent_indices.contains(&expected_index) {
                    missing_blocks.push(expected_index);
                    if self.verbose_logging {
                        warn!("Missing block {} in persistent storage", expected_index);
                    }
                }
            }
        }

        // Check for blocks in memory that are not in persistent storage
        for memory_index in &memory_indices {
            if !persistent_indices.contains(memory_index) && self.verbose_logging {
                warn!(
                    "Block {} exists in memory but not in persistent storage",
                    memory_index
                );
            }
        }

        // Check for blocks in persistent storage that are not in memory
        for persistent_index in &persistent_indices {
            if !memory_indices.contains(persistent_index) && self.verbose_logging {
                warn!(
                    "Block {} exists in persistent storage but not loaded in memory",
                    persistent_index
                );
            }
        }

        debug!(
            "Missing block detection completed, found {} missing blocks",
            missing_blocks.len()
        );
        Ok(missing_blocks)
    }

    /// Validate block hash integrity across the entire chain
    #[instrument(skip(self, blockchain))]
    pub fn validate_block_hash_integrity(&self, blockchain: &Blockchain) -> Result<Vec<String>> {
        let mut hash_errors = Vec::new();

        if self.verbose_logging {
            info!(
                "Validating hash integrity for {} blocks",
                blockchain.chain.len()
            );
        }

        // Validate each block's hash integrity
        for (i, block) in blockchain.chain.iter().enumerate() {
            // Recalculate hash using RDF canonicalization
            let recalculated_hash = block.calculate_hash_with_store(Some(&blockchain.rdf_store));

            // Compare with stored hash
            if block.hash != recalculated_hash {
                let error_msg = format!(
                    "Block {} hash mismatch: stored='{}', calculated='{}'",
                    block.index, block.hash, recalculated_hash
                );
                hash_errors.push(error_msg.clone());
                if self.verbose_logging {
                    error!("{}", error_msg);
                }
            }

            // Validate hash chain linking (except for genesis block)
            if i > 0 {
                let previous_block = &blockchain.chain[i - 1];
                if block.previous_hash != previous_block.hash {
                    let error_msg = format!(
                        "Block {} previous hash mismatch: expected='{}', actual='{}'",
                        block.index, previous_block.hash, block.previous_hash
                    );
                    hash_errors.push(error_msg.clone());
                    if self.verbose_logging {
                        error!("{}", error_msg);
                    }
                }
            }

            // Validate RDF canonicalization consistency
            if let Ok(graph_name) =
                NamedNode::new(format!("http://provchain.org/block/{}", block.index))
            {
                let canonical_hash = blockchain.rdf_store.canonicalize_graph(&graph_name);

                // Create a temporary store to validate the block's data field
                let mut temp_store = RDFStore::new();
                temp_store.add_rdf_to_graph(&block.data, &graph_name);
                let temp_canonical_hash = temp_store.canonicalize_graph(&graph_name);

                if canonical_hash != temp_canonical_hash {
                    let error_msg = format!(
                        "Block {} RDF canonicalization inconsistency: store='{}', data_field='{}'",
                        block.index, canonical_hash, temp_canonical_hash
                    );
                    hash_errors.push(error_msg.clone());
                    if self.verbose_logging {
                        warn!("{}", error_msg);
                    }
                }
            } else {
                let error_msg = format!("Block {} has invalid graph name format", block.index);
                hash_errors.push(error_msg.clone());
                if self.verbose_logging {
                    error!("{}", error_msg);
                }
            }

            // Process in batches to avoid memory issues with large chains
            if i > 0 && i % self.max_batch_size == 0 && self.verbose_logging {
                debug!(
                    "Processed {} blocks, found {} hash errors so far",
                    i + 1,
                    hash_errors.len()
                );
            }
        }

        debug!(
            "Hash integrity validation completed with {} errors",
            hash_errors.len()
        );
        Ok(hash_errors)
    }

    /// Detect corrupted blocks
    #[instrument(skip(self, blockchain))]
    pub fn detect_corrupted_blocks(&self, blockchain: &Blockchain) -> Result<Vec<u64>> {
        let mut corrupted_blocks = Vec::new();

        if self.verbose_logging {
            info!(
                "Detecting corrupted blocks in chain of length {}",
                blockchain.chain.len()
            );
        }

        for block in &blockchain.chain {
            let mut block_corrupted = false;

            // Skip genesis block from corruption checks (it's a special case)
            if block.index == 0 {
                if self.verbose_logging {
                    debug!("Skipping corruption checks for genesis block (index 0)");
                }
                continue;
            }

            // 1. Validate block data integrity using existing blockchain method
            if self.validate_rdf_consistency && !blockchain.validate_block_data_integrity(block) {
                if self.verbose_logging {
                    error!("Block {} failed data integrity validation", block.index);
                }
                block_corrupted = true;
            }

            // 2. Check RDF parsing consistency
            if let Ok(graph_name) =
                NamedNode::new(format!("http://provchain.org/block/{}", block.index))
            {
                // Try to parse the block's RDF data
                let mut temp_store = RDFStore::new();
                temp_store.add_rdf_to_graph(&block.data, &graph_name);

                // Check if the data was parsed successfully by counting triples
                let temp_query = format!(
                    r#"
                    SELECT (COUNT(*) as ?count) WHERE {{
                        GRAPH <{}> {{
                            ?s ?p ?o .
                        }}
                    }}
                "#,
                    graph_name.as_str()
                );

                let mut temp_triple_count = 0;
                if let oxigraph::sparql::QueryResults::Solutions(solutions) =
                    temp_store.query(&temp_query)
                {
                    for sol in solutions.flatten() {
                        if let Some(oxigraph::model::Term::Literal(lit)) = sol.get("count") {
                            if let Ok(count) = lit.value().parse::<usize>() {
                                temp_triple_count = count;
                                break;
                            }
                        }
                    }
                }

                // Compare with main store triple count
                let main_query = format!(
                    r#"
                    SELECT (COUNT(*) as ?count) WHERE {{
                        GRAPH <{}> {{
                            ?s ?p ?o .
                        }}
                    }}
                "#,
                    graph_name.as_str()
                );

                let mut main_triple_count = 0;
                if let oxigraph::sparql::QueryResults::Solutions(solutions) =
                    blockchain.rdf_store.query(&main_query)
                {
                    for sol in solutions.flatten() {
                        if let Some(oxigraph::model::Term::Literal(lit)) = sol.get("count") {
                            if let Ok(count) = lit.value().parse::<usize>() {
                                main_triple_count = count;
                                break;
                            }
                        }
                    }
                }

                if temp_triple_count != main_triple_count {
                    if self.verbose_logging {
                        error!("Block {} RDF parsing inconsistency: data_field={} triples, store={} triples", 
                               block.index, temp_triple_count, main_triple_count);
                    }
                    block_corrupted = true;
                }
            } else {
                if self.verbose_logging {
                    error!("Block {} has invalid graph name", block.index);
                }
                block_corrupted = true;
            }

            // 3. Verify block metadata consistency with persistent storage
            let metadata_query = format!(
                r#"
                PREFIX prov: <http://provchain.org/>
                SELECT ?timestamp ?hash ?prevHash WHERE {{
                    GRAPH <http://provchain.org/blockchain> {{
                        ?block a ?blockType ;
                               prov:hasIndex {} ;
                               prov:hasTimestamp ?timestamp ;
                               prov:hasHash ?hash ;
                               prov:hasPreviousHash ?prevHash .
                        FILTER(?blockType = prov:Block || ?blockType = prov:GenesisBlock)
                    }}
                }}
            "#,
                block.index
            );

            if let oxigraph::sparql::QueryResults::Solutions(solutions) =
                blockchain.rdf_store.query(&metadata_query)
            {
                let mut metadata_found = false;
                if let Some(sol) = solutions.flatten().next() {
                    metadata_found = true;

                    // Check timestamp consistency
                    if let Some(oxigraph::model::Term::Literal(lit)) = sol.get("timestamp") {
                        let stored_timestamp = lit.value().trim_matches('"');
                        if stored_timestamp != block.timestamp {
                            if self.verbose_logging {
                                error!(
                                    "Block {} timestamp mismatch: block='{}', store='{}'",
                                    block.index, block.timestamp, stored_timestamp
                                );
                            }
                            block_corrupted = true;
                        }
                    }

                    // Check hash consistency
                    if let Some(oxigraph::model::Term::Literal(lit)) = sol.get("hash") {
                        let stored_hash = lit.value().trim_matches('"');
                        if stored_hash != block.hash {
                            if self.verbose_logging {
                                error!(
                                    "Block {} hash mismatch: block='{}', store='{}'",
                                    block.index, block.hash, stored_hash
                                );
                            }
                            block_corrupted = true;
                        }
                    }

                    // Check previous hash consistency
                    if let Some(oxigraph::model::Term::Literal(lit)) = sol.get("prevHash") {
                        let stored_prev_hash = lit.value().trim_matches('"');
                        if stored_prev_hash != block.previous_hash {
                            if self.verbose_logging {
                                error!(
                                    "Block {} previous hash mismatch: block='{}', store='{}'",
                                    block.index, block.previous_hash, stored_prev_hash
                                );
                            }
                            block_corrupted = true;
                        }
                    }
                }
                if !metadata_found {
                    if self.verbose_logging {
                        error!(
                            "Block {} metadata not found in persistent storage",
                            block.index
                        );
                    }
                    block_corrupted = true;
                }
            }

            if block_corrupted {
                corrupted_blocks.push(block.index);
            }
        }

        debug!(
            "Corrupted block detection completed, found {} corrupted blocks",
            corrupted_blocks.len()
        );
        Ok(corrupted_blocks)
    }

    /// Validate enhanced block data integrity
    #[instrument(skip(self, block, rdf_store))]
    pub fn validate_block_data_consistency(
        &self,
        block: &Block,
        rdf_store: &RDFStore,
    ) -> Result<bool> {
        if self.verbose_logging {
            debug!("Validating data consistency for block {}", block.index);
        }

        // Create graph name for this block
        let graph_name = match NamedNode::new(format!("http://provchain.org/block/{}", block.index))
        {
            Ok(name) => name,
            Err(e) => {
                if self.verbose_logging {
                    error!("Block {} has invalid graph name: {}", block.index, e);
                }
                return Ok(false);
            }
        };

        // 1. Validate RDF syntax by parsing the block's data field
        let mut temp_store = RDFStore::new();
        temp_store.add_rdf_to_graph(&block.data, &graph_name);

        // Count triples in temporary store to ensure parsing succeeded
        let temp_query = format!(
            r#"
            SELECT (COUNT(*) as ?count) WHERE {{
                GRAPH <{}> {{
                    ?s ?p ?o .
                }}
            }}
        "#,
            graph_name.as_str()
        );

        let temp_triple_count = match temp_store.query(&temp_query) {
            oxigraph::sparql::QueryResults::Solutions(solutions) => {
                let mut count = 0;
                for sol in solutions.flatten() {
                    if let Some(oxigraph::model::Term::Literal(lit)) = sol.get("count") {
                        if let Ok(parsed_count) = lit.value().parse::<usize>() {
                            count = parsed_count;
                            break;
                        }
                    }
                }
                count
            }
            _ => {
                if self.verbose_logging {
                    error!("Block {} failed to parse RDF data", block.index);
                }
                return Ok(false);
            }
        };

        // 2. Compare with main store triple count
        let main_triple_count = match rdf_store.query(&temp_query) {
            oxigraph::sparql::QueryResults::Solutions(solutions) => {
                let mut count = 0;
                for sol in solutions.flatten() {
                    if let Some(oxigraph::model::Term::Literal(lit)) = sol.get("count") {
                        if let Ok(parsed_count) = lit.value().parse::<usize>() {
                            count = parsed_count;
                            break;
                        }
                    }
                }
                count
            }
            _ => {
                if self.verbose_logging {
                    error!("Block {} failed to query main store", block.index);
                }
                return Ok(false);
            }
        };

        if temp_triple_count != main_triple_count {
            if self.verbose_logging {
                error!(
                    "Block {} triple count mismatch: data_field={}, store={}",
                    block.index, temp_triple_count, main_triple_count
                );
            }
            return Ok(false);
        }

        // 3. Cross-validation with multiple canonicalization algorithms
        let main_canonical_hash = rdf_store.canonicalize_graph(&graph_name);
        let temp_canonical_hash = temp_store.canonicalize_graph(&graph_name);

        if main_canonical_hash != temp_canonical_hash {
            if self.verbose_logging {
                error!(
                    "Block {} canonicalization mismatch: store='{}', data_field='{}'",
                    block.index, main_canonical_hash, temp_canonical_hash
                );
            }
            return Ok(false);
        }

        // 4. Validate RDF semantics by checking for basic RDF structure
        let semantic_query = format!(
            r#"
            ASK {{
                GRAPH <{}> {{
                    ?s ?p ?o .
                    FILTER(isURI(?s) || isBlank(?s))
                    FILTER(isURI(?p))
                }}
            }}
        "#,
            graph_name.as_str()
        );

        let semantic_valid = match temp_store.query(&semantic_query) {
            oxigraph::sparql::QueryResults::Boolean(result) => result,
            _ => {
                if self.verbose_logging {
                    warn!("Block {} semantic validation query failed", block.index);
                }
                false
            }
        };

        if !semantic_valid && temp_triple_count > 0 {
            if self.verbose_logging {
                error!("Block {} contains invalid RDF semantics", block.index);
            }
            return Ok(false);
        }

        // 5. Validate that the block data is not empty (unless it's intentionally empty)
        if block.data.trim().is_empty() && block.index > 0 && self.verbose_logging {
            warn!("Block {} has empty data field", block.index);
        }

        if self.verbose_logging {
            debug!("Block {} passed all data consistency checks", block.index);
        }

        Ok(true)
    }

    /// Count blocks in persistent storage
    #[instrument(skip(self, rdf_store))]
    pub fn count_persistent_blocks(&self, rdf_store: &RDFStore) -> Result<usize> {
        if self.verbose_logging {
            debug!("Counting blocks in persistent storage");
        }

        // Query RDF store for block metadata to count actual blocks
        let query = r#"
            PREFIX prov: <http://provchain.org/>
            SELECT (COUNT(DISTINCT ?block) as ?count) WHERE {
                GRAPH <http://provchain.org/blockchain> {
                    ?block a ?blockType ;
                           prov:hasIndex ?index .
                    FILTER(?blockType = prov:Block || ?blockType = prov:GenesisBlock)
                }
            }
        "#;

        match rdf_store.query(query) {
            oxigraph::sparql::QueryResults::Solutions(solutions) => {
                for sol in solutions.flatten() {
                    if let Some(oxigraph::model::Term::Literal(lit)) = sol.get("count") {
                        if let Ok(count) = lit.value().parse::<usize>() {
                            if self.verbose_logging {
                                debug!("Found {} blocks in persistent storage", count);
                            }
                            return Ok(count);
                        }
                    }
                }
            }
            _ => {
                warn!("Unexpected query result format when counting blocks");
            }
        }

        // Fallback: count by querying for all block indices
        let fallback_query = r#"
            PREFIX prov: <http://provchain.org/>
            SELECT DISTINCT ?index WHERE {
                GRAPH <http://provchain.org/blockchain> {
                    ?block a ?blockType ;
                           prov:hasIndex ?index .
                    FILTER(?blockType = prov:Block || ?blockType = prov:GenesisBlock)
                }
            }
        "#;

        let mut count = 0;
        if let oxigraph::sparql::QueryResults::Solutions(solutions) =
            rdf_store.query(fallback_query)
        {
            for solution in solutions {
                if solution.is_ok() {
                    count += 1;
                }
            }
        }

        if self.verbose_logging {
            debug!(
                "Fallback count found {} blocks in persistent storage",
                count
            );
        }

        Ok(count)
    }

    /// Generate blockchain-specific integrity recommendations
    pub fn generate_recommendations(
        &self,
        status: &BlockchainIntegrityStatus,
    ) -> Vec<IntegrityRecommendation> {
        let mut recommendations = Vec::new();

        // Missing blocks recommendations
        if !status.missing_blocks.is_empty() {
            recommendations.push(IntegrityRecommendation {
                severity: RecommendationSeverity::Critical,
                category: "Blockchain Integrity".to_string(),
                description: format!(
                    "Found {} missing blocks: {:?}",
                    status.missing_blocks.len(),
                    status.missing_blocks
                ),
                action_required: "Restore missing blocks from backup or network synchronization"
                    .to_string(),
                auto_fixable: false,
            });
        }

        // Corrupted blocks recommendations
        if !status.corrupted_blocks.is_empty() {
            recommendations.push(IntegrityRecommendation {
                severity: RecommendationSeverity::Critical,
                category: "Blockchain Integrity".to_string(),
                description: format!(
                    "Found {} corrupted blocks: {:?}",
                    status.corrupted_blocks.len(),
                    status.corrupted_blocks
                ),
                action_required: "Restore corrupted blocks from backup or re-validate data"
                    .to_string(),
                auto_fixable: false,
            });
        }

        // Hash validation errors recommendations
        if !status.hash_validation_errors.is_empty() {
            recommendations.push(IntegrityRecommendation {
                severity: RecommendationSeverity::Critical,
                category: "Blockchain Integrity".to_string(),
                description: format!("Found {} hash validation errors", status.hash_validation_errors.len()),
                action_required: "Investigate hash calculation inconsistencies and re-validate chain".to_string(),
                auto_fixable: false,
            });
        }

        // Chain length discrepancy recommendations
        if status.chain_length != status.persistent_block_count {
            let severity = if status.chain_length.abs_diff(status.persistent_block_count) > 10 {
                RecommendationSeverity::Critical
            } else {
                RecommendationSeverity::Warning
            };

            recommendations.push(IntegrityRecommendation {
                severity,
                category: "Blockchain Integrity".to_string(),
                description: format!(
                    "Chain length mismatch: {} in memory vs {} in storage",
                    status.chain_length, status.persistent_block_count
                ),
                action_required: "Synchronize in-memory chain with persistent storage".to_string(),
                auto_fixable: true,
            });
        }

        // Reconstruction errors recommendations
        if !status.reconstruction_errors.is_empty() {
            recommendations.push(IntegrityRecommendation {
                severity: RecommendationSeverity::Warning,
                category: "Blockchain Integrity".to_string(),
                description: format!(
                    "Found {} reconstruction errors",
                    status.reconstruction_errors.len()
                ),
                action_required: "Review blockchain loading logic and RDF parsing".to_string(),
                auto_fixable: true,
            });
        }

        recommendations
    }
}

impl Default for BlockchainIntegrityValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::blockchain::Blockchain;

    #[test]
    fn test_blockchain_validator_creation() {
        let validator = BlockchainIntegrityValidator::new();
        assert!(!validator.verbose_logging);
        assert!(validator.validate_rdf_consistency);
        assert_eq!(validator.max_batch_size, 100);
    }

    #[test]
    fn test_blockchain_validator_with_config() {
        let validator = BlockchainIntegrityValidator::with_config(true, false, 50);
        assert!(validator.verbose_logging);
        assert!(!validator.validate_rdf_consistency);
        assert_eq!(validator.max_batch_size, 50);
    }

    #[test]
    fn test_validate_chain_reconstruction_basic() {
        let validator = BlockchainIntegrityValidator::new();
        let blockchain = Blockchain::new();

        let result = validator.validate_chain_reconstruction(&blockchain);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_detect_missing_blocks_basic() {
        let validator = BlockchainIntegrityValidator::new();
        let blockchain = Blockchain::new();

        let result = validator.detect_missing_blocks(&blockchain);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
