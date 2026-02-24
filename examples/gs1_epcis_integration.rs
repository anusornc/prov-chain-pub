//! GS1 EPCIS Integration Example for UHT Supply Chain
//!
//! This example demonstrates:
//! 1. Loading GS1 EPCIS ontologies
//! 2. Creating EPCIS-compliant UHT supply chain events
//! 3. Exporting to EPCIS JSON-LD format
//!
//! ## Running the Example
//!
//! ```bash
//! cargo run --example gs1_epcis_integration
//! ```

use provchain_org::semantic::gs1_epcis::{
    biz_steps, create_epcis_document, dispositions, EpcisEventBuilder, EpcisEventType,
    generate_uht_supply_chain_events, namespaces,
};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║           GS1 EPCIS Integration for UHT Supply Chain                ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();

    // Step 1: Show namespaces
    println!("📦 Step 1: GS1 EPCIS Namespaces");
    println!("   EPCIS: {}", namespaces::EPCIS);
    println!("   CBV:   {}", namespaces::CBV);
    println!("   GS1:   {}", namespaces::GS1);
    println!("   UHT:   {}", namespaces::UHT);

    // Step 2: Check ontologies
    println!("\n📚 Step 2: Checking GS1 EPCIS Ontologies...");
    let ontology_dir = Path::new("docs/ontologies/gs1_epcis");
    
    if ontology_dir.exists() {
        println!("   ✓ Ontology directory found");
        
        let files = ["epcis.ttl", "cbv.ttl", "epcis-shacl.ttl", "gs1-web-vocab.ttl"];
        for file in &files {
            let path = ontology_dir.join(file);
            if path.exists() {
                let size = std::fs::metadata(&path)?.len();
                println!("   ✓ {} ({} KB)", file, size / 1024);
            } else {
                println!("   ✗ {} not found", file);
            }
        }
    } else {
        println!("   ⚠ Ontology directory not found at {:?}", ontology_dir);
    }

    // Step 3: Check UHT extension ontology
    let uht_ontology = Path::new("docs/ontologies/uht-supply-chain.ttl");
    if uht_ontology.exists() {
        println!("\n   ✓ UHT Supply Chain Ontology found");
    }

    // Step 4: Create UHT supply chain events
    println!("\n🥛 Step 3: Creating UHT Supply Chain Events...");
    let batch_id = "UHT-BATCH-2024-001";
    
    let events = vec![
        create_milk_collection_event(batch_id),
        create_quality_test_event(batch_id),
        create_uht_processing_event(batch_id),
        create_packaging_event(batch_id),
        create_cold_storage_event(batch_id),
        create_shipping_event(batch_id),
        create_receiving_event(batch_id),
    ];

    println!("   ✓ Created {} supply chain events", events.len());

    // Step 5: Display events in different formats
    println!("\n📝 Step 4: Event Formats Demo...");
    
    println!("\n   [Turtle RDF Format - Event 1: Milk Collection]");
    println!("   {:-<60}", "");
    println!("{}", events[0].0);
    println!();

    println!("   [JSON-LD Format - Event 1: Milk Collection]");
    println!("   {:-<60}", "");
    let json_str = serde_json::to_string_pretty(&events[0].1)?;
    println!("{}", json_str);
    println!();

    // Step 6: Create full EPCIS Document
    println!("📄 Step 5: Creating EPCIS Document...");
    let jsonld_events: Vec<_> = events.iter().map(|(_, jsonld)| jsonld.clone()).collect();
    let epcis_doc = create_epcis_document(jsonld_events);
    
    println!("   ✓ EPCIS Document created with {} events", events.len());
    
    // Save to file
    let output_path = "data/gs1_epcis_uht_demo.json";
    std::fs::create_dir_all("data")?;
    std::fs::write(output_path, serde_json::to_string_pretty(&epcis_doc)?)?;
    println!("   ✓ Saved to: {}", output_path);

    // Step 7: Generate events using helper function
    println!("\n🔄 Step 6: Auto-generated Supply Chain Events...");
    let auto_events = generate_uht_supply_chain_events(batch_id);
    println!("   ✓ Generated {} events using helper function", auto_events.len());
    
    for (i, event) in auto_events.iter().enumerate() {
        println!("   Event {}: {} - {}", 
            i + 1,
            event["@type"].as_str().unwrap_or("Unknown"),
            event.get("epcis:bizStep")
                .and_then(|v| v.as_str())
                .map(|s| s.split('/').last().unwrap_or(s))
                .unwrap_or("N/A")
        );
    }

    // Step 8: Business Vocabulary Reference
    println!("\n📚 Step 7: CBV (Core Business Vocabulary) Reference");
    println!("   {:-<60}", "");
    println!("   Business Steps:");
    println!("     • commissioning     - Milk collection");
    println!("     • inspecting        - Quality testing");
    println!("     • creating_class_instance - UHT processing");
    println!("     • packing           - Packaging into cartons");
    println!("     • storing           - Cold storage");
    println!("     • shipping          - Transport to DC");
    println!("     • receiving         - DC receipt");
    println!();
    println!("   Dispositions:");
    println!("     • active            - New batch created");
    println!("     • in_progress       - Processing");
    println!("     • in_transit        - Transporting");
    println!("     • conformant        - Passed quality test");

    // Summary
    println!("\n╔══════════════════════════════════════════════════════════════════════╗");
    println!("║                        ✅ DEMO COMPLETE                              ║");
    println!("╠══════════════════════════════════════════════════════════════════════╣");
    println!("║  GS1 EPCIS Integration Features Demonstrated:                        ║");
    println!("║  • EPCIS 2.0 compliant event creation                                ║");
    println!("║  • CBV business steps and dispositions                               ║");
    println!("║  • Turtle RDF and JSON-LD export                                     ║");
    println!("║  • UHT supply chain specific extensions                              ║");
    println!("║  • EPCIS Document generation                                         ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");

    Ok(())
}

