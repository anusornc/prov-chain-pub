#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceQueryKind {
    EntityLookup,
    MultiHop,
    AggregationByProducer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceQueryScenario {
    pub kind: TraceQueryKind,
    pub name: &'static str,
    pub batch_ids: Vec<String>,
    pub hops: Option<usize>,
}

impl TraceQueryScenario {
    pub fn entity_lookup(batch_ids: &[String]) -> Self {
        Self {
            kind: TraceQueryKind::EntityLookup,
            name: "Simple Product Lookup",
            batch_ids: batch_ids.to_vec(),
            hops: None,
        }
    }

    pub fn multi_hop(batch_ids: &[String], hops: usize) -> Self {
        Self {
            kind: TraceQueryKind::MultiHop,
            name: "Multi-hop Traceability",
            batch_ids: batch_ids.to_vec(),
            hops: Some(hops),
        }
    }

    pub fn aggregation_by_producer() -> Self {
        Self {
            kind: TraceQueryKind::AggregationByProducer,
            name: "Aggregation by Producer",
            batch_ids: Vec::new(),
            hops: None,
        }
    }
}

pub fn parse_batch_ids(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

pub fn default_trace_query_scenarios(batch_ids: &[String]) -> Vec<TraceQueryScenario> {
    vec![
        TraceQueryScenario::entity_lookup(batch_ids),
        TraceQueryScenario::multi_hop(batch_ids, 10),
        TraceQueryScenario::aggregation_by_producer(),
    ]
}
