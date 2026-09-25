//! Production-mode honesty tests for public trace/query endpoints.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use provchain_org::config::RuntimeMode;
use provchain_org::core::blockchain::Blockchain;
use provchain_org::web::handlers::{
    delete_sparql_query, get_blockchain_status, get_product_analytics, get_product_by_id,
    get_product_provenance, get_product_trace, get_product_trace_path, get_products,
    get_products_by_type, get_saved_sparql_queries, save_sparql_query,
    toggle_favorite_sparql_query, trace_path_api, validate_item, AppState,
};
use provchain_org::web::models::{ProductsQueryParams, TraceQueryParams, UserClaims};
use serde_json::json;

const TEST_PRODUCT_IRI: &str = "http://example.org/product-honesty";

fn production_state() -> AppState {
    AppState::new(Blockchain::new()).expect("production AppState should initialize")
}

fn demo_state() -> AppState {
    AppState::with_runtime_mode(Blockchain::new(), RuntimeMode::Demo)
        .expect("demo AppState should initialize")
}

fn product_params(product_type: Option<&str>) -> ProductsQueryParams {
    ProductsQueryParams {
        q: None,
        page: None,
        limit: None,
        sort_by: None,
        sort_order: None,
        product_type: product_type.map(str::to_string),
        participant: None,
        location: None,
        status: None,
        start_date: None,
        end_date: None,
    }
}

fn test_claims() -> UserClaims {
    UserClaims {
        sub: "runtime-honesty-test-user".to_string(),
        role: "user".to_string(),
        exp: 4_102_444_800,
    }
}

async fn seed_core_product(state: &AppState) {
    let mut blockchain = state.blockchain.write().await;
    blockchain
        .add_block(format!(
            r#"
            @prefix ex: <http://example.org/> .
            @prefix core: <http://provchain.org/core#> .
            @prefix trace: <http://provchain.org/trace#> .
            @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

            <{TEST_PRODUCT_IRI}> a core:Product ;
                trace:name "Runtime Honesty Product" ;
                trace:status "released" ;
                trace:participant "participant:test" ;
                trace:location "warehouse:test" ;
                trace:timestamp "2026-05-13T00:00:00Z"^^xsd:dateTime ;
                trace:description "Fixture product for runtime honesty checks" .
            "#
        ))
        .expect("fixture product should be admitted");
}

async fn seed_product_analytics_steps(state: &AppState) {
    let mut blockchain = state.blockchain.write().await;
    blockchain
        .add_block(format!(
            r#"
            @prefix ex: <http://example.org/> .
            @prefix trace: <http://provchain.org/trace#> .
            @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

            ex:analytics-step-1 trace:product <{TEST_PRODUCT_IRI}> ;
                trace:participant "participant:test" ;
                trace:location "warehouse:test" ;
                trace:timestamp "2026-05-13T00:00:00Z"^^xsd:dateTime .

            ex:analytics-step-2 trace:product <{TEST_PRODUCT_IRI}> ;
                trace:participant "participant:test-2" ;
                trace:location "retail:test" ;
                trace:timestamp "2026-05-14T00:00:00Z"^^xsd:dateTime .
            "#
        ))
        .expect("fixture analytics steps should be admitted");
}

#[tokio::test]
async fn production_mode_trace_endpoints_fail_closed_without_data() {
    let state = production_state();

    let trace = get_product_trace(
        Query(TraceQueryParams {
            batch_id: Some("MISSING-BATCH".to_string()),
            product_name: None,
        }),
        State(state.clone()),
    )
    .await;
    assert_eq!(trace.unwrap_err().0, StatusCode::NOT_FOUND);

    let path_trace = get_product_trace_path(
        Path("http://example.org/missing_product".to_string()),
        State(state.clone()),
    )
    .await;
    assert_eq!(path_trace.unwrap_err().0, StatusCode::NOT_FOUND);

    let provenance = get_product_provenance(
        Path("http://example.org/missing_product".to_string()),
        State(state.clone()),
    )
    .await;
    assert_eq!(provenance.unwrap_err().0, StatusCode::NOT_FOUND);

    let legacy_trace = trace_path_api(
        Query(TraceQueryParams {
            batch_id: Some("MISSING-BATCH".to_string()),
            product_name: None,
        }),
        State(state),
    )
    .await;
    assert_eq!(legacy_trace.unwrap_err().0, StatusCode::NOT_IMPLEMENTED);
}

