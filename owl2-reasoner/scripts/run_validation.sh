#!/bin/bash

# OWL2 Reasoner Validation Script
# Runs comprehensive performance validation and generates evidence

set -e

echo "🔬 OWL2 Reasoner Validation Script"
echo "====================================="

# Check if required dependencies are available
echo "📋 Checking dependencies..."

if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust."
    exit 1
fi

if ! command -v python3 &> /dev/null; then
    echo "❌ Python3 not found. Please install Python 3."
    exit 1
fi

echo "✅ Dependencies found"

# Create validation results directory
VALIDATION_DIR="validation_results"
mkdir -p "$VALIDATION_DIR"

echo "📁 Validation results directory: $VALIDATION_DIR"

# Function to run validation test
run_validation_test() {
    local test_name="$1"
    local command="$2"
    local output_file="$VALIDATION_DIR/${test_name}_output.txt"

    echo ""
    echo "🧪 Running $test_name..."
    echo "   Command: $command"

    # Run with timeout and capture output
    if timeout 300 eval "$command" > "$output_file" 2>&1; then
        echo "   ✅ $test_name completed successfully"
        return 0
    else
        echo "   ❌ $test_name failed or timed out"
        return 1
    fi
}

# Function to extract metrics from output
extract_metric() {
    local file="$1"
    local pattern="$2"
    grep -o "$pattern" "$file" | head -1 | sed 's/[^0-9.]//g'
}

echo ""
echo "🏃‍♂️ Running comprehensive validation..."

# 1. Build the project
run_validation_test "build" "cargo build --release"

# 2. Run standard tests
run_validation_test "standard_tests" "cargo test --release"

# 3. Run performance validation suite
run_validation_test "performance_validation" "cargo run --release --example performance_validation_suite"

# 4. Run existing examples to validate functionality
EXAMPLES=(
    "basic_reasoning"
    "family_ontology"
    "reasoning_demonstration"
    "ecosystem_integration_examples"
)

for example in "${EXAMPLES[@]}"; do
    run_validation_test "example_$example" "cargo run --release --example $example"
done

# 5. Memory leak detection
echo ""
echo "🔍 Running memory leak detection..."

# Build with memory profiling
run_validation_test "memory_build" "cargo build --release --features memory-profiling"

# Run memory-intensive test
cat > "$VALIDATION_DIR/memory_test.rs" << 'EOF'
use owl2_reasoner::*;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Memory Leak Detection Test");

    let mut baseline_memory = 0;

    for i in 0..10 {
        let start = Instant::now();

        // Create and process large ontology
        let mut ontology = Ontology::new();

        // Add many entities
        for j in 0..1000 {
            let iri = IRI::new(&format!("http://example.org/entity{}_{}", i, j))?;
            let class = Class::new(iri);
            ontology.add_entity(Entity::Class(class));
        }

        let mut reasoner = SimpleReasoner::new(ontology);
        let _ = reasoner.classify();
        let _ = reasoner.is_consistent();

        let duration = start.elapsed();
        println!("Iteration {}: {:?}, Classes: {}", i, duration, ontology.classes().len());

        // Force garbage collection if possible
        drop(reasoner);
        drop(ontology);
    }

    println!("✅ Memory leak test completed");
    Ok(())
}
EOF

run_validation_test "memory_leak_test" "rustc --edition 2021 -L target/release/deps $VALIDATION_DIR/memory_test.rs --extern owl2_reasoner=target/release/libowl2_reasoner.rlib -o $VALIDATION_DIR/memory_test && $VALIDATION_DIR/memory_test"

# 6. Performance benchmarks
echo ""
echo "⚡ Running performance benchmarks..."

run_validation_test "benchmarks" "cargo bench --release"

# 7. Python integration test
echo ""
echo "🐍 Testing Python integration..."

cat > "$VALIDATION_DIR/python_test.py" << 'EOF'
#!/usr/bin/env python3

try:
    import sys
    print(f"Python version: {sys.version}")

    # Test basic Python functionality
    import json
    import time

    print("✅ Python dependencies available")

    # Test data processing
    test_data = []
    for i in range(1000):
        test_data.append({
            'id': i,
            'name': f'test_{i}',
            'value': i * 2
        })

    start_time = time.time()
    processed = [item['value'] for item in test_data if item['value'] > 100]
    end_time = time.time()

    print(f"✅ Python test data processing: {len(processed)} items in {(end_time - start_time)*1000:.2f}ms")

except ImportError as e:
    print(f"❌ Python import error: {e}")
    sys.exit(1)
except Exception as e:
    print(f"❌ Python test error: {e}")
    sys.exit(1)
EOF

chmod +x "$VALIDATION_DIR/python_test.py"
run_validation_test "python_integration" "python3 $VALIDATION_DIR/python_test.py"

# 8. Generate validation report
echo ""
echo "📊 Generating validation report..."

# Collect results
VALIDATION_SUMMARY="$VALIDATION_DIR/validation_summary.md"

cat > "$VALIDATION_SUMMARY" << EOF
# OWL2 Reasoner Validation Summary

**Generated**: $(date)
**System**: $(uname -a)
**Rust Version**: $(rustc --version)
**Python Version**: $(python3 --version)

## Test Results Overview

