//! EPCIS Data Generator for Supply Chain Testing
//! 
//! This module provides comprehensive data generation capabilities for EPCIS events
//! across different scales (small, medium, large) with realistic supply chain scenarios.

use crate::epcis::*;
use crate::*;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// EPCIS Data Generator with realistic supply chain simulation
pub struct EPCISDataGenerator {
    config: EPCISDataConfig,
    participants: Vec<SupplyChainParticipant>,
    locations: Vec<BusinessLocation>,
    read_points: Vec<ReadPoint>,
    rng: rand::rngs::ThreadRng,
    start_time: SystemTime,
}

impl EPCISDataGenerator {
    /// Create a new data generator with configuration
    pub fn new(config: EPCISDataConfig) -> Self {
        let mut generator = Self {
            config,
            participants: Vec::new(),
            locations: Vec::new(),
            read_points: Vec::new(),
            rng: rand::thread_rng(),
            start_time: SystemTime::now(),
        };
        
        generator.initialize_supply_chain();
        generator
    }

    /// Generate EPCIS ontology with events
    pub fn generate_ontology(&mut self) -> OwlResult<Ontology> {
        let mut ontology = Ontology::new();
        ontology.set_iri("http://example.org/epcis/supply-chain");

        // Add core EPCIS classes
        self.add_epcis_classes(&mut ontology)?;
        
        // Add participants
        self.add_participants_to_ontology(&mut ontology)?;
        
        // Add locations and read points
        self.add_locations_to_ontology(&mut ontology)?;
        
        // Generate events
        let event_count = self.calculate_event_count();
        let events = self.generate_events(event_count);
        
        // Add events to ontology
        for event in events {
            self.add_event_to_ontology(&mut ontology, &event)?;
        }

        Ok(ontology)
    }

    /// Generate events as separate data structure
    pub fn generate_events(&mut self, count: usize) -> Vec<EPCISEvent> {
        let mut events = Vec::new();
        let mut epc_pool = self.generate_epc_pool(count);

        for i in 0..count {
            let pattern = self.select_event_pattern();
            let event = self.generate_single_event(i, pattern, &mut epc_pool);
            events.push(event);
        }

        events
    }

    /// Generate small scale test data
    pub fn generate_small_scale() -> OwlResult<(Ontology, Vec<EPCISEvent>)> {
        let config = EPCISDataConfig::small_scale();
        let mut generator = Self::new(config);
        let ontology = generator.generate_ontology()?;
        let events = generator.generate_events(250); // Small scale
        Ok((ontology, events))
    }

    /// Generate medium scale test data
    pub fn generate_medium_scale() -> OwlResult<(Ontology, Vec<EPCISEvent>)> {
        let config = EPCISDataConfig::medium_scale();
        let mut generator = Self::new(config);
        let ontology = generator.generate_ontology()?;
        let events = generator.generate_events(3000); // Medium scale
        Ok((ontology, events))
    }

    /// Generate large scale test data
    pub fn generate_large_scale() -> OwlResult<(Ontology, Vec<EPCISEvent>)> {
        let config = EPCISDataConfig::large_scale();
        let mut generator = Self::new(config);
        let ontology = generator.generate_ontology()?;
        let events = generator.generate_events(25000); // Large scale
        Ok((ontology, events))
    }

    fn initialize_supply_chain(&mut self) {
        self.generate_participants();
        self.generate_locations();
        self.generate_read_points();
    }

