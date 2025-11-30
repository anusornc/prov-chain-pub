# 🎓 Interactive OWL2 Reasoner Tutorial

*Learn OWL2 reasoning step-by-step with hands-on examples*

## 📋 Tutorial Overview

This tutorial will teach you:
1. **OWL2 Basics** - Classes, properties, individuals
2. **Reasoning Fundamentals** - Consistency, classification, inference
3. **Advanced Patterns** - Complex axioms, rules, optimizations
4. **Real-World Applications** - Supply chain, biomedical, enterprise

## 🏁 Lesson 1: Your First Ontology

### Learning Objectives
- Create a simple ontology
- Add classes and properties
- Perform basic reasoning

```rust
// Let's create a simple animal taxonomy!
use owl2_reasoner::ontology::Ontology;
use owl2_reasoner::axioms::*;
use owl2_reasoner::reasoning::SimpleReasoner;
use owl2_reasoner::iri::IRI;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create a new ontology
    let mut ontology = Ontology::new();

    // 2. Define our vocabulary
    let animal = IRI::new("http://example.org/Animal")?;
    let mammal = IRI::new("http://example.org/Mammal")?;
    let dog = IRI::new("http://example.org/Dog")?;
    let cat = IRI::new("http://example.org/Cat")?;

    // 3. Add classes (try it yourself!)
    // TODO: Add Animal as a class
    // TODO: Add Mammal as a subclass of Animal
    // TODO: Add Dog and Cat as subclasses of Mammal

    // 4. Create reasoner and check consistency
    let reasoner = SimpleReasoner::new(ontology);
    println!("Is our ontology consistent? {}", reasoner.is_consistent()?);

    Ok(())
}
```

**✏️ Exercise**: Complete the TODO sections above. Then run:
```bash
cargo run --example tutorial_1
```

**💡 Expected Output**: "Is our ontology consistent? true"

---

## 🏗️ Lesson 2: Properties and Restrictions

### Learning Objectives
- Add object and data properties
- Create property restrictions
- Understand existential restrictions

```rust
// Let's add properties to our animal ontology!
use owl2_reasoner::axioms::property_expressions::*;
use owl2_reasoner::axioms::ClassExpression;

fn add_properties_to_ontology(ontology: &mut Ontology) -> Result<(), Box<dyn std::error::Error>> {
    // Define property IRIs
    let eats = IRI::new("http://example.org/eats")?;
    let has_color = IRI::new("http://example.org/hasColor")?;
    let animal = IRI::new("http://example.org/Animal")?;
    let plant = IRI::new("http://example.org/Plant")?;

    // TODO: Add object property "eats"
    // Hint: Use ObjectPropertyDeclarationAxiom

    // TODO: Add data property "hasColor"
    // Hint: Use DataPropertyDeclarationAxiom

    // TODO: Create existential restriction
    // Animals eat at least one plant
    // Hint: Use ObjectSomeValuesFrom and SubClassOfAxiom

    Ok(())
}
```

**✏️ Exercise**:
1. Complete the TODO sections
2. Add restrictions like "Mammals eat something"
3. Test if your ontology is still consistent

**🤔 Think**: What happens if you create contradictory restrictions? Try it!

---

## 🧠 Lesson 3: Reasoning and Inference

### Learning Objectives
- Perform automatic classification
- Discover inferred relationships
- Use different reasoning tasks

```rust
// Let's explore what our reasoner can discover!
fn explore_reasoning(reasoner: &SimpleReasoner) -> Result<(), Box<dyn std::error::Error>> {
    let dog = IRI::new("http://example.org/Dog")?;
    let animal = IRI::new("http://example.org/Animal")?;

    // 1. Check subclass relationships
    // TODO: Test if Dog is a subclass of Animal

    // 2. Get all superclasses of Dog
    // TODO: Get the complete hierarchy

    // 3. Find equivalent classes
    // TODO: Check if any classes are equivalent

    // 4. Test class consistency
    // TODO: Create a contradictory class and test consistency

    Ok(())
}
```

**🔬 Experiment**:
- Create a class `Carnivore` that eats only animals
- Create a class `Herbivore` that eats only plants
- What happens if an animal is both?

---

## 🚀 Lesson 4: Performance and Optimization

### Learning Objectives
- Measure reasoning performance
- Optimize large ontologies
- Use memory management

```rust
// Let's build a larger ontology and optimize it!
fn performance_exploration() -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Instant;

    // 1. Create a larger ontology
    let mut ontology = Ontology::new();

    // TODO: Add 100+ classes in a hierarchy
    // Hint: Use a loop to create classes like Class1, Class2, etc.

    let start = Instant::now();
    let reasoner = SimpleReasoner::new(ontology);
    let reasoning_time = start.elapsed();

    println!("Reasoning took: {:?}", reasoning_time);

    // 2. Test memory usage
    // TODO: Use MemoryManager to track memory changes

    // 3. Try profile optimization
    // TODO: Check if ontology fits EL profile

    Ok(())
}
```

