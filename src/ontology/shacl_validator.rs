use crate::ontology::error::{ConstraintType, ShapeViolation, ValidationError, ValidationResult};
use owl2_reasoner::iri::IRI;
use owl2_reasoner::reasoning::{OwlReasoner, Reasoner};
use oxigraph::model::*;
use oxigraph::sparql::QueryResults;
use oxigraph::store::Store;
use std::fs;
use std::sync::{Arc, Mutex};

/// SHACL validator for RDF transaction data
pub struct ShaclValidator {
    /// Core SHACL shapes that apply to all domains
    pub core_shapes: Vec<ShaclShape>,
    /// Domain-specific SHACL shapes
    pub domain_shapes: Vec<ShaclShape>,
    /// Hash of the ontology for consistency checking
    pub ontology_hash: String,
    /// Whether validation is enabled
    pub validation_enabled: bool,
    /// OWL2 Reasoner for advanced validation
    pub reasoner: Option<Arc<Mutex<OwlReasoner>>>,
    /// Oxigraph store for SHACL validation queries
    #[allow(dead_code)]
    store: Store,
}

impl std::fmt::Debug for ShaclValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShaclValidator")
            .field("core_shapes", &self.core_shapes)
            .field("domain_shapes", &self.domain_shapes)
            .field("ontology_hash", &self.ontology_hash)
            .field("validation_enabled", &self.validation_enabled)
            .field("reasoner", &self.reasoner.is_some())
            .field("store", &"<Store>")
            .finish()
    }
}

impl Clone for ShaclValidator {
    fn clone(&self) -> Self {
        // Since Store doesn't implement Clone, we need to recreate it
        let store = Store::new()
            .unwrap_or_else(|_| panic!("Failed to create store for ShaclValidator clone"));

        // Reload shapes into the new store
        let _ = Self::load_shapes_into_store(&store, &self.core_shapes);
        let _ = Self::load_shapes_into_store(&store, &self.domain_shapes);

        ShaclValidator {
            core_shapes: self.core_shapes.clone(),
            domain_shapes: self.domain_shapes.clone(),
            ontology_hash: self.ontology_hash.clone(),
            validation_enabled: self.validation_enabled,
            reasoner: self.reasoner.clone(),
            store,
        }
    }
}

impl ShaclValidator {
    /// Create a new SHACL validator
    pub fn new(
        core_shacl_path: &str,
        domain_shacl_path: &str,
        ontology_hash: String,
        reasoner: Option<Arc<Mutex<OwlReasoner>>>,
    ) -> Result<Self, ValidationError> {
        let core_shapes = Self::load_shacl_shapes(core_shacl_path)?;
        let domain_shapes = Self::load_shacl_shapes(domain_shacl_path)?;
        let store = Store::new()?;

        // Load SHACL shapes into the store for validation
        Self::load_shapes_into_store(&store, &core_shapes)?;
        Self::load_shapes_into_store(&store, &domain_shapes)?;

        Ok(ShaclValidator {
            core_shapes,
            domain_shapes,
            ontology_hash,
            validation_enabled: true,
            reasoner,
            store,
        })
    }

    /// Load SHACL shapes from a Turtle file
    fn load_shacl_shapes(file_path: &str) -> Result<Vec<ShaclShape>, ValidationError> {
        let content = fs::read_to_string(file_path).map_err(|e| {
            ValidationError::new(format!("Failed to read SHACL file '{}': {}", file_path, e))
        })?;

        let store = Store::new()
            .map_err(|e| ValidationError::new(format!("Failed to create RDF store: {}", e)))?;

        // Parse the SHACL Turtle content
        use std::io::Cursor;
        let reader = Cursor::new(content.as_bytes());
        store
            .load_from_reader(oxigraph::io::RdfFormat::Turtle, reader)
            .map_err(|e| {
                ValidationError::new(format!("Failed to parse SHACL file '{}': {}", file_path, e))
            })?;

        // Extract SHACL shapes from the store
        Self::extract_shapes_from_store(&store)
    }

