//! Demo Runner for ProvChainOrg
//!
//! This module provides a unified interface for running various demos
//! and showcasing the capabilities of the ProvChainOrg system.

use anyhow::Result;
use std::io::{self, Write};

use crate::uht_demo::run_uht_demo;

/// Available demo types
#[derive(Debug, Clone)]
pub enum DemoType {
    /// UHT Manufacturing Supply Chain Demo
    UHTManufacturing,
    /// Basic blockchain demo
    BasicBlockchain,
    /// Transaction signing demo
    TransactionSigning,
    /// Multi-participant demo
    MultiParticipant,
}

/// Demo runner for showcasing ProvChainOrg capabilities
pub struct DemoRunner {
    /// Available demos
    demos: Vec<(DemoType, String, String)>,
}

impl Default for DemoRunner {
    fn default() -> Self {
        let demos = vec![
            (
                DemoType::UHTManufacturing,
                "UHT Manufacturing Supply Chain".to_string(),
                "Complete milk traceability from farm to shelf with multiple participants"
                    .to_string(),
            ),
            (
                DemoType::BasicBlockchain,
                "Basic Blockchain Operations".to_string(),
                "Demonstrate basic blockchain creation, transaction submission, and validation"
                    .to_string(),
            ),
            (
                DemoType::TransactionSigning,
                "Transaction Signing & Validation".to_string(),
                "Show how transactions are signed by participants and validated".to_string(),
            ),
            (
                DemoType::MultiParticipant,
                "Multi-Participant Network".to_string(),
                "Demonstrate a network with multiple types of participants".to_string(),
            ),
        ];

        Self { demos }
    }
}

impl DemoRunner {
    /// Create a new demo runner
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the interactive demo menu
    pub fn run_interactive(&self) -> Result<()> {
        println!("🚀 ProvChainOrg Demo Runner");
        println!("{}", "=".repeat(50));
        println!("Welcome to the ProvChainOrg demonstration system!");
        println!("This system showcases blockchain-based supply chain traceability\n");

        loop {
            self.display_menu();

            print!("Enter your choice (1-{}, q to quit): ", self.demos.len());
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.eq_ignore_ascii_case("q") || input.eq_ignore_ascii_case("quit") {
                println!("Thank you for using ProvChainOrg Demo Runner!");
                break;
            }

            match input.parse::<usize>() {
                Ok(choice) if choice >= 1 && choice <= self.demos.len() => {
                    let demo_type = &self.demos[choice - 1].0;
                    self.run_demo(demo_type)?;

                    println!("\nPress Enter to continue...");
                    let mut _input = String::new();
                    io::stdin().read_line(&mut _input)?;
                }
                _ => {
                    println!("Invalid choice. Please try again.\n");
                }
            }
        }

        Ok(())
    }

    /// Display the demo menu
    fn display_menu(&self) {
        println!("\nAvailable Demos:");
        println!("{}", "-".repeat(30));

        for (i, (_, name, description)) in self.demos.iter().enumerate() {
            println!("{}. {}", i + 1, name);
            println!("   {}", description);
            println!();
        }
    }

    /// Run a specific demo
    pub fn run_demo(&self, demo_type: &DemoType) -> Result<()> {
        println!("\n{}", "=".repeat(60));

        match demo_type {
            DemoType::UHTManufacturing => {
                println!("🥛 Running UHT Manufacturing Supply Chain Demo");
                run_uht_demo()?;
            }
            DemoType::BasicBlockchain => {
                println!("⛓️  Running Basic Blockchain Demo");
                self.run_basic_blockchain_demo()?;
            }
            DemoType::TransactionSigning => {
                println!("✍️  Running Transaction Signing Demo");
                self.run_transaction_signing_demo()?;
            }
            DemoType::MultiParticipant => {
                println!("👥 Running Multi-Participant Network Demo");
                self.run_multi_participant_demo()?;
            }
        }

        println!("{}", "=".repeat(60));
        Ok(())
    }

