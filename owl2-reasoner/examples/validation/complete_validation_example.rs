//! Complete Validation Example
//!
//! This example demonstrates how to use the OWL2 reasoner validation framework
//! to run comprehensive validation and generate reports.

use owl2_reasoner::validation::execution_engine::ValidationExecutionEngine;
use owl2_reasoner::Ontology;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🔬 OWL2 Reasoner Complete Validation Example");
    println!("==========================================");

    // Create validation execution engine
    let mut engine = ValidationExecutionEngine::new()?;
    println!("✅ Validation execution engine created successfully");

    // Run comprehensive validation
    println!("\n🚀 Starting comprehensive validation...");
    let validation_result = engine.run_comprehensive_validation()?;
    
    println!("✅ Comprehensive validation completed in {:?}", validation_result.total_duration);
    println!("📊 Overall score: {:.1}%", validation_result.overall_score * 100.0);
    println!("🎯 Success: {}", validation_result.success);

    // Display validation results
    display_validation_results(&validation_result);

    // Save results to file
    let output_path = PathBuf::from("validation_results.json");
    engine.save_results(&validation_result, &output_path)?;
    println!("💾 Results saved to {:?}", output_path);

    // Generate and display summary
    let summary = engine.generate_summary(&validation_result);
    println!("\n📋 Validation Summary:");
    println!("{}", summary);

    // Run quick validation for comparison
    println!("\n⚡ Running quick validation...");
    let quick_result = engine.run_quick_validation()?;
    println!("✅ Quick validation completed in {:?}", quick_result.total_duration);

    if let Some(ref quick_benchmarks) = quick_result.quick_benchmarks {
        println!("📈 Quick benchmark results:");
        println!("   Reasoning time: {:?}", quick_benchmarks.reasoning_time);
        println!("   Memory usage: {:.1} MB", quick_benchmarks.memory_usage_mb);
        println!("   Success: {}", quick_benchmarks.success);
    }

    println!("\n🎉 Validation example completed successfully!");
    Ok(())
}

fn display_validation_results(result: &owl2_reasoner::validation::execution_engine::ValidationExecutionResult) {
    println!("\n📊 Detailed Validation Results:");
    println!("==============================");

    // Framework validation
    if let Some(ref framework) = result.framework_validation {
        println!("\n🏗️  Framework Validation:");
        if let Some(ref assessment) = framework.overall_assessment {
            println!("   Overall Score: {:.1}%", assessment.overall_score * 100.0);
            println!("   Readiness Level: {:?}", assessment.readiness_level);
            println!("   Strengths: {}", assessment.strengths.len());
            println!("   Areas for Improvement: {}", assessment.areas_for_improvement.len());
        }
    }

    // Performance benchmarks
    if let Some(ref benchmarks) = result.performance_benchmarks {
        println!("\n⚡ Performance Benchmarks:");
        println!("   Performance Score: {:.1}%", benchmarks.performance_score * 100.0);
        println!("   Memory Efficiency: {:.1}%", benchmarks.memory_efficiency_score * 100.0);
        println!("   Scalability Score: {:.1}%", benchmarks.scalability_score * 100.0);
        println!("   Correctness Score: {:.1}%", benchmarks.correctness_score * 100.0);
        println!("   Total Duration: {:?}", benchmarks.total_duration);
    }

    // Empirical validation
    if let Some(ref empirical) = result.empirical_validation {
        println!("\n🔬 Empirical Validation:");
        println!("   Assessment: {}", empirical.overall_assessment);
        println!("   Generated: {}", empirical.generated_at.format("%Y-%m-%d %H:%M:%S UTC"));
        
        if let Some(ref reasoning) = empirical.reasoning_result {
            println!("   Reasoning Operations: {}", reasoning.operation_count);
            println!("   Avg Time/Operation: {:.3} ms", reasoning.avg_time_per_operation_ms);
            println!("   Operations/Second: {:.0}", reasoning.operations_per_second);
        }
        
        if let Some(ref memory) = empirical.memory_result {
            println!("   Memory Efficiency: {:.4} MB/entity", memory.memory_per_entity_mb);
            println!("   Entity Count: {}", memory.entity_count);
        }
        
        if let Some(ref cache) = empirical.cache_result {
            println!("   Cache Hit Rate: {:.1}%", cache.hit_rate * 100.0);
            println!("   Avg Response Time: {:.3} ms", cache.avg_response_time_ms);
        }
    }

    // Issues and warnings
    if !result.critical_issues.is_empty() {
        println!("\n⚠️  Critical Issues:");
        for issue in &result.critical_issues {
            println!("   • {}", issue);
        }
    }

    if !result.warnings.is_empty() {
        println!("\n⚠️  Warnings:");
        for warning in &result.warnings {
            println!("   • {}", warning);
        }
    }

    if !result.errors.is_empty() {
        println!("\n❌ Errors:");
        for error in &result.errors {
            println!("   • {}", error);
        }
    }

    // Performance profile
    if let Some(ref profile) = result.performance_profile {
        println!("\n📈 Performance Profile:");
        println!("   Profile ID: {}", profile.id);
        println!("   Profile Name: {}", profile.name);
        println!("   Total Duration: {:?}", profile.total_duration);
        println!("   Total Operations: {}", profile.performance_metrics.total_operations);
        println!("   Successful Operations: {}", profile.performance_metrics.successful_operations);
        println!("   Failed Operations: {}", profile.performance_metrics.failed_operations);
        println!("   Peak Memory Usage: {} MB", profile.performance_metrics.peak_memory_usage_mb);
        println!("   Operations/Second: {:.0}", profile.performance_metrics.operations_per_second);
    }
}