    /// Extract SHACL shapes from an RDF store
    fn extract_shapes_from_store(store: &Store) -> Result<Vec<ShaclShape>, ValidationError> {
        let mut shapes = Vec::new();

        // SPARQL query to find all SHACL shapes
        let query = r#"
            PREFIX sh: <http://www.w3.org/ns/shacl#>
            SELECT ?shape ?targetClass WHERE {
                ?shape a sh:NodeShape ;
                       sh:targetClass ?targetClass .
            }
        "#;

        let results = store
            .query(query)
            .map_err(|e| ValidationError::new(format!("Failed to query SHACL shapes: {}", e)))?;

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution.map_err(|e| {
                    ValidationError::new(format!("Failed to process SHACL query result: {}", e))
                })?;

                if let (Some(shape_term), Some(target_class_term)) =
                    (solution.get("shape"), solution.get("targetClass"))
                {
                    // Extract IRI strings without angle brackets
                    let shape_id = match shape_term {
                        Term::NamedNode(node) => node.as_str().to_string(),
                        _ => shape_term.to_string(),
                    };
                    let target_class = match target_class_term {
                        Term::NamedNode(node) => node.as_str().to_string(),
                        _ => target_class_term.to_string(),
                    };

                    // Extract properties and constraints for this shape
                    let properties = Self::extract_shape_properties(store, shape_term)?;
                    let constraints = Self::extract_shape_constraints(store, shape_term)?;

                    shapes.push(ShaclShape {
                        id: shape_id,
                        target_class,
                        properties,
                        constraints,
                    });
                }
            }
        }

        Ok(shapes)
    }

    /// Extract properties for a SHACL shape
    fn extract_shape_properties(
        store: &Store,
        shape_term: &Term,
    ) -> Result<Vec<ShaclProperty>, ValidationError> {
        let mut properties = Vec::new();

        // Convert Term to proper IRI string without angle brackets
        let shape_iri = match shape_term {
            Term::NamedNode(node) => node.as_str(),
            _ => return Ok(properties), // Skip non-IRI shapes
        };

        let query = format!(
            r#"
            PREFIX sh: <http://www.w3.org/ns/shacl#>
            SELECT ?property ?path ?datatype ?minCount ?maxCount ?class WHERE {{
                <{}> sh:property ?property .
                ?property sh:path ?path .
                OPTIONAL {{ ?property sh:datatype ?datatype }}
                OPTIONAL {{ ?property sh:minCount ?minCount }}
                OPTIONAL {{ ?property sh:maxCount ?maxCount }}
                OPTIONAL {{ ?property sh:class ?class }}
            }}
        "#,
            shape_iri
        );

        let results = store.query(&query).map_err(|e| {
            ValidationError::new(format!("Failed to query shape properties: {}", e))
        })?;

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution.map_err(|e| {
                    ValidationError::new(format!("Failed to process property query result: {}", e))
                })?;

                if let (Some(property_term), Some(path_term)) =
                    (solution.get("property"), solution.get("path"))
                {
                    let property_id = match property_term {
                        Term::NamedNode(node) => node.as_str().to_string(),
                        _ => property_term.to_string(),
                    };
                    let path = match path_term {
                        Term::NamedNode(node) => node.as_str().to_string(),
                        _ => path_term.to_string(),
                    };
                    let datatype = solution.get("datatype").map(|t| t.to_string());
                    let min_count = solution.get("minCount").and_then(|t| {
                        if let Term::Literal(lit) = t {
                            lit.value().parse::<u32>().ok()
                        } else {
                            None
                        }
                    });
                    let max_count = solution.get("maxCount").and_then(|t| {
                        if let Term::Literal(lit) = t {
                            lit.value().parse::<u32>().ok()
                        } else {
                            None
                        }
                    });

                    let class = solution.get("class").map(|t| match t {
                        Term::NamedNode(node) => node.as_str().to_string(),
                        _ => t.to_string(),
                    });

                    properties.push(ShaclProperty {
                        id: property_id,
                        path,
                        datatype,
                        min_count,
                        max_count,
                        class,
                        constraints: Vec::new(), // Will be populated separately
                    });
                }
            }
        }

        Ok(properties)
    }

    /// Extract constraints for a SHACL shape
    fn extract_shape_constraints(
        store: &Store,
        shape_term: &Term,
    ) -> Result<Vec<ShaclConstraint>, ValidationError> {
        let mut constraints = Vec::new();

        // Convert Term to proper IRI string without angle brackets
        let shape_iri = match shape_term {
            Term::NamedNode(node) => node.as_str(),
            _ => return Ok(constraints), // Skip non-IRI shapes
        };

        // Query for various SHACL constraints
        let constraint_queries = vec![
            ("minCount", "sh:minCount"),
            ("maxCount", "sh:maxCount"),
            ("datatype", "sh:datatype"),
            ("class", "sh:class"),
            ("nodeKind", "sh:nodeKind"),
            ("pattern", "sh:pattern"),
            ("minInclusive", "sh:minInclusive"),
            ("maxInclusive", "sh:maxInclusive"),
        ];

        for (constraint_name, constraint_predicate) in constraint_queries {
            let query = format!(
                r#"
                PREFIX sh: <http://www.w3.org/ns/shacl#>
                SELECT ?value WHERE {{
                    <{}> {} ?value .
                }}
            "#,
                shape_iri, constraint_predicate
            );

            let results = store.query(&query).map_err(|e| {
                ValidationError::new(format!(
                    "Failed to query {} constraint: {}",
                    constraint_name, e
                ))
            })?;

            if let QueryResults::Solutions(solutions) = results {
                for solution in solutions {
                    let solution = solution.map_err(|e| {
                        ValidationError::new(format!(
                            "Failed to process constraint query result: {}",
                            e
                        ))
                    })?;

                    if let Some(value_term) = solution.get("value") {
                        let constraint_type = match constraint_name {
                            "minCount" => ConstraintType::MinCount,
                            "maxCount" => ConstraintType::MaxCount,
                            "datatype" => ConstraintType::Datatype,
                            "class" => ConstraintType::Class,
                            "nodeKind" => ConstraintType::NodeKind,
                            "pattern" => ConstraintType::Pattern,
                            "minInclusive" => ConstraintType::MinInclusive,
                            "maxInclusive" => ConstraintType::MaxInclusive,
                            _ => ConstraintType::Custom(constraint_name.to_string()),
                        };

                        let value = match value_term {
                            Term::NamedNode(node) => node.as_str().to_string(),
                            Term::Literal(lit) => lit.value().to_string(),
                            _ => value_term.to_string(),
                        };

                        constraints.push(ShaclConstraint {
                            constraint_type,
                            value,
                            message: None,
                        });
                    }
                }
            }
        }

        Ok(constraints)
    }

    /// Load SHACL shapes into the validation store
    fn load_shapes_into_store(store: &Store, shapes: &[ShaclShape]) -> Result<(), ValidationError> {
        // Convert shapes back to RDF and load into store
        // This is a simplified implementation - in practice, you might want to
        // keep the original RDF representation
        for shape in shapes {
            let shape_iri = NamedNode::new(&shape.id).map_err(|e| {
                ValidationError::new(format!("Invalid shape IRI '{}': {}", shape.id, e))
            })?;

            let target_class_iri = NamedNode::new(&shape.target_class).map_err(|e| {
                ValidationError::new(format!(
                    "Invalid target class IRI '{}': {}",
                    shape.target_class, e
                ))
            })?;

            // Add basic shape triples
            store
                .insert(&Quad::new(
                    shape_iri.clone(),
                    NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
                    NamedNode::new("http://www.w3.org/ns/shacl#NodeShape").unwrap(),
                    oxigraph::model::GraphName::DefaultGraph,
                ))
                .map_err(|e| {
                    ValidationError::new(format!("Failed to insert shape triple: {}", e))
                })?;

            store
                .insert(&Quad::new(
                    shape_iri.clone(),
                    NamedNode::new("http://www.w3.org/ns/shacl#targetClass").unwrap(),
                    target_class_iri,
                    oxigraph::model::GraphName::DefaultGraph,
                ))
                .map_err(|e| {
                    ValidationError::new(format!("Failed to insert target class triple: {}", e))
                })?;
        }

        Ok(())
    }

    /// Validate RDF transaction data against SHACL shapes
    pub fn validate_transaction(
        &self,
        rdf_data: &str,
    ) -> Result<ValidationResult, ValidationError> {
        let start_time = std::time::Instant::now();

        if !self.validation_enabled {
            return Ok(ValidationResult::success(0)
                .with_execution_time(start_time.elapsed().as_millis() as u64)
                .with_metadata("validation_enabled".to_string(), "false".to_string()));
        }

        // Create a temporary store for the transaction data
        let data_store = Store::new().map_err(|e| {
            ValidationError::new(format!("Failed to create validation store: {}", e))
        })?;

        // Load the transaction RDF data
        use std::io::Cursor;
        let reader = Cursor::new(rdf_data.as_bytes());
        data_store
            .load_from_reader(oxigraph::io::RdfFormat::Turtle, reader)
            .map_err(|e| ValidationError::new(format!("Failed to parse transaction RDF: {}", e)))?;

        let mut violations = Vec::new();
        let mut constraints_checked = 0u32;

        // Validate against core shapes
        for shape in &self.core_shapes {
            constraints_checked += shape.properties.len() as u32 + shape.constraints.len() as u32;
            if let Err(mut shape_violations) = self.validate_against_shape(&data_store, shape) {
                violations.append(&mut shape_violations);
            }
        }

        // Validate against domain shapes
        for shape in &self.domain_shapes {
            constraints_checked += shape.properties.len() as u32 + shape.constraints.len() as u32;
            if let Err(mut shape_violations) = self.validate_against_shape(&data_store, shape) {
                violations.append(&mut shape_violations);
            }
        }

        let execution_time = start_time.elapsed().as_millis() as u64;

        if violations.is_empty() {
            Ok(ValidationResult::success(constraints_checked)
                .with_execution_time(execution_time)
                .with_metadata(
                    "core_shapes".to_string(),
                    self.core_shapes.len().to_string(),
                )
                .with_metadata(
                    "domain_shapes".to_string(),
                    self.domain_shapes.len().to_string(),
                ))
        } else {
            Ok(ValidationResult::failure(violations, constraints_checked)
                .with_execution_time(execution_time)
                .with_metadata(
                    "core_shapes".to_string(),
                    self.core_shapes.len().to_string(),
                )
                .with_metadata(
                    "domain_shapes".to_string(),
                    self.domain_shapes.len().to_string(),
                ))
        }
    }

    /// Validate data against a specific SHACL shape
    fn validate_against_shape(
        &self,
        data_store: &Store,
        shape: &ShaclShape,
    ) -> Result<(), Vec<ShapeViolation>> {
        let mut violations = Vec::new();

        // Find all instances of the target class
        let query = format!(
            r#"
            SELECT ?instance WHERE {{
                ?instance a <{}> .
            }}
        "#,
            shape.target_class
        );

        let results = data_store.query(&query).map_err(|e| {
            vec![ShapeViolation::new(
                shape.id.clone(),
                ConstraintType::Custom("QueryError".to_string()),
                format!("Failed to query target class instances: {}", e),
            )]
        })?;

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution.map_err(|e| {
                    vec![ShapeViolation::new(
                        shape.id.clone(),
                        ConstraintType::Custom("QueryError".to_string()),
                        format!("Failed to process query result: {}", e),
                    )]
                })?;

                if let Some(instance_term) = solution.get("instance") {
                    // Validate this instance against all shape properties
                    for property in &shape.properties {
                        if let Err(mut property_violations) =
                            self.validate_property(data_store, &shape.id, instance_term, property)
                        {
                            violations.append(&mut property_violations);
                        }
                    }

                    // Validate against shape-level constraints
                    for constraint in &shape.constraints {
                        if let Err(constraint_violation) = self.validate_constraint(
                            data_store,
                            &shape.id,
                            instance_term,
                            constraint,
                        ) {
                            violations.push(*constraint_violation);
                        }
                    }
                }
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }

    /// Validate a property constraint
    fn validate_property(
        &self,
        data_store: &Store,
        shape_id: &str,
        instance: &Term,
        property: &ShaclProperty,
    ) -> Result<(), Vec<ShapeViolation>> {
        let mut violations = Vec::new();

        // Query for property values - properly format IRIs
        let instance_iri = match instance {
            Term::NamedNode(node) => node.as_str(),
            _ => return Ok(()), // Skip non-IRI instances
        };

        let query = format!(
            r#"
            SELECT ?value WHERE {{
                <{}> <{}> ?value .
            }}
        "#,
            instance_iri, property.path
        );

        let results = data_store.query(&query).map_err(|e| {
            vec![ShapeViolation::new(
                shape_id.to_string(),
                ConstraintType::Custom("QueryError".to_string()),
                format!("Failed to query property values: {}", e),
            )]
        })?;

        let mut value_count = 0;
        let mut values = Vec::new();

        if let QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let solution = solution.map_err(|e| {
                    vec![ShapeViolation::new(
                        shape_id.to_string(),
                        ConstraintType::Custom("QueryError".to_string()),
                        format!("Failed to process property query result: {}", e),
                    )]
                })?;

                if let Some(value_term) = solution.get("value") {
                    value_count += 1;
                    let value = match value_term {
                        Term::NamedNode(node) => node.as_str().to_string(),
                        Term::Literal(lit) => lit.value().to_string(),
                        _ => value_term.to_string(),
                    };
                    values.push(value);
                }
            }
        }

        // Check minCount constraint
        if let Some(min_count) = property.min_count {
            if value_count < min_count {
                violations.push(
                    ShapeViolation::new(
                        shape_id.to_string(),
                        ConstraintType::MinCount,
                        format!(
                            "Property {} has {} values, minimum required: {}",
                            property.path, value_count, min_count
                        ),
                    )
                    .with_property_path(property.path.clone()),
                );
            }
        }

        // Check maxCount constraint
        if let Some(max_count) = property.max_count {
            if value_count > max_count {
                violations.push(
                    ShapeViolation::new(
                        shape_id.to_string(),
                        ConstraintType::MaxCount,
                        format!(
                            "Property {} has {} values, maximum allowed: {}",
                            property.path, value_count, max_count
                        ),
                    )
                    .with_property_path(property.path.clone()),
                );
            }
        }

        // Check datatype constraint
        if let Some(expected_datatype) = &property.datatype {
            for value in &values {
                // This is a simplified datatype check
                // In practice, you'd want more sophisticated datatype validation
                if !self.validate_datatype(value, expected_datatype) {
                    violations.push(
                        ShapeViolation::new(
                            shape_id.to_string(),
                            ConstraintType::Datatype,
                            format!(
                                "Property {} value '{}' does not match expected datatype {}",
                                property.path, value, expected_datatype
                            ),
                        )
                        .with_property_path(property.path.clone())
                        .with_value(value.clone()),
                    );
                }
            }
        }

        // Check class constraint using OWL2 Reasoner
        if let Some(expected_class) = &property.class {
            if let Some(reasoner_lock) = &self.reasoner {
                if let Ok(mut reasoner) = reasoner_lock.lock() {
                    if let Ok(expected_class_iri) = IRI::new(expected_class) {
                        for value in &values {
                            // Only check if value is an IRI
                            if value.starts_with("http") {
                                if let Ok(_value_iri) = IRI::new(value) {
                                    // Check if the value is an instance of the expected class
                                    // or if the value's type is a subclass of the expected class
                                    // Since we don't have instance types easily available here without querying the store again,
                                    // we'll assume for now we are checking if the object is of the right type.
                                    // BUT, `sh:class` on a property means the object of the triple must be an instance of the class.
                                    // In our data generation, we have `core:batch1 a core:Batch`.
                                    // And `prov:wasAttributedTo core:org1`. `core:org1 a core:Organization`.
                                    // The property `prov:wasAttributedTo` has `sh:class core:Organization`.
                                    // So we need to check if `core:org1` is an instance of `core:Organization`.

                                    // To do this properly with the reasoner, we need to know the types of the value.
                                    // We can query the data_store for the type of the value.
                                    let type_query =
                                        format!("SELECT ?type WHERE {{ <{}> a ?type }}", value);

                                    let mut is_instance = false;
                                    if let Ok(type_results) = data_store.query(&type_query) {
                                        if let QueryResults::Solutions(type_solutions) =
                                            type_results
                                        {
                                            for type_sol in type_solutions {
                                                if let Ok(ts) = type_sol {
                                                    if let Some(type_term) = ts.get("type") {
                                                        let type_str = match type_term {
                                                            Term::NamedNode(n) => {
                                                                n.as_str().to_string()
                                                            }
                                                            _ => type_term.to_string(),
                                                        };

                                                        if let Ok(type_iri) = IRI::new(&type_str) {
                                                            // Check if type_iri is subclass of expected_class_iri (or same)
                                                            if let Ok(is_sub) = reasoner
                                                                .is_subclass_of(
                                                                    &type_iri,
                                                                    &expected_class_iri,
                                                                )
                                                            {
                                                                if is_sub {
                                                                    is_instance = true;
                                                                    break;
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    if !is_instance {
                                        violations.push(
                                            ShapeViolation::new(
                                                shape_id.to_string(),
                                                ConstraintType::Class,
                                                format!(
                                                    "Property {} value '{}' is not an instance of class {}",
                                                    property.path, value, expected_class
                                                ),
                                            )
                                            .with_property_path(property.path.clone())
                                            .with_value(value.clone()),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }

    /// Validate a shape-level constraint
    fn validate_constraint(
        &self,
        _data_store: &Store,
        shape_id: &str,
        _instance: &Term,
        constraint: &ShaclConstraint,
    ) -> Result<(), Box<ShapeViolation>> {
        // This is a placeholder for shape-level constraint validation
        // In practice, you'd implement specific validation logic for each constraint type
        match constraint.constraint_type {
            ConstraintType::Custom(ref name) => {
                // Custom constraint validation would go here
                if name == "example_failing_constraint" {
                    return Err(Box::new(ShapeViolation::new(
                        shape_id.to_string(),
                        constraint.constraint_type.clone(),
                        format!("Custom constraint '{}' failed", name),
                    )));
                }
            }
            _ => {
                // Other constraint types would be handled here
            }
        }

        Ok(())
    }

    pub fn reload(&mut self) -> Result<(), ValidationError> {
        let store = Store::new().map_err(|e| {
            ValidationError::new(format!("Failed to create store for reload: {}", e))
        })?;

        // Reload shapes into the new store
        let _ = Self::load_shapes_into_store(&store, &self.core_shapes);
        let _ = Self::load_shapes_into_store(&store, &self.domain_shapes);

        self.store = store;
        Ok(())
    }

    /// Simple datatype validation
    fn validate_datatype(&self, value: &str, expected_datatype: &str) -> bool {
        match expected_datatype {
            "http://www.w3.org/2001/XMLSchema#string" => true, // All values can be strings
            "http://www.w3.org/2001/XMLSchema#integer" => value.parse::<i64>().is_ok(),
            "http://www.w3.org/2001/XMLSchema#decimal" => value.parse::<f64>().is_ok(),
            "http://www.w3.org/2001/XMLSchema#boolean" => value == "true" || value == "false",
            "http://www.w3.org/2001/XMLSchema#dateTime" => {
                // Simplified datetime validation
                value.contains('T') && value.len() >= 19
            }
            _ => true, // Unknown datatypes pass validation
        }
    }

    /// Get all loaded shapes (core + domain)
    pub fn get_all_shapes(&self) -> Vec<&ShaclShape> {
        let mut shapes = Vec::new();
        shapes.extend(&self.core_shapes);
        shapes.extend(&self.domain_shapes);
        shapes
    }

    /// Enable or disable validation
    pub fn set_validation_enabled(&mut self, enabled: bool) {
        self.validation_enabled = enabled;
    }
}

/// SHACL shape definition
#[derive(Debug, Clone)]
pub struct ShaclShape {
    /// Unique identifier for the shape
    pub id: String,
    /// Target class that this shape applies to
    pub target_class: String,
    /// Properties defined by this shape
    pub properties: Vec<ShaclProperty>,
    /// Shape-level constraints
    pub constraints: Vec<ShaclConstraint>,
}

/// SHACL property definition
#[derive(Debug, Clone)]
pub struct ShaclProperty {
    /// Property identifier
    pub id: String,
    /// Property path (IRI)
    pub path: String,
    /// Expected datatype
    pub datatype: Option<String>,
    /// Minimum cardinality
    pub min_count: Option<u32>,
    /// Maximum cardinality
    pub max_count: Option<u32>,
    /// Expected class (for object properties)
    pub class: Option<String>,
    /// Property-specific constraints
    pub constraints: Vec<ShaclConstraint>,
}

/// SHACL constraint definition
#[derive(Debug, Clone)]
pub struct ShaclConstraint {
    /// Type of constraint
    pub constraint_type: ConstraintType,
    /// Constraint value
    pub value: String,
    /// Custom error message
    pub message: Option<String>,
}

impl From<oxigraph::store::StorageError> for ValidationError {
    fn from(error: oxigraph::store::StorageError) -> Self {
        ValidationError::new(format!("RDF store error: {}", error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_shacl_validator_creation() {
        let temp_dir = TempDir::new().unwrap();

        // Create minimal SHACL files
        let core_shacl_path = temp_dir.path().join("core.shacl.ttl");
        let domain_shacl_path = temp_dir.path().join("domain.shacl.ttl");

        let shacl_content = r#"
            @prefix sh: <http://www.w3.org/ns/shacl#> .
            @prefix ex: <http://example.org/> .
            
            ex:TestShape a sh:NodeShape ;
                sh:targetClass ex:TestClass ;
                sh:property [
                    sh:path ex:name ;
                    sh:datatype <http://www.w3.org/2001/XMLSchema#string> ;
                    sh:minCount 1 ;
                ] .
        "#;

        fs::write(&core_shacl_path, shacl_content).unwrap();
        fs::write(&domain_shacl_path, shacl_content).unwrap();

        let validator = ShaclValidator::new(
            &core_shacl_path.to_string_lossy(),
            &domain_shacl_path.to_string_lossy(),
            "test_hash".to_string(),
            None,
        );

        assert!(validator.is_ok());
        let validator = validator.unwrap();
        assert!(validator.validation_enabled);
        assert_eq!(validator.ontology_hash, "test_hash");
    }

    #[test]
    fn test_datatype_validation() {
        let temp_dir = TempDir::new().unwrap();
        let core_shacl_path = temp_dir.path().join("core.shacl.ttl");
        let domain_shacl_path = temp_dir.path().join("domain.shacl.ttl");

        fs::write(
            &core_shacl_path,
            "@prefix sh: <http://www.w3.org/ns/shacl#> .",
        )
        .unwrap();
        fs::write(
            &domain_shacl_path,
            "@prefix sh: <http://www.w3.org/ns/shacl#> .",
        )
        .unwrap();

        let validator = ShaclValidator::new(
            &core_shacl_path.to_string_lossy(),
            &domain_shacl_path.to_string_lossy(),
            "test_hash".to_string(),
            None,
        )
        .unwrap();

        // Test integer validation
        assert!(validator.validate_datatype("123", "http://www.w3.org/2001/XMLSchema#integer"));
        assert!(!validator.validate_datatype("abc", "http://www.w3.org/2001/XMLSchema#integer"));

        // Test boolean validation
        assert!(validator.validate_datatype("true", "http://www.w3.org/2001/XMLSchema#boolean"));
        assert!(validator.validate_datatype("false", "http://www.w3.org/2001/XMLSchema#boolean"));
        assert!(!validator.validate_datatype("maybe", "http://www.w3.org/2001/XMLSchema#boolean"));
    }
}
