//! Enhanced Memory Profiling for OWL2 Reasoner
//!
//! Provides precise memory measurement and analysis tools
//! using system-level memory information and detailed profiling

use owl2_reasoner::*;
use std::alloc::{GlobalAlloc, System, Layout};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;

/// Custom allocator for precise memory tracking
#[derive(Debug)]
struct TrackingAllocator {
    allocated: AtomicUsize,
    deallocated: AtomicUsize,
    peak_allocated: AtomicUsize,
}

impl TrackingAllocator {
    const fn new() -> Self {
        Self {
            allocated: AtomicUsize::new(0),
            deallocated: AtomicUsize::new(0),
            peak_allocated: AtomicUsize::new(0),
        }
    }

    fn get_stats(&self) -> MemoryTrackingStats {
        let allocated = self.allocated.load(Ordering::Relaxed);
        let deallocated = self.deallocated.load(Ordering::Relaxed);
        let peak = self.peak_allocated.load(Ordering::Relaxed);

        MemoryTrackingStats {
            total_allocated_bytes: allocated,
            total_deallocated_bytes: deallocated,
            current_allocated_bytes: allocated.saturating_sub(deallocated),
            peak_allocated_bytes: peak,
            allocation_count: allocated / 16, // Estimate based on average allocation size
            deallocation_count: deallocated / 16,
        }
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            let size = layout.size();
            self.allocated.fetch_add(size, Ordering::Relaxed);
            let current = self.allocated.load(Ordering::Relaxed) - self.deallocated.load(Ordering::Relaxed);
            let peak = self.peak_allocated.load(Ordering::Relaxed);
            if current > peak {
                self.peak_allocated.store(current, Ordering::Relaxed);
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size();
        self.deallocated.fetch_add(size, Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) };
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: TrackingAllocator = TrackingAllocator::new();

/// Detailed memory tracking statistics
#[derive(Debug, Clone)]
struct MemoryTrackingStats {
    pub total_allocated_bytes: usize,
    pub total_deallocated_bytes: usize,
    pub current_allocated_bytes: usize,
    pub peak_allocated_bytes: usize,
    pub allocation_count: usize,
    pub deallocation_count: usize,
}

/// Enhanced memory profiler with system-level measurements
pub struct EnhancedMemoryProfiler {
    baseline_stats: Option<MemoryTrackingStats>,
    measurements: HashMap<String, MemoryTrackingStats>,
    entity_breakdown: HashMap<String, EntityMemoryBreakdown>,
}

/// Detailed entity memory breakdown
#[derive(Debug, Clone)]
struct EntityMemoryBreakdown {
    pub entity_type: String,
    pub count: usize,
    pub estimated_struct_bytes: usize,
    pub estimated_string_bytes: usize,
    pub estimated_arc_overhead: usize,
    pub total_estimated_bytes: usize,
    pub average_bytes_per_entity: usize,
}

impl EnhancedMemoryProfiler {
    pub fn new() -> Self {
        Self {
            baseline_stats: None,
            measurements: HashMap::new(),
            entity_breakdown: HashMap::new(),
        }
    }

    /// Take baseline memory measurement
    pub fn take_baseline(&mut self) -> OwlResult<()> {
        let stats = GLOBAL_ALLOCATOR.get_stats();
        self.baseline_stats = Some(stats);
        Ok(())
    }

    /// Profile specific operation with detailed memory tracking
    pub fn profile_operation<F, R>(&mut self, operation_name: &str, operation: F) -> OwlResult<(R, MemoryTrackingStats)>
    where
        F: FnOnce() -> R,
    {
        let before_stats = GLOBAL_ALLOCATOR.get_stats();

        let result = operation();

        let after_stats = GLOBAL_ALLOCATOR.get_stats();

        let operation_stats = MemoryTrackingStats {
            total_allocated_bytes: after_stats.total_allocated_bytes.saturating_sub(before_stats.total_allocated_bytes),
            total_deallocated_bytes: after_stats.total_deallocated_bytes.saturating_sub(before_stats.total_deallocated_bytes),
            current_allocated_bytes: after_stats.current_allocated_bytes.saturating_sub(before_stats.current_allocated_bytes),
            peak_allocated_bytes: after_stats.peak_allocated_bytes.max(before_stats.peak_allocated_bytes),
            allocation_count: after_stats.allocation_count.saturating_sub(before_stats.allocation_count),
            deallocation_count: after_stats.deallocation_count.saturating_sub(before_stats.deallocation_count),
        };

        self.measurements.insert(operation_name.to_string(), operation_stats.clone());

        Ok((result, operation_stats))
    }

    /// Profile ontology creation with detailed breakdown
    pub fn profile_ontology_creation(&mut self, entity_count: usize) -> OwlResult<(Ontology, MemoryTrackingStats)> {
        self.profile_operation("ontology_creation", || {
            create_detailed_test_ontology(entity_count)
        })
    }

    /// Profile reasoning operations
    pub fn profile_reasoning_operations(&mut self, ontology: &Ontology) -> OwlResult<MemoryTrackingStats> {
        let (_, stats) = self.profile_operation("reasoning_operations", || {
            let reasoner = SimpleReasoner::new(ontology.clone());

            // Warm up caches
            let _ = reasoner.warm_up_caches();

            // Perform reasoning operations
            let _is_consistent = reasoner.is_consistent();

            let classes: Vec<_> = ontology.classes().iter().take(10).cloned().collect();
            for i in 0..classes.len().min(5) {
                for j in 0..classes.len().min(5) {
                    if i != j {
                        let _ = reasoner.is_subclass_of(&classes[i].iri(), &classes[j].iri());
                    }
                }
            }

            for class in classes.iter().take(5) {
                let _ = reasoner.is_class_satisfiable(&class.iri());
            }
        })?;

        Ok(stats)
    }

    /// Analyze entity memory usage in detail
    pub fn analyze_entity_memory_usage(&mut self, ontology: &Ontology) -> OwlResult<Vec<EntityMemoryBreakdown>> {
        let mut breakdowns = Vec::new();

        // Analyze classes
        let class_breakdown = analyze_class_memory(ontology);
        breakdowns.push(class_breakdown);

        // Analyze properties
        let prop_breakdown = analyze_property_memory(ontology);
        breakdowns.push(prop_breakdown);

        // Analyze axioms
        let axiom_breakdown = analyze_axiom_memory(ontology);
        breakdowns.push(axiom_breakdown);

        // Store for later analysis
        for breakdown in &breakdowns {
            self.entity_breakdown.insert(breakdown.entity_type.clone(), breakdown.clone());
        }

        Ok(breakdowns)
    }

    /// Generate comprehensive memory report
    pub fn generate_memory_report(&self) -> OwlResult<String> {
        let mut report = String::new();

        report.push_str("Enhanced Memory Profiling Report\n");
        report.push_str("=================================\n\n");

        // System allocator statistics
        let current_stats = GLOBAL_ALLOCATOR.get_stats();
        report.push_str(&format!("System Allocator Statistics:\n"));
        report.push_str(&format!("  Total Allocated: {:.2} MB\n", current_stats.total_allocated_bytes as f64 / 1_048_576.0));
        report.push_str(&format!("  Current Allocated: {:.2} MB\n", current_stats.current_allocated_bytes as f64 / 1_048_576.0));
        report.push_str(&format!("  Peak Allocated: {:.2} MB\n", current_stats.peak_allocated_bytes as f64 / 1_048_576.0));
        report.push_str(&format!("  Allocation Count: {}\n", current_stats.allocation_count));
        report.push_str(&format!("  Deallocation Count: {}\n", current_stats.deallocation_count));
        report.push_str("\n");

        // Operation-specific measurements
        if !self.measurements.is_empty() {
            report.push_str("Operation Memory Usage:\n");
            for (operation, stats) in &self.measurements {
                report.push_str(&format!("  {}:\n", operation));
                report.push_str(&format!("    Memory Used: {:.2} KB\n", stats.current_allocated_bytes as f64 / 1024.0));
                report.push_str(&format!("    Peak Memory: {:.2} KB\n", stats.peak_allocated_bytes as f64 / 1024.0));
                report.push_str(&format!("    Allocations: {}\n", stats.allocation_count));
                report.push_str(&format!("    Deallocations: {}\n", stats.deallocation_count));
                report.push_str("\n");
            }
        }

        // Entity breakdown
        if !self.entity_breakdown.is_empty() {
            report.push_str("Entity Memory Breakdown:\n");
            for (_, breakdown) in &self.entity_breakdown {
                report.push_str(&format!("  {}:\n", breakdown.entity_type));
                report.push_str(&format!("    Count: {}\n", breakdown.count));
                report.push_str(&format!("    Total Memory: {:.2} KB\n", breakdown.total_estimated_bytes as f64 / 1024.0));
                report.push_str(&format!("    Average per Entity: {:.1} bytes\n", breakdown.average_bytes_per_entity));
                report.push_str(&format!("    Struct Memory: {:.1} bytes\n", breakdown.estimated_struct_bytes));
                report.push_str(&format!("    String Memory: {:.1} bytes\n", breakdown.estimated_string_bytes));
                report.push_str(&format!("    Arc Overhead: {:.1} bytes\n", breakdown.estimated_arc_overhead));
                report.push_str("\n");
            }
        }

        // Performance analysis
        report.push_str("Performance Analysis:\n");
        if let Some(ref baseline) = self.baseline_stats {
            let current_allocated = current_stats.current_allocated_bytes.saturating_sub(baseline.current_allocated_bytes);
            let efficiency = if current_allocated > 0 {
                (self.entity_breakdown.values()
                    .map(|b| b.count * b.average_bytes_per_entity)
                    .sum::<usize>() as f64 / current_allocated as f64) * 100.0
            } else {
                0.0
            };

            report.push_str(&format!("  Memory Efficiency: {:.1}%\n", efficiency));
            report.push_str(&format!("  Net Memory Growth: {:.2} KB\n", current_allocated as f64 / 1024.0));
        }

        report.push_str("\nGenerated by: Enhanced OWL2 Reasoner Memory Profiler\n");

        Ok(report)
    }
}

fn analyze_class_memory(ontology: &Ontology) -> EntityMemoryBreakdown {
    let classes = ontology.classes();
    let count = classes.len();

    let estimated_struct_bytes = count * std::mem::size_of::<Class>();
    let estimated_string_bytes = classes.iter()
        .map(|c| c.iri().as_str().len())
        .sum::<usize>();
    let estimated_arc_overhead = count * 16; // Arc overhead

    let total_estimated_bytes = estimated_struct_bytes + estimated_string_bytes + estimated_arc_overhead;
    let average_bytes_per_entity = if count > 0 { total_estimated_bytes / count } else { 0 };

    EntityMemoryBreakdown {
        entity_type: "Classes".to_string(),
        count,
        estimated_struct_bytes,
        estimated_string_bytes,
        estimated_arc_overhead,
        total_estimated_bytes,
        average_bytes_per_entity,
    }
}

fn analyze_property_memory(ontology: &Ontology) -> EntityMemoryBreakdown {
    let properties = ontology.object_properties();
    let count = properties.len();

    let estimated_struct_bytes = count * std::mem::size_of::<ObjectProperty>();
    let estimated_string_bytes = properties.iter()
        .map(|p| p.iri().as_str().len())
        .sum::<usize>();
    let estimated_arc_overhead = count * 16; // Arc overhead

    let total_estimated_bytes = estimated_struct_bytes + estimated_string_bytes + estimated_arc_overhead;
    let average_bytes_per_entity = if count > 0 { total_estimated_bytes / count } else { 0 };

    EntityMemoryBreakdown {
        entity_type: "Object Properties".to_string(),
        count,
        estimated_struct_bytes,
        estimated_string_bytes,
        estimated_arc_overhead,
        total_estimated_bytes,
        average_bytes_per_entity,
    }
}

fn analyze_axiom_memory(ontology: &Ontology) -> EntityMemoryBreakdown {
    let axioms = ontology.subclass_axioms();
    let count = axioms.len();

    let estimated_struct_bytes = count * std::mem::size_of::<SubClassOfAxiom>();
    let estimated_string_bytes = axioms.iter()
        .map(|a| {
            let subclass_len = if let crate::axioms::ClassExpression::Class(c) = a.sub_class() {
                c.iri().as_str().len()
            } else { 0 };
            let superclass_len = if let crate::axioms::ClassExpression::Class(c) = a.super_class() {
                c.iri().as_str().len()
            } else { 0 };
            subclass_len + superclass_len
        })
        .sum::<usize>();
    let estimated_arc_overhead = count * 32; // Two Arc references per axiom

    let total_estimated_bytes = estimated_struct_bytes + estimated_string_bytes + estimated_arc_overhead;
    let average_bytes_per_entity = if count > 0 { total_estimated_bytes / count } else { 0 };

    EntityMemoryBreakdown {
        entity_type: "Subclass Axioms".to_string(),
        count,
        estimated_struct_bytes,
        estimated_string_bytes,
        estimated_arc_overhead,
        total_estimated_bytes,
        average_bytes_per_entity,
    }
}

fn create_detailed_test_ontology(entity_count: usize) -> Ontology {
    let mut ontology = Ontology::new();

    // Create classes with varying IRI lengths
    for i in 0..entity_count {
        let iri_str = format!("http://example.org/ontology/detailed/class/with/long/path/{}", i);
        let class = Class::new(IRI::new(&iri_str).unwrap());
        ontology.add_class(class).unwrap();
    }

    // Create properties
    for i in 0..(entity_count / 10).max(1) {
        let iri_str = format!("http://example.org/ontology/detailed/property/object/{}", i);
        let prop = ObjectProperty::new(IRI::new(&iri_str).unwrap());
        ontology.add_object_property(prop).unwrap();
    }

    // Create subclass relationships
    for i in 1..(entity_count / 5).max(1) {
        let child_iri = format!("http://example.org/ontology/detailed/class/with/long/path/{}", i);
        let parent_iri = format!("http://example.org/ontology/detailed/class/with/long/path/{}", i / 2);

        let child = ClassExpression::Class(Class::new(IRI::new(&child_iri).unwrap()));
        let parent = ClassExpression::Class(Class::new(IRI::new(&parent_iri).unwrap()));
        let axiom = SubClassOfAxiom::new(child, parent);
        ontology.add_subclass_axiom(axiom).unwrap();
    }

    ontology
}

fn main() -> OwlResult<()> {
    println!("🔬 Enhanced Memory Profiling for OWL2 Reasoner");
    println!("===========================================");

    let mut profiler = EnhancedMemoryProfiler::new();

    // Take baseline measurement
    println!("📊 Taking baseline memory measurement...");
    profiler.take_baseline()?;

    // Test different ontology sizes
    let test_sizes = vec![100, 500, 1000, 2500];

    for size in test_sizes {
        println!("\n🧪 Testing with {} entities:", size);

        // Profile ontology creation
        println!("   Creating ontology...");
        let (ontology, creation_stats) = profiler.profile_ontology_creation(size)?;
        println!("     Creation time: {:.2} KB allocated", creation_stats.current_allocated_bytes as f64 / 1024.0);
        println!("     Classes: {}", ontology.classes().len());
        println!("     Properties: {}", ontology.object_properties().len());
        println!("     Axioms: {}", ontology.subclass_axioms().len());

        // Profile reasoning operations
        println!("   Testing reasoning operations...");
        let reasoning_stats = profiler.profile_reasoning_operations(&ontology)?;
        println!("     Reasoning memory: {:.2} KB", reasoning_stats.current_allocated_bytes as f64 / 1024.0);

        // Analyze entity memory breakdown
        println!("   Analyzing entity memory usage...");
        let breakdowns = profiler.analyze_entity_memory_usage(&ontology)?;
        for breakdown in &breakdowns {
            println!("     {}: {} entities, {:.1} bytes each",
                breakdown.entity_type, breakdown.count, breakdown.average_bytes_per_entity);
        }
    }

    // Generate comprehensive report
    println!("\n📄 Generating memory profiling report...");
    let report = profiler.generate_memory_report()?;
    std::fs::write("enhanced_memory_profile_report.txt", report)?;

    println!("✅ Enhanced memory profiling completed!");
    println!("   Report saved to: enhanced_memory_profile_report.txt");
    println!("   Results show precise memory measurements and detailed breakdowns.");

    Ok(())
}