#[tokio::test]
async fn production_mode_saved_query_demo_endpoints_are_unsupported() {
    let state = production_state();

    let list = get_saved_sparql_queries(State(state.clone())).await;
    assert_eq!(list.unwrap_err().0, StatusCode::NOT_IMPLEMENTED);

    let save = save_sparql_query(State(state.clone()), axum::Json(json!({"name": "demo"}))).await;
    assert_eq!(save.unwrap_err().0, StatusCode::NOT_IMPLEMENTED);

    let delete = delete_sparql_query(State(state.clone()), Path("q_demo".to_string())).await;
    assert_eq!(delete.unwrap_err().0, StatusCode::NOT_IMPLEMENTED);

    let toggle = toggle_favorite_sparql_query(State(state), Path("q_demo".to_string())).await;
    assert_eq!(toggle.unwrap_err().0, StatusCode::NOT_IMPLEMENTED);
}

#[tokio::test]
async fn production_mode_status_does_not_emit_synthetic_hash_rate() {
    let status = get_blockchain_status(State(production_state()))
        .await
        .expect("status should be available")
        .0;

    assert!(status["network_hash_rate"].is_null());
    assert_eq!(status["network_hash_rate_source"], "unavailable");
}

#[tokio::test]
async fn demo_mode_keeps_legacy_trace_fallback_explicitly_labelled() {
    let response = trace_path_api(
        Query(TraceQueryParams {
            batch_id: Some("DEMO-BATCH".to_string()),
            product_name: None,
        }),
        State(demo_state()),
    )
    .await
    .expect("demo fallback should remain available")
    .0;

    assert_eq!(response["batch_id"], "DEMO-BATCH");
    assert_eq!(response["source"], "demo_fallback");
}

#[tokio::test]
async fn production_mode_product_metrics_are_unavailable_not_synthetic() {
    let state = production_state();
    seed_core_product(&state).await;

    let products = get_products(Query(product_params(None)), State(state.clone()))
        .await
        .expect("product listing should be available")
        .0;
    let product = &products["items"][0];
    assert!(product["quality_score"].is_null());
    assert_eq!(product["quality_score_source"], "unavailable");
    assert!(product["compliance_status"].is_null());
    assert_eq!(product["compliance_status_source"], "unavailable");

    let detail = get_product_by_id(
        Path(TEST_PRODUCT_IRI.to_string()),
        axum::Extension(test_claims()),
        State(state),
    )
    .await
    .expect("product detail should be available")
    .0;
    assert!(detail["quality_score"].is_null());
    assert_eq!(detail["quality_score_source"], "unavailable");
    assert!(detail["compliance_status"].is_null());
    assert_eq!(detail["compliance_status_source"], "unavailable");
}

#[tokio::test]
async fn demo_mode_product_metrics_are_explicitly_labelled_synthetic_demo() {
    let state = demo_state();
    seed_core_product(&state).await;

    let products = get_products(Query(product_params(None)), State(state.clone()))
        .await
        .expect("product listing should be available")
        .0;
    let product = &products["items"][0];
    assert_eq!(product["quality_score"], json!(85.0));
    assert_eq!(product["quality_score_source"], "synthetic_demo");
    assert_eq!(product["compliance_status"], "compliant");
    assert_eq!(product["compliance_status_source"], "synthetic_demo");

    let detail = get_product_by_id(
        Path(TEST_PRODUCT_IRI.to_string()),
        axum::Extension(test_claims()),
        State(state),
    )
    .await
    .expect("product detail should be available")
    .0;
    assert_eq!(detail["quality_score"], json!(85.0));
    assert_eq!(detail["quality_score_source"], "synthetic_demo");
    assert_eq!(detail["compliance_status"], "compliant");
    assert_eq!(detail["compliance_status_source"], "synthetic_demo");
}

