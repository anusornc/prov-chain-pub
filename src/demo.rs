use crate::core::blockchain::Blockchain;
use std::fs;

pub fn run_demo() {
    let mut bc = Blockchain::new();

    // Farmer RDF using ontology classes
    let farmer_data = r#"
        @prefix ex: <http://example.org/> .
        @prefix core: <http://provchain.org/core#> .
        @prefix prov: <http://www.w3.org/ns/prov#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        ex:milkBatch1 a core:Batch ;
            core:hasIdentifier "MB001" ;
            core:producedAt "2025-08-08T10:00:00Z"^^xsd:dateTime ;
            prov:wasAttributedTo ex:FarmerJohn .

        ex:FarmerJohn a core:Supplier ;
            rdfs:label "John's Dairy Farm" .
    "#;
    let _ = bc.add_block(farmer_data.into());

    // Manufacturer RDF using ontology classes
    let manufacturer_data = r#"
        @prefix ex: <http://example.org/> .
        @prefix core: <http://provchain.org/core#> .
        @prefix prov: <http://www.w3.org/ns/prov#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        ex:uhtProcess1 a core:ManufacturingProcess ;
            core:recordedAt "2025-08-08T12:00:00Z"^^xsd:dateTime ;
            prov:used ex:milkBatch1 ;
            prov:wasAssociatedWith ex:UHTFactory .

        ex:uhtMilk1 a core:Batch ;
            core:hasIdentifier "UHT001" ;
            core:producedAt "2025-08-08T12:30:00Z"^^xsd:dateTime ;
            prov:wasGeneratedBy ex:uhtProcess1 ;
            core:derivedFrom ex:milkBatch1 .

        ex:UHTFactory a core:Manufacturer ;
            rdfs:label "UHT Processing Factory A" .
    "#;
    let _ = bc.add_block(manufacturer_data.into());

    // Transport RDF with environmental conditions
    let transport_data = r#"
        @prefix ex: <http://example.org/> .
        @prefix core: <http://provchain.org/core#> .
        @prefix prov: <http://www.w3.org/ns/prov#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        ex:transport1 a core:TransportProcess ;
            core:recordedAt "2025-08-08T14:00:00Z"^^xsd:dateTime ;
            prov:used ex:uhtMilk1 ;
            prov:wasAssociatedWith ex:LogisticsCorp ;
            core:hasCondition ex:condition1 .

        ex:condition1 a core:EnvironmentalCondition ;
            core:hasTemperature "4.2"^^xsd:decimal ;
            core:hasHumidity "65.0"^^xsd:decimal ;
            core:hasConditionTimestamp "2025-08-08T14:00:00Z"^^xsd:dateTime .

        ex:LogisticsCorp a core:LogisticsProvider ;
            rdfs:label "Cold Chain Logistics Corp" .
    "#;
    let _ = bc.add_block(transport_data.into());

    println!("Blockchain valid? {}", bc.is_valid());
    println!("\n--- Blockchain Dump ---");
    println!("{}", bc.dump());
    println!("\n--- Running Queries ---");


    // Run Step 4 queries if present in ./queries/
    let queries = vec![
        "trace_by_batch.sparql",
        "trace_by_batch_ontology.sparql",
        "trace_origin.sparql",
        "env_conditions_for_batch.sparql",
        "blockchain_metadata.sparql",
    ];

    for qfile in queries {
        let path = format!("queries/{qfile}");
        if let Ok(qtext) = fs::read_to_string(&path) {
            println!("\n=== Running query: {qfile} ===");
            if let oxigraph::sparql::QueryResults::Solutions(solutions) = bc.rdf_store.query(&qtext) {
                for solution in solutions {
                    println!("{:?}", solution.unwrap());
                }
            }
        }
    }
}