    fn generate_participants(&mut self) {
        self.participants = match self.config.scale {
            DataScale::Small => {
                vec![
                    SupplyChainParticipant::new(
                        "mfg-001".to_string(),
                        "ACME Manufacturing".to_string(),
                        ParticipantRole::Manufacturer,
                    ),
                    SupplyChainParticipant::new(
                        "dist-001".to_string(),
                        "Global Distributors".to_string(),
                        ParticipantRole::Distributor,
                    ),
                    SupplyChainParticipant::new(
                        "ret-001".to_string(),
                        "City Supermarket".to_string(),
                        ParticipantRole::Retailer,
                    ),
                    SupplyChainParticipant::new(
                        "log-001".to_string(),
                        "FastShip Logistics".to_string(),
                        ParticipantRole::LogisticsProvider,
                    ),
                    SupplyChainParticipant::new(
                        "reg-001".to_string(),
                        "Quality Assurance Bureau".to_string(),
                        ParticipantRole::Regulator,
                    ),
                ]
            }
            DataScale::Medium => {
                let mut participants = Vec::new();
                // Manufacturers
                for i in 0..3 {
                    participants.push(SupplyChainParticipant::new(
                        format!("mfg-{:03}", i),
                        format!("Manufacturer {}", i + 1),
                        ParticipantRole::Manufacturer,
                    ));
                }
                // Distributors
                for i in 0..5 {
                    participants.push(SupplyChainParticipant::new(
                        format!("dist-{:03}", i),
                        format!("Distributor {}", i + 1),
                        ParticipantRole::Distributor,
                    ));
                }
                // Retailers
                for i in 0..4 {
                    participants.push(SupplyChainParticipant::new(
                        format!("ret-{:03}", i),
                        format!("Retail Store {}", i + 1),
                        ParticipantRole::Retailer,
                    ));
                }
                // Service providers
                for i in 0..3 {
                    participants.push(SupplyChainParticipant::new(
                        format!("svc-{:03}", i),
                        format!("Service Provider {}", i + 1),
                        ParticipantRole::ServiceProvider,
                    ));
                }
                participants
            }
            DataScale::Large => {
                let mut participants = Vec::new();
                let role_counts = [
                    (ParticipantRole::Manufacturer, 10),
                    (ParticipantRole::Distributor, 15),
                    (ParticipantRole::Retailer, 12),
                    (ParticipantRole::LogisticsProvider, 8),
                    (ParticipantRole::Regulator, 3),
                    (ParticipantRole::ServiceProvider, 5),
                ];

                for (role, count) in role_counts {
                    for i in 0..count {
                        let role_name = match role {
                            ParticipantRole::Manufacturer => "Manufacturer",
                            ParticipantRole::Distributor => "Distributor",
                            ParticipantRole::Retailer => "Retailer",
                            ParticipantRole::LogisticsProvider => "Logistics",
                            ParticipantRole::Regulator => "Regulator",
                            ParticipantRole::ServiceProvider => "Service",
                            ParticipantRole::Custom(_) => "Custom",
                        };
                        
                        participants.push(SupplyChainParticipant::new(
                            format!("{}-{:03}", role.to_lowercase().chars().next().unwrap_or('x'), i),
                            format!("{} {}", role_name, i + 1),
                            role,
                        ));
                    }
                }
                participants
            }
        };

        // Assign locations to participants
        for (i, participant) in self.participants.iter_mut().enumerate() {
            if let Some(location) = self.locations.get(i % self.locations.len()) {
                participant.location = Some(location.clone());
            }
        }
    }

    fn generate_locations(&mut self) {
        let location_count = match self.config.scale {
            DataScale::Small => 5,
            DataScale::Medium => 15,
            DataScale::Large => 40,
        };

        let cities = vec![
            ("New York", "NY", "10001"),
            ("Los Angeles", "CA", "90001"),
            ("Chicago", "IL", "60601"),
            ("Houston", "TX", "77001"),
            ("Phoenix", "AZ", "85001"),
            ("Philadelphia", "PA", "19101"),
            ("San Antonio", "TX", "78201"),
            ("San Diego", "CA", "92101"),
            ("Dallas", "TX", "75201"),
            ("San Jose", "CA", "95101"),
        ];

        for i in 0..location_count {
            let city_data = cities[i % cities.len()];
            let address = Address::new(
                format!("{} Main St", 100 + i),
                city_data.0.to_string(),
                city_data.1.to_string(),
                city_data.2.to_string(),
                "USA".to_string(),
            );

            let capabilities = match i % 5 {
                0 => vec![LocationCapability::Manufacturing],
                1 => vec![LocationCapability::Warehousing, LocationCapability::Distribution],
                2 => vec![LocationCapability::Retail],
                3 => vec![LocationCapability::QualityTesting, LocationCapability::Certification],
                4 => vec![LocationCapability::Customs],
                _ => vec![LocationCapability::Warehousing],
            };

            let location = BusinessLocation::new(
                format!("loc-{:03}", i),
                format!("{} Facility", city_data.0),
                address,
            )
            .with_coordinates(40.0 + (i as f64) * 0.1, -74.0 + (i as f64) * 0.1)
            .with_capabilities(capabilities);

            self.locations.push(location);
        }
    }

    fn generate_read_points(&mut self) {
        for (i, location) in self.locations.iter().enumerate() {
            let reader_types = vec![
                ReaderType::RFID,
                ReaderType::Barcode,
                ReaderType::Manual,
                ReaderType::API,
            ];

            for j in 0..2.min(reader_types.len()) { // 2 read points per location
                let read_point = ReadPoint::new(
                    format!("rp-{:03}-{:02}", i, j),
                    format!("{} Reader {}", location.name, j + 1),
                    location.clone(),
                    reader_types[j].clone(),
                );
                self.read_points.push(read_point);
            }
        }
    }

    fn generate_epc_pool(&self, count: usize) -> Vec<String> {
        let mut epcs = Vec::new();
        
        for i in 0..count.max(100) { // Generate at least 100 EPCs
            let company_prefix = "0614141"; // GS1 company prefix
            let item_reference = format!("{:06}", i % 1000000);
            let serial_number = format!("{:08}", self.rng.gen_range(0..100000000));
            
            let epc = format!("urn:epc:id:sgtin:{}:{}.{}", company_prefix, item_reference, serial_number);
            epcs.push(epc);
        }
        
        epcs
    }