#[tokio::test]
async fn production_mode_product_analytics_does_not_emit_synthetic_metrics() {
    let state = production_state();
    seed_core_product(&state).await;
    seed_product_analytics_steps(&state).await;

    let analytics = get_product_analytics(Path(TEST_PRODUCT_IRI.to_string()), State(state))
        .await
        .expect("analytics with real trace steps should be available")
        .0;

    assert_eq!(analytics["total_steps"], 2);
    assert_eq!(analytics["total_participants"], 2);
    assert_eq!(analytics["duration_days_source"], "derived");
    assert!(analytics["carbon_footprint"].is_null());
    assert_eq!(analytics["carbon_footprint_source"], "unavailable");
    assert!(analytics["quality_scores"].is_null());
    assert_eq!(analytics["quality_scores_source"], "unavailable");
    assert!(analytics["compliance_status"].is_null());
    assert_eq!(analytics["compliance_status_source"], "unavailable");
}

#[tokio::test]
async fn production_mode_product_analytics_fails_closed_without_real_data() {
    let state = production_state();
    seed_core_product(&state).await;

    let no_analytics = get_product_analytics(Path(TEST_PRODUCT_IRI.to_string()), State(state))
        .await
        .expect_err("production analytics must not fabricate missing trace data");

    assert_eq!(no_analytics.0, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn demo_mode_product_analytics_labels_synthetic_metrics() {
    let state = demo_state();

    let analytics = get_product_analytics(Path(TEST_PRODUCT_IRI.to_string()), State(state))
        .await
        .expect("demo analytics fallback should remain available")
        .0;

    assert_eq!(analytics["carbon_footprint"], json!(2.5));
    assert_eq!(analytics["carbon_footprint_source"], "synthetic_demo");
    assert_eq!(analytics["quality_scores_source"], "synthetic_demo");
    assert_eq!(analytics["compliance_status"], "compliant");
    assert_eq!(analytics["compliance_status_source"], "synthetic_demo");
}

#[tokio::test]
async fn production_mode_product_analytics_rejects_invalid_product_id() {
    let invalid = get_product_analytics(
        Path("http://example.org/product> ?s ?p ?o".to_string()),
        State(production_state()),
    )
    .await
    .expect_err("invalid product IRI must be rejected before SPARQL construction");

    assert_eq!(invalid.0, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn production_mode_item_validation_is_explicitly_unsupported() {
    let state = production_state();
    seed_core_product(&state).await;

    let validation = validate_item(Path(TEST_PRODUCT_IRI.to_string()), State(state))
        .await
        .expect_err(
            "production item validation must not report chain validation as item validation",
        );

    assert_eq!(validation.0, StatusCode::NOT_IMPLEMENTED);
}

#[tokio::test]
async fn demo_mode_item_validation_labels_chain_level_fallback() {
    let state = demo_state();
    seed_core_product(&state).await;

    let validation = validate_item(Path(TEST_PRODUCT_IRI.to_string()), State(state))
        .await
        .expect("demo validation fallback should remain available")
        .0;

    assert_eq!(validation["requested_item_id"], TEST_PRODUCT_IRI);
    assert_eq!(validation["validation_scope"], "chain_demo_fallback");
    assert_eq!(validation["source"], "demo_fallback");
    assert_eq!(validation["item_specific_validation"], "not_implemented");
}

#[tokio::test]
async fn product_type_filters_use_canonical_core_ontology_types() {
    let state = production_state();
    seed_core_product(&state).await;

    let filtered = get_products(Query(product_params(Some("product"))), State(state.clone()))
        .await
        .expect("lowercase product type alias should map to core Product")
        .0;
    assert_eq!(filtered["total_count"], 1);
    assert_eq!(filtered["items"][0]["type"], "Product");

    let by_type = get_products_by_type(Path("product".to_string()), State(state.clone()))
        .await
        .expect("by-type endpoint should use same canonical type mapper")
        .0;
    assert_eq!(by_type.len(), 1);
    assert_eq!(by_type[0]["type"], "Product");

    let invalid = get_products(
        Query(product_params(Some("UnknownDemoType"))),
        State(state.clone()),
    )
    .await;
    assert_eq!(invalid.unwrap_err().0, StatusCode::BAD_REQUEST);

    let invalid_by_type =
        get_products_by_type(Path("UnknownDemoType".to_string()), State(state)).await;
    assert_eq!(invalid_by_type.unwrap_err().0, StatusCode::BAD_REQUEST);
}
