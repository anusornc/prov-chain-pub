# Querying

This chapter covers how to query ontologies using the OWL2 Reasoner's SPARQL-like query engine.

## Query Engine Overview

The OWL2 Reasoner provides a powerful query engine that supports:

- **Pattern Matching**: Complex graph patterns with variables
- **Filter Expressions**: Value-based filtering and constraints
- **Join Optimization**: Efficient hash-based join algorithms
- **Inference Integration**: Queries work with inferred knowledge
- **Multiple Result Formats**: Structured results for different use cases

## Creating a Query Engine

```rust
use owl2_reasoner::{Ontology, QueryEngine};

// Create a query engine for an ontology
let mut query_engine = QueryEngine::new(ontology);

// Or create with a reasoner for inferred knowledge
let reasoner = SimpleReasoner::new(ontology);
let mut query_engine = QueryEngine::with_reasoner(reasoner);
```

## Basic Queries

### Find All Classes

```rust
use owl2_reasoner::query::{Query, TriplePattern, Variable};

// Query for all classes
let query = Query::select()
    .triple(TriplePattern {
        subject: Variable::new("class"),
        predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".into(),
        object: "http://www.w3.org/2002/07/owl#Class".into(),
    })
    .build();

let results = query_engine.execute(&query)?;
println!("Found {} classes:", results.len());

for result in results {
    if let Some(class) = result.get("class") {
        println!("  - {}", class);
    }
}
```

### Find All Individuals of a Class

```rust
// Find all instances of Person class
let query = Query::select()
    .triple(TriplePattern {
        subject: Variable::new("person"),
        predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".into(),
        object: "http://example.org/Person".into(),
    })
    .build();

let results = query_engine.execute(&query)?;
println!("Found {} persons:", results.len());

for result in results {
    if let Some(person) = result.get("person") {
        println!("  - {}", person);
    }
}
```

### Property-Based Queries

```rust
// Find all parent-child relationships
let query = Query::select()
    .triple(TriplePattern {
        subject: Variable::new("child"),
        predicate: "http://example.org/hasParent".into(),
        object: Variable::new("parent"),
    })
    .build();

let results = query_engine.execute(&query)?;
println!("Found {} parent-child relationships:", results.len());

for result in results {
    if let (Some(child), Some(parent)) = (result.get("child"), result.get("parent")) {
        println!("  {} has parent {}", child, parent);
    }
}
```

## Complex Queries

### Multi-Join Queries

```rust
use owl2_reasoner::query::{Query, TriplePattern, Variable};

// Find all grandparents and their grandchildren
let query = Query::select()
    .triple(TriplePattern {
        subject: Variable::new("grandchild"),
        predicate: "http://example.org/hasParent".into(),
        object: Variable::new("parent"),
    })
    .triple(TriplePattern {
        subject: Variable::new("parent"),
        predicate: "http://example.org/hasParent".into(),
        object: Variable::new("grandparent"),
    })
    .build();

let results = query_engine.execute(&query)?;
println!("Found {} grandparent-grandchild relationships:", results.len());

for result in results {
    if let (Some(gc), Some(gp)) = (result.get("grandchild"), result.get("grandparent")) {
        println!("  {} is grandchild of {}", gc, gp);
    }
}
```

### Optional Patterns

```rust
use owl2_reasoner::query::{Query, TriplePattern, Variable, OptionalPattern};

// Find all persons with optional age information
let query = Query::select()
    .triple(TriplePattern {
        subject: Variable::new("person"),
        predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".into(),
        object: "http://example.org/Person".into(),
    })
    .optional(OptionalPattern::triple(TriplePattern {
        subject: Variable::new("person"),
        predicate: "http://example.org/hasAge".into(),
        object: Variable::new("age"),
    }))
    .build();

let results = query_engine.execute(&query)?;

for result in results {
    let person = result.get("person").unwrap();
    if let Some(age) = result.get("age") {
        println!("  {} (age: {})", person, age);
    } else {
        println!("  {} (age unknown)", person);
    }
}
```

