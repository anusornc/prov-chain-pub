#!/bin/bash

# Documentation Update Script
# Usage: ./update_docs.sh "Description of changes"

set -e

DESCRIPTION=${1:-"Documentation update"}

echo "📚 Updating OWL2 Reasoner Documentation"
echo "📝 Changes: $DESCRIPTION"

# 1. Update Rustdoc documentation
echo "🔧 Building Rustdoc documentation..."
cargo doc --no-deps

# 2. Build and test examples
echo "🧪 Testing examples..."
cargo check --example simple_example
cargo check --example family_ontology
cargo check --example biomedical_ontology
cargo check --example performance_benchmarking
cargo check --example benchmark_cli
cargo check --example comparative_analysis
cargo check --example cache_usage
# Note: test_arena_allocation and test_sroiq_reasoning need API updates

# 3. Build mdbook documentation
echo "📖 Building mdbook documentation..."
mdbook build docs

# 4. Build technical documentation (if Typst is available)
echo "📄 Building technical documentation..."
if command -v typst &> /dev/null; then
    ./scripts/build-technical-docs.sh
else
    echo "⚠️  Typst not installed, skipping technical documentation build"
    echo "   Install Typst: https://github.com/typst/typst"
fi

# 5. Run tests to ensure everything works
echo "✅ Running tests..."
cargo test

# 6. Validate benchmarks compile
echo "📊 Validating benchmarks..."
cargo bench --bench tableaux_main --no-run
cargo bench --bench basic_benchmarks --no-run

# 7. Show documentation locations
echo ""
echo "📚 Documentation Generated Successfully!"
echo ""
echo "📖 mdbook Documentation:"
echo "   file://$(pwd)/docs/book/index.html"
echo ""
echo "🛠️ Rustdoc API Documentation:"
echo "   file://$(pwd)/target/doc/owl2_reasoner/index.html"
echo ""
if [ -f "$(pwd)/docs/technical-documentation/output/OWL2_Reasoner_Technical_Documentation.pdf" ]; then
    echo "📄 Technical Documentation (PDF):"
    echo "   $(pwd)/docs/technical-documentation/output/OWL2_Reasoner_Technical_Documentation.pdf"
    echo ""
fi
echo "💻 Examples:"
echo "   $(pwd)/examples/"
echo ""
echo "🔄 Next steps:"
echo "   1. Review generated documentation"
echo "   2. Test new features manually"
echo "   3. Commit changes with: git commit -m \"docs: $DESCRIPTION\""
echo "   4. Push to repository"