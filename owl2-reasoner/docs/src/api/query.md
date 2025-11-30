# Query Engine

This chapter covers the query engine components and APIs for performing SPARQL-like queries over OWL2 ontologies.

## Overview

The query engine provides:

- **SPARQL-like query syntax** for familiar semantic web querying
- **Pattern matching** with variables and complex graph patterns
- **Filter expressions** for value-based constraints
- **Aggregation and grouping** for data analysis
- **Inference integration** for querying inferred knowledge
- **Performance optimization** with caching and indexing

## Creating a Query Engine

### Basic Query Engine

```rust
use owl2_reasoner::{Ontology, QueryEngine};

// Create a query engine for an ontology
let mut query_engine = QueryEngine::new(ontology);

// Create with a reasoner for inferred knowledge
let reasoner = SimpleReasoner::new(ontology);
let mut query_engine = QueryEngine::with_reasoner(reasoner);
```

### Configuration

```rust
use owl2_reasoner::query::{QueryConfig, OptimizationHint};
use std::time::Duration;

let config = QueryConfig {
    enable_hash_joins: true,
    enable_index_lookups: true,
    cache_results: true,
    cache_size: 5000,
    cache_ttl: Some(Duration::from_secs(300)), // 5 minutes
    optimization_hint: OptimizationHint::Performance,
    max_results: Some(10000),
    enable_parallel_execution: true,
};

let mut query_engine = QueryEngine::with_config(ontology, config);
```

## Query Patterns

### Basic Triple Patterns

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
for result in results {
    if let (Some(child), Some(parent)) = (result.get("child"), result.get("parent")) {
        println!("{} has parent {}", child, parent);
    }
}
```

### Complex Patterns

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
for result in results {
    if let (Some(gc), Some(gp)) = (result.get("grandchild"), result.get("grandparent")) {
        println!("{} is grandchild of {}", gc, gp);
    }
}
```

## Optional Patterns

### Optional Properties

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

### Multiple Optional Patterns

```rust
// Find persons with optional name and age
let query = Query::select()
    .triple(TriplePattern {
        subject: Variable::new("person"),
        predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".into(),
        object: "http://example.org/Person".into(),
    })
    .optional(OptionalPattern::triple(TriplePattern {
        subject: Variable::new("person"),
        predicate: "http://example.org/hasName".into(),
        object: Variable::new("name"),
    }))
    .optional(OptionalPattern::triple(TriplePattern {
        subject: Variable::new("person"),
        predicate: "http://example.org/hasAge".into(),
        object: Variable::new("age"),
    }))
    .build();
```

## Union Patterns

### Alternative Patterns

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
for result in results {
    println!("Educator: {}", result.get("person").unwrap());
}
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
for result in results {
    if let (Some(person), Some(age)) = (result.get("person"), result.get("age")) {
        println!("  {} (age: {})", person, age);
    }
}
```

### String Filtering

```rust
// Find people whose names start with "John"
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
```

### Complex Logical Filters

```rust
use owl2_reasoner::query::{Query, TriplePattern, Variable, FilterExpression};

// Find people aged 25-65 with names containing "John"
let query = Query::select()
    .triple(TriplePattern {
        subject: Variable::new("person"),
        predicate: "http://example.org/hasAge".into(),
        object: Variable::new("age"),
    })
    .triple(TriplePattern {
        subject: Variable::new("person"),
        predicate: "http://example.org/hasName".into(),
        object: Variable::new("name"),
    })
    .filter(FilterExpression::and(
        Box::new(FilterExpression::greater_than_or_equal(Variable::new("age"), 25)),
        Box::new(FilterExpression::less_than_or_equal(Variable::new("age"), 65))
    ))
    .filter(FilterExpression::contains(
        Variable::new("name"),
        "John"
    ))
    .build();
```

## Aggregation Queries

### Count Queries

```rust
use owl2_reasoner::query::{Query, TriplePattern, Variable, AggregateFunction};

// Count individuals per class
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
```

### Multiple Aggregations

```rust
// Get count, min, max, and average age per class
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
    .aggregate(AggregateFunction::count(Variable::new("person")))
    .aggregate(AggregateFunction::min(Variable::new("age")))
    .aggregate(AggregateFunction::max(Variable::new("age")))
    .aggregate(AggregateFunction::average(Variable::new("age")))
    .group_by(Variable::new("class"))
    .build();
```

## Inference-Aware Queries

### Querying Inferred Knowledge

```rust
use owl2_reasoner::{Ontology, SimpleReasoner, QueryEngine};

// Create ontology with subclass relationships
let mut ontology = Ontology::new();
// ... add classes and subclass axioms ...

// Create reasoner and compute inferred relationships
let reasoner = SimpleReasoner::new(ontology);
reasoner.classify()?; // Compute inferred relationships

// Query engine with reasoner returns inferred results
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

### Explicit vs Inferred Results