    fn calculate_event_count(&self) -> usize {
        match self.config.scale {
            DataScale::Small => self.rng.gen_range(100..=500),
            DataScale::Medium => self.rng.gen_range(1000..=5000),
            DataScale::Large => self.rng.gen_range(10000..=50000),
        }
    }

    fn select_event_pattern(&mut self) -> &EventPattern {
        let total_weight: f64 = self.config.event_patterns.iter().map(|p| p.weight).sum();
        let mut selection = self.rng.gen_range(0.0..total_weight);
        
        for pattern in &self.config.event_patterns {
            if selection < pattern.weight {
                return pattern;
            }
            selection -= pattern.weight;
        }
        
        &self.config.event_patterns[0]
    }

    fn generate_single_event(&mut self, index: usize, pattern: &EventPattern, epc_pool: &mut Vec<String>) -> EPCISEvent {
        let event_id = format!("evt-{:06}", index);
        let event_type = pattern.event_type.clone();
        
        let event_time = self.start_time + 
            Duration::from_secs(self.rng.gen_range(0..self.config.time_span.as_secs()));
        
        let mut event = EPCISEvent::new(event_id, event_type);
        event.event_time = event_time;
        
        // Select random participant
        let participant = self.participants
            .choose(&mut self.rng)
            .expect("No participants available");
        
        // Select random location
        let read_point = self.read_points
            .choose(&mut self.rng)
            .expect("No read points available");
        
        // Set business step and disposition
        event.biz_step = pattern.business_steps.choose(&mut self.rng).cloned();
        event.disposition = pattern.dispositions.choose(&mut self.rng).cloned();
        
        // Set read point and business location
        event.read_point = Some(read_point.clone());
        event.business_location = Some(read_point.location.clone());
        
        // Add EPCs based on event type
        match event_type {
            EPCISEventType::ObjectEvent => {
                // Add 1-5 EPCs
                let epc_count = self.rng.gen_range(1..=5.min(epc_pool.len()));
                for _ in 0..epc_count {
                    if let Some(epc) = epc_pool.pop() {
                        event.add_epc(epc);
                    }
                }
            }
            EPCISEventType::AggregationEvent => {
                // Add parent EPC and child EPCs
                if let Some(parent_epc) = epc_pool.pop() {
                    event.parent_id = Some(parent_epc.clone());
                    event.add_epc(parent_epc);
                    
                    // Add 2-10 child EPCs
                    let child_count = self.rng.gen_range(2..=10.min(epc_pool.len()));
                    let mut child_epcs = Vec::new();
                    for _ in 0..child_count {
                        if let Some(child_epc) = epc_pool.pop() {
                            child_epcs.push(child_epc);
                        }
                    }
                    event.child_epcs = Some(child_epcs);
                }
            }
            EPCISEventType::TransactionEvent => {
                // Add 1-3 EPCs with quantities
                let epc_count = self.rng.gen_range(1..=3.min(epc_pool.len()));
                for _ in 0..epc_count {
                    if let Some(epc) = epc_pool.pop() {
                        let quantity = self.rng.gen_range(1..=100);
                        event.add_quantity(epc, quantity);
                    }
                }
            }
            EPCISEventType::TransformationEvent => {
                // Add input and output EPCs
                for _ in 0..self.rng.gen_range(1..=3.min(epc_pool.len() / 2)) {
                    if let Some(input_epc) = epc_pool.pop() {
                        event.add_epc(format!("input_{}", input_epc));
                    }
                }
                for _ in 0..self.rng.gen_range(1..=3.min(epc_pool.len() / 2)) {
                    if let Some(output_epc) = epc_pool.pop() {
                        event.add_epc(format!("output_{}", output_epc));
                    }
                }
            }
        }
        
        // Add business transactions
        if self.rng.gen_bool(0.3) { // 30% chance
            event.business_transaction_list.push(BusinessTransaction {
                transaction_type: "po".to_string(),
                transaction_id: format!("PO-{:06}", self.rng.gen_range(1..10000)),
            });
        }
        
        // Add extensions if enabled
        if self.config.include_extensions && self.rng.gen_bool(0.2) { // 20% chance
            event.extension.insert(
                "temperature".to_string(),
                format!("{}°C", self.rng.gen_range(15..=25)),
            );
            event.extension.insert(
                "humidity".to_string(),
                format!("{}%", self.rng.gen_range(30..=70)),
            );
        }
        
        event
    }