    /// Run basic blockchain demo
    fn run_basic_blockchain_demo(&self) -> Result<()> {
        use crate::transaction::blockchain::TransactionBlockchain;
        use crate::wallet::Participant;

        println!("Creating a basic blockchain with simple transactions...");

        let mut blockchain = TransactionBlockchain::new("./demo_basic_blockchain")?;

        // Create a simple participant
        let farmer =
            Participant::new_farmer("Demo Farmer".to_string(), "Demo Location".to_string());

        let farmer_id = blockchain.register_participant(farmer)?;
        println!("✅ Registered demo farmer: {}", farmer_id);

        // Create a simple production transaction
        let tx = blockchain.create_production_transaction(
            farmer_id,
            "DEMO-BATCH-001".to_string(),
            100.0,
            "Demo Farm".to_string(),
            None,
            false,
        )?;

        let tx_id = blockchain.submit_transaction(tx)?;
        println!("✅ Created production transaction: {}", tx_id);

        // Create a block
        blockchain.create_block(10, farmer_id)?;
        println!("✅ Created block with transactions");

        // Display statistics
        let stats = blockchain.get_statistics();
        println!("\nBlockchain Statistics:");
        println!("  Blocks: {}", stats.total_blocks);
        println!("  Participants: {}", stats.total_participants);
        println!("  UTXOs: {}", stats.total_utxos);

        // Validate blockchain
        if blockchain.validate() {
            println!("✅ Blockchain validation: PASSED");
        } else {
            println!("❌ Blockchain validation: FAILED");
        }

        Ok(())
    }

    /// Run transaction signing demo
    fn run_transaction_signing_demo(&self) -> Result<()> {
        use crate::transaction::blockchain::TransactionBlockchain;
        use crate::wallet::Participant;

        println!("Demonstrating transaction signing and validation...");

        let mut blockchain = TransactionBlockchain::new("./demo_signing")?;

        // Create participants
        let farmer = Participant::new_farmer(
            "Signing Demo Farmer".to_string(),
            "Demo Location".to_string(),
        );

        let farmer_id = blockchain.register_participant(farmer)?;
        println!("✅ Registered farmer for signing demo: {}", farmer_id);

        // Create and sign a transaction
        let tx = blockchain.create_production_transaction(
            farmer_id,
            "SIGNED-BATCH-001".to_string(),
            200.0,
            "Demo Farm".to_string(),
            None,
            false,
        )?;

        println!("✅ Created transaction with digital signature");
        println!("   Transaction ID: {}", tx.id);
        println!("   Signatures: {}", tx.signatures.len());

        // Verify signatures
        if tx.verify_signatures()? {
            println!("✅ Transaction signatures verified successfully");
        } else {
            println!("❌ Transaction signature verification failed");
        }

        let tx_id = blockchain.submit_transaction(tx)?;
        println!("✅ Transaction submitted to blockchain: {}", tx_id);

        Ok(())
    }