```rust
// Query only explicit assertions
let explicit_engine = QueryEngine::new(ontology);
let explicit_results = explicit_engine.execute(&query)?;

// Query both explicit and inferred assertions
let inferred_engine = QueryEngine::with_reasoner(reasoner);
let inferred_results = inferred_engine.execute(&query)?;

println!("Explicit: {}, Inferred: {}", explicit_results.len(), inferred_results.len());
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
    enable_parallel_execution: true,
    ..Default::default()
};

let query_engine = QueryEngine::with_config(ontology, config);
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

// Get estimated cost
let cost = query_engine.estimate_cost(&query)?;
println!("Estimated query cost: {}", cost);
```

### Join Ordering

```rust
// Manually specify join order for better performance
let query = Query::select()
    .triple(TriplePattern {
        subject: Variable::new("person"),
        predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".into(),
        object: "http://example.org/Person".into(),
    })
    .triple(TriplePattern {
        subject: Variable::new("person"),
        predicate: "http://example.org/hasAge".into(),
        object: Variable::new("age"),
    })
    .join_order(vec![0, 1]) // Execute first triple, then second
    .build();
```

## Performance Tuning

### Query Caching

```rust
use std::time::Duration;

let config = QueryConfig {
    cache_results: true,
    cache_size: 1000,
    cache_ttl: Some(Duration::from_secs(300)), // 5 minutes
    enable_parallel_execution: true,
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

### Parallel Execution

```rust
// Enable parallel query execution
let config = QueryConfig {
    enable_parallel_execution: true,
    max_parallel_threads: Some(4),
    parallel_threshold: 1000, // Use parallel for result sets > 1000
    ..Default::default()
};

let query_engine = QueryEngine::with_config(ontology, config);
```

## Advanced Query Features

### Subqueries

```rust
use owl2_reasoner::query::{Query, TriplePattern, Variable, SubQuery};

// Find people who are parents of adults
let adult_query = Query::select()
    .triple(TriplePattern {
        subject: Variable::new("adult"),
        predicate: "http://example.org/hasAge".into(),
        object: Variable::new("age"),
    })
    .filter(FilterExpression::greater_than_or_equal(Variable::new("age"), 18))
    .build();

let parent_query = Query::select()
    .triple(TriplePattern {
        subject: Variable::new("parent"),
        predicate: "http://example.org/hasChild".into(),
        object: Variable::new("child"),
    })
    .filter(FilterExpression::in_set(
        Variable::new("parent"),
        SubQuery::new(adult_query, "adult")
    ))
    .build();
```

### Property Paths

```rust
// Find ancestors using property paths
let query = Query::select()
    .triple(TriplePattern {
        subject: Variable::new("descendant"),
        predicate: QueryExpression::property_path(vec![
            "http://example.org/hasParent".into(),
            "http://example.org/hasParent".into() // hasParent+
        ]),
        object: Variable::new("ancestor"),
    })
    .build();
```

### Graph Patterns

```rust
use owl2_reasoner::query::{Query, TriplePattern, Variable, GraphPattern};

// Complex graph pattern with constraints
let pattern = GraphPattern::and(vec![
    TriplePattern::basic(
        Variable::new("person"),
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".into(),
        "http://example.org/Person".into(),
    ),
    TriplePattern::basic(
        Variable::new("person"),
        "http://example.org/hasParent".into(),
        Variable::new("parent"),
    ),
    TriplePattern::basic(
        Variable::new("parent"),
        "http://example.org/hasAge".into(),
        Variable::new("parent_age"),
    ),
])
.filter(FilterExpression::greater_than(Variable::new("parent_age"), 30));

let query = Query::select().graph_pattern(pattern).build();
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
    Err(QueryError::MemoryError) => {
        eprintln!("Insufficient memory for query");
    }
    Err(err) => {
        eprintln!("Other query error: {}", err);
    }
}
```

### Validation

```rust
// Validate query before execution
match query_engine.validate(&query) {
    Ok(()) => {
        let results = query_engine.execute(&query)?;
        println!("Query executed successfully");
    }
    Err(validation_errors) => {
        for error in validation_errors {
            println!("Validation error: {}", error);
        }
    }
}
```

## Best Practices

1. **Use specific patterns**: More selective patterns are faster
2. **Enable inference when needed**: Use reasoner for inferred knowledge
3. **Optimize join order**: Most selective patterns first
4. **Use caching**: Enable result caching for repeated queries
5. **Batch operations**: Use batch queries for multiple similar operations
6. **Monitor performance**: Use execution plans to understand query behavior
7. **Handle timeouts**: Set reasonable timeouts for complex queries

## Summary

The query engine provides comprehensive SPARQL-like querying capabilities:

- **Pattern matching** with variables and complex graph patterns
- **Filter expressions** for value-based constraints
- **Aggregation and grouping** for data analysis
- **Inference integration** for querying inferred knowledge
- **Performance optimization** with caching and parallel execution
- **Advanced features** like subqueries and property paths

These query capabilities enable powerful information retrieval and analysis over OWL2 ontologies.