### Union Patterns

```rust
use owl2_reasoner::query::{Query, TriplePattern, Variable, UnionPattern};

// Find all teachers or professors
let query = Query::select()
    .union(UnionPattern::triple(TriplePattern {
        subject: Variable::new("person"),
        predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".into(),
        object: "http://example.org/Teacher".into(),
    }))
    .union(UnionPattern::triple(TriplePattern {
        subject: Variable::new("person"),
        predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".into(),
        object: "http://example.org/Professor".into(),
    }))
    .build();

let results = query_engine.execute(&query)?;
println!("Found {} teachers/professors:", results.len());
```

## Filter Expressions

### Basic Filtering

```rust
use owl2_reasoner::query::{Query, TriplePattern, Variable, FilterExpression};

// Find all adults (age >= 18)
let query = Query::select()
    .triple(TriplePattern {
        subject: Variable::new("person"),
        predicate: "http://example.org/hasAge".into(),
        object: Variable::new("age"),
    })
    .filter(FilterExpression::greater_than_or_equal(
        Variable::new("age"),
        18
    ))
    .build();

let results = query_engine.execute(&query)?;
println!("Found {} adults:", results.len());

for result in results {
    if let (Some(person), Some(age)) = (result.get("person"), result.get("age")) {
        println!("  {} (age: {})", person, age);
    }
}
```

### String Filtering

```rust
use owl2_reasoner::query::{Query, TriplePattern, Variable, FilterExpression};

// Find all people whose names start with "John"
let query = Query::select()
    .triple(TriplePattern {
        subject: Variable::new("person"),
        predicate: "http://example.org/hasName".into(),
        object: Variable::new("name"),
    })
    .filter(FilterExpression::starts_with(
        Variable::new("name"),
        "John"
    ))
    .build();

let results = query_engine.execute(&query)?;
println!("Found {} people named 'John*':", results.len());
```

### Complex Filters

```rust
use owl2_reasoner::query::{Query, TriplePattern, Variable, FilterExpression, LogicalExpression};

// Find people between 25 and 65 years old
let query = Query::select()
    .triple(TriplePattern {
        subject: Variable::new("person"),
        predicate: "http://example.org/hasAge".into(),
        object: Variable::new("age"),
    })
    .filter(FilterExpression::and(
        Box::new(FilterExpression::greater_than_or_equal(Variable::new("age"), 25)),
        Box::new(FilterExpression::less_than_or_equal(Variable::new("age"), 65))
    ))
    .build();

let results = query_engine.execute(&query)?;
println!("Found {} people aged 25-65:", results.len());
```

## Aggregation Queries

### Count Queries

```rust
use owl2_reasoner::query::{Query, TriplePattern, Variable, AggregateFunction};

// Count number of individuals per class
let query = Query::select()
    .triple(TriplePattern {
        subject: Variable::new("individual"),
        predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".into(),
        object: Variable::new("class"),
    })
    .aggregate(AggregateFunction::count(Variable::new("individual")))
    .group_by(Variable::new("class"))
    .build();

let results = query_engine.execute(&query)?;

for result in results {
    if let (Some(class), Some(count)) = (result.get("class"), result.get("count")) {
        println!("  {}: {} instances", class, count);
    }
}
```

### Average Queries

```rust
// Find average age per class
let query = Query::select()
    .triple(TriplePattern {
        subject: Variable::new("person"),
        predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".into(),
        object: Variable::new("class"),
    })
    .triple(TriplePattern {
        subject: Variable::new("person"),
        predicate: "http://example.org/hasAge".into(),
        object: Variable::new("age"),
    })
    .aggregate(AggregateFunction::average(Variable::new("age")))
    .group_by(Variable::new("class"))
    .build();

let results = query_engine.execute(&query)?;

for result in results {
    if let (Some(class), Some(avg_age)) = (result.get("class"), result.get("avg_age")) {
        println!("  {}: average age {}", class, avg_age);
    }
}
```

