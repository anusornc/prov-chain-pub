# Parser Interface

This chapter covers the parser components and APIs for reading and writing OWL2 ontologies in different serialization formats.

## Overview

The parser system provides:

- **Multi-format support** for Turtle, RDF/XML, OWL/XML, and N-Triples
- **Unified parser interface** with format auto-detection
- **Streaming and memory-efficient parsing** for large files
- **Import resolution** for handling ontology dependencies
- **Error reporting** with detailed diagnostics
- **Performance optimization** with caching and parallel processing

## Parser Factory

### Creating Parsers

```rust
use owl2_reasoner::parser::{ParserFactory, SerializationFormat};

// Create parser for specific format
let turtle_parser = ParserFactory::create_parser(SerializationFormat::Turtle)?;
let rdfxml_parser = ParserFactory::create_parser(SerializationFormat::RdfXml)?;
let owlexml_parser = ParserFactory::create_parser(SerializationFormat::OwlXml)?;
let ntriples_parser = ParserFactory::create_parser(SerializationFormat::NTriples)?;

// Create parser with configuration
let config = ParserConfig {
    strict_mode: false,
    resolve_imports: true,
    cache_imports: true,
    enable_validation: true,
    max_file_size: Some(100 * 1024 * 1024), // 100MB
};

let configured_parser = ParserFactory::create_parser_with_config(
    SerializationFormat::Turtle,
    config
)?;
```

### Auto-Detection

```rust
use owl2_reasoner::parser::ParserFactory;

// Auto-detect format from file extension
let parser = ParserFactory::create_parser_for_file("ontology.ttl")?;

// Auto-detect format from MIME type
let parser = ParserFactory::create_parser_for_mime_type("text/turtle")?;

// Auto-detect format from content
let content = "<rdf:RDF ...";
let parser = ParserFactory::detect_format(content)?;
```

## Turtle Parser

### Basic Parsing

```rust
use owl2_reasoner::parser::TurtleParser;

let parser = TurtleParser::new();

// Parse from string
let content = r#"
@prefix ex: <http://example.org/> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .

ex:Person a owl:Class .
ex:hasParent a owl:ObjectProperty .
"#;

let ontology = parser.parse_str(content)?;

// Parse from file
let ontology = parser.parse_file("ontology.ttl")?;

// Parse from reader
let file = std::fs::File::open("ontology.ttl")?;
let ontology = parser.parse_reader(file)?;
```

### Advanced Turtle Features

```rust
// Parse with base IRI
let base_iri = "http://example.org/ontology/";
let ontology = parser.parse_str_with_base(content, base_iri)?;

// Parse with prefixes
let mut prefixes = std::collections::HashMap::new();
prefixes.insert("ex".to_string(), "http://example.org/".to_string());
prefixes.insert("owl".to_string(), "http://www.w3.org/2002/07/owl#".to_string());

let ontology = parser.parse_str_with_prefixes(content, prefixes)?;

// Parse with streaming for large files
let mut parser = TurtleParser::new();
parser.enable_streaming(1024 * 1024); // 1MB chunks

let file = std::fs::File::open("large_ontology.ttl")?;
let ontology = parser.parse_streaming(file)?;
```

### Turtle Validation

```rust
// Validate Turtle syntax without creating ontology
match parser.validate_str(content) {
    Ok(()) => println!("Turtle syntax is valid"),
    Err(errors) => {
        for error in errors {
            println!("Line {}: {}", error.line_number, error.message);
        }
    }
}
```

## RDF/XML Parser

### Legacy RDF/XML Parser

```rust
use owl2_reasoner::parser::RdfXmlLegacyParser;

let parser = RdfXmlLegacyParser::new();

// Parse RDF/XML content
let content = r#"<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
                   xmlns:owl="http://www.w3.org/2002/07/owl#">
    <owl:Class rdf:about="http://example.org/Person"/>
</rdf:RDF>"#;

let ontology = parser.parse_str(content)?;

// Parse with configuration
let config = RdfXmlConfig {
    preserve_bnode_ids: true,
    normalize_datatypes: true,
    enable_validation: false,
    chunk_size: 8192,
};

let parser = RdfXmlLegacyParser::with_config(config);
let ontology = parser.parse_str(content)?;
```

### Streaming RDF/XML Parser

```rust
use owl2_reasoner::parser::RdfXmlStreamingParser;

// Use Rio-based streaming parser for better performance
let parser = RdfXmlStreamingParser::new();

// Parse large files efficiently
let file = std::fs::File::open("large_ontology.rdf")?;
let ontology = parser.parse_file(file)?;

// Parse with custom handler
let mut handler = CustomTripleHandler::new();
parser.parse_with_handler(file, &mut handler)?;
```