**⚡ Performance Challenge**:
- Create the deepest hierarchy possible
- Measure reasoning time as depth increases
- What's the relationship between depth and performance?

---

## 🏭 Lesson 5: Real-World Application

### Learning Objectives
- Model a real domain
- Handle complex relationships
- Apply enterprise patterns

```rust
// Let's model a simple supply chain!
fn supply_chain_ontology() -> Result<(), Box<dyn std::error::Error>> {
    let mut ontology = Ontology::new();

    // Define supply chain concepts
    let product = IRI::new("http://supply-chain.example.org/Product")?;
    let supplier = IRI::new("http://supply-chain.example.org/Supplier")?;
    let shipment = IRI::new("http://supply-chain.example.org/Shipment")?;

    // TODO: Model the supply chain relationships
    // - Products have suppliers
    // - Shipments contain products
    // - Suppliers can ship products

    // TODO: Add business rules
    // - All products must have a supplier
    // - Shipments must contain at least one product

    // TODO: Test the model
    // - Create instances and check consistency
    // - Verify business rules are enforced

    Ok(())
}
```

**🎯 Real-World Challenge**:
- Model your own domain (hobby, work, research)
- Add at least 5 classes and 3 properties
- Create 2 business rules/restrictions
- Test the model with real data

---

## 🔧 Lesson 6: Advanced Features

### Learning Objectives
- Use profile optimizations
- Apply real-time validation
- Handle complex data types

```rust
// Advanced features showcase!
fn advanced_features() -> Result<(), Box<dyn std::error::Error>> {
    let mut ontology = Ontology::new();

    // 1. Profile Optimization
    // TODO: Check EL, QL, RL profile compatibility
    // TODO: Apply optimization suggestions

    // 2. Real-time Validation
    // TODO: Set up RealtimeMonitor
    // TODO: Validate axioms as they're added

    // 3. Complex Data Types
    // TODO: Add data ranges and restrictions
    // TODO: Use cardinality restrictions

    // 4. Memory Management
    // TODO: Implement rollback functionality
    // TODO: Track memory mutations

    Ok(())
}
```

**🏆 Advanced Challenge**:
- Create an ontology that uses all major OWL2 features
- Optimize it for a specific profile (EL/QL/RL)
- Add comprehensive validation
- Implement rollback for error recovery

---

## 📊 Solutions and Answers

### Lesson 1 Solution
```rust
// Add classes to ontology
ontology.add_axiom(Axiom::Declaration(DeclarationAxiom::Class(
    ClassExpression::Class(Class::new(animal))
)))?;

ontology.add_axiom(Axiom::SubClassOf(SubClassOfAxiom::new(
    ClassExpression::Class(Class::new(mammal)),
    ClassExpression::Class(Class::new(animal))
)))?;

ontology.add_axiom(Axiom::SubClassOf(SubClassOfAxiom::new(
    ClassExpression::Class(Class::new(dog)),
    ClassExpression::Class(Class::new(mammal))
)))?;
```

### Lesson 2 Solution
```rust
// Add properties
ontology.add_axiom(Axiom::Declaration(DeclarationAxiom::ObjectProperty(
    ObjectProperty::new(eats)
)))?;

ontology.add_axiom(Axiom::SubClassOf(SubClassOfAxiom::new(
    ClassExpression::Class(Class::new(animal)),
    ClassExpression::ObjectSomeValuesFrom(
        ObjectPropertyExpression::ObjectProperty(ObjectProperty::new(eats)),
        ClassExpression::Class(Class::new(plant))
    )
)))?;
```

### Performance Benchmarks
Here are some expected performance metrics:

| Operation | Expected Time | Notes |
|-----------|---------------|-------|
| Small ontology consistency check | ~80ns | Constant time |
| 100 classes classification | ~1-5ms | Linear scaling |
| 1000 classes classification | ~10-50ms | Efficient algorithms |
| Memory allocation (1000 entities) | ~1-5ms | Arena allocation |

---

## 🎯 Next Steps

**Congratulations!** 🎉 You've completed the OWL2 Reasoner tutorial. You can now:

1. **Build Your Own Ontology**: Model a domain you care about
2. **Contribute to the Project**: See CONTRIBUTING.md
3. **Explore Advanced Topics**: Read the API documentation
4. **Real-World Applications**: Check the examples directory

**Need Help?**
- Join our community discussions
- Check the API docs: `cargo doc --open`
- Browse examples in `examples/` directory
- Report issues on GitHub

**Happy Reasoning!** 🦉✨