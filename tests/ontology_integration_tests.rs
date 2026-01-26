use oxigraph::model::NamedNode;
use provchain_org::core::blockchain::Blockchain;
use provchain_org::storage::rdf_store::RDFStore;

#[test]
fn test_ontology_loading() {
    let bc = Blockchain::new();

    // Check that ontology classes are loaded
    let classes = bc.rdf_store.get_ontology_classes();
    println!("Loaded ontology classes: {classes:?}");

    // Should contain our main ontology classes
    let class_strings = classes.join(" ");
    assert!(class_strings.contains("Batch"));
    assert!(class_strings.contains("ManufacturingProcess"));
    assert!(class_strings.contains("Supplier"));
    assert!(class_strings.contains("Manufacturer"));
}

#[test]
fn test_ontology_validation() {
    let mut bc = Blockchain::new();

    // Add valid ontology-based data
    let valid_data = r#"
        @prefix ex: <http://example.org/> .
        @prefix core: <http://provchain.org/core#> .
        @prefix prov: <http://www.w3.org/ns/prov#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        ex:testBatch a core:Batch ;
            core:hasIdentifier "TEST001" ;
            core:producedAt "2025-08-08T10:00:00Z"^^xsd:dateTime .

        ex:testActivity a core:ManufacturingProcess ;
            core:recordedAt "2025-08-08T12:00:00Z"^^xsd:dateTime ;
            prov:wasAssociatedWith ex:testAgent .

        ex:testAgent a core:Manufacturer ;
            rdfs:label "Test Manufacturer" .
    "#;

    let _ = bc.add_block(valid_data.into());

    // Validate the last block's data
    let last_block_index = bc.chain.len() - 1;
    let graph_name =
        NamedNode::new(format!("http://provchain.org/block/{last_block_index}")).unwrap();

    // Test ontology validation
    let is_valid = bc.rdf_store.validate_against_ontology(&graph_name);
    assert!(is_valid, "Valid ontology data should pass validation");

    // Test required properties validation
    let validation_errors = bc.rdf_store.validate_required_properties(&graph_name);
    assert!(
        validation_errors.is_empty(),
        "Valid data should have no validation errors: {validation_errors:?}"
    );
}

#[test]
fn test_ontology_validation_failures() {
    let mut rdf_store = RDFStore::new();

    // Load ontology first
    if let Ok(ontology_data) = std::fs::read_to_string("src/semantic/ontologies/generic_core.owl") {
        let ontology_graph = NamedNode::new("http://provchain.org/ontology").unwrap();
        rdf_store.load_ontology(&ontology_data, &ontology_graph);
    }

    // Add invalid data (Batch without required hasIdentifier)
    let invalid_data = r#"
        @prefix ex: <http://example.org/> .
        @prefix core: <http://provchain.org/core#> .
        @prefix prov: <http://www.w3.org/ns/prov#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

        ex:invalidBatch a core:Batch ;
            core:producedAt "2025-08-08T10:00:00Z"^^xsd:dateTime .

        ex:invalidActivity a core:ManufacturingProcess ;
            prov:wasAssociatedWith ex:testAgent .
    "#;

    let graph_name = NamedNode::new("http://provchain.org/test").unwrap();
    rdf_store.add_rdf_to_graph(invalid_data, &graph_name);

    // Test validation should find errors
    let validation_errors = rdf_store.validate_required_properties(&graph_name);
    assert!(
        !validation_errors.is_empty(),
        "Invalid data should have validation errors"
    );

    // Should find missing hasIdentifier and recordedAt
    let error_text = validation_errors.join(" ");
    assert!(
        error_text.contains("hasIdentifier"),
        "Should detect missing hasIdentifier"
    );
    assert!(
        error_text.contains("recordedAt"),
        "Should detect missing recordedAt"
    );
}