## OWL/XML Parser

### Parsing OWL/XML

```rust
use owl2_reasoner::parser::OwlXmlParser;

let parser = OwlXmlParser::new();

// Parse OWL/XML content
let content = r#"<Ontology xmlns="http://www.w3.org/2002/07/owl#"
               xml:base="http://example.org/ontology">
    <Declaration>
        <Class IRI="http://example.org/Person"/>
    </Declaration>
</Ontology>"#;

let ontology = parser.parse_str(content)?;

// Parse with validation
let parser = OwlXmlParser::with_validation(true);
let ontology = parser.parse_str(content)?;
```

### OWL/XML Features

```rust
// Parse complete OWL/XML document
let parser = OwlXmlParser::new();
let ontology = parser.parse_file("ontology.owlx")?;

// Extract specific components
let classes = parser.extract_classes(&ontology)?;
let properties = parser.extract_properties(&ontology)?;
let individuals = parser.extract_individuals(&ontology)?;
let axioms = parser.extract_axioms(&ontology)?;
```

## N-Triples Parser

### Basic N-Triples Parsing

```rust
use owl2_reasoner::parser::NTriplesParser;

let parser = NTriplesParser::new();

// Parse N-Triples content
let content = r#"<http://example.org/Person> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/2002/07/owl#Class> .
<http://example.org/John> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example.org/Person> .
<http://example.org/John> <http://example.org/hasAge> "30" ."#;

let ontology = parser.parse_str(content)?;

// Parse line by line for streaming
let reader = std::io::BufReader::new(std::fs::File::open("data.nt")?);
let ontology = parser.parse_streaming(reader)?;
```

## Manchester Syntax Parser

### Parsing Manchester Syntax

```rust
use owl2_reasoner::parser::manchester::ManchesterParser;

let parser = ManchesterParser::new();

// Parse class expression
let expression = parser.parse_class_expression("Person and (hasParent some Adult)")?;

// Parse axiom
let axiom = parser.parse_axiom("Person subClassOf Animal")?;

// Parse complete ontology
let ontology = parser.parse_ontology(manchester_content)?;
```

## OWL Functional Syntax Parser

### Parsing OWL Functional Syntax

```rust
use owl2_reasoner::parser::owl_functional::OwlFunctionalSyntaxParser;

let parser = OwlFunctionalSyntaxParser::new();

// Parse axiom
let axiom = parser.parse_axiom("SubClassOf( ObjectIntersectionOf(Person, Male), Animal )")?;

// Parse ontology
let ontology = parser.parse_file("ontology.ofn")?;

// Parse with detailed error reporting
let result = parser.parse_with_detailed_errors(content);
match result {
    Ok(ontology) => println!("Parsed successfully"),
    Err(errors) => {
        for error in errors {
            println!("Error at {}: {}", error.position, error.message);
        }
    }
}
```

## Serialization

### Writing Ontologies

```rust
use owl2_reasoner::parser::{ParserFactory, SerializationFormat};

// Create Turtle parser for writing
let turtle_parser = ParserFactory::create_parser(SerializationFormat::Turtle)?;

// Serialize ontology to string
let turtle_output = turtle_parser.serialize_ontology(&ontology)?;

// Write to file
turtle_parser.serialize_to_file(&ontology, "output.ttl")?;

// Serialize to writer
let mut file = std::fs::File::create("output.ttl")?;
turtle_parser.serialize_to_writer(&ontology, &mut file)?;
```

### Format Conversion

```rust
// Convert between formats
let input_format = SerializationFormat::Turtle;
let output_format = SerializationFormat::RdfXml;

let input_parser = ParserFactory::create_parser(input_format)?;
let output_parser = ParserFactory::create_parser(output_format)?;

// Read in one format
let ontology = input_parser.parse_file("input.ttl")?;

// Write in another format
output_parser.serialize_to_file(&ontology, "output.rdf")?;
```

## Import Resolution

### Basic Import Resolution

```rust
use owl2_reasoner::parser::ImportResolver;

let resolver = ImportResolver::new();

// Resolve imports automatically
let ontology = resolver.resolve_imports(&base_ontology)?;

// Get import dependencies
let dependencies = resolver.get_dependencies(&ontology)?;
for import in dependencies {
    println!("Import: {}", import);
}

// Cache imports for performance
resolver.enable_caching(true);
let ontology = resolver.resolve_imports_cached(&base_ontology)?;
```

### Custom Import Handler