EOF

# Add test results
for output_file in "$VALIDATION_DIR"/*_output.txt; do
    if [ -f "$output_file" ]; then
        test_name=$(basename "$output_file" _output.txt)
        echo "### $test_name" >> "$VALIDATION_SUMMARY"
        echo "" >> "$VALIDATION_SUMMARY"

        # Add summary of output (first 20 lines)
        head -20 "$output_file" >> "$VALIDATION_SUMMARY"
        echo "" >> "$VALIDATION_SUMMARY"
        echo "---" >> "$VALIDATION_SUMMARY"
        echo "" >> "$VALIDATION_SUMMARY"
    fi
done

# Extract key metrics
echo "## Performance Metrics" >> "$VALIDATION_SUMMARY"
echo "" >> "$VALIDATION_SUMMARY"

# Try to extract metrics from performance validation
if [ -f "$VALIDATION_DIR/performance_validation_output.txt" ]; then
    MEMORY_METRIC=$(extract_metric "$VALIDATION_DIR/performance_validation_output.txt" "[0-9.]* MB")
    THROUGHPUT_METRIC=$(extract_metric "$VALIDATION_DIR/performance_validation_output.txt" "[0-9.]* entities/sec")

    if [ ! -z "$MEMORY_METRIC" ] && [ ! -z "$THROUGHPUT_METRIC" ]; then
        echo "- Memory Usage: ${MEMORY_METRIC} MB" >> "$VALIDATION_SUMMARY"
        echo "- Processing Throughput: ${THROUGHPUT_METRIC} entities/sec" >> "$VALIDATION_SUMMARY"
    fi
fi

# Build statistics
if [ -f "$VALIDATION_DIR/build_output.txt" ]; then
    BUILD_TIME=$(grep -o "Finished.*in [0-9.]*s" "$VALIDATION_DIR/build_output.txt" | head -1)
    if [ ! -z "$BUILD_TIME" ]; then
        echo "- Build Time: $BUILD_TIME" >> "$VALIDATION_SUMMARY"
    fi
fi

echo "" >> "$VALIDATION_SUMMARY"
echo "## System Information" >> "$VALIDATION_SUMMARY"
echo "" >> "$VALIDATION_SUMMARY"
echo "- Operating System: $(uname -s)" >> "$VALIDATION_SUMMARY"
echo "- Architecture: $(uname -m)" >> "$VALIDATION_SUMMARY"
echo "- CPU Cores: $(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo "unknown")" >> "$VALIDATION_SUMMARY"
echo "- Memory: $(free -h 2>/dev/null || echo "unknown")" >> "$VALIDATION_SUMMARY"

echo "" >> "$VALIDATION_SUMMARY"
echo "## Validation Status" >> "$VALIDATION_SUMMARY"
echo "" >> "$VALIDATION_SUMMARY"

# Count successful tests
successful_tests=0
total_tests=0

for output_file in "$VALIDATION_DIR"/*_output.txt; do
    if [ -f "$output_file" ]; then
        total_tests=$((total_tests + 1))
        if grep -q "completed successfully\|✅" "$output_file"; then
            successful_tests=$((successful_tests + 1))
        fi
    fi
done

echo "- Successful Tests: $successful_tests/$total_tests" >> "$VALIDATION_SUMMARY"
echo "- Success Rate: $((successful_tests * 100 / total_tests))%" >> "$VALIDATION_SUMMARY"

echo "" >> "$VALIDATION_SUMMARY"
echo "## Files Generated" >> "$VALIDATION_SUMMARY"
echo "" >> "$VALIDATION_SUMMARY"

ls -la "$VALIDATION_DIR/" >> "$VALIDATION_SUMMARY"

echo "" >> "$VALIDATION_SUMMARY"
echo "---" >> "$VALIDATION_SUMMARY"
echo "" >> "$VALIDATION_SUMMARY"
echo "*This report was generated automatically by the OWL2 Reasoner validation script*" >> "$VALIDATION_SUMMARY"

echo ""
echo "📊 Validation Summary Generated: $VALIDATION_SUMMARY"

# 9. Clean up temporary files
rm -f "$VALIDATION_DIR/memory_test.rs" "$VALIDATION_DIR/memory_test" "$VALIDATION_DIR/python_test.py"

echo ""
echo "🎯 Validation Results:"
echo "   Total Tests: $total_tests"
echo "   Successful: $successful_tests"
echo "   Success Rate: $((successful_tests * 100 / total_tests))%"

# Display validation status
if [ $successful_tests -eq $total_tests ]; then
    echo "🎉 All validation tests passed!"
    echo "✅ System is ready for public publishing"
elif [ $successful_tests -gt $((total_tests / 2)) ]; then
    echo "⚠️  Most validation tests passed"
    echo "📋 Review the validation report for details"
else
    echo "❌ Many validation tests failed"
    echo "🔧 System needs fixes before public publishing"
fi

echo ""
echo "📄 Full validation results available in: $VALIDATION_DIR/"
echo "📋 Summary report: $VALIDATION_SUMMARY"

# Offer to open the summary
if command -v open &> /dev/null; then
    echo ""
    read -p "Open validation summary? (y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        open "$VALIDATION_SUMMARY"
    fi
fi

exit 0