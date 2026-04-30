//! Shared SPARQL query definitions for ProvChain benchmark scenarios.

/// Build the benchmark query for simple product lookup by batch id.
pub fn entity_lookup_query(id: &str) -> String {
    format!(
        "PREFIX ex: <http://example.org/supplychain/> \
         SELECT ?product WHERE {{ \
             GRAPH ?g1 {{ ?product a ex:Product . }} \
             GRAPH ?g2 {{ ?product ex:batchId \"{}\" . }} \
         }}",
        id
    )
}

/// Build the benchmark query for a short multi-hop transaction trace.
pub fn multi_hop_query(id: &str) -> String {
    format!(
        "PREFIX ex: <http://example.org/supplychain/> \
         PREFIX trace: <http://example.org/traceability#> \
         SELECT ?product ?tx1 ?tx2 ?tx3 \
         WHERE {{ \
             GRAPH ?g1 {{ ?product ex:batchId \"{}\" . }} \
             GRAPH ?g2 {{ ?product trace:hasTransaction ?tx1 . }} \
             OPTIONAL {{ GRAPH ?g3 {{ ?tx1 trace:nextTransaction ?tx2 . }} }} \
             OPTIONAL {{ GRAPH ?g4 {{ ?tx2 trace:nextTransaction ?tx3 . }} }} \
         }}",
        id
    )
}

/// Build the benchmark query for producer-level aggregation.
pub fn aggregation_by_producer_query() -> String {
    r#"
        PREFIX ex: <http://example.org/supplychain/>
        PREFIX trace: <http://example.org/traceability#>
        PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
        SELECT ?producer (SUM(xsd:decimal(?quantity)) AS ?total)
        WHERE {
            GRAPH ?g1 { ?product trace:hasProducer ?producer . }
            GRAPH ?g2 { ?product trace:hasTransaction ?tx . }
            GRAPH ?g3 { ?tx trace:quantity ?quantity . }
        }
        GROUP BY ?producer
    "#
    .to_string()
}
