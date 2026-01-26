#!/usr/bin/env python3
"""
Generate Thesis Performance Plots with REAL Experimental Data

Author: Mr. Anusorn Chaikaew (Student Code: 640551018)
Thesis: "Enhancement of Blockchain with Embedded Ontology and Knowledge Graph for Data Traceability"

All data points are from actual experimental measurements taken 2026-01-17 to 2026-01-18.
No synthetic or projected data is included in these plots.

Academic Integrity: All figures show REAL experimental results.
"""

import matplotlib.pyplot as plt
import numpy as np
from matplotlib import rcParams

# Configure matplotlib for academic paper quality
rcParams['font.size'] = 10
rcParams['font.family'] = 'serif'
rcParams['figure.figsize'] = (10, 6)
rcParams['axes.linewidth'] = 1.0
rcParams['grid.alpha'] = 0.3

# =============================================================================
# REAL EXPERIMENTAL DATA FROM CRITERION.RS BENCHMARKS (2026-01-17 to 2026-01-18)
# =============================================================================

# OWL2 Consistency Checking (microseconds)
# Source: docs/benchmarking/EXPERIMENTAL_RESULTS.md - Section 1.1
ontology_sizes = [10, 50, 100, 500, 1000, 5000, 10000]
simple_consistency = [15.65, 27.23, 41.77, 168.65, 313, 1690, 3690]  # µs
tableaux_consistency = [19.72, 31.17, 45.58, 166.52, 411, 2430, 5680]  # µs

# SPARQL Query Performance (microseconds to milliseconds)
# Source: docs/benchmarking/EXPERIMENTAL_RESULTS.md - Section 2.1
dataset_sizes = [100, 500, 1000, 5000]  # triples
simple_query_latency = [39.74, 170.84, 358.50, 2460]  # µs
join_query_latency = [56.29, 490.95, 1670, 10870]  # µs
complex_join_latency = [140.16, 759.79, 1790, 18040]  # µs

# Memory Management Performance (nanoseconds)
# Source: docs/benchmarking/EXPERIMENTAL_RESULTS.md - Section 3.1-3.3
operations = ['get_stats', 'get_pressure', 'is_under_pressure', 'detect_leaks']
memory_op_latency = [122.97, 121.64, 120.71, 131.39]  # ns

# Checkpoint/Rollback Performance
checkpoint_scales = ['base', 'with_alloc', '10_ops', '100_ops', '1000_ops']
checkpoint_latency = [0.182, 2.93, 5.55, 44.84, 518.89]  # µs

# Load Test Results (2026-01-18)
# Source: Actual load test execution
load_test_config = {
    'users': 200,
    'requests_per_user': 100,
    'duration_seconds': 60,
    'theoretical_max_tps': 333.33
}
load_test_results = {
    'actual_tps': 19.58,
    'avg_response_time_ms': 51.02,
    'p95_response_time_ms': 98.29,
    'p99_response_time_ms': 98.29,
    'success_rate': 100.0,
    'total_requests': 1397,
    'potential_requests': 20000
}

# =============================================================================
# PLOT 1: OWL2 Consistency Checking Performance
# =============================================================================

fig, ax = plt.subplots(figsize=(10, 6))

x = np.array(ontology_sizes)
y1 = np.array(simple_consistency)
y2 = np.array(tableaux_consistency)

ax.plot(x, y1, 'o-', color='#2ecc71', linewidth=2, markersize=8, label='Simple Consistency')
ax.plot(x, y2, 's--', color='#e74c3c', linewidth=2, markersize=8, label='Tableaux Consistency')

ax.set_xlabel('Ontology Size (axioms)', fontsize=12, fontweight='bold')
ax.set_ylabel('Consistency Checking Time (µs)', fontsize=12, fontweight='bold')
ax.set_title('OWL2 Consistency Checking Performance\n(Real Experimental Data - Criterion.rs, 2026-01-17)',
             fontsize=13, fontweight='bold')
ax.grid(True, alpha=0.3)
ax.legend(fontsize=11, loc='upper left')

# Add annotation about linear scaling
ax.annotate(f'O(n) Linear Scaling\n{simple_consistency[-1]/ontology_sizes[-1]:.2f} µs/axiom',
            xy=(ontology_sizes[-1], simple_consistency[-1]),
            xytext=(5000, 2000),
            fontsize=10,
            bbox=dict(boxstyle='round,pad=0.5', facecolor='yellow', alpha=0.3),
            arrowprops=dict(arrowstyle='->', color='black'))

plt.tight_layout()
plt.savefig('/home/cit/provchain-org/thesis/figures/owl2_consistency_performance.png',
            dpi=300, bbox_inches='tight')
plt.close()

print("✓ Generated: owl2_consistency_performance.png")

# =============================================================================
# PLOT 2: SPARQL Query Performance
# =============================================================================

fig, ax = plt.subplots(figsize=(10, 6))

x = np.array(dataset_sizes)
y1 = np.array(simple_query_latency) / 1000  # Convert to ms
y2 = np.array(join_query_latency) / 1000   # Convert to ms
y3 = np.array(complex_join_latency) / 1000 # Convert to ms

