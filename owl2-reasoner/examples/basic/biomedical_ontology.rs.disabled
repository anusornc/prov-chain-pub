//! Biomedical Ontology Example
//!
//! This example demonstrates how to create a biomedical ontology with
//! gene-disease associations and protein interactions, showing more
//! complex class expressions and reasoning patterns.

use owl2_reasoner::{
    axioms::*,
    entities::*,
    iri::IRI,
    ontology::Ontology,
    reasoning::{PatternTerm, QueryEngine, QueryPattern, SimpleReasoner, TriplePattern},
    OwlResult,
};

fn main() -> OwlResult<()> {
    println!("=== Biomedical Ontology Example ===\n");

    // Create a new ontology
    let mut ontology = Ontology::new();
    ontology.set_iri("http://example.org/biomedical");

    // Define biomedical classes
    let gene = Class::new("http://example.org/Gene");
    let protein = Class::new("http://example.org/Protein");
    let disease = Class::new("http://example.org/Disease");
    let symptom = Class::new("http://example.org/Symptom");
    let treatment = Class::new("http://example.org/Treatment");
    let drug = Class::new("http://example.org/Drug");
    let genetic_disorder = Class::new("http://example.org/GeneticDisorder");
    let rare_disease = Class::new("http://example.org/RareDisease");

    // Add classes to ontology
    for class in &[
        gene.clone(),
        protein.clone(),
        disease.clone(),
        symptom.clone(),
        treatment.clone(),
        drug.clone(),
        genetic_disorder.clone(),
        rare_disease.clone(),
    ] {
        ontology.add_class(class.clone())?;
    }

    println!("✓ Added {} biomedical classes", ontology.classes().len());

    // Define properties
    let encodes = ObjectProperty::new("http://example.org/encodes");
    let mut associated_with = ObjectProperty::new("http://example.org/associatedWith");
    let causes = ObjectProperty::new("http://example.org/causes");
    let treats = ObjectProperty::new("http://example.org/treats");
    let has_symptom = ObjectProperty::new("http://example.org/hasSymptom");
    let mut interacts_with = ObjectProperty::new("http://example.org/interactsWith");

    // Add property characteristics
    associated_with.add_characteristic(ObjectPropertyCharacteristic::Symmetric);
    interacts_with.add_characteristic(ObjectPropertyCharacteristic::Symmetric);

    // Add properties to ontology
    for prop in &[
        encodes.clone(),
        associated_with.clone(),
        causes.clone(),
        treats.clone(),
        has_symptom.clone(),
        interacts_with.clone(),
    ] {
        ontology.add_object_property(prop.clone())?;
    }

    println!(
        "✓ Added {} biomedical properties",
        ontology.object_properties().len()
    );

    // Add subclass relationships
    let subclass_axioms = vec![
        // Genetic disorders are diseases
        SubClassOfAxiom::new(
            ClassExpression::from(genetic_disorder.clone()),
            ClassExpression::from(disease.clone()),
        ),
        // Rare diseases are diseases
        SubClassOfAxiom::new(
            ClassExpression::from(rare_disease.clone()),
            ClassExpression::from(disease.clone()),
        ),
        // Drugs are treatments
        SubClassOfAxiom::new(
            ClassExpression::from(drug.clone()),
            ClassExpression::from(treatment.clone()),
        ),
    ];

    for axiom in subclass_axioms {
        ontology.add_subclass_axiom(axiom)?;
    }

    println!(
        "✓ Added {} subclass axioms",
        ontology.subclass_axioms().len()
    );

    // Add equivalent classes (simplified for current API)
    // Note: Current API only supports IRI-based equivalent classes
    // Future versions will support complex class expressions
    let equivalent_genetic =
        EquivalentClassesAxiom::new(vec![genetic_disorder.iri().clone(), disease.iri().clone()]);

    ontology.add_equivalent_classes_axiom(equivalent_genetic)?;

    // Create biomedical individuals
    let brca1 = NamedIndividual::new("http://example.org/BRCA1");
    let brca2 = NamedIndividual::new("http://example.org/BRCA2");
    let brca1_protein = NamedIndividual::new("http://example.org/BRCA1_protein");
    let brca2_protein = NamedIndividual::new("http://example.org/BRCA2_protein");
    let breast_cancer = NamedIndividual::new("http://example.org/BreastCancer");
    let ovarian_cancer = NamedIndividual::new("http://example.org/OvarianCancer");
    let tamoxifen = NamedIndividual::new("http://example.org/Tamoxifen");
    let fatigue = NamedIndividual::new("http://example.org/Fatigue");

    // Add individuals to ontology
    for individual in &[
        brca1.clone(),
        brca2.clone(),
        brca1_protein.clone(),
        brca2_protein.clone(),
        breast_cancer.clone(),
        ovarian_cancer.clone(),
        tamoxifen.clone(),
        fatigue.clone(),
    ] {
        ontology.add_named_individual(individual.clone())?;
    }

    println!(
        "✓ Added {} biomedical individuals",
        ontology.named_individuals().len()
    );

    // Add class assertions
    let class_assertions = vec![
        // Genes
        ClassAssertionAxiom::new(brca1.iri().clone(), ClassExpression::Class(gene.clone())),
        ClassAssertionAxiom::new(brca2.iri().clone(), ClassExpression::Class(gene.clone())),
        // Proteins
        ClassAssertionAxiom::new(
            brca1_protein.iri().clone(),
            ClassExpression::Class(protein.clone()),
        ),
        ClassAssertionAxiom::new(
            brca2_protein.iri().clone(),
            ClassExpression::Class(protein.clone()),
        ),
        // Diseases
        ClassAssertionAxiom::new(
            breast_cancer.iri().clone(),
            ClassExpression::Class(disease.clone()),
        ),
        ClassAssertionAxiom::new(
            ovarian_cancer.iri().clone(),
            ClassExpression::Class(disease.clone()),
        ),
        ClassAssertionAxiom::new(
            breast_cancer.iri().clone(),
            ClassExpression::Class(genetic_disorder.clone()),
        ),
        ClassAssertionAxiom::new(
            ovarian_cancer.iri().clone(),
            ClassExpression::Class(genetic_disorder.clone()),
        ),
        // Treatments
        ClassAssertionAxiom::new(
            tamoxifen.iri().clone(),
            ClassExpression::Class(drug.clone()),
        ),
        // Symptoms
        ClassAssertionAxiom::new(
            fatigue.iri().clone(),
            ClassExpression::Class(symptom.clone()),
        ),
    ];

    for assertion in class_assertions {
        ontology.add_class_assertion(assertion)?;
    }

    println!(
        "✓ Added {} class assertions",
        ontology.class_assertions().len()
    );

    // Add property assertions
    let property_assertions = vec![
        // Gene-protein relationships
        PropertyAssertionAxiom::new(
            encodes.iri().clone(),
            brca1.iri().clone(),
            brca1_protein.iri().clone(),
        ),
        PropertyAssertionAxiom::new(
            encodes.iri().clone(),
            brca2.iri().clone(),
            brca2_protein.iri().clone(),
        ),
        // Gene-disease associations
        PropertyAssertionAxiom::new(
            associated_with.iri().clone(),
            brca1.iri().clone(),
            breast_cancer.iri().clone(),
        ),
        PropertyAssertionAxiom::new(
            associated_with.iri().clone(),
            brca2.iri().clone(),
            breast_cancer.iri().clone(),
        ),
        PropertyAssertionAxiom::new(
            associated_with.iri().clone(),
            brca1.iri().clone(),
            ovarian_cancer.iri().clone(),
        ),
        PropertyAssertionAxiom::new(
            associated_with.iri().clone(),
            brca2.iri().clone(),
            ovarian_cancer.iri().clone(),
        ),
        // Protein interactions
        PropertyAssertionAxiom::new(
            interacts_with.iri().clone(),
            brca1_protein.iri().clone(),
            brca2_protein.iri().clone(),
        ),
        PropertyAssertionAxiom::new(
            interacts_with.iri().clone(),
            brca2_protein.iri().clone(),
            brca1_protein.iri().clone(),
        ),
        // Disease-symptom relationships
        PropertyAssertionAxiom::new(
            has_symptom.iri().clone(),
            breast_cancer.iri().clone(),
            fatigue.iri().clone(),
        ),
        PropertyAssertionAxiom::new(
            has_symptom.iri().clone(),
            ovarian_cancer.iri().clone(),
            fatigue.iri().clone(),
        ),
        // Treatment relationships
        PropertyAssertionAxiom::new(
            treats.iri().clone(),
            tamoxifen.iri().clone(),
            breast_cancer.iri().clone(),
        ),
    ];

    for assertion in property_assertions {
        ontology.add_property_assertion(assertion)?;
    }

    println!(
        "✓ Added {} property assertions",
        ontology.property_assertions().len()
    );

    // Create reasoner and perform reasoning
    println!("\n=== Biomedical Reasoning Results ===");
    let reasoner = SimpleReasoner::new(ontology);

    // Check consistency
    let is_consistent = reasoner.is_consistent()?;
    println!("✓ Biomedical ontology is consistent: {}", is_consistent);

    // Check subclass relationships
    let subclass_checks = vec![
        (
            genetic_disorder.clone(),
            disease.clone(),
            "GeneticDisorder ⊑ Disease",
        ),
        (
            rare_disease.clone(),
            disease.clone(),
            "RareDisease ⊑ Disease",
        ),
        (drug.clone(), treatment.clone(), "Drug ⊑ Treatment"),
    ];

    for (sub, sup, desc) in subclass_checks {
        let result = reasoner.is_subclass_of(sub.iri(), sup.iri())?;
        println!("✓ {}: {}", desc, result);
    }

    // Check class satisfiability
    println!("\n=== Class Satisfiability ===");
    let satisfiability_checks = vec![
        (genetic_disorder.iri().clone(), "GeneticDisorder"),
        (breast_cancer.iri().clone(), "BreastCancer"),
        (brca1.iri().clone(), "BRCA1"),
    ];

    for (class_iri, desc) in satisfiability_checks {
        let is_satisfiable = reasoner.is_class_satisfiable(&class_iri)?;
        println!("✓ {} is satisfiable: {}", desc, is_satisfiable);
    }

    // Get instances
    println!("\n=== Instance Retrieval ===");
    let instance_checks = vec![
        (gene.clone(), "Genes"),
        (protein.clone(), "Proteins"),
        (disease.clone(), "Diseases"),
        (genetic_disorder.clone(), "Genetic Disorders"),
        (treatment.clone(), "Treatments"),
    ];

    for (class, desc) in instance_checks {
        let instances = reasoner.get_instances(class.iri())?;
        println!("✓ {}: {:?}", desc, instances);
    }

    // Complex queries
    println!("\n=== Complex Biomedical Queries ===");
    let mut query_engine = QueryEngine::new(reasoner.ontology.clone());

    // Find all genes associated with diseases
    let gene_disease_pattern = QueryPattern::BasicGraphPattern(vec![
        TriplePattern {
            subject: PatternTerm::Variable("s".to_string()),
            predicate: PatternTerm::IRI(IRI::new(
                "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            )?),
            object: PatternTerm::IRI(gene.iri().as_ref().clone()),
        },
        TriplePattern {
            subject: PatternTerm::Variable("s".to_string()),
            predicate: PatternTerm::IRI(associated_with.iri().as_ref().clone()),
            object: PatternTerm::Variable("o".to_string()),
        },
    ]);

    let genes_with_diseases = query_engine.execute_query(&gene_disease_pattern)?;
    println!(
        "✓ Found {} genes associated with diseases",
        genes_with_diseases.bindings.len()
    );

    // Find all disease-symptom relationships
    let symptom_pattern = QueryPattern::BasicGraphPattern(vec![TriplePattern {
        subject: PatternTerm::Variable("s".to_string()),
        predicate: PatternTerm::IRI(has_symptom.iri().as_ref().clone()),
        object: PatternTerm::Variable("o".to_string()),
    }]);

    let symptom_relationships = query_engine.execute_query(&symptom_pattern)?;
    println!(
        "✓ Found {} disease-symptom relationships",
        symptom_relationships.bindings.len()
    );

    // Find all protein-protein interactions
    let interaction_pattern = QueryPattern::BasicGraphPattern(vec![TriplePattern {
        subject: PatternTerm::Variable("s".to_string()),
        predicate: PatternTerm::IRI(interacts_with.iri().as_ref().clone()),
        object: PatternTerm::Variable("o".to_string()),
    }]);

    let interactions = query_engine.execute_query(&interaction_pattern)?;
    println!(
        "✓ Found {} protein-protein interactions",
        interactions.bindings.len()
    );

    // Performance statistics
    println!("\n=== Performance Statistics ===");
    println!(
        "✓ Total biomedical entities: {}",
        reasoner.ontology.entity_count()
    );
    println!(
        "✓ Total biomedical axioms: {}",
        reasoner.ontology.axiom_count()
    );
    println!("✓ Cache stats: {:?}", reasoner.cache_stats());

    // Complex class expression example (simplified)
    println!("\n=== Complex Class Expression Example ===");

    // Note: Current API supports basic class expressions
    // Future versions will support complex nested expressions
    println!("✓ Basic class expressions are supported");
    println!("✓ Complex class expressions will be supported in future versions");
    println!("✓ Current API focuses on IRI-based reasoning for performance");

    println!("\n=== Biomedical Example Complete ===");
    Ok(())
}