    /// Run multi-participant demo
    fn run_multi_participant_demo(&self) -> Result<()> {
        use crate::transaction::blockchain::TransactionBlockchain;
        use crate::wallet::Participant;

        println!("Demonstrating multi-participant network...");

        let mut blockchain = TransactionBlockchain::new("./demo_multi_participant")?;

        // Create multiple participants
        let farmer =
            Participant::new_farmer("Multi-Demo Farmer".to_string(), "Farm Location".to_string());

        let processor = Participant::new_uht_manufacturer(
            "Multi-Demo Processor".to_string(),
            "Processing Plant".to_string(),
        );

        let lab = Participant::new_quality_lab(
            "Multi-Demo Lab".to_string(),
            "Testing Facility".to_string(),
        );

        let farmer_id = blockchain.register_participant(farmer)?;
        let processor_id = blockchain.register_participant(processor)?;
        let lab_id = blockchain.register_participant(lab)?;

        println!("✅ Registered {} participants", 3);

        // Create transactions between participants
        let tx1 = blockchain.create_production_transaction(
            farmer_id,
            "MULTI-BATCH-001".to_string(),
            300.0,
            "Farm Location".to_string(),
            None,
            false,
        )?;

        let tx1_id = blockchain.submit_transaction(tx1)?;
        println!("✅ Farmer created production transaction: {}", tx1_id);

        let tx2 = blockchain.create_quality_transaction(
            lab_id,
            "MULTI-BATCH-001".to_string(),
            "QUALITY_CHECK".to_string(),
            "PASSED".to_string(),
            Some(95.0),
            false,
        )?;

        let tx2_id = blockchain.submit_transaction(tx2)?;
        println!("✅ Lab created quality transaction: {}", tx2_id);

        let tx3 = blockchain.create_processing_transaction(
            processor_id,
            vec!["MULTI-BATCH-001".to_string()],
            "PROCESSED-BATCH-001".to_string(),
            "UHT_PROCESSING".to_string(),
            None,
            false,
        )?;

        let tx3_id = blockchain.submit_transaction(tx3)?;
        println!("✅ Processor created processing transaction: {}", tx3_id);

        // Create block and finalize
        blockchain.create_block(10, processor_id)?;
        println!("✅ Created block with all transactions");

        // Display network statistics
        let stats = blockchain.get_statistics();
        println!("\nMulti-Participant Network Statistics:");
        println!("  Total Participants: {}", stats.total_participants);
        println!("  Participant Types:");
        for (participant_type, count) in &stats.participant_distribution {
            println!("    {:?}: {}", participant_type, count);
        }
        println!("  Transaction Types:");
        for (tx_type, count) in &stats.transaction_distribution {
            println!("    {:?}: {}", tx_type, count);
        }

        Ok(())
    }

    /// Run all demos in sequence
    pub fn run_all_demos(&self) -> Result<()> {
        println!("🚀 Running All ProvChainOrg Demos");
        println!("{}", "=".repeat(50));

        for (demo_type, name, _) in &self.demos {
            println!("\n🎯 Starting: {}", name);
            self.run_demo(demo_type)?;
            println!("✅ Completed: {}", name);
        }

        println!("\n🎉 All demos completed successfully!");
        Ok(())
    }
}

/// Run the demo runner with command line arguments
pub fn run_demo_with_args(args: Vec<String>) -> Result<()> {
    let runner = DemoRunner::new();

    if args.len() < 2 {
        // No specific demo requested, run interactive mode
        runner.run_interactive()
    } else {
        match args[1].as_str() {
            "uht" | "manufacturing" => runner.run_demo(&DemoType::UHTManufacturing),
            "basic" | "blockchain" => runner.run_demo(&DemoType::BasicBlockchain),
            "signing" | "signatures" => runner.run_demo(&DemoType::TransactionSigning),
            "multi" | "participants" => runner.run_demo(&DemoType::MultiParticipant),
            "all" => runner.run_all_demos(),
            "interactive" | "menu" => runner.run_interactive(),
            _ => {
                println!("Unknown demo type: {}", args[1]);
                println!("Available options: uht, basic, signing, multi, all, interactive");
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn should_skip_demo_profile() -> bool {
        match std::env::var("PROVCHAIN_RUN_DEMO_TESTS") {
            Ok(value) if matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES") => false,
            _ => {
                eprintln!("skipping ignored demo test; set PROVCHAIN_RUN_DEMO_TESTS=1 to run this profile");
                true
            }
        }
    }

    #[test]
    fn test_demo_runner_creation() {
        let runner = DemoRunner::new();
        assert_eq!(runner.demos.len(), 4);
    }

    #[test]
    #[ignore]
    fn test_basic_blockchain_demo() {
        if should_skip_demo_profile() {
            return;
        }
        let runner = DemoRunner::new();
        assert!(runner.run_basic_blockchain_demo().is_ok());
    }

    #[test]
    #[ignore]
    fn test_transaction_signing_demo() {
        if should_skip_demo_profile() {
            return;
        }
        let runner = DemoRunner::new();
        assert!(runner.run_transaction_signing_demo().is_ok());
    }

    #[test]
    #[ignore]
    fn test_multi_participant_demo() {
        if should_skip_demo_profile() {
            return;
        }
        let runner = DemoRunner::new();
        assert!(runner.run_multi_participant_demo().is_ok());
    }
}
