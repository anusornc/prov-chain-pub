//! Data models for web API responses and requests

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Response model for blockchain status
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockchainStatus {
    pub height: usize,
    pub latest_block_hash: String,
    pub total_transactions: usize,
    pub network_peers: usize,
    pub last_updated: DateTime<Utc>,
}

/// Response model for block information
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockInfo {
    pub index: usize,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: DateTime<Utc>,
    pub transaction_count: usize,
    pub size_bytes: usize,
}

/// Response model for transaction/triple information
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub block_index: usize,
    pub timestamp: DateTime<Utc>,
}

/// Request model for adding new triples
#[derive(Debug, Serialize, Deserialize)]
pub struct AddTripleRequest {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub graph_name: Option<String>,
    pub privacy_key_id: Option<String>, // Optional key ID for encryption
}

/// Request model for SPARQL queries
#[derive(Debug, Serialize, Deserialize)]
pub struct SparqlQueryRequest {
    pub query: String,
    pub format: Option<String>, // json, xml, turtle, etc.
}

/// Response model for SPARQL query results
#[derive(Debug, Serialize, Deserialize)]
pub struct SparqlQueryResponse {
    pub results: serde_json::Value,
    pub execution_time_ms: u64,
    pub result_count: usize,
}

/// Response model for product traceability
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductTrace {
    pub batch_id: String,
    pub product_name: String,
    pub origin: String,
    pub current_location: String,
    pub status: String,
    pub timeline: Vec<TraceEvent>,
    pub certifications: Vec<String>,
    pub environmental_data: Option<EnvironmentalData>,
}

/// Individual trace event in product journey
#[derive(Debug, Serialize, Deserialize)]
pub struct TraceEvent {
    pub timestamp: DateTime<Utc>,
    pub location: String,
    pub actor: String,
    pub action: String,
    pub details: String,
    pub block_hash: String,
}

/// Environmental conditions data
#[derive(Debug, Serialize, Deserialize)]
pub struct EnvironmentalData {
    pub temperature: Option<f64>,
    pub humidity: Option<f64>,
    pub co2_footprint: Option<f64>,
    pub certifications: Vec<String>,
}

/// API error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// Authentication request
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

/// Authentication response
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub user_role: String,
}

/// User claims for JWT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    pub sub: String, // user id
    pub role: String,
    pub exp: usize, // expiration timestamp
}

/// Supply chain actor roles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActorRole {
    Farmer,
    Processor,
    Transporter,
    Retailer,
    Consumer,
    Auditor,
    Admin,
}

impl std::fmt::Display for ActorRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActorRole::Farmer => write!(f, "farmer"),
            ActorRole::Processor => write!(f, "processor"),
            ActorRole::Transporter => write!(f, "transporter"),
            ActorRole::Retailer => write!(f, "retailer"),
            ActorRole::Consumer => write!(f, "consumer"),
            ActorRole::Auditor => write!(f, "auditor"),
            ActorRole::Admin => write!(f, "admin"),
        }
    }
}

/// Wallet registration request
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletRegistrationRequest {
    pub name: String,
    pub participant_type: String,
    pub contact_info: Option<ContactInfo>,
    pub location: Option<String>,
}

/// Contact information for participants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub website: Option<String>,
}

/// Wallet registration response
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletRegistrationResponse {
    pub participant_id: String,
    pub public_key: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// Transaction creation request
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTransactionRequest {
    pub tx_type: String,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub rdf_data: String,
    pub metadata: TransactionMetadata,
}

/// Transaction input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInput {
    pub prev_tx_id: String,
    pub output_index: u32,
}

/// Transaction output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionOutput {
    pub id: String,
    pub owner: String, // Participant ID
    pub asset_type: String,
    pub value: f64,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Transaction metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMetadata {
    pub location: Option<String>,
    pub environmental_conditions: Option<EnvironmentalConditions>,
    pub compliance_info: Option<ComplianceInfo>,
    pub quality_data: Option<QualityData>,
    pub custom_fields: std::collections::HashMap<String, String>,
}

/// Environmental conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalConditions {
    pub temperature: Option<f64>,
    pub humidity: Option<f64>,
    pub pressure: Option<f64>,
    pub timestamp: DateTime<Utc>,
    pub sensor_id: Option<String>,
}

/// Compliance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceInfo {
    pub regulation_type: String,
    pub compliance_status: String,
    pub certificate_id: Option<String>,
    pub auditor_id: Option<String>, // Participant ID
    pub expiry_date: Option<DateTime<Utc>>,
}

/// Quality data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityData {
    pub test_type: String,
    pub test_result: String,
    pub test_value: Option<f64>,
    pub test_unit: Option<String>,
    pub lab_id: Option<String>, // Participant ID
    pub test_timestamp: DateTime<Utc>,
}

/// Transaction creation response
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTransactionResponse {
    pub tx_id: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// Transaction signing request
#[derive(Debug, Serialize, Deserialize)]
pub struct SignTransactionRequest {
    pub tx_id: String,
    pub participant_id: String,
}

/// Transaction signing response
#[derive(Debug, Serialize, Deserialize)]
pub struct SignTransactionResponse {
    pub tx_id: String,
    pub signatures: Vec<TransactionSignatureInfo>,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// Transaction signature information
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionSignatureInfo {
    pub signer_id: String,
    pub timestamp: DateTime<Utc>,
}

/// Transaction submission request
#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitTransactionRequest {
    pub tx_id: String,
}

/// Transaction submission response
#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitTransactionResponse {
    pub tx_id: String,
    pub block_index: Option<usize>,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

// --- Query Parameters ---

/// Query parameters for product trace
#[derive(Debug, Serialize, Deserialize)]
pub struct TraceQueryParams {
    pub batch_id: Option<String>,
    pub product_name: Option<String>,
}

fn default_optimization_level() -> u8 {
    1
}

/// Query parameters for enhanced product trace
#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedTraceQueryParams {
    pub batch_id: String,
    #[serde(default = "default_optimization_level")]
    pub optimization_level: u8,
}

/// Query parameters for products listing
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductsQueryParams {
    pub q: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    #[serde(rename = "type")]
    pub product_type: Option<String>,
    pub participant: Option<String>,
    pub location: Option<String>,
    pub status: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

/// Query parameters for knowledge graph
#[derive(Debug, Serialize, Deserialize)]
pub struct KnowledgeGraphParams {
    pub item_id: Vec<String>,
}

/// Analytics query parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsQueryParams {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub participant_type: Option<String>,
    pub transaction_type: Option<String>,
    pub granularity: Option<String>, // hour | day | week | month
}
