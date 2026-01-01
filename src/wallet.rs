//! Wallet system for ProvChainOrg participants
//!
//! This module implements:
//! - Multi-participant wallet management
//! - Secure key storage and management
//! - Participant identity and role management
//! - Transaction signing capabilities

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Participant types in the supply chain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ParticipantType {
    /// Raw material producers (farmers, suppliers)
    Producer,
    /// Manufacturing facilities (UHT processors, packagers)
    Manufacturer,
    /// Logistics and transport providers
    LogisticsProvider,
    /// Quality control laboratories
    QualityLab,
    /// Regulatory authorities and auditors
    Auditor,
    /// Retail and distribution
    Retailer,
    /// System administrators
    Administrator,
}

/// Participant permissions for different operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantPermissions {
    /// Can create production transactions
    pub can_produce: bool,
    /// Can create processing transactions
    pub can_process: bool,
    /// Can create transport transactions
    pub can_transport: bool,
    /// Can create quality control transactions
    pub can_quality_test: bool,
    /// Can create compliance transactions
    pub can_audit: bool,
    /// Can transfer ownership
    pub can_transfer: bool,
    /// Can view all transactions (admin privilege)
    pub can_view_all: bool,
    /// Can manage other participants (admin privilege)
    pub can_manage_participants: bool,
}

impl ParticipantPermissions {
    /// Get default permissions for a participant type
    pub fn for_type(participant_type: &ParticipantType) -> Self {
        match participant_type {
            ParticipantType::Producer => Self {
                can_produce: true,
                can_process: false,
                can_transport: false,
                can_quality_test: false,
                can_audit: false,
                can_transfer: true,
                can_view_all: false,
                can_manage_participants: false,
            },
            ParticipantType::Manufacturer => Self {
                can_produce: false,
                can_process: true,
                can_transport: false,
                can_quality_test: false,
                can_audit: false,
                can_transfer: true,
                can_view_all: false,
                can_manage_participants: false,
            },
            ParticipantType::LogisticsProvider => Self {
                can_produce: false,
                can_process: false,
                can_transport: true,
                can_quality_test: false,
                can_audit: false,
                can_transfer: false,
                can_view_all: false,
                can_manage_participants: false,
            },
            ParticipantType::QualityLab => Self {
                can_produce: false,
                can_process: false,
                can_transport: false,
                can_quality_test: true,
                can_audit: false,
                can_transfer: false,
                can_view_all: false,
                can_manage_participants: false,
            },
            ParticipantType::Auditor => Self {
                can_produce: false,
                can_process: false,
                can_transport: false,
                can_quality_test: false,
                can_audit: true,
                can_transfer: false,
                can_view_all: true,
                can_manage_participants: false,
            },
            ParticipantType::Retailer => Self {
                can_produce: false,
                can_process: false,
                can_transport: false,
                can_quality_test: false,
                can_audit: false,
                can_transfer: true,
                can_view_all: false,
                can_manage_participants: false,
            },
            ParticipantType::Administrator => Self {
                can_produce: true,
                can_process: true,
                can_transport: true,
                can_quality_test: true,
                can_audit: true,
                can_transfer: true,
                can_view_all: true,
                can_manage_participants: true,
            },
        }
    }
}

/// Certificate information for participants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    /// Certificate ID
    pub id: String,
    /// Certificate type (e.g., "ORGANIC", "FDA_APPROVED", "ISO_9001")
    pub cert_type: String,
    /// Issuing authority
    pub issuer: String,
    /// Issue date
    pub issued_at: DateTime<Utc>,
    /// Expiration date
    pub expires_at: DateTime<Utc>,
    /// Certificate status
    pub status: CertificateStatus,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Certificate status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CertificateStatus {
    Active,
    Expired,
    Revoked,
    Suspended,
}

/// Participant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    /// Unique participant identifier
    pub id: Uuid,
    /// Participant name/label
    pub name: String,
    /// Participant type
    pub participant_type: ParticipantType,
    /// Contact information
    pub contact_info: ContactInfo,
    /// Location information
    pub location: Option<String>,
    /// Permissions for this participant
    pub permissions: ParticipantPermissions,
    /// Certificates held by this participant
    pub certificates: Vec<Certificate>,
    /// Registration timestamp
    pub registered_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity: Option<DateTime<Utc>>,
    /// Reputation score (0.0 to 1.0)
    pub reputation: f64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Contact information for participants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub website: Option<String>,
}

/// Wallet containing cryptographic keys and participant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    /// Participant information
    pub participant: Participant,
    /// Signing key for transactions (stored encrypted)
    #[serde(skip)]
    pub signing_key: Option<SigningKey>,
    /// Public key for verification
    pub public_key: VerifyingKey,
    /// Key derivation path (for HD wallets)
    pub derivation_path: Option<String>,
    /// Wallet creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last backup timestamp
    pub last_backup: Option<DateTime<Utc>>,
    /// Shared secrets for data privacy (KeyID -> HexEncodedSecret)
    pub shared_secrets: HashMap<String, String>,
}

