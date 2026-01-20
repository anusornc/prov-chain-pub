//! UHT Manufacturing Supply Chain Demo
//!
//! This module demonstrates a complete UHT milk manufacturing supply chain
//! with multiple participants, transactions, and traceability features.

use anyhow::Result;
use chrono::{Duration, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use crate::transaction::blockchain::TransactionBlockchain;
use crate::transaction::transaction::EnvironmentalConditions;
use crate::wallet::{Certificate, CertificateStatus, Participant};

/// UHT Manufacturing Demo
pub struct UHTDemo {
    /// The transaction blockchain
    pub blockchain: TransactionBlockchain,
    /// Participant IDs for easy reference
    pub participants: DemoParticipants,
}

/// Demo participant IDs
#[derive(Debug, Clone)]
pub struct DemoParticipants {
    pub farmer_john: Uuid,
    pub farmer_mary: Uuid,
    pub uht_processor: Uuid,
    pub quality_lab: Uuid,
    pub logistics_provider: Uuid,
    pub retailer: Uuid,
}

impl UHTDemo {
    /// Create a new UHT manufacturing demo
    pub fn new(data_dir: &str) -> Result<Self> {
        let mut blockchain = TransactionBlockchain::new(data_dir)?;

        // Register all participants
        let participants = Self::setup_participants(&mut blockchain)?;

        Ok(Self {
            blockchain,
            participants,
        })
    }

    /// Setup demo participants with certificates
    fn setup_participants(blockchain: &mut TransactionBlockchain) -> Result<DemoParticipants> {
        println!("🏭 Setting up UHT Manufacturing Supply Chain Participants...");

        // Create Farmer John
        let mut farmer_john = Participant::new_farmer(
            "John's Organic Dairy Farm".to_string(),
            "Vermont, USA".to_string(),
        );

        // Add organic certification
        farmer_john.certificates.push(Certificate {
            id: "USDA-ORG-2025-001".to_string(),
            cert_type: "ORGANIC".to_string(),
            issuer: "USDA Organic Program".to_string(),
            issued_at: Utc::now() - Duration::days(30),
            expires_at: Utc::now() + Duration::days(335),
            status: CertificateStatus::Active,
            metadata: HashMap::new(),
        });

        let farmer_john_id = blockchain.register_participant(farmer_john)?;
        println!(
            "✅ Registered Farmer John (Organic Dairy) - ID: {}",
            farmer_john_id
        );

        // Create Farmer Mary
        let farmer_mary = Participant::new_farmer(
            "Mary's Premium Dairy".to_string(),
            "Wisconsin, USA".to_string(),
        );
        let farmer_mary_id = blockchain.register_participant(farmer_mary)?;
        println!(
            "✅ Registered Farmer Mary (Premium Dairy) - ID: {}",
            farmer_mary_id
        );

        // Create UHT Processor
        let mut uht_processor = Participant::new_uht_manufacturer(
            "Alpine UHT Processing Corp".to_string(),
            "Wisconsin, USA".to_string(),
        );

        // Add FDA certification
        uht_processor.certificates.push(Certificate {
            id: "FDA-DAIRY-2025-042".to_string(),
            cert_type: "FDA_APPROVED".to_string(),
            issuer: "U.S. Food and Drug Administration".to_string(),
            issued_at: Utc::now() - Duration::days(60),
            expires_at: Utc::now() + Duration::days(305),
            status: CertificateStatus::Active,
            metadata: HashMap::new(),
        });

        let uht_processor_id = blockchain.register_participant(uht_processor)?;
        println!(
            "✅ Registered UHT Processor (Alpine Corp) - ID: {}",
            uht_processor_id
        );

        // Create Quality Lab
        let mut quality_lab = Participant::new_quality_lab(
            "Midwest Dairy Testing Laboratory".to_string(),
            "Illinois, USA".to_string(),
        );

        // Add ISO certification
        quality_lab.certificates.push(Certificate {
            id: "ISO-17025-2025-078".to_string(),
            cert_type: "ISO_17025".to_string(),
            issuer: "International Organization for Standardization".to_string(),
            issued_at: Utc::now() - Duration::days(90),
            expires_at: Utc::now() + Duration::days(275),
            status: CertificateStatus::Active,
            metadata: HashMap::new(),
        });

        let quality_lab_id = blockchain.register_participant(quality_lab)?;
        println!(
            "✅ Registered Quality Lab (Midwest Testing) - ID: {}",
            quality_lab_id
        );

        // Create Logistics Provider
        let logistics_provider = Participant::new_logistics_provider(
            "ColdChain Express Logistics".to_string(),
            "Illinois, USA".to_string(),
        );
        let logistics_provider_id = blockchain.register_participant(logistics_provider)?;
        println!(
            "✅ Registered Logistics Provider (ColdChain Express) - ID: {}",
            logistics_provider_id
        );

        // Create Retailer
        let retailer = Participant::new_retailer(
            "FreshMart Supermarket Chain".to_string(),
            "Nationwide, USA".to_string(),
        );
        let retailer_id = blockchain.register_participant(retailer)?;
        println!("✅ Registered Retailer (FreshMart) - ID: {}", retailer_id);

        Ok(DemoParticipants {
            farmer_john: farmer_john_id,
            farmer_mary: farmer_mary_id,
            uht_processor: uht_processor_id,
            quality_lab: quality_lab_id,
            logistics_provider: logistics_provider_id,
            retailer: retailer_id,
        })
    }

    /// Run the complete UHT manufacturing demo
    pub fn run_complete_demo(&mut self) -> Result<()> {
        println!("\n🥛 Starting Complete UHT Manufacturing Supply Chain Demo");
        println!("{}", "=".repeat(60));

        // Step 1: Milk Production
        self.demo_milk_production()?;

        // Step 2: Quality Testing
        self.demo_quality_testing()?;

        // Step 3: UHT Processing
        self.demo_uht_processing()?;

        // Step 4: Post-Processing Quality Control
        self.demo_post_processing_quality()?;

        // Step 5: Transport to Distribution Center
        self.demo_transport_to_distribution()?;

        // Step 6: Final Distribution to Retailer
        self.demo_final_distribution()?;

        // Step 7: Create blocks and finalize
        self.finalize_transactions()?;

        // Step 8: Display results
        self.display_results()?;

        Ok(())
    }

    /// Demo milk production from farmers
    fn demo_milk_production(&mut self) -> Result<()> {
        println!("\n📍 Step 1: Milk Production");
        println!("{}", "-".repeat(30));

        // Farmer John produces organic milk
        let env_conditions_john = EnvironmentalConditions {
            temperature: Some(18.5), // Celsius
            humidity: Some(65.0),
            pressure: Some(1013.25),
            timestamp: Utc::now(),
            sensor_id: Some("FARM-SENSOR-001".to_string()),
        };

        let tx1 = self.blockchain.create_production_transaction(
            self.participants.farmer_john,
            "ORGANIC-MILK-BATCH-001".to_string(),
            2000.0, // 2000 liters
            "Vermont, USA".to_string(),
            Some(env_conditions_john),
            false,
        )?;

        let tx1_id = self.blockchain.submit_transaction(tx1)?;
        println!(
            "✅ Farmer John produced 2000L organic milk - TX: {}",
            tx1_id
        );

        // Farmer Mary produces premium milk
        let env_conditions_mary = EnvironmentalConditions {
            temperature: Some(16.2),
            humidity: Some(68.0),
            pressure: Some(1015.30),
            timestamp: Utc::now(),
            sensor_id: Some("FARM-SENSOR-002".to_string()),
        };

        let tx2 = self.blockchain.create_production_transaction(
            self.participants.farmer_mary,
            "PREMIUM-MILK-BATCH-001".to_string(),
            1500.0, // 1500 liters
            "Wisconsin, USA".to_string(),
            Some(env_conditions_mary),
            false,
        )?;

        let tx2_id = self.blockchain.submit_transaction(tx2)?;
        println!(
            "✅ Farmer Mary produced 1500L premium milk - TX: {}",
            tx2_id
        );

        Ok(())
    }

    /// Demo quality testing of raw milk
    fn demo_quality_testing(&mut self) -> Result<()> {
        println!("\n🔬 Step 2: Quality Testing of Raw Milk");
        println!("{}", "-".repeat(40));

        // Test organic milk batch
        let tx3 = self.blockchain.create_quality_transaction(
            self.participants.quality_lab,
            "ORGANIC-MILK-BATCH-001".to_string(),
            "MICROBIOLOGICAL".to_string(),
            "PASSED".to_string(),
            Some(5.2), // Bacterial count (log CFU/mL)
            false,
        )?;

        let tx3_id = self.blockchain.submit_transaction(tx3)?;
        println!(
            "✅ Quality test for organic milk batch - PASSED - TX: {}",
            tx3_id
        );

        // Test premium milk batch
        let tx4 = self.blockchain.create_quality_transaction(
            self.participants.quality_lab,
            "PREMIUM-MILK-BATCH-001".to_string(),
            "MICROBIOLOGICAL".to_string(),
            "PASSED".to_string(),
            Some(4.8), // Bacterial count (log CFU/mL)
            false,
        )?;

        let tx4_id = self.blockchain.submit_transaction(tx4)?;
        println!(
            "✅ Quality test for premium milk batch - PASSED - TX: {}",
            tx4_id
        );

        Ok(())
    }

    /// Demo UHT processing
    fn demo_uht_processing(&mut self) -> Result<()> {
        println!("\n🏭 Step 3: UHT Processing");
        println!("{}", "-".repeat(25));

        // UHT processing conditions
        let uht_conditions = EnvironmentalConditions {
            temperature: Some(138.0), // UHT temperature in Celsius
            humidity: Some(45.0),
            pressure: Some(1012.0),
            timestamp: Utc::now(),
            sensor_id: Some("UHT-PROCESSOR-001".to_string()),
        };

        // Process both milk batches together
        let tx5 = self.blockchain.create_processing_transaction(
            self.participants.uht_processor,
            vec![
                "ORGANIC-MILK-BATCH-001".to_string(),
                "PREMIUM-MILK-BATCH-001".to_string(),
            ],
            "UHT-MILK-BATCH-001".to_string(),
            "UHT_PASTEURIZATION".to_string(),
            Some(uht_conditions),
            false,
        )?;

        let tx5_id = self.blockchain.submit_transaction(tx5)?;
        println!(
            "✅ UHT processing completed - 3500L processed - TX: {}",
            tx5_id
        );

        Ok(())
    }

    /// Demo post-processing quality control
    fn demo_post_processing_quality(&mut self) -> Result<()> {
        println!("\n🔍 Step 4: Post-Processing Quality Control");
        println!("{}", "-".repeat(45));

        // Microbiological test
        let tx6 = self.blockchain.create_quality_transaction(
            self.participants.quality_lab,
            "UHT-MILK-BATCH-001".to_string(),
            "POST_UHT_MICROBIOLOGICAL".to_string(),
            "PASSED".to_string(),
            Some(0.0), // Should be sterile after UHT
            false,
        )?;

        let tx6_id = self.blockchain.submit_transaction(tx6)?;
        println!("✅ Post-UHT microbiological test - PASSED - TX: {}", tx6_id);

        // Nutritional analysis
        let tx7 = self.blockchain.create_quality_transaction(
            self.participants.quality_lab,
            "UHT-MILK-BATCH-001".to_string(),
            "NUTRITIONAL_ANALYSIS".to_string(),
            "PASSED".to_string(),
            Some(3.2), // Fat content percentage
            false,
        )?;

        let tx7_id = self.blockchain.submit_transaction(tx7)?;
        println!(
            "✅ Nutritional analysis - PASSED (3.2% fat) - TX: {}",
            tx7_id
        );

        Ok(())
    }

    /// Demo transport to distribution center
    fn demo_transport_to_distribution(&mut self) -> Result<()> {
        println!("\n🚛 Step 5: Transport to Distribution Center");
        println!("{}", "-".repeat(45));

        // Cold chain transport conditions
        let transport_conditions = EnvironmentalConditions {
            temperature: Some(4.0), // Refrigerated transport
            humidity: Some(60.0),
            pressure: Some(1010.0),
            timestamp: Utc::now(),
            sensor_id: Some("TRUCK-SENSOR-001".to_string()),
        };

        let tx8 = self.blockchain.create_transport_transaction(
            self.participants.logistics_provider,
            "UHT-MILK-BATCH-001".to_string(),
            "Alpine UHT Processing Corp, Wisconsin".to_string(),
            "ColdChain Distribution Center, Illinois".to_string(),
            Some(transport_conditions),
            false,
        )?;

        let tx8_id = self.blockchain.submit_transaction(tx8)?;
        println!(
            "✅ Transport to distribution center - Cold chain maintained - TX: {}",
            tx8_id
        );

        Ok(())
    }

    /// Demo final distribution to retailer
    fn demo_final_distribution(&mut self) -> Result<()> {
        println!("\n🏪 Step 6: Final Distribution to Retailer");
        println!("{}", "-".repeat(40));

        // Final distribution
        let final_transport_conditions = EnvironmentalConditions {
            temperature: Some(3.8),
            humidity: Some(58.0),
            pressure: Some(1008.0),
            timestamp: Utc::now(),
            sensor_id: Some("DELIVERY-TRUCK-001".to_string()),
        };

        let tx9 = self.blockchain.create_transport_transaction(
            self.participants.logistics_provider,
            "UHT-MILK-BATCH-001".to_string(),
            "ColdChain Distribution Center, Illinois".to_string(),
            "FreshMart Supermarket Chain, Nationwide".to_string(),
            Some(final_transport_conditions),
            false,
        )?;

        let tx9_id = self.blockchain.submit_transaction(tx9)?;
        println!("✅ Final distribution to retailer - TX: {}", tx9_id);

        Ok(())
    }

    /// Finalize all transactions by creating blocks
    fn finalize_transactions(&mut self) -> Result<()> {
        println!("\n⛓️  Step 7: Finalizing Transactions on Blockchain");
        println!("{}", "-".repeat(50));

        // Create blocks with all pending transactions
        // Use UHT processor as the validator for this block
        self.blockchain
            .create_block(10, self.participants.uht_processor)?; // Process up to 10 transactions per block

        // Save to disk
        self.blockchain.save_to_disk()?;

        println!("✅ All transactions finalized and saved to blockchain");

        Ok(())
    }

    /// Display comprehensive results
    fn display_results(&self) -> Result<()> {
        println!("\n📊 Demo Results and Statistics");
        println!("{}", "=".repeat(40));

        let stats = self.blockchain.get_statistics();

        println!("Blockchain Statistics:");
        println!("  📦 Total Blocks: {}", stats.total_blocks);
        println!("  ⏳ Pending Transactions: {}", stats.pending_transactions);
        println!("  👥 Total Participants: {}", stats.total_participants);
        println!("  💰 Total UTXOs: {}", stats.total_utxos);

        println!("\nParticipant Distribution:");
        for (participant_type, count) in &stats.participant_distribution {
            println!("  {:?}: {}", participant_type, count);
        }

        println!("\nTransaction Distribution:");
        for (tx_type, count) in &stats.transaction_distribution {
            println!("  {:?}: {}", tx_type, count);
        }

        println!("\nParticipant Details:");
        self.display_participant_info()?;

        println!("\n🎉 UHT Manufacturing Supply Chain Demo Completed Successfully!");
        println!("   Complete traceability from farm to shelf achieved.");
        println!("   All transactions signed and verified by participants.");
        println!("   Environmental conditions monitored throughout the chain.");
        println!("   Quality control checkpoints passed at each stage.");

        Ok(())
    }

    /// Display detailed participant information
    fn display_participant_info(&self) -> Result<()> {
        let participants = [
            ("Farmer John (Organic)", self.participants.farmer_john),
            ("Farmer Mary (Premium)", self.participants.farmer_mary),
            ("UHT Processor", self.participants.uht_processor),
            ("Quality Lab", self.participants.quality_lab),
            ("Logistics Provider", self.participants.logistics_provider),
            ("Retailer", self.participants.retailer),
        ];

        for (name, id) in participants {
            if let Some(wallet) = self.blockchain.get_participant_wallet(id) {
                println!("  {} ({})", name, id);
                println!("    Type: {:?}", wallet.participant.participant_type);
                println!(
                    "    Location: {}",
                    wallet.participant.location.as_deref().unwrap_or("N/A")
                );
                println!(
                    "    Certificates: {}",
                    wallet.participant.certificates.len()
                );

                for cert in &wallet.participant.certificates {
                    println!("      - {} ({:?})", cert.cert_type, cert.status);
                }
            }
        }

        Ok(())
    }

    /// Query traceability information for a specific batch
    pub fn trace_batch(&self, batch_id: &str) -> Result<()> {
        println!("\n🔍 Tracing Batch: {}", batch_id);
        println!("{}", "-".repeat(30));

        // In a full implementation, this would query the RDF store
        // For now, we'll show a simplified trace
        println!("Traceability information would be queried from the RDF store");
        println!(
            "showing the complete journey of batch {} from farm to shelf.",
            batch_id
        );

        Ok(())
    }

    /// Validate the entire blockchain
    pub fn validate_blockchain(&self) -> bool {
        self.blockchain.validate()
    }
}

/// Run the UHT manufacturing demo
pub fn run_uht_demo() -> Result<()> {
    println!("🥛 ProvChainOrg UHT Manufacturing Supply Chain Demo");
    println!("{}", "=".repeat(60));
    println!("Demonstrating complete milk traceability from farm to shelf");
    println!("with multiple participants, digital signatures, and environmental monitoring.\n");

    let mut demo = UHTDemo::new("./demo_data")?;
    demo.run_complete_demo()?;

    // Validate the blockchain
    if demo.validate_blockchain() {
        println!("\n✅ Blockchain validation: PASSED");
    } else {
        println!("\n❌ Blockchain validation: FAILED");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_uht_demo_setup() {
        let temp_dir = tempdir().unwrap();
        let demo = UHTDemo::new(temp_dir.path().to_str().unwrap()).unwrap();

        let stats = demo.blockchain.get_statistics();
        assert_eq!(stats.total_participants, 6);
        assert!(demo.validate_blockchain());
    }

    #[test]
    fn test_milk_production() {
        let temp_dir = tempdir().unwrap();
        let mut demo = UHTDemo::new(temp_dir.path().to_str().unwrap()).unwrap();

        assert!(demo.demo_milk_production().is_ok());

        let stats = demo.blockchain.get_statistics();
        assert_eq!(stats.pending_transactions, 2);
    }
}