## Inference-Aware Queries

### Querying Inferred Knowledge

```rust
use owl2_reasoner::{Ontology, SimpleReasoner, QueryEngine};

// Create ontology with subclass relationships
let mut ontology = Ontology::new();
// ... add classes and subclass axioms ...

// Create reasoner and enable inference
let reasoner = SimpleReasoner::new(ontology);
reasoner.classify()?; // Compute inferred relationships

// Query engine with reasoner will return inferred results
let mut query_engine = QueryEngine::with_reasoner(reasoner);

// Find all instances of Person (including inferred)
let query = Query::select()
    .triple(TriplePattern {
        subject: Variable::new("person"),
        predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".into(),
        object: "http://example.org/Person".into(),
    })
    .build();

let results = query_engine.execute(&query)?;
println!("Found {} persons (including inferred):", results.len());
```

## Query Optimization

### Index Utilization

```rust
use owl2_reasoner::query::{QueryConfig, OptimizationHint};

let config = QueryConfig {
    enable_hash_joins: true,
    enable_index_lookups: true,
    cache_results: true,
    optimization_hint: OptimizationHint::Performance,
};

let mut query_engine = QueryEngine::with_config(ontology, config);
```

### Query Planning

```rust
// Get query execution plan
let query = Query::select()
    .triple(TriplePattern {
        subject: Variable::new("person"),
        predicate: "http://example.org/hasParent".into(),
        object: Variable::new("parent"),
    })
    .triple(TriplePattern {
        subject: Variable::new("parent"),
        predicate: "http://example.org/hasAge".into(),
        object: Variable::new("age"),
    })
    .build();

let plan = query_engine.explain_plan(&query)?;
println!("Query execution plan: {:?}", plan);
```

## Performance Tuning

### Query Caching

```rust
use std::time::Duration;

let config = QueryConfig {
    cache_results: true,
    cache_ttl: Some(Duration::from_secs(300)), // 5 minutes
    cache_size: 1000,
    ..Default::default()
};

let mut query_engine = QueryEngine::with_config(ontology, config);

// Execute same query multiple times (cached after first)
let results1 = query_engine.execute(&query)?;
let results2 = query_engine.execute(&query)?; // Faster, from cache
```

### Batch Queries

```rust
// Execute multiple queries efficiently
let queries = vec![
    query_for_all_classes(),
    query_for_all_individuals(),
    query_for_parent_child_relationships(),
];

let results = query_engine.execute_batch(&queries)?;

for (i, result) in results.iter().enumerate() {
    println!("Query {} returned {} results", i + 1, result.len());
}
```

## Error Handling

### Query Errors

```rust
use owl2_reasoner::query::QueryError;

match query_engine.execute(&query) {
    Ok(results) => {
        println!("Query successful: {} results", results.len());
    }
    Err(QueryError::SyntaxError(msg)) => {
        eprintln!("Query syntax error: {}", msg);
    }
    Err(QueryError::TypeError(msg)) => {
        eprintln!("Query type error: {}", msg);
    }
    Err(QueryError::TimeoutError) => {
        eprintln!("Query timed out");
    }
    Err(err) => {
        eprintln!("Other query error: {}", err);
    }
}
```

## Best Practices

1. **Use specific patterns**: More specific queries are faster
2. **Enable inference when needed**: Use reasoner for inferred knowledge
3. **Optimize joins**: Order patterns from most selective to least selective
4. **Use caching**: Enable result caching for repeated queries
5. **Batch operations**: Use batch queries for multiple similar queries
6. **Monitor performance**: Use execution plans to understand query behavior
7. **Handle timeouts**: Set reasonable timeouts for complex queries

## Summary

This chapter covered the comprehensive querying capabilities of the OWL2 Reasoner:

- Basic and complex query patterns
- Filter expressions and constraints
- Aggregation and grouping
- Inference-aware querying
- Performance optimization and caching
- Error handling and best practices

**Next**: [Performance Optimization](performance.md) - Learn how to optimize reasoning and query performance.