impl Wallet {
    /// Create a new wallet for a participant
    pub fn new(participant: Participant) -> Self {
        let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
        let public_key = signing_key.verifying_key();

        Self {
            participant,
            signing_key: Some(signing_key),
            public_key,
            derivation_path: None,
            created_at: Utc::now(),
            last_backup: None,
            shared_secrets: HashMap::new(),
        }
    }

    /// Create a wallet from an existing signing key
    pub fn from_signing_key(participant: Participant, signing_key: SigningKey) -> Self {
        let public_key = signing_key.verifying_key();

        Self {
            participant,
            signing_key: Some(signing_key),
            public_key,
            derivation_path: None,
            created_at: Utc::now(),
            last_backup: None,
            shared_secrets: HashMap::new(),
        }
    }

    /// Add a shared secret for data privacy
    pub fn add_secret(&mut self, key_id: String, secret: String) {
        self.shared_secrets.insert(key_id, secret);
    }

    /// Get a shared secret by Key ID
    pub fn get_secret(&self, key_id: &str) -> Option<&String> {
        self.shared_secrets.get(key_id)
    }

    /// Get the participant ID
    pub fn participant_id(&self) -> Uuid {
        self.participant.id
    }

    /// Get the participant type
    pub fn participant_type(&self) -> &ParticipantType {
        &self.participant.participant_type
    }

    /// Check if the wallet has permission for an operation
    pub fn has_permission(&self, operation: &str) -> bool {
        match operation {
            "produce" => self.participant.permissions.can_produce,
            "process" => self.participant.permissions.can_process,
            "transport" => self.participant.permissions.can_transport,
            "quality_test" => self.participant.permissions.can_quality_test,
            "audit" => self.participant.permissions.can_audit,
            "transfer" => self.participant.permissions.can_transfer,
            "view_all" => self.participant.permissions.can_view_all,
            "manage_participants" => self.participant.permissions.can_manage_participants,
            _ => false,
        }
    }

    /// Sign data with the wallet's private key
    pub fn sign(&self, data: &[u8]) -> Result<Signature> {
        let signing_key = self
            .signing_key
            .as_ref()
            .ok_or_else(|| anyhow!("No signing key available"))?;

        Ok(signing_key.sign(data))
    }

    /// Verify a signature against this wallet's public key
    pub fn verify(&self, data: &[u8], signature: &Signature) -> bool {
        use ed25519_dalek::Verifier;
        self.public_key.verify(data, signature).is_ok()
    }

    /// Update last activity timestamp
    pub fn update_activity(&mut self) {
        self.participant.last_activity = Some(Utc::now());
    }

    /// Add a certificate to the participant
    pub fn add_certificate(&mut self, certificate: Certificate) {
        self.participant.certificates.push(certificate);
    }

    /// Check if participant has a valid certificate of a specific type
    pub fn has_valid_certificate(&self, cert_type: &str) -> bool {
        let now = Utc::now();
        self.participant.certificates.iter().any(|cert| {
            cert.cert_type == cert_type
                && cert.status == CertificateStatus::Active
                && cert.expires_at > now
        })
    }

    /// Get active certificates
    pub fn get_active_certificates(&self) -> Vec<&Certificate> {
        let now = Utc::now();
        self.participant
            .certificates
            .iter()
            .filter(|cert| cert.status == CertificateStatus::Active && cert.expires_at > now)
            .collect()
    }
}

/// Wallet manager for handling multiple participant wallets
#[derive(Debug)]
pub struct WalletManager {
    /// Map of participant ID to wallet
    wallets: HashMap<Uuid, Wallet>,
    /// Storage directory for wallet files
    storage_dir: PathBuf,
    /// Encryption key for wallet storage (in production, this should be derived from user input)
    #[allow(dead_code)]
    encryption_key: [u8; 32],
}

impl WalletManager {
    /// Create a new wallet manager
    pub fn new<P: AsRef<Path>>(storage_dir: P) -> Result<Self> {
        let storage_dir = storage_dir.as_ref().to_path_buf();

        // Create storage directory if it doesn't exist
        if !storage_dir.exists() {
            fs::create_dir_all(&storage_dir)?;
        }

        // In production, this should be derived from user input or secure key management
        let encryption_key = rand::random::<[u8; 32]>();

        Ok(Self {
            wallets: HashMap::new(),
            storage_dir,
            encryption_key,
        })
    }

