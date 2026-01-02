use provchain_org::core::blockchain::Blockchain;
use provchain_org::interop::bridge::BridgeManager;
use provchain_org::network::consensus::ConsensusManager;
use provchain_org::network::NetworkManager;
use provchain_org::utils::config::{ConsensusConfig, NodeConfig};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[tokio::test]
async fn test_consensus_switching_mechanism() {
    // 1. Test PoA Configuration
    let mut poa_config = ConsensusConfig::default();
    poa_config.consensus_type = "poa".to_string();

    let node_config = NodeConfig::default();
    let network = Arc::new(NetworkManager::new(node_config.clone()));
    let blockchain = Arc::new(RwLock::new(Blockchain::new()));

    let poa_manager = ConsensusManager::new(poa_config, network.clone(), blockchain.clone())
        .await
        .expect("Failed to create PoA manager");

    let stats = poa_manager.get_consensus_stats().await;
    assert_eq!(stats.protocol_type, "PoA", "Should be running PoA protocol");

    // 2. Test PBFT Configuration
    let mut pbft_config = ConsensusConfig::default();
    pbft_config.consensus_type = "pbft".to_string();

    let pbft_manager = ConsensusManager::new(pbft_config, network.clone(), blockchain.clone())
        .await
        .expect("Failed to create PBFT manager");

    let stats_pbft = pbft_manager.get_consensus_stats().await;
    assert_eq!(stats_pbft.protocol_type, "PBFT", "Should be running PBFT protocol");
}

#[tokio::test]
async fn test_cross_chain_foundation() {
    // Setup Source Chain
    let source_chain = Arc::new(RwLock::new(Blockchain::new()));
    let source_bridge = BridgeManager::new(source_chain.clone());

    // Setup Destination Chain
    let dest_chain = Arc::new(RwLock::new(Blockchain::new()));
    let dest_bridge = BridgeManager::new(dest_chain.clone());

    // 1. Create a "transfer" block on Source Chain
    // In a real scenario, this block would contain specific burn/lock transaction data
    let mut source_chain_write = source_chain.write().await;
    let index = 1;
    let data = "TRANSFER_ASSET_TO_DEST_NET";
    let previous_hash = "0".repeat(64);
    let state_root = "root_hash".to_string();
    let node_id = Uuid::new_v4().to_string();
    
    // Create a mock block manually for testing
    let mut block = provchain_org::core::blockchain::Block::new(
        index, 
        data.to_string(), 
        previous_hash, 
        state_root, 
        node_id
    );
    // Mock signature (usually done by authority)
    block.signature = hex::encode([0u8; 64]); 
    source_chain_write.chain.push(block);
    drop(source_chain_write); // release lock

    // 2. Export Proof from Source
    let transfer_id = Uuid::new_v4();
    let source_signing_key = ed25519_dalek::SigningKey::from_bytes(&[2u8; 32]);
    let proof = source_bridge.export_proof(1, transfer_id, &source_signing_key)
        .await
        .expect("Failed to export proof");

    assert_eq!(proof.message.transfer_id, transfer_id);
    assert_eq!(proof.message.payload, data);

    // 3. Import Proof to Destination
    // First, register a trusted authority so the bridge accepts the proof
    dest_bridge.add_trusted_authority("local-net", source_signing_key.verifying_key().as_bytes())
        .await
        .expect("Failed to add trusted authority");

    let result = dest_bridge.import_proof(&proof).await.expect("Import failed");
    
    assert!(result, "Proof should be accepted and verified");
}