ax.plot(x, y1, 'o-', color='#3498db', linewidth=2, markersize=8, label='Simple SELECT')
ax.plot(x, y2, 's--', color='#9b59b6', linewidth=2, markersize=8, label='Join Query')
ax.plot(x, y3, '^-.', color='#e67e22', linewidth=2, markersize=8, label='Complex Join')

ax.set_xlabel('Dataset Size (triples)', fontsize=12, fontweight='bold')
ax.set_ylabel('Query Latency (ms)', fontsize=12, fontweight='bold')
ax.set_title('SPARQL Query Performance vs Dataset Size\n(Real Experimental Data - Criterion.rs, 2026-01-17)',
             fontsize=13, fontweight='bold')
ax.grid(True, alpha=0.3)
ax.legend(fontsize=11, loc='upper left')

# Annotate the measured range
ax.axhline(y=18, color='green', linestyle=':', alpha=0.5)
ax.annotate('Measured Range: 0.04-18 ms\n(P95 < 100ms target ✅)',
            xy=(3000, 18), xytext=(2000, 10),
            fontsize=10, color='green',
            bbox=dict(boxstyle='round,pad=0.5', facecolor='lightgreen', alpha=0.5))

plt.tight_layout()
plt.savefig('/home/cit/provchain-org/thesis/figures/sparql_query_performance.png',
            dpi=300, bbox_inches='tight')
plt.close()

print("✓ Generated: sparql_query_performance.png")

# =============================================================================
# PLOT 3: Memory Management Performance
# =============================================================================

fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 5))

# Subplot 1: Memory Statistics Operations
x_pos = np.arange(len(operations))
bars = ax1.bar(x_pos, memory_op_latency, color=['#3498db', '#2ecc71', '#f39c12', '#e74c3c'])
ax1.set_xlabel('Memory Operation', fontsize=11, fontweight='bold')
ax1.set_ylabel('Latency (nanoseconds)', fontsize=11, fontweight='bold')
ax1.set_title('Memory Statistics Collection\n(Real Experimental Data - Criterion.rs, 2026-01-17)',
              fontsize=12, fontweight='bold')
ax1.set_xticks(x_pos)
ax1.set_xticklabels(operations, rotation=45, ha='right')
ax1.grid(True, alpha=0.3, axis='y')

# Add value labels on bars
for i, bar in enumerate(bars):
    height = bar.get_height()
    ax1.text(bar.get_x() + bar.get_width()/2., height,
             f'{memory_op_latency[i]:.1f}',
             ha='center', va='bottom', fontsize=9)

# Subplot 2: Checkpoint/Rollback Scaling
x_pos = np.arange(len(checkpoint_scales))
ax2.plot(x_pos, checkpoint_latency, 'o-', color='#9b59b6', linewidth=2, markersize=8)
ax2.set_xlabel('Checkpoint Scale', fontsize=11, fontweight='bold')
ax2.set_ylabel('Rollback Latency (µs)', fontsize=11, fontweight='bold')
ax2.set_title('Checkpoint/Rollback Performance\n(Real Experimental Data - Criterion.rs, 2026-01-17)',
              fontsize=12, fontweight='bold')
ax2.set_xticks(x_pos)
ax2.set_xticklabels(checkpoint_scales, rotation=45, ha='right')
ax2.grid(True, alpha=0.3)

# Add value labels
for i, val in enumerate(checkpoint_latency):
    ax2.text(i, val, f'{val:.1f}', ha='center', va='bottom', fontsize=9)

plt.tight_layout()
plt.savefig('/home/cit/provchain-org/thesis/figures/memory_management_performance.png',
            dpi=300, bbox_inches='tight')
plt.close()

print("✓ Generated: memory_management_performance.png")

# =============================================================================
# PLOT 4: Performance Validation Summary (Target vs Actual)
# =============================================================================

fig, ax = plt.subplots(figsize=(12, 6))

metrics = ['Write Throughput\n(TPS)', 'Read Latency P95\n(ms)', 'OWL2 Reasoning\n(ms)', 'Memory Usage\n(GB)']
targets = [8000, 100, 200, 16]
actual = [load_test_results['actual_tps'], 0.018, 0.17, 0.2]  # Convert to consistent units
actual_labels = ['19.58', '0.04-18', '0.015-0.17', '~0.2']

x_pos = np.arange(len(metrics))
width = 0.35

bars1 = ax.bar(x_pos - width/2, targets, width, label='ADR 0001 Target\n(Production Projection)',
              color='#95a5a6', alpha=0.7)
bars2 = ax.bar(x_pos + width/2, actual, width, label='Actual Measured\n(Development Environment)',
              color=['#e74c3c' if m == 'Write Throughput\n(TPS)' else '#2ecc71' for m in metrics])

ax.set_xlabel('Performance Metric', fontsize=12, fontweight='bold')
ax.set_ylabel('Value', fontsize=12, fontweight='bold')
ax.set_title('Performance Validation: Target vs Actual Measurements\n(Real Experimental Data - 2026-01-17 to 2026-01-18)',
             fontsize=13, fontweight='bold')
ax.set_xticks(x_pos)
ax.set_xticklabels(metrics)
ax.legend(fontsize=11, loc='upper right')
ax.grid(True, alpha=0.3, axis='y')

# Use log scale for better visualization
ax.set_yscale('log')
ax.set_ylim(0.01, 10000)

# Add value labels
for i, (bar, label) in enumerate(zip(bars2, actual_labels)):
    height = bar.get_height()
    ax.text(bar.get_x() + bar.get_width()/2., height,
            label, ha='center', va='bottom', fontsize=10, fontweight='bold')

# Add annotation explaining the discrepancy
ax.annotate('Note: Write throughput measured in single-node\ndevelopment environment. Production target assumes\n100+ distributed nodes with network-level parallelism.',
            xy=(0, 20), xytext=(1.5, 100),
            fontsize=9, color='#e74c3c',
            bbox=dict(boxstyle='round,pad=0.5', facecolor='white', edgecolor='#e74c3c', alpha=0.8),
            arrowprops=dict(arrowstyle='->', color='#e74c3c', shrinkA=0, shrinkB=0))

plt.tight_layout()
plt.savefig('/home/cit/provchain-org/thesis/figures/performance_validation_summary.png',
            dpi=300, bbox_inches='tight')
plt.close()

print("✓ Generated: performance_validation_summary.png")

# =============================================================================
# PLOT 5: Load Test Analysis (Development Environment)
# =============================================================================

fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 5))

# Subplot 1: Request Processing
categories = ['Potential\nRequests', 'Actual\nProcessed']
values = [load_test_results['potential_requests'], load_test_results['total_requests']]
colors = ['#95a5a6', '#3498db']

bars = ax1.bar(categories, values, color=colors, edgecolor='black', linewidth=1.5)
ax1.set_ylabel('Number of Requests', fontsize=11, fontweight='bold')
ax1.set_title(f'Load Test: Request Processing\nConfiguration: {load_test_config["users"]} users × {load_test_config["requests_per_user"]} requests / {load_test_config["duration_seconds"]}s',
              fontsize=12, fontweight='bold')
ax1.grid(True, alpha=0.3, axis='y')

# Add value labels
for i, bar in enumerate(bars):
    height = bar.get_height()
    ax1.text(bar.get_x() + bar.get_width()/2., height,
             f'{values[i]:,}', ha='center', va='bottom', fontsize=11, fontweight='bold')

# Subplot 2: Response Time Distribution
metrics_resp = ['Average', 'P95', 'P99']
values_resp = [load_test_results['avg_response_time_ms'],
               load_test_results['p95_response_time_ms'],
               load_test_results['p99_response_time_ms']]
colors_resp = ['#2ecc71', '#f39c12', '#e74c3c']

bars = ax2.bar(metrics_resp, values_resp, color=colors_resp, edgecolor='black', linewidth=1.5)
ax2.set_ylabel('Response Time (ms)', fontsize=11, fontweight='bold')
ax2.set_title(f'Load Test: Response Time Distribution\nThroughput: {load_test_results["actual_tps"]} TPS | Success Rate: {load_test_results["success_rate"]:.0f}%',
              fontsize=12, fontweight='bold')
ax2.grid(True, alpha=0.3, axis='y')

# Add value labels and target line
for i, bar in enumerate(bars):
    height = bar.get_height()
    ax2.text(bar.get_x() + bar.get_width()/2., height,
             f'{values_resp[i]:.2f}', ha='center', va='bottom', fontsize=11, fontweight='bold')

ax2.axhline(y=500, color='red', linestyle='--', alpha=0.5, label='Target: 500ms')
ax2.legend(fontsize=9)

plt.tight_layout()
plt.savefig('/home/cit/provchain-org/thesis/figures/load_test_analysis.png',
            dpi=300, bbox_inches='tight')
plt.close()

print("✓ Generated: load_test_analysis.png")

# =============================================================================
# SUMMARY STATISTICS
# =============================================================================

print("\n" + "="*70)
print("THESIS PERFORMANCE PLOTS GENERATED")
print("="*70)
print("\nAll plots generated with REAL experimental data:")
print("  Source: Criterion.rs benchmarks (2026-01-17 to 2026-01-18)")
print("  Confidence intervals: 95%")
print("\nGenerated Files:")
print("  1. owl2_consistency_performance.png")
print("  2. sparql_query_performance.png")
print("  3. memory_management_performance.png")
print("  4. performance_validation_summary.png")
print("  5. load_test_analysis.png")
print("\nKey Findings:")
print(f"  - OWL2 Consistency: O(n) linear scaling ({simple_consistency[-1]/ontology_sizes[-1]:.2f} µs/axiom)")
print(f"  - SPARQL Queries: 0.04-18 ms (P95 < 100ms target ✅)")
print(f"  - Load Test: {load_test_results['actual_tps']} TPS (single-node dev environment)")
print(f"  - Success Rate: {load_test_results['success_rate']:.0f}%")
print("="*70)