    /// Create a new participant wallet
    pub fn create_wallet(&mut self, participant: Participant) -> Result<Uuid> {
        let participant_id = participant.id;
        let wallet = Wallet::new(participant);

        // Save wallet to disk
        self.save_wallet(&wallet)?;

        // Add to memory
        self.wallets.insert(participant_id, wallet);

        Ok(participant_id)
    }

    /// Load a wallet from storage
    pub fn load_wallet(&mut self, participant_id: Uuid) -> Result<()> {
        let wallet_path = self.get_wallet_path(participant_id);

        if !wallet_path.exists() {
            return Err(anyhow!(
                "Wallet file not found for participant {}",
                participant_id
            ));
        }

        let wallet_data = fs::read(&wallet_path)?;
        let wallet = self.decrypt_wallet_data(&wallet_data)?;

        self.wallets.insert(participant_id, wallet);
        Ok(())
    }

    /// Save a wallet to storage
    pub fn save_wallet(&self, wallet: &Wallet) -> Result<()> {
        let wallet_path = self.get_wallet_path(wallet.participant_id());
        let encrypted_data = self.encrypt_wallet_data(wallet)?;

        fs::write(wallet_path, encrypted_data)?;
        Ok(())
    }

    /// Get a wallet by participant ID
    pub fn get_wallet(&self, participant_id: Uuid) -> Option<&Wallet> {
        self.wallets.get(&participant_id)
    }

    /// Get a mutable wallet by participant ID
    pub fn get_wallet_mut(&mut self, participant_id: Uuid) -> Option<&mut Wallet> {
        self.wallets.get_mut(&participant_id)
    }

    /// List all participant IDs
    pub fn list_participants(&self) -> Vec<Uuid> {
        self.wallets.keys().cloned().collect()
    }

    /// Get participants by type
    pub fn get_participants_by_type(&self, participant_type: &ParticipantType) -> Vec<&Wallet> {
        self.wallets
            .values()
            .filter(|wallet| &wallet.participant.participant_type == participant_type)
            .collect()
    }

    /// Remove a wallet
    pub fn remove_wallet(&mut self, participant_id: Uuid) -> Result<()> {
        // Remove from memory
        self.wallets.remove(&participant_id);

        // Remove from storage
        let wallet_path = self.get_wallet_path(participant_id);
        if wallet_path.exists() {
            fs::remove_file(wallet_path)?;
        }

        Ok(())
    }

    /// Create a backup of all wallets
    pub fn create_backup(&self) -> Result<String> {
        let backup_dir = self.storage_dir.join("backups");
        if !backup_dir.exists() {
            fs::create_dir_all(&backup_dir)?;
        }

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = backup_dir.join(format!("wallet_backup_{}.json", timestamp));

        let backup_data = serde_json::to_string_pretty(&self.wallets)?;
        fs::write(&backup_path, backup_data)?;

        Ok(backup_path.to_string_lossy().to_string())
    }

    /// Get wallet file path
    fn get_wallet_path(&self, participant_id: Uuid) -> PathBuf {
        self.storage_dir.join(format!("{}.wallet", participant_id))
    }

    /// Encrypt wallet data (simplified - in production use proper encryption)
    fn encrypt_wallet_data(&self, wallet: &Wallet) -> Result<Vec<u8>> {
        // In production, implement proper encryption
        let json_data = serde_json::to_string(wallet)?;
        Ok(json_data.into_bytes())
    }

    /// Decrypt wallet data (simplified - in production use proper decryption)
    fn decrypt_wallet_data(&self, data: &[u8]) -> Result<Wallet> {
        // In production, implement proper decryption
        let json_str = String::from_utf8(data.to_vec())?;
        let wallet: Wallet = serde_json::from_str(&json_str)?;
        Ok(wallet)
    }

    /// Get wallet statistics
    pub fn get_statistics(&self) -> WalletManagerStats {
        let mut type_counts = HashMap::new();
        let mut total_certificates = 0;
        let mut active_participants = 0;

        for wallet in self.wallets.values() {
            *type_counts
                .entry(wallet.participant.participant_type.clone())
                .or_insert(0) += 1;
            total_certificates += wallet.participant.certificates.len();

            if wallet.participant.last_activity.is_some() {
                active_participants += 1;
            }
        }

        WalletManagerStats {
            total_participants: self.wallets.len(),
            type_distribution: type_counts,
            total_certificates,
            active_participants,
        }
    }
}

/// Wallet manager statistics
#[derive(Debug, Clone)]
pub struct WalletManagerStats {
    pub total_participants: usize,
    pub type_distribution: HashMap<ParticipantType, usize>,
    pub total_certificates: usize,
    pub active_participants: usize,
}

