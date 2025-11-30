# Git Workflow Strategy for OWL2 Reasoner

## Branch Strategy
- **main**: Stable production-ready code
- **develop**: Integration branch for features
- **feature/***: Individual feature branches
- **hotfix/***: Emergency fixes

## Feature Branch Workflow

### 1. Core Data Model
```bash
git checkout -b feature/core-data-model develop
# Implement IRI management, entities, ontology structure
git checkout develop
git merge --no-ff feature/core-data-model
git branch -d feature/core-data-model
```

### 2. OWL2 Parsers
```bash
git checkout -b feature/owl2-parsers develop
# Implement Turtle, RDF/XML, OWL/XML parsers
git checkout develop
git merge --no-ff feature/owl2-parsers
git branch -d feature/owl2-parsers
```

### 3. Tableaux Reasoning Engine
```bash
git checkout -b feature/tableaux-reasoning develop
# Implement tableaux algorithm and classification
git checkout develop
git merge --no-ff feature/tableaux-reasoning
git branch -d feature/tableaux-reasoning
```

### 4. Rule-based Reasoning
```bash
git checkout -b feature/rule-based-reasoning develop
# Implement SWRL rules and custom rule support
git checkout develop
git merge --no-ff feature/rule-based-reasoning
git branch -d feature/rule-based-reasoning
```

### 5. SPARQL Query Engine
```bash
git checkout -b feature/sparql-query-engine develop
# Implement SPARQL 1.1 query support
git checkout develop
git merge --no-ff feature/sparql-query-engine
git branch -d feature/sparql-query-engine
```

### 6. Test Suite
```bash
git checkout -b feature/test-suite develop
# Create comprehensive OWL2 test cases
git checkout develop
git merge --no-ff feature/test-suite
git branch -d feature/test-suite
```

### 7. Performance Optimization
```bash
git checkout -b feature/performance-optimization develop
# Add indexing, caching, parallelization
git checkout develop
git merge --no-ff feature/performance-optimization
git branch -d feature/performance-optimization
```

### 8. Documentation
```bash
git checkout -b feature/documentation develop
# Create docs, examples, API documentation
git checkout develop
git merge --no-ff feature/documentation
git branch -d feature/documentation
```

## Branch Protection Rules
- **main**: Protected, require PR review and CI checks
- **develop**: Protected, require CI checks
- **feature/***: No protection, developer workflow

## CI/CD Pipeline
- Run tests on all branches
- Run benchmarks on develop and main
- Deploy documentation from main

## Commit Message Format
```
<type>(<scope>): <description>

[optional body]

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

Types: feat, fix, docs, style, refactor, test, chore