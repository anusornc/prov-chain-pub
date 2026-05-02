use crate::core::adapter::{AdapterCapabilities, BenchmarkAdapter, TraceQueryAdapter};
use crate::core::result::TraceQueryResult;
use crate::dataset::normalize_turtle;
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use rio_api::parser::TriplesParser;
use rio_turtle::TurtleParser;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const EX_NS: &str = "http://example.org/supplychain/";
const TRACE_NS: &str = "http://example.org/traceability#";

#[derive(Debug, Clone)]
pub struct TigerGraphConfig {
    pub base_url: String,
    pub graph_name: String,
    pub timeout_secs: u64,
}

impl Default for TigerGraphConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:19000".to_string(),
            graph_name: "ProvChainTrace".to_string(),
            timeout_secs: 30,
        }
    }
}

impl TigerGraphConfig {
    pub fn from_env() -> Self {
        Self {
            base_url: std::env::var("TIGERGRAPH_URL")
                .unwrap_or_else(|_| "http://localhost:19000".to_string()),
            graph_name: std::env::var("TIGERGRAPH_GRAPH")
                .unwrap_or_else(|_| "ProvChainTrace".to_string()),
            timeout_secs: std::env::var("TIGERGRAPH_TIMEOUT_SECONDS")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(30),
        }
    }
}

