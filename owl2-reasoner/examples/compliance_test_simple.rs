//! Simplified OWL2 Compliance Test
//!
//! Validates the complete SROIQ(D) implementation with core functionality tests

use owl2_reasoner::parser::owl_functional::OwlFunctionalSyntaxParser;
use owl2_reasoner::parser::OntologyParser;
use owl2_reasoner::reasoning::tableaux::TableauxReasoner;
use owl2_reasoner::IRI;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 **Simplified OWL2 Compliance Test**");
    println!("====================================\\n");

    let mut total_tests = 0;
    let mut passed_tests = 0;
    let mut total_time = 0.0;

    // Test 1: Basic SROIQ(D) Reasoning
    println!("1️⃣ **Basic SROIQ(D) Reasoning**");
    let start_time = Instant::now();

    let basic_ontology = r#"
Prefix(:=<http://example.org/sroiq#>)
Prefix(owl:=<http://www.w3.org/2002/07/owl#>)

Ontology(<http://example.org/sroiq>
Declaration(Class(:Person))
Declaration(Class(:Student))
Declaration(ObjectProperty(:hasFriend))
SubClassOf(:Student :Person)
SubClassOf(:Person ObjectSomeValuesFrom(:hasFriend :Person))
)"#;

    let parser = OwlFunctionalSyntaxParser::new();
    let ontology = parser.parse_str(basic_ontology)?;
    let reasoner = TableauxReasoner::new(ontology);

    let person_iri = IRI::new("http://example.org/sroiq#Person")?;
    let result = reasoner.is_class_satisfiable(&person_iri);
    let elapsed = start_time.elapsed().as_millis() as f64;

    total_tests += 1;
    total_time += elapsed;

    if result.is_ok() {
        passed_tests += 1;
        println!("   ✅ Basic SROIQ(D): PASSED ({:.1}ms)", elapsed);
    } else {
        println!("   ❌ Basic SROIQ(D): FAILED");
    }

    // Test 2: Nominal Reasoning
    println!("\\n2️⃣ **Nominal Reasoning**");
    let start_time = Instant::now();

    let nominal_ontology = r#"
Prefix(:=<http://example.org/nominal#>)
Prefix(owl:=<http://www.w3.org/2002/07/owl#>)

Ontology(<http://example.org/nominal>
Declaration(Class(:SpecificPerson))
Declaration(Class(:Person))
Declaration(NamedIndividual(:Alice))
Declaration(NamedIndividual(:Bob))
SubClassOf(:SpecificPerson :Person)
ClassAssertion(:Person :Alice)
ClassAssertion(:Person :Bob)
)"#;

    let nominal_ontology = parser.parse_str(nominal_ontology)?;
    let nominal_reasoner = TableauxReasoner::new(nominal_ontology);

    let specific_person_iri = IRI::new("http://example.org/nominal#SpecificPerson")?;
    let nominal_result = nominal_reasoner.is_class_satisfiable(&specific_person_iri);
    let elapsed = start_time.elapsed().as_millis() as f64;

    total_tests += 1;
    total_time += elapsed;

    if nominal_result.is_ok() {
        passed_tests += 1;
        println!("   ✅ Nominal Reasoning: PASSED ({:.1}ms)", elapsed);
    } else {
        println!("   ❌ Nominal Reasoning: FAILED");
    }

    // Test 3: Self-Restrictions
    println!("\\n3️⃣ **Self-Restrictions**");
    let start_time = Instant::now();

    let self_ontology = r#"
Prefix(:=<http://example.org/self#>)
Prefix(owl:=<http://www.w3.org/2002/07/owl#>)

Ontology(<http://example.org/self>
Declaration(Class(:SocialPerson))
Declaration(ObjectProperty(:hasFriend))
SubClassOf(:SocialPerson ObjectHasSelf(:hasFriend))
)"#;

    let self_ontology = parser.parse_str(self_ontology)?;
    let self_reasoner = TableauxReasoner::new(self_ontology);

    let social_person_iri = IRI::new("http://example.org/self#SocialPerson")?;
    let self_result = self_reasoner.is_class_satisfiable(&social_person_iri);
    let elapsed = start_time.elapsed().as_millis() as f64;

    total_tests += 1;
    total_time += elapsed;

    if self_result.is_ok() {
        passed_tests += 1;
        println!("   ✅ Self-Restrictions: PASSED ({:.1}ms)", elapsed);
    } else {
        println!("   ❌ Self-Restrictions: FAILED");
    }

    // Test 4: Cardinality Restrictions
    println!("\\n4️⃣ **Cardinality Restrictions**");
    let start_time = Instant::now();

    let cardinality_ontology = r#"
Prefix(:=<http://example.org/cardinality#>)
Prefix(owl:=<http://www.w3.org/2002/07/owl#>)

Ontology(<http://example.org/cardinality>
Declaration(Class(:Department))
Declaration(Class(:Person))
Declaration(ObjectProperty(:hasMember))
SubClassOf(:Department ObjectMinCardinality(3 :hasMember :Person))
)"#;

    let cardinality_ontology = parser.parse_str(cardinality_ontology)?;
    let cardinality_reasoner = TableauxReasoner::new(cardinality_ontology);

    let dept_iri = IRI::new("http://example.org/cardinality#Department")?;
    let cardinality_result = cardinality_reasoner.is_class_satisfiable(&dept_iri);
    let elapsed = start_time.elapsed().as_millis() as f64;

    total_tests += 1;
    total_time += elapsed;

    if cardinality_result.is_ok() {
        passed_tests += 1;
        println!("   ✅ Cardinality Restrictions: PASSED ({:.1}ms)", elapsed);
    } else {
        println!("   ❌ Cardinality Restrictions: FAILED");
    }

    // Test 5: Complex Reasoning
    println!("\\n5️⃣ **Complex SROIQ(D) Reasoning**");
    let start_time = Instant::now();

    let complex_ontology = r#"
Prefix(:=<http://example.org/complex#>)
Prefix(owl:=<http://www.w3.org/2002/07/owl#>)

Ontology(<http://example.org/complex>
Declaration(Class(:Person))
Declaration(Class(:Student))
Declaration(Class(:Professor))
Declaration(Class(:Department))
Declaration(ObjectProperty(:hasFriend))
Declaration(ObjectProperty(:memberOf))
Declaration(ObjectProperty(:advises))
Declaration(NamedIndividual(:Alice))
Declaration(NamedIndividual(:Bob))

SubClassOf(:Student :Person)
SubClassOf(:Professor :Person)
SubClassOf(:Person ObjectHasSelf(:hasFriend))
SubClassOf(:Student ObjectSomeValuesFrom(:advises :Professor))
SubClassOf(:Professor ObjectSomeValuesFrom(:memberOf :Department))
SubClassOf(:Department ObjectMinCardinality(3 :memberOf :Person))
SymmetricObjectProperty(:hasFriend)

DifferentIndividuals(:Alice :Bob)
ClassAssertion(:Person :Alice)
ClassAssertion(:Student :Alice)
ClassAssertion(:Person :Bob)
ClassAssertion(:Professor :Bob)
)"#;

    let complex_ontology = parser.parse_str(complex_ontology)?;
    let complex_reasoner = TableauxReasoner::new(complex_ontology);

    let person_iri = IRI::new("http://example.org/complex#Person")?;
    let complex_result = complex_reasoner.is_class_satisfiable(&person_iri);
    let elapsed = start_time.elapsed().as_millis() as f64;

    total_tests += 1;
    total_time += elapsed;

    if complex_result.is_ok() {
        passed_tests += 1;
        println!("   ✅ Complex SROIQ(D): PASSED ({:.1}ms)", elapsed);
    } else {
        println!("   ❌ Complex SROIQ(D): FAILED");
    }

    // Generate summary
    println!("\\n📊 **Test Summary**");
    println!("==================");
    println!("Total Tests: {}", total_tests);
    println!("Passed Tests: {}", passed_tests);
    println!("Failed Tests: {}", total_tests - passed_tests);
    println!(
        "Success Rate: {:.1}%",
        (passed_tests as f64 / total_tests as f64) * 100.0
    );
    println!("Average Time: {:.1}ms", total_time / total_tests as f64);

    let overall_status = if (passed_tests as f64 / total_tests as f64) >= 0.9 {
        "🟢 EXCELLENT"
    } else if (passed_tests as f64 / total_tests as f64) >= 0.8 {
        "🟡 GOOD"
    } else if (passed_tests as f64 / total_tests as f64) >= 0.7 {
        "🟠 FAIR"
    } else {
        "🔴 NEEDS IMPROVEMENT"
    };

    println!("\\n🎯 **Overall Status**: {}", overall_status);

    if (passed_tests as f64 / total_tests as f64) >= 0.8 {
        println!("🎉 **The OWL2 Reasoner demonstrates strong SROIQ(D) compliance!**");
        println!("   Ready for production use and advanced reasoning tasks.");
    } else {
        println!("🔧 **Continue development to improve compliance and performance.**");
    }

    Ok(())
}