    fn add_epcis_classes(&self, ontology: &mut Ontology) -> OwlResult<()> {
        let epcis_classes = vec![
            ("EPCISEvent", "http://example.org/epcis/EPCISEvent"),
            ("ObjectEvent", "http://example.org/epcis/ObjectEvent"),
            ("AggregationEvent", "http://example.org/epcis/AggregationEvent"),
            ("TransactionEvent", "http://example.org/epcis/TransactionEvent"),
            ("TransformationEvent", "http://example.org/epcis/TransformationEvent"),
            ("EPC", "http://example.org/epcis/EPC"),
            ("BusinessLocation", "http://example.org/epcis/BusinessLocation"),
            ("ReadPoint", "http://example.org/epcis/ReadPoint"),
            ("Participant", "http://example.org/epcis/Participant"),
            ("SupplyChainParticipant", "http://example.org/epcis/SupplyChainParticipant"),
        ];

        for (name, iri) in epcis_classes {
            let class = Class::new(iri);
            ontology.add_class(class)?;
        }

        // Add subclass relationships
        let event_class = Class::new("http://example.org/epcis/EPCISEvent");
        let subclasses = vec![
            "http://example.org/epcis/ObjectEvent",
            "http://example.org/epcis/AggregationEvent", 
            "http://example.org/epcis/TransactionEvent",
            "http://example.org/epcis/TransformationEvent",
        ];

        for subclass_iri in subclasses {
            let subclass = Class::new(subclass_iri);
            let axiom = SubClassOfAxiom::new(
                ClassExpression::from(subclass),
                ClassExpression::from(event_class.clone()),
            );
            ontology.add_subclass_axiom(axiom)?;
        }

        Ok(())
    }

    fn add_participants_to_ontology(&self, ontology: &mut Ontology) -> OwlResult<()> {
        let participant_class = Class::new("http://example.org/epcis/Participant");
        
        for participant in &self.participants {
            let individual = NamedIndividual::new(format!(
                "http://example.org/epcis/participants/{}", 
                participant.id
            ));
            ontology.add_named_individual(individual.clone())?;

            let assertion = ClassAssertionAxiom::new(
                ClassExpression::from(participant_class.clone()),
                individual(*(*iri())).clone(),
            );
            ontology.add_class_assertion(assertion)?;
        }

        Ok(())
    }

    fn add_locations_to_ontology(&self, ontology: &mut Ontology) -> OwlResult<()> {
        let location_class = Class::new("http://example.org/epcis/BusinessLocation");
        let read_point_class = Class::new("http://example.org/epcis/ReadPoint");
        
        for location in &self.locations {
            let individual = NamedIndividual::new(format!(
                "http://example.org/epcis/locations/{}", 
                location.id
            ));
            ontology.add_named_individual(individual.clone())?;

            let assertion = ClassAssertionAxiom::new(
                ClassExpression::from(location_class.clone()),
                individual(*(*iri())).clone(),
            );
            ontology.add_class_assertion(assertion)?;
        }

        for read_point in &self.read_points {
            let individual = NamedIndividual::new(format!(
                "http://example.org/epcis/readpoints/{}", 
                read_point.id
            ));
            ontology.add_named_individual(individual.clone())?;

            let assertion = ClassAssertionAxiom::new(
                ClassExpression::from(read_point_class.clone()),
                individual(*(*iri())).clone(),
            );
            ontology.add_class_assertion(assertion)?;
        }

        Ok(())
    }

    fn add_event_to_ontology(&self, ontology: &mut Ontology, event: &EPCISEvent) -> OwlResult<()> {
        // Convert event to OWL2 and merge with main ontology
        let (event_ontology, _) = event.to_owl2()?;
        
        // Merge all entities and axioms
        for class in event_ontology.classes() {
            ontology.add_class(class.clone())?;
        }
        
        for individual in event_ontology.named_individuals() {
            ontology.add_named_individual(individual.clone())?;
        }
        
        for axiom in event_ontology.class_assertions() {
            ontology.add_class_assertion(axiom.clone())?;
        }
        
        for axiom in event_ontology.property_assertions() {
            ontology.add_property_assertion(axiom.clone())?;
        }

        Ok(())
    }

    /// Get statistics about generated data
    pub fn get_statistics(&self) -> EPCISStatistics {
        EPCISStatistics {
            participant_count: self.participants.len(),
            location_count: self.locations.len(),
            read_point_count: self.read_points.len(),
            scale: self.config.scale.clone(),
        }
    }
}

/// Statistics about generated EPCIS data
#[derive(Debug, Clone)]
pub struct EPCISStatistics {
    pub participant_count: usize,
    pub location_count: usize,
    pub read_point_count: usize,
    pub scale: DataScale,
}

impl EPCISStatistics {
    pub fn summary(&self) -> String {
        format!(
            "EPCIS Data Generation - {}: {} participants, {} locations, {} read points",
            self.scale, self.participant_count, self.location_count, self.read_point_count
        )
    }
}