#[test]
fn test_environmental_conditions_integration() {
    let mut bc = Blockchain::new();

    // Add data with environmental conditions
    let env_data = r#"
        @prefix ex: <http://example.org/> .
        @prefix core: <http://provchain.org/core#> .
        @prefix prov: <http://www.w3.org/ns/prov#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

        ex:coldTransport a core:TransportProcess ;
            core:recordedAt "2025-08-08T14:00:00Z"^^xsd:dateTime ;
            core:hasCondition ex:coldCondition .

        ex:coldCondition a core:EnvironmentalCondition ;
            core:hasTemperature "2.5"^^xsd:decimal ;
            core:hasHumidity "70.0"^^xsd:decimal ;
            core:hasConditionTimestamp "2025-08-08T14:00:00Z"^^xsd:dateTime .
    "#;

    let _ = bc.add_block(env_data.into());

    // Query for environmental conditions across all graphs
    let env_query = r#"
        PREFIX core: <http://provchain.org/core#>
        
        SELECT ?activity ?temp ?humidity ?graph WHERE {
            GRAPH ?graph {
                ?activity a core:TransportProcess ;
                          core:hasCondition ?condition .
                ?condition core:hasTemperature ?temp ;
                           core:hasHumidity ?humidity .
            }
        }
    "#;

    if let oxigraph::sparql::QueryResults::Solutions(solutions) = bc.rdf_store.query(env_query) {
        let mut found_conditions = false;
        for solution in solutions {
            if let Ok(sol) = solution {
                if let (Some(temp), Some(humidity)) = (sol.get("temp"), sol.get("humidity")) {
                    println!("Found environmental conditions: temp={temp}, humidity={humidity}");
                    found_conditions = true;
                }
            }
        }
        assert!(
            found_conditions,
            "Should find environmental conditions in the data"
        );
    } else {
        // Fallback: check if the data was stored at all
        let simple_query = r#"
            PREFIX core: <http://provchain.org/core#>
            
            SELECT ?s ?p ?o WHERE {
                GRAPH ?g {
                    ?s ?p ?o .
                    FILTER(?p = trace:hasTemperature || ?p = trace:hasHumidity)
                }
            }
        "#;

        if let oxigraph::sparql::QueryResults::Solutions(solutions) =
            bc.rdf_store.query(simple_query)
        {
            let mut count = 0;
            for solution in solutions {
                if let Ok(_sol) = solution {
                    count += 1;
                }
            }
            assert!(
                count > 0,
                "Should find at least some environmental data properties"
            );
        }
    }
}

#[test]
fn test_supply_chain_traceability() {
    let mut bc = Blockchain::new();

    // Add complete supply chain data using ontology
    let farmer_data = r#"
        @prefix ex: <http://example.org/> .
        @prefix core: <http://provchain.org/core#> .
        @prefix prov: <http://www.w3.org/ns/prov#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        ex:rawMilk a core:Batch ;
            core:hasIdentifier "RAW001" ;
            core:producedAt "2025-08-08T08:00:00Z"^^xsd:dateTime ;
            prov:wasAttributedTo ex:dairyFarm .

        ex:dairyFarm a core:Supplier ;
            rdfs:label "Green Valley Dairy Farm" .
    "#;
    let _ = bc.add_block(farmer_data.into());

    let processing_data = r#"
        @prefix ex: <http://example.org/> .
        @prefix core: <http://provchain.org/core#> .
        @prefix prov: <http://www.w3.org/ns/prov#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        ex:uhtProcess a core:ManufacturingProcess ;
            core:recordedAt "2025-08-08T10:00:00Z"^^xsd:dateTime ;
            prov:used ex:rawMilk ;
            prov:wasAssociatedWith ex:processor .

        ex:processedMilk a core:Batch ;
            core:hasIdentifier "UHT001" ;
            core:producedAt "2025-08-08T10:30:00Z"^^xsd:dateTime ;
            prov:wasGeneratedBy ex:uhtProcess ;
            core:derivedFrom ex:rawMilk .

        ex:processor a core:Manufacturer ;
            rdfs:label "UHT Processing Co." .
    "#;
    let _ = bc.add_block(processing_data.into());

    // Query for complete traceability chain across all graphs
    let trace_query = r#"
        PREFIX core: <http://provchain.org/core#>
        PREFIX prov: <http://www.w3.org/ns/prov#>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
        
        SELECT ?batch ?batchId ?derivedFrom ?agent ?agentLabel ?graph WHERE {
            GRAPH ?graph {
                ?batch a core:Batch ;
                       core:hasIdentifier ?batchId .
                
                OPTIONAL {
                    ?batch core:derivedFrom ?derivedFrom .
                }
                
                OPTIONAL {
                    ?batch prov:wasAttributedTo ?agent .
                    ?agent rdfs:label ?agentLabel .
                }
            }
        }
        ORDER BY ?batchId
    "#;

    if let oxigraph::sparql::QueryResults::Solutions(solutions) = bc.rdf_store.query(trace_query) {
        let mut batch_count = 0;
        for solution in solutions {
            if let Ok(sol) = solution {
                if let Some(batch_id) = sol.get("batchId") {
                    println!("Found batch: {batch_id}");
                    batch_count += 1;
                }
            }
        }
        assert!(
            batch_count >= 2,
            "Should find at least 2 batches in the supply chain"
        );
    } else {
        // Fallback: check if any Batch data exists
        let simple_query = r#"
            PREFIX core: <http://provchain.org/core#>
            
            SELECT ?batch ?batchId WHERE {
                GRAPH ?g {
                    ?batch a core:Batch ;
                           core:hasIdentifier ?batchId .
                }
            }
        "#;

        if let oxigraph::sparql::QueryResults::Solutions(solutions) =
            bc.rdf_store.query(simple_query)
        {
            let mut count = 0;
            for solution in solutions {
                if let Ok(sol) = solution {
                    if let Some(batch_id) = sol.get("batchId") {
                        println!("Found batch in fallback: {batch_id}");
                        count += 1;
                    }
                }
            }
            assert!(
                count >= 2,
                "Should find at least 2 batches in the supply chain (fallback check)"
            );
        }
    }
}