pub struct TigerGraphAdapter {
    config: TigerGraphConfig,
    client: Client,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TigerGraphTranslatedDataset {
    pub products: Vec<TigerGraphProduct>,
    pub actors: Vec<TigerGraphActor>,
    pub transactions: Vec<TigerGraphTransaction>,
    pub product_actor_edges: Vec<TigerGraphProductActorEdge>,
    pub product_transaction_edges: Vec<TigerGraphProductTransactionEdge>,
    pub transaction_party_edges: Vec<TigerGraphTransactionPartyEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TigerGraphProduct {
    pub id: String,
    pub batch_id: String,
    pub product_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TigerGraphActor {
    pub id: String,
    pub actor_type: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TigerGraphTransaction {
    pub id: String,
    pub quantity: String,
    pub transaction_date: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TigerGraphProductActorEdge {
    pub product_id: String,
    pub actor_id: String,
    pub edge_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TigerGraphProductTransactionEdge {
    pub product_id: String,
    pub transaction_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TigerGraphTransactionPartyEdge {
    pub transaction_id: String,
    pub actor_id: String,
    pub edge_type: String,
}

#[derive(Debug, Clone, Default)]
struct ResourceRecord {
    types: BTreeSet<String>,
    literals: BTreeMap<String, String>,
    links: BTreeMap<String, Vec<String>>,
}

impl TigerGraphAdapter {
    pub fn new(config: TigerGraphConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("TigerGraph reqwest client should be constructible");
        Self { config, client }
    }

    pub fn config(&self) -> &TigerGraphConfig {
        &self.config
    }

    pub async fn health_check(&self) -> Result<bool> {
        let response = self
            .client
            .get(self.url("/echo"))
            .send()
            .await
            .context("failed to call TigerGraph RESTPP /echo")?;
        let status = response.status();
        let payload: Value = response
            .json()
            .await
            .unwrap_or_else(|_| serde_json::json!({}));
        Ok(status.is_success()
            && payload
                .get("error")
                .and_then(Value::as_bool)
                .map(|value| !value)
                .unwrap_or(false))
    }

    pub async fn execute_installed_query(
        &self,
        query_name: &str,
        params: &[(&str, &str)],
    ) -> Result<TraceQueryResult> {
        let start = Instant::now();
        let response = self
            .client
            .get(self.query_url(query_name))
            .query(params)
            .send()
            .await
            .with_context(|| format!("failed to call TigerGraph query {query_name}"))?;
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        let status = response.status();
        let payload: Value = response
            .json()
            .await
            .unwrap_or_else(|_| serde_json::json!({}));

        let scenario = match query_name {
            "product_lookup" => "Simple Product Lookup",
            "multi_hop_trace" => "Multi-hop Traceability (10 hops)",
            "aggregation_by_producer" => "Aggregation by Producer",
            _ => query_name,
        };

        if !status.is_success()
            || payload
                .get("error")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        {
            return Ok(TraceQueryResult::new("TigerGraph", scenario, duration_ms)
                .with_error(format!("http status {status}: {payload}")));
        }

        Ok(TraceQueryResult::new("TigerGraph", scenario, duration_ms)
            .with_record_count(tigergraph_result_count(&payload))
            .with_metadata("graph", self.config.graph_name.clone())
            .with_metadata("query_name", query_name.to_string()))
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.config.base_url.trim_end_matches('/'), path)
    }

    fn query_url(&self, query_name: &str) -> String {
        self.url(&format!("/query/{}/{}", self.config.graph_name, query_name))
    }
}

#[async_trait]
impl BenchmarkAdapter for TigerGraphAdapter {
    fn system_name(&self) -> &'static str {
        "TigerGraph"
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            supports_trace_query: true,
            supports_ledger_write: false,
            supports_semantic_pipeline: false,
            supports_native_rdf: false,
            supports_native_jsonld: false,
            supports_native_shacl: false,
            supports_finality_measurement: false,
        }
    }
}

#[async_trait]
impl TraceQueryAdapter for TigerGraphAdapter {
    async fn entity_lookup(&self, id: &str) -> Result<TraceQueryResult> {
        self.execute_installed_query("product_lookup", &[("product_id", id)])
            .await
    }

    async fn trace_multi_hop(&self, id: &str, hops: usize) -> Result<TraceQueryResult> {
        let hops = hops.to_string();
        self.execute_installed_query(
            "multi_hop_trace",
            &[("product_id", id), ("max_hops", &hops)],
        )
        .await
    }

    async fn aggregation_by_producer(&self) -> Result<TraceQueryResult> {
        self.execute_installed_query("aggregation_by_producer", &[])
            .await
    }
}

pub fn translate_turtle_to_tigergraph(dataset_path: &Path) -> Result<TigerGraphTranslatedDataset> {
    let raw_content = fs::read_to_string(dataset_path).with_context(|| {
        format!(
            "failed to read TigerGraph Turtle dataset {:?}",
            dataset_path
        )
    })?;
    translate_turtle_content_to_tigergraph(&raw_content)
}

pub fn translate_turtle_content_to_tigergraph(
    content: &str,
) -> Result<TigerGraphTranslatedDataset> {
    let normalized = normalize_turtle(content);
    let records = parse_resource_records(&normalized)?;
    Ok(build_translated_dataset(records))
}

fn parse_resource_records(content: &str) -> Result<BTreeMap<String, ResourceRecord>> {
    let mut records: BTreeMap<String, ResourceRecord> = BTreeMap::new();
    let mut parser = TurtleParser::new(content.as_bytes(), None);
    parser
        .parse_all(&mut |triple| {
            let subject = match triple.subject {
                rio_api::model::Subject::NamedNode(node) => node.iri.to_string(),
                rio_api::model::Subject::BlankNode(node) => format!("_:{}", node.id),
                _ => return Ok(()) as Result<(), rio_turtle::TurtleError>,
            };
            let predicate = triple.predicate.iri.to_string();
            let record = records.entry(subject).or_default();
            if predicate == RDF_TYPE {
                if let rio_api::model::Term::NamedNode(node) = triple.object {
                    record.types.insert(local_name(node.iri));
                }
            } else {
                match triple.object {
                    rio_api::model::Term::NamedNode(node) => record
                        .links
                        .entry(predicate)
                        .or_default()
                        .push(node.iri.to_string()),
                    rio_api::model::Term::BlankNode(node) => record
                        .links
                        .entry(predicate)
                        .or_default()
                        .push(format!("_:{}", node.id)),
                    rio_api::model::Term::Literal(literal) => {
                        record.literals.insert(predicate, literal_value(&literal));
                    }
                    _ => {}
                }
            }
            Ok(()) as Result<(), rio_turtle::TurtleError>
        })
        .map_err(|error| anyhow::anyhow!("TigerGraph Turtle parsing error: {error:?}"))?;
    Ok(records)
}

fn build_translated_dataset(
    records: BTreeMap<String, ResourceRecord>,
) -> TigerGraphTranslatedDataset {
    let mut dataset = TigerGraphTranslatedDataset::default();

    for (iri, record) in &records {
        if record.types.contains("Product") {
            let product_id = resource_id(iri);
            dataset.products.push(TigerGraphProduct {
                id: product_id.clone(),
                batch_id: literal(record, "batchId").unwrap_or_else(|| product_id.clone()),
                product_type: literal(record, "productType").unwrap_or_default(),
            });

            for (predicate, edge_type) in [
                (format!("{TRACE_NS}hasProducer"), "HAS_PRODUCER"),
                (format!("{TRACE_NS}processedBy"), "PROCESSED_BY"),
                (format!("{TRACE_NS}distributedBy"), "DISTRIBUTED_BY"),
                (format!("{TRACE_NS}soldBy"), "SOLD_BY"),
            ] {
                for actor in record.links.get(&predicate).into_iter().flatten() {
                    dataset
                        .product_actor_edges
                        .push(TigerGraphProductActorEdge {
                            product_id: product_id.clone(),
                            actor_id: resource_id(actor),
                            edge_type: edge_type.to_string(),
                        });
                }
            }

            for transaction in record
                .links
                .get(&format!("{TRACE_NS}hasTransaction"))
                .into_iter()
                .flatten()
            {
                dataset
                    .product_transaction_edges
                    .push(TigerGraphProductTransactionEdge {
                        product_id: product_id.clone(),
                        transaction_id: resource_id(transaction),
                    });
            }
        }

        if let Some(actor_type) = actor_type(record) {
            dataset.actors.push(TigerGraphActor {
                id: resource_id(iri),
                actor_type,
                name: literal(record, "name").unwrap_or_default(),
            });
        }

        if record.types.contains("Transaction") {
            let transaction_id = resource_id(iri);
            dataset.transactions.push(TigerGraphTransaction {
                id: transaction_id.clone(),
                quantity: literal(record, "quantity").unwrap_or_default(),
                transaction_date: literal(record, "transactionDate").unwrap_or_default(),
            });

            for (predicate, edge_type) in [
                (format!("{EX_NS}from"), "FROM_ACTOR"),
                (format!("{EX_NS}to"), "TO_ACTOR"),
            ] {
                for actor in record.links.get(&predicate).into_iter().flatten() {
                    dataset
                        .transaction_party_edges
                        .push(TigerGraphTransactionPartyEdge {
                            transaction_id: transaction_id.clone(),
                            actor_id: resource_id(actor),
                            edge_type: edge_type.to_string(),
                        });
                }
            }
        }
    }

    dataset.products.sort_by(|a, b| a.id.cmp(&b.id));
    dataset.actors.sort_by(|a, b| a.id.cmp(&b.id));
    dataset.transactions.sort_by(|a, b| a.id.cmp(&b.id));
    dataset.product_actor_edges.sort_by(|a, b| {
        (&a.product_id, &a.edge_type, &a.actor_id).cmp(&(&b.product_id, &b.edge_type, &b.actor_id))
    });
    dataset.product_transaction_edges.sort_by(|a, b| {
        (&a.product_id, &a.transaction_id).cmp(&(&b.product_id, &b.transaction_id))
    });
    dataset.transaction_party_edges.sort_by(|a, b| {
        (&a.transaction_id, &a.edge_type, &a.actor_id).cmp(&(
            &b.transaction_id,
            &b.edge_type,
            &b.actor_id,
        ))
    });
    dataset
}

fn actor_type(record: &ResourceRecord) -> Option<String> {
    ["Producer", "Processor", "Distributor", "Retailer"]
        .iter()
        .find(|kind| record.types.contains(**kind))
        .map(|kind| (*kind).to_string())
}

fn literal(record: &ResourceRecord, local: &str) -> Option<String> {
    record
        .literals
        .get(&format!("{EX_NS}{local}"))
        .or_else(|| record.literals.get(&format!("{TRACE_NS}{local}")))
        .cloned()
}

fn literal_value(literal: &rio_api::model::Literal<'_>) -> String {
    match literal {
        rio_api::model::Literal::Simple { value } => value.to_string(),
        rio_api::model::Literal::LanguageTaggedString { value, language: _ } => value.to_string(),
        rio_api::model::Literal::Typed { value, datatype: _ } => value.to_string(),
    }
}

fn local_name(iri: &str) -> String {
    iri.rsplit(['#', '/']).next().unwrap_or(iri).to_string()
}

fn resource_id(iri: &str) -> String {
    iri.rsplit('/').next().unwrap_or(iri).to_string()
}

fn tigergraph_result_count(payload: &Value) -> usize {
    payload
        .get("results")
        .and_then(Value::as_array)
        .map(|results| {
            results
                .iter()
                .map(|entry| match entry {
                    Value::Array(items) => items.len(),
                    Value::Object(map) => map
                        .values()
                        .map(|value| value.as_array().map(Vec::len).unwrap_or(1))
                        .sum(),
                    _ => 1,
                })
                .sum()
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    fn test_adapter(server: &Server) -> TigerGraphAdapter {
        TigerGraphAdapter::new(TigerGraphConfig {
            base_url: server.url(),
            graph_name: "ProvChainTrace".to_string(),
            timeout_secs: 5,
        })
    }

    #[tokio::test]
    async fn health_check_accepts_echo_success() {
        let mut server = Server::new_async().await;
        let _echo = server
            .mock("GET", "/echo")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error":false,"message":"Hello GSQL"}"#)
            .create_async()
            .await;

        assert!(test_adapter(&server).health_check().await.unwrap());
    }

    #[tokio::test]
    async fn product_lookup_calls_installed_query_endpoint() {
        let mut server = Server::new_async().await;
        let _query = server
            .mock("GET", "/query/ProvChainTrace/product_lookup")
            .match_query(mockito::Matcher::UrlEncoded(
                "product_id".to_string(),
                "BATCH001".to_string(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                  "error": false,
                  "results": [
                    {"matched": [{"v_id": "BATCH001", "v_type": "Product"}]}
                  ]
                }"#,
            )
            .create_async()
            .await;

        let result = test_adapter(&server)
            .entity_lookup("BATCH001")
            .await
            .unwrap();
        assert_eq!(result.system, "TigerGraph");
        assert_eq!(result.scenario, "Simple Product Lookup");
        assert_eq!(result.record_count, 1);
        assert_eq!(
            result.metadata.get("graph").and_then(Value::as_str),
            Some("ProvChainTrace")
        );
    }

    #[tokio::test]
    async fn query_error_is_returned_as_failed_trace_result() {
        let mut server = Server::new_async().await;
        let _query = server
            .mock("GET", "/query/ProvChainTrace/aggregation_by_producer")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error":true,"message":"query not installed"}"#)
            .create_async()
            .await;

        let result = test_adapter(&server)
            .aggregation_by_producer()
            .await
            .unwrap();
        assert!(!result.success);
        assert!(result
            .error_message
            .as_deref()
            .unwrap_or_default()
            .contains("query not installed"));
    }

    #[test]
    fn translator_maps_turtle_to_property_graph_rows() {
        let ttl = r#"@prefix ex: <http://example.org/supplychain/> .
@prefix trace: <http://example.org/traceability#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:Producer/Farm001 a ex:Producer ;
    ex:name "Farm 001" .

ex:Product/BATCH001 a ex:Product ;
    ex:productType "Tomato" ;
    ex:batchId "BATCH001" ;
    trace:hasProducer ex:Producer/Farm001 ;
    trace:hasTransaction ex:Transaction/TX001 .

ex:Transaction/TX001 a ex:Transaction ;
    trace:transactionDate "2024-01-02T08:00:00"^^xsd:dateTime ;
    trace:quantity 100.0 ;
    ex:from ex:Producer/Farm001 .
"#;

        let translated = translate_turtle_content_to_tigergraph(ttl).unwrap();
        assert_eq!(translated.products.len(), 1);
        assert_eq!(translated.products[0].id, "BATCH001");
        assert_eq!(translated.actors.len(), 1);
        assert_eq!(translated.actors[0].actor_type, "Producer");
        assert_eq!(translated.transactions.len(), 1);
        assert_eq!(translated.product_actor_edges.len(), 1);
        assert_eq!(translated.product_actor_edges[0].edge_type, "HAS_PRODUCER");
        assert_eq!(translated.product_transaction_edges.len(), 1);
        assert_eq!(translated.transaction_party_edges.len(), 1);
    }

    #[test]
    fn translator_normalizes_slashed_curies_before_parsing() {
        let ttl = r#"@prefix ex: <http://example.org/supplychain/> .
ex:Product/BATCH001 a ex:Product ; ex:batchId "BATCH001" .
"#;
        let translated = translate_turtle_content_to_tigergraph(ttl).unwrap();
        assert_eq!(translated.products[0].id, "BATCH001");
    }
}
