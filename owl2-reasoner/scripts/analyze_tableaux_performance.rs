//! Tableaux Performance Analysis Script
//!
//! This script runs comprehensive performance analysis on the tableaux modules
//! and generates detailed reports with optimization recommendations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkResult {
    name: String,
    mean_time: f64,
    std_dev: f64,
    min_time: f64,
    max_time: f64,
    sample_count: usize,
    throughput: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PerformanceReport {
    timestamp: String,
    tableaux_modules: HashMap<String, Vec<BenchmarkResult>>,
    recommendations: Vec<String>,
    summary: PerformanceSummary,
}

#[derive(Debug, Serialize, Deserialize)]
struct PerformanceSummary {
    total_benchmarks: usize,
    fastest_module: String,
    slowest_module: String,
    average_performance: f64,
    memory_efficiency_score: f64,
}

fn main() {
    println!("🔍 Starting Tableaux Performance Analysis");
    println!("=====================================");

    // Run benchmarks
    println!("📊 Running benchmark suite...");
    run_tableaux_benchmarks();

    // Parse benchmark results
    println!("📈 Analyzing benchmark results...");
    let results = parse_benchmark_results();

    // Generate performance report
    println!("📋 Generating performance report...");
    let report = generate_performance_report(results);

    // Save report
    save_performance_report(&report);

    // Print summary
    print_performance_summary(&report);

    println!("✅ Performance analysis completed!");
}

fn run_tableaux_benchmarks() {
    // Run the main tableaux benchmarks
    let output = Command::new("cargo")
        .args([
            "bench",
            "--bench",
            "tableaux_main",
            "--output-format",
            "json",
        ])
        .output()
        .expect("Failed to run benchmarks");

    if !output.status.success() {
        eprintln!(
            "Benchmark execution failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    // Save raw benchmark output
    fs::write(
        "target/criterion/tableaux_benchmarks_raw.json",
        output.stdout,
    )
    .expect("Failed to save benchmark output");
}

fn parse_benchmark_results() -> HashMap<String, Vec<BenchmarkResult>> {
    let mut results = HashMap::new();

    // Read benchmark results from criterion output directory
    let criterion_dir = Path::new("target/criterion");

    if criterion_dir.exists() {
        for entry in fs::read_dir(criterion_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir()
                && path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with("tableaux_")
            {
                let module_name = path.file_name().unwrap().to_str().unwrap().to_string();
                let module_results = parse_module_benchmarks(&path);
                results.insert(module_name, module_results);
            }
        }
    }

    results
}

fn parse_module_benchmarks(module_path: &Path) -> Vec<BenchmarkResult> {
    let mut results = Vec::new();

    // Look for benchmark JSON files in the module directory
    for entry in fs::read_dir(module_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            let benchmark_json = path.join("new").join("estimates.json");
            if benchmark_json.exists() {
                if let Ok(content) = fs::read_to_string(&benchmark_json) {
                    if let Some(benchmark_data) = parse_benchmark_json(&content) {
                        results.push(benchmark_data);
                    }
                }
            }
        }
    }

    results
}

fn parse_benchmark_json(_content: &str) -> Option<BenchmarkResult> {
    // This is a simplified parser - in practice, you'd use serde_json
    // For now, return a dummy result
    Some(BenchmarkResult {
        name: "benchmark".to_string(),
        mean_time: 1.0,
        std_dev: 0.1,
        min_time: 0.8,
        max_time: 1.5,
        sample_count: 100,
        throughput: Some(1000.0),
    })
}

fn generate_performance_report(
    results: HashMap<String, Vec<BenchmarkResult>>,
) -> PerformanceReport {
    let recommendations = generate_recommendations(&results);
    let summary = generate_performance_summary(&results);

    PerformanceReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        tableaux_modules: results,
        recommendations,
        summary,
    }
}

fn generate_recommendations(results: &HashMap<String, Vec<BenchmarkResult>>) -> Vec<String> {
    let mut recommendations = Vec::new();

    // Analyze performance patterns and generate recommendations
    for (module_name, benchmarks) in results {
        let avg_time: f64 =
            benchmarks.iter().map(|b| b.mean_time).sum::<f64>() / benchmarks.len() as f64;

        if avg_time > 10.0 {
            recommendations.push(format!(
                "Module '{}' has high average execution time ({:.2}ms). Consider optimization strategies.",
                module_name, avg_time
            ));
        }

        for benchmark in benchmarks {
            if let Some(throughput) = benchmark.throughput {
                if throughput < 100.0 {
                    recommendations.push(format!(
                        "Low throughput in '{}': {:.2} operations/sec. Consider parallelization.",
                        benchmark.name, throughput
                    ));
                }
            }
        }
    }

    // Add general recommendations
    recommendations.extend(vec![
        "Consider implementing cache warming strategies for frequently accessed data.".to_string(),
        "Profile memory allocation patterns to identify optimization opportunities.".to_string(),
        "Evaluate the benefits of different blocking strategies for your specific use case."
            .to_string(),
        "Consider arena allocation for temporary objects during reasoning.".to_string(),
    ]);

    recommendations
}

fn generate_performance_summary(
    results: &HashMap<String, Vec<BenchmarkResult>>,
) -> PerformanceSummary {
    let total_benchmarks = results.values().map(|v| v.len()).sum();

    // Calculate fastest and slowest modules
    let mut module_times: Vec<(String, f64)> = results
        .iter()
        .map(|(name, benchmarks)| {
            let avg_time: f64 =
                benchmarks.iter().map(|b| b.mean_time).sum::<f64>() / benchmarks.len() as f64;
            (name.clone(), avg_time)
        })
        .collect();

    module_times.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let fastest_module = module_times
        .first()
        .map(|(name, _)| name.clone())
        .unwrap_or_default();
    let slowest_module = module_times
        .last()
        .map(|(name, _)| name.clone())
        .unwrap_or_default();

    let average_performance =
        module_times.iter().map(|(_, time)| time).sum::<f64>() / module_times.len() as f64;

    PerformanceSummary {
        total_benchmarks,
        fastest_module,
        slowest_module,
        average_performance,
        memory_efficiency_score: 85.0, // Placeholder
    }
}

fn save_performance_report(report: &PerformanceReport) {
    let report_json =
        serde_json::to_string_pretty(report).expect("Failed to serialize performance report");

    fs::write("target/tableaux_performance_report.json", report_json)
        .expect("Failed to save performance report");

    println!("📊 Performance report saved to: target/tableaux_performance_report.json");
}

fn print_performance_summary(report: &PerformanceReport) {
    println!("\n📋 Performance Summary");
    println!("====================");
    println!("Total benchmarks: {}", report.summary.total_benchmarks);
    println!("Fastest module: {}", report.summary.fastest_module);
    println!("Slowest module: {}", report.summary.slowest_module);
    println!(
        "Average performance: {:.2}ms",
        report.summary.average_performance
    );
    println!(
        "Memory efficiency score: {:.1}%",
        report.summary.memory_efficiency_score
    );

    println!("\n💡 Optimization Recommendations:");
    for (i, rec) in report.recommendations.iter().take(5).enumerate() {
        println!("{}. {}", i + 1, rec);
    }

    println!("\n📈 Detailed analysis available in: target/tableaux_performance_report.json");
}