/// Helper functions for creating demo participants
impl Participant {
    /// Create a farmer participant
    pub fn new_farmer(name: String, location: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            participant_type: ParticipantType::Producer,
            contact_info: ContactInfo {
                email: None,
                phone: None,
                address: Some(location.clone()),
                website: None,
            },
            location: Some(location),
            permissions: ParticipantPermissions::for_type(&ParticipantType::Producer),
            certificates: vec![],
            registered_at: Utc::now(),
            last_activity: None,
            reputation: 1.0,
            metadata: HashMap::new(),
        }
    }

    /// Create a UHT manufacturer participant
    pub fn new_uht_manufacturer(name: String, location: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            participant_type: ParticipantType::Manufacturer,
            contact_info: ContactInfo {
                email: None,
                phone: None,
                address: Some(location.clone()),
                website: None,
            },
            location: Some(location),
            permissions: ParticipantPermissions::for_type(&ParticipantType::Manufacturer),
            certificates: vec![],
            registered_at: Utc::now(),
            last_activity: None,
            reputation: 1.0,
            metadata: HashMap::new(),
        }
    }

    /// Create a logistics provider participant
    pub fn new_logistics_provider(name: String, location: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            participant_type: ParticipantType::LogisticsProvider,
            contact_info: ContactInfo {
                email: None,
                phone: None,
                address: Some(location.clone()),
                website: None,
            },
            location: Some(location),
            permissions: ParticipantPermissions::for_type(&ParticipantType::LogisticsProvider),
            certificates: vec![],
            registered_at: Utc::now(),
            last_activity: None,
            reputation: 1.0,
            metadata: HashMap::new(),
        }
    }

    /// Create a quality lab participant
    pub fn new_quality_lab(name: String, location: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            participant_type: ParticipantType::QualityLab,
            contact_info: ContactInfo {
                email: None,
                phone: None,
                address: Some(location.clone()),
                website: None,
            },
            location: Some(location),
            permissions: ParticipantPermissions::for_type(&ParticipantType::QualityLab),
            certificates: vec![],
            registered_at: Utc::now(),
            last_activity: None,
            reputation: 1.0,
            metadata: HashMap::new(),
        }
    }

    /// Create a retailer participant
    pub fn new_retailer(name: String, location: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            participant_type: ParticipantType::Retailer,
            contact_info: ContactInfo {
                email: None,
                phone: None,
                address: Some(location.clone()),
                website: None,
            },
            location: Some(location),
            permissions: ParticipantPermissions::for_type(&ParticipantType::Retailer),
            certificates: vec![],
            registered_at: Utc::now(),
            last_activity: None,
            reputation: 1.0,
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_participant_creation() {
        let farmer =
            Participant::new_farmer("John's Dairy Farm".to_string(), "Vermont, USA".to_string());

        assert_eq!(farmer.participant_type, ParticipantType::Producer);
        assert!(farmer.permissions.can_produce);
        assert!(!farmer.permissions.can_process);
    }

    #[test]
    fn test_wallet_creation() {
        let farmer =
            Participant::new_farmer("John's Dairy Farm".to_string(), "Vermont, USA".to_string());

        let wallet = Wallet::new(farmer);
        assert!(wallet.signing_key.is_some());
        assert!(wallet.has_permission("produce"));
        assert!(!wallet.has_permission("process"));
    }

    #[test]
    fn test_wallet_signing() {
        let farmer =
            Participant::new_farmer("John's Dairy Farm".to_string(), "Vermont, USA".to_string());

        let wallet = Wallet::new(farmer);
        let data = b"test message";

        let signature = wallet.sign(data).unwrap();
        assert!(wallet.verify(data, &signature));
    }

    #[test]
    fn test_wallet_manager() {
        let temp_dir = tempdir().unwrap();
        let mut manager = WalletManager::new(temp_dir.path()).unwrap();

        let farmer =
            Participant::new_farmer("John's Dairy Farm".to_string(), "Vermont, USA".to_string());

        let participant_id = manager.create_wallet(farmer).unwrap();
        assert!(manager.get_wallet(participant_id).is_some());

        let stats = manager.get_statistics();
        assert_eq!(stats.total_participants, 1);
    }

    #[test]
    fn test_certificate_management() {
        let farmer =
            Participant::new_farmer("John's Dairy Farm".to_string(), "Vermont, USA".to_string());

        let mut wallet = Wallet::new(farmer);

        let cert = Certificate {
            id: "ORGANIC-001".to_string(),
            cert_type: "ORGANIC".to_string(),
            issuer: "USDA".to_string(),
            issued_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::days(365),
            status: CertificateStatus::Active,
            metadata: HashMap::new(),
        };

        wallet.add_certificate(cert);
        assert!(wallet.has_valid_certificate("ORGANIC"));
        assert!(!wallet.has_valid_certificate("FDA_APPROVED"));
    }
}