/// Create milk collection event
fn create_milk_collection_event(batch_id: &str) -> (String, serde_json::Value) {
    let builder = EpcisEventBuilder::new(EpcisEventType::ObjectEvent)
        .action("ADD")
        .biz_step(biz_steps::COMMISSIONING)
        .disposition(dispositions::ACTIVE)
        .read_point("urn:epc:id:sgln:1234567.12345.1")
        .biz_location("urn:epc:id:sgln:1234567.12345.0")
        .batch_number(batch_id);

    let event_id = format!("https://provchain.org/uht/{}/collection", batch_id);
    (
        builder.build_turtle(&event_id),
        builder.build_jsonld(&event_id),
    )
}

/// Create quality test event
fn create_quality_test_event(batch_id: &str) -> (String, serde_json::Value) {
    let builder = EpcisEventBuilder::new(EpcisEventType::ObjectEvent)
        .action("OBSERVE")
        .biz_step(biz_steps::INSPECTING)
        .disposition(dispositions::CONFORMANT)
        .read_point("urn:epc:id:sgln:1234567.12346.1")
        .biz_location("urn:epc:id:sgln:1234567.12346.0")
        .batch_number(batch_id);

    let event_id = format!("https://provchain.org/uht/{}/quality-test", batch_id);
    (
        builder.build_turtle(&event_id),
        builder.build_jsonld(&event_id),
    )
}

/// Create UHT processing event
fn create_uht_processing_event(batch_id: &str) -> (String, serde_json::Value) {
    let builder = EpcisEventBuilder::new(EpcisEventType::TransformationEvent)
        .biz_step(biz_steps::CREATING_CLASS_INSTANCE)
        .disposition(dispositions::IN_PROGRESS)
        .read_point("urn:epc:id:sgln:1234567.12347.1")
        .biz_location("urn:epc:id:sgln:1234567.12347.0")
        .batch_number(batch_id);

    let event_id = format!("https://provchain.org/uht/{}/processing", batch_id);
    (
        builder.build_turtle(&event_id),
        builder.build_jsonld(&event_id),
    )
}

/// Create packaging event
fn create_packaging_event(batch_id: &str) -> (String, serde_json::Value) {
    let builder = EpcisEventBuilder::new(EpcisEventType::AggregationEvent)
        .action("ADD")
        .biz_step(biz_steps::PACKING)
        .disposition(dispositions::IN_PROGRESS)
        .read_point("urn:epc:id:sgln:1234567.12347.2")
        .biz_location("urn:epc:id:sgln:1234567.12347.0")
        .batch_number(batch_id);

    let event_id = format!("https://provchain.org/uht/{}/packaging", batch_id);
    (
        builder.build_turtle(&event_id),
        builder.build_jsonld(&event_id),
    )
}

/// Create cold storage event
fn create_cold_storage_event(batch_id: &str) -> (String, serde_json::Value) {
    let builder = EpcisEventBuilder::new(EpcisEventType::ObjectEvent)
        .action("OBSERVE")
        .biz_step(biz_steps::STORING)
        .disposition(dispositions::IN_PROGRESS)
        .read_point("urn:epc:id:sgln:1234567.12347.3")
        .biz_location("urn:epc:id:sgln:1234567.12347.0")
        .batch_number(batch_id);

    let event_id = format!("https://provchain.org/uht/{}/storage", batch_id);
    (
        builder.build_turtle(&event_id),
        builder.build_jsonld(&event_id),
    )
}

/// Create shipping event
fn create_shipping_event(batch_id: &str) -> (String, serde_json::Value) {
    let builder = EpcisEventBuilder::new(EpcisEventType::ObjectEvent)
        .action("OBSERVE")
        .biz_step(biz_steps::SHIPPING)
        .disposition(dispositions::IN_TRANSIT)
        .read_point("urn:epc:id:sgln:1234567.12347.4")
        .biz_location("urn:epc:id:sgln:1234567.12347.0")
        .batch_number(batch_id);

    let event_id = format!("https://provchain.org/uht/{}/shipping", batch_id);
    (
        builder.build_turtle(&event_id),
        builder.build_jsonld(&event_id),
    )
}

/// Create receiving event at DC
fn create_receiving_event(batch_id: &str) -> (String, serde_json::Value) {
    let builder = EpcisEventBuilder::new(EpcisEventType::ObjectEvent)
        .action("OBSERVE")
        .biz_step(biz_steps::RECEIVING)
        .disposition(dispositions::IN_PROGRESS)
        .read_point("urn:epc:id:sgln:1234567.12348.1")
        .biz_location("urn:epc:id:sgln:1234567.12348.0")
        .batch_number(batch_id);

    let event_id = format!("https://provchain.org/uht/{}/receiving", batch_id);
    (
        builder.build_turtle(&event_id),
        builder.build_jsonld(&event_id),
    )
}