```rust
// Custom import resolution
struct CustomImportHandler;

impl ImportHandler for CustomImportHandler {
    fn resolve_import(&self, iri: &IRI) -> OwlResult<Ontology> {
        // Custom logic for loading ontologies
        if iri.as_str().starts_with("http://example.org/") {
            // Load from local cache
            self.load_local(iri)
        } else {
            // Load from network
            self.load_from_network(iri)
        }
    }
}

let resolver = ImportResolver::with_handler(Box::new(CustomImportHandler));
```

## Error Handling

### Parser Errors

```rust
use owl2_reasoner::parser::ParseError;

match parser.parse_str(content) {
    Ok(ontology) => {
        println!("Parsed successfully with {} axioms", ontology.axiom_count());
    }
    Err(ParseError::SyntaxError { line, column, message }) => {
        eprintln!("Syntax error at line {}, column {}: {}", line, column, message);
    }
    Err(ParseError::InvalidToken { expected, found, position }) => {
        eprintln!("Expected '{}' but found '{}' at position {}", expected, found, position);
    }
    Err(ParseError::UnsupportedFeature(feature)) => {
        eprintln!("Unsupported feature: {}", feature);
    }
    Err(ParseError::IOError(io_err)) => {
        eprintln!("IO error: {}", io_err);
    }
}
```

### Error Recovery

```rust
// Configure parser for error recovery
let config = ParserConfig {
    strict_mode: false,
    error_recovery: true,
    continue_on_error: true,
    collect_errors: true,
};

let parser = TurtleParser::with_config(config);
let result = parser.parse_str_with_errors(content)?;

match result {
    Ok(ontology) => {
        println!("Parsed with {} warnings", parser.error_count());
    }
    Err(errors) => {
        println!("Failed to parse with {} errors", errors.len());
        for error in errors {
            println!("  {}: {}", error.severity, error.message);
        }
    }
}
```

## Performance Optimization

### Streaming for Large Files

```rust
// Configure for large file processing
let config = ParserConfig {
    enable_streaming: true,
    chunk_size: 64 * 1024, // 64KB chunks
    buffer_size: 1024 * 1024, // 1MB buffer
    max_memory: Some(512 * 1024 * 1024), // 512MB limit
};

let parser = TurtleParser::with_config(config);

// Process large file in chunks
let file = std::fs::File::open("very_large_ontology.ttl")?;
let ontology = parser.parse_streaming(file)?;
```

### Parallel Processing

```rust
// Enable parallel parsing for multiple files
let resolver = ImportResolver::new();
resolver.enable_parallel_processing(true);

let files = vec!["ontology1.ttl", "ontology2.ttl", "ontology3.ttl"];
let ontologies: Vec<OwlResult<Ontology>> = files
    .into_iter()
    .map(|file| resolver.resolve_imports_file(file))
    .collect();
```

### Caching

```rust
// Enable import caching
let resolver = ImportResolver::new();
resolver.enable_caching(true);
resolver.set_cache_size(100); // Cache up to 100 ontologies
resolver.set_cache_ttl(Duration::from_secs(3600)); // 1 hour TTL
```

## Validation

### Syntax Validation

```rust
// Validate syntax without full parsing
let validator = TurtleSyntaxValidator::new();
match validator.validate(content) {
    Ok(()) => println!("Valid Turtle syntax"),
    Err(errors) => {
        for error in errors {
            println!("Line {}: {}", error.line, error.message);
        }
    }
}
```

### Semantic Validation

```rust
// Validate OWL2 constraints
let validator = Owl2Validator::new();
let validation_result = validator.validate(&ontology)?;

if validation_result.is_valid {
    println!("Ontology is semantically valid");
} else {
    println!("Validation issues:");
    for issue in validation_result.issues {
        println!("  - {}: {}", issue.severity, issue.description);
    }
}
```

## Best Practices

1. **Use appropriate parser**: Choose parser based on file format and size
2. **Enable streaming**: For large files to avoid memory issues
3. **Resolve imports**: Handle ontology dependencies properly
4. **Handle errors gracefully**: Provide useful error messages and recovery
5. **Validate input**: Check syntax and semantics before processing
6. **Use caching**: Cache imports for better performance

## Summary

The parser interface provides comprehensive OWL2 serialization support:

- **Multi-format parsing** for all major OWL2 serialization formats
- **Unified interface** with format auto-detection
- **Memory-efficient streaming** for large files
- **Import resolution** for handling dependencies
- **Error handling** with detailed diagnostics
- **Performance optimization** with caching and parallel processing

These parsing capabilities enable the OWL2 Reasoner to work with ontologies from various sources and formats.