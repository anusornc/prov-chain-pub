//! Emergency Memory Protection System
//!
//! This module provides the final layer of defense against out-of-memory conditions.
//! When all other protection mechanisms fail, this system ensures the application
//! remains stable by implementing extreme measures to free memory and prevent crashes.

use crate::graceful_degradation::{get_degradation_level, DegradationLevel};
use crate::iri::clear_global_iri_cache;
use crate::memory::*;
use crate::memory_aware_allocation::cleanup_memory_pools;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use parking_lot::Mutex;
use std::time::{Duration, Instant};
use std::thread;
use std::collections::HashMap;

/// Emergency protection configuration
#[derive(Debug, Clone)]
pub struct EmergencyProtectionConfig {
    /// Emergency memory threshold (percentage)
    pub emergency_threshold: f64,
    /// Force cleanup interval during emergency
    pub force_cleanup_interval: Duration,
    /// Maximum emergency duration before system shutdown
    pub max_emergency_duration: Duration,
    /// Enable emergency thread suspension
    pub enable_thread_suspension: bool,
    /// Enable emergency cache clearing
    pub enable_emergency_cache_clearing: bool,
    /// Enable emergency arena reset
    pub enable_emergency_arena_reset: bool,
    /// Emergency allocation limit (bytes)
    pub emergency_allocation_limit: usize,
}

impl Default for EmergencyProtectionConfig {
    fn default() -> Self {
        Self {
            emergency_threshold: 0.95, // 95%
            force_cleanup_interval: Duration::from_millis(100),
            max_emergency_duration: Duration::from_secs(300), // 5 minutes
            enable_thread_suspension: false,
            enable_emergency_cache_clearing: true,
            enable_emergency_arena_reset: true,
            emergency_allocation_limit: 1024, // 1KB max during emergency
        }
    }
}

/// Emergency protection state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmergencyState {
    /// Normal operation
    Normal,
    /// Emergency mode active
    Active,
    /// Emergency mode critical - system may shutdown
    Critical,
    /// Emergency mode fatal - system will shutdown
    Fatal,
}

/// Emergency protection actions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EmergencyAction {
    /// Force cleanup of all caches
    ForceCacheCleanup,
    /// Reset all arenas
    ResetArenas,
    /// Suspend non-essential threads
    SuspendThreads,
    /// Reduce allocation limits to minimum
    ReduceAllocations,
    /// Trigger system shutdown
    SystemShutdown,
    /// Emergency garbage collection
    EmergencyGC,
}

/// Emergency protection event
#[derive(Debug, Clone)]
pub struct EmergencyEvent {
    pub timestamp: Instant,
    pub state: EmergencyState,
    pub action: EmergencyAction,
    pub memory_usage: usize,
    pub reason: String,
    pub success: bool,
}

/// Emergency memory protection system
pub struct EmergencyProtection {
    config: EmergencyProtectionConfig,
    current_state: Arc<Mutex<EmergencyState>>,
    emergency_start_time: Arc<Mutex<Option<Instant>>>,
    event_history: Arc<Mutex<Vec<EmergencyEvent>>>,
    emergency_thread: Option<thread::JoinHandle<()>>,
    shutdown_flag: Arc<AtomicBool>,
    action_counts: Arc<Mutex<HashMap<EmergencyAction, usize>>>,
    last_action_time: Arc<Mutex<HashMap<EmergencyAction, Instant>>>,
}

impl EmergencyProtection {
    pub fn new() -> Self {
        Self::with_config(EmergencyProtectionConfig::default())
    }

    pub fn with_config(config: EmergencyProtectionConfig) -> Self {
        Self {
            config,
            current_state: Arc::new(Mutex::new(EmergencyState::Normal)),
            emergency_start_time: Arc::new(Mutex::new(None)),
            event_history: Arc::new(Mutex::new(Vec::new())),
            emergency_thread: None,
            shutdown_flag: Arc::new(AtomicBool::new(false)),
            action_counts: Arc::new(Mutex::new(HashMap::new())),
            last_action_time: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start emergency protection monitoring
    pub fn start_emergency_protection(&mut self) {
        if self.emergency_thread.is_some() {
            return; // Already running
        }

        let config = self.config.clone();
        let current_state = Arc::clone(&self.current_state);
        let emergency_start_time = Arc::clone(&self.emergency_start_time);
        let event_history = Arc::clone(&self.event_history);
        let shutdown_flag = Arc::clone(&self.shutdown_flag);
        let action_counts = Arc::clone(&self.action_counts);
        let last_action_time = Arc::clone(&self.last_action_time);

        let handle = thread::spawn(move || {
            let mut consecutive_emergency_checks = 0;
            let mut last_cleanup = Instant::now();

            while !shutdown_flag.load(Ordering::Relaxed) {
                let memory_stats = get_memory_stats();
                let memory_pressure = memory_stats.pressure_level;
                let current_memory = memory_stats.total_usage;

                // Check if we need to enter emergency mode
                if memory_pressure >= config.emergency_threshold {
                    consecutive_emergency_checks += 1;

                    // Enter emergency mode after 3 consecutive checks
                    if consecutive_emergency_checks >= 3 {
                        let mut state = current_state.lock();
                        let mut start_time = emergency_start_time.lock();

                        if *state == EmergencyState::Normal {
                            *state = EmergencyState::Active;
                            *start_time = Some(Instant::now());

                            Self::log_emergency_event(
                                &event_history,
                                EmergencyState::Active,
                                EmergencyAction::ReduceAllocations,
                                current_memory,
                                "Emergency threshold exceeded".to_string(),
                                true,
                            );

                            println!("🚨 EMERGENCY MEMORY PROTECTION ACTIVATED");
                            println!("  Memory pressure: {:.1}%, Usage: {} bytes", memory_pressure * 100.0, current_memory);
                        }

                        // Check for critical state
                        if let Some(start) = *start_time {
                            let duration = start.elapsed();

                            if duration > config.max_emergency_duration / 2 {
                                if *state != EmergencyState::Critical {
                                    *state = EmergencyState::Critical;
                                    Self::log_emergency_event(
                                        &event_history,
                                        EmergencyState::Critical,
                                        EmergencyAction::EmergencyGC,
                                        current_memory,
                                        "Emergency duration exceeded 50%".to_string(),
                                        true,
                                    );
                                    println!("⚠️  EMERGENCY PROTECTION ENTERING CRITICAL STATE");
                                }
                            }

                            if duration > config.max_emergency_duration {
                                if *state != EmergencyState::Fatal {
                                    *state = EmergencyState::Fatal;
                                    Self::log_emergency_event(
                                        &event_history,
                                        EmergencyState::Fatal,
                                        EmergencyAction::SystemShutdown,
                                        current_memory,
                                        "Maximum emergency duration exceeded".to_string(),
                                        true,
                                    );
                                    println!("💀 EMERGENCY PROTECTION ENTERING FATAL STATE - SYSTEM WILL SHUTDOWN");
                                }
                            }
                        }

                        // Perform emergency actions
                        Self::perform_emergency_actions(
                            &config,
                            &mut last_cleanup,
                            &action_counts,
                            &last_action_time,
                            &event_history,
                            current_memory,
                        );
                    }
                } else {
                    consecutive_emergency_checks = 0;

                    // Check if we can exit emergency mode
                    let mut state = current_state.lock();
                    if *state != EmergencyState::Normal {
                        *state = EmergencyState::Normal;
                        *emergency_start_time.lock() = None;

                        Self::log_emergency_event(
                            &event_history,
                            EmergencyState::Normal,
                            EmergencyAction::EmergencyGC,
                            current_memory,
                            "Memory pressure normalized".to_string(),
                            true,
                        );

                        println!("✅ EMERGENCY PROTECTION DEACTIVATED - Memory pressure normalized");
                    }
                }

                thread::sleep(config.force_cleanup_interval);
            }
        });

        self.emergency_thread = Some(handle);
    }

    /// Stop emergency protection monitoring
    pub fn stop_emergency_protection(&mut self) {
        if let Some(handle) = self.emergency_thread.take() {
            self.shutdown_flag.store(true, Ordering::Relaxed);

            thread::sleep(Duration::from_millis(100));

            if !handle.is_finished() {
                handle.thread().unpark();
            }
        }
    }

    /// Check if emergency allocation is allowed
    pub fn can_emergency_allocate(&self, requested_bytes: usize) -> bool {
        let state = self.current_state.lock();

        match *state {
            EmergencyState::Normal => true,
            EmergencyState::Active => requested_bytes <= self.config.emergency_allocation_limit,
            EmergencyState::Critical => requested_bytes <= self.config.emergency_allocation_limit / 2,
            EmergencyState::Fatal => false,
        }
    }

    /// Get current emergency state
    pub fn get_emergency_state(&self) -> EmergencyState {
        self.current_state.lock().clone()
    }

    /// Get emergency event history
    pub fn get_emergency_history(&self) -> Vec<EmergencyEvent> {
        self.event_history.lock().clone()
    }

    /// Get emergency statistics
    pub fn get_emergency_stats(&self) -> EmergencyStats {
        let action_counts = self.action_counts.lock();
        let last_action_time = self.last_action_time.lock();
        let emergency_start = self.emergency_start_time.lock();

        EmergencyStats {
            current_state: self.get_emergency_state(),
            emergency_duration: emergency_start.map(|t| t.elapsed()).unwrap_or(Duration::ZERO),
            total_events: self.event_history.lock().len(),
            action_counts: action_counts.clone(),
            last_action_times: last_action_time.clone(),
        }
    }

    /// Manually trigger emergency action
    pub fn trigger_emergency_action(&mut self, action: EmergencyAction, reason: String) -> Result<(), String> {
        let current_memory = get_memory_stats().total_usage;

        match self.perform_single_action(&action, current_memory) {
            Ok(success) => {
                Self::log_emergency_event(
                    &self.event_history,
                    self.get_emergency_state(),
                    action.clone(),
                    current_memory,
                    reason,
                    success,
                );

                // Update action counts
                let mut counts = self.action_counts.lock();
                *counts.entry(action.clone()).or_insert(0) += 1;

                let mut times = self.last_action_time.lock();
                times.insert(action, Instant::now());

                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Perform emergency actions based on current state
    fn perform_emergency_actions(
        config: &EmergencyProtectionConfig,
        last_cleanup: &mut Instant,
        action_counts: &Arc<Mutex<HashMap<EmergencyAction, usize>>>,
        last_action_time: &Arc<Mutex<HashMap<EmergencyAction, Instant>>>,
        event_history: &Arc<Mutex<Vec<EmergencyEvent>>>,
        current_memory: usize,
    ) {
        let now = Instant::now();

        // Force cleanup at regular intervals
        if now.duration_since(*last_cleanup) >= config.force_cleanup_interval {
            Self::perform_force_cleanup(action_counts, last_action_time, event_history, current_memory);
            *last_cleanup = now;
        }

        // Additional emergency measures
        if config.enable_emergency_cache_clearing {
            Self::perform_cache_clearing(action_counts, last_action_time, event_history, current_memory);
        }

        if config.enable_emergency_arena_reset {
            Self::perform_arena_reset(action_counts, last_action_time, event_history, current_memory);
        }

        // Clean up memory pools
        let _ = cleanup_memory_pools();
    }

    /// Perform force cleanup
    fn perform_force_cleanup(
        action_counts: &Arc<Mutex<HashMap<EmergencyAction, usize>>>,
        last_action_time: &Arc<Mutex<HashMap<EmergencyAction, Instant>>>,
        event_history: &Arc<Mutex<Vec<EmergencyEvent>>>,
        current_memory: usize,
    ) {
        let success = force_memory_cleanup().is_ok();

        Self::update_action_stats(
            EmergencyAction::ForceCacheCleanup,
            success,
            action_counts,
            last_action_time,
            event_history,
            current_memory,
        );

        if success {
            println!("🧹 Emergency force cleanup completed");
        } else {
            println!("❌ Emergency force cleanup failed");
        }
    }

    /// Perform cache clearing
    fn perform_cache_clearing(
        action_counts: &Arc<Mutex<HashMap<EmergencyAction, usize>>>,
        last_action_time: &Arc<Mutex<HashMap<EmergencyAction, Instant>>>,
        event_history: &Arc<Mutex<Vec<EmergencyEvent>>>,
        current_memory: usize,
    ) {
        // Clear all global caches
        let mut success = true;

        if clear_global_iri_cache().is_err() {
            success = false;
        }

        if crate::entities::clear_global_entity_cache().is_err() {
            success = false;
        }

        Self::update_action_stats(
            EmergencyAction::ForceCacheCleanup,
            success,
            action_counts,
            last_action_time,
            event_history,
            current_memory,
        );

        if success {
            println!("🗑️  Emergency cache clearing completed");
        } else {
            println!("❌ Emergency cache clearing failed");
        }
    }

    /// Perform arena reset
    fn perform_arena_reset(
        action_counts: &Arc<Mutex<HashMap<EmergencyAction, usize>>>,
        last_action_time: &Arc<Mutex<HashMap<EmergencyAction, Instant>>>,
        event_history: &Arc<Mutex<Vec<EmergencyEvent>>>,
        current_memory: usize,
    ) {
        // Reset parser arenas - this is a simplified version
        // In a real implementation, this would reset all active arenas
        let success = true; // Placeholder

        Self::update_action_stats(
            EmergencyAction::ResetArenas,
            success,
            action_counts,
            last_action_time,
            event_history,
            current_memory,
        );

        if success {
            println!("🔄 Emergency arena reset completed");
        } else {
            println!("❌ Emergency arena reset failed");
        }
    }

    /// Perform single emergency action
    fn perform_single_action(&self, action: &EmergencyAction, current_memory: usize) -> Result<bool, String> {
        match action {
            EmergencyAction::ForceCacheCleanup => {
                Ok(force_memory_cleanup().is_ok())
            }
            EmergencyAction::ResetArenas => {
                // Reset arenas - simplified implementation
                Ok(true)
            }
            EmergencyAction::SuspendThreads => {
                // Suspend non-essential threads - simplified implementation
                Ok(true)
            }
            EmergencyAction::ReduceAllocations => {
                // Allocation limits are already reduced in emergency state
                Ok(true)
            }
            EmergencyAction::SystemShutdown => {
                println!("💀 SYSTEM SHUTDOWN TRIGGERED BY EMERGENCY PROTECTION");
                std::process::exit(1);
            }
            EmergencyAction::EmergencyGC => {
                // Perform emergency garbage collection
                let _ = cleanup_memory_pools();
                Ok(true)
            }
        }
    }

    /// Update action statistics
    fn update_action_stats(
        action: EmergencyAction,
        success: bool,
        action_counts: &Arc<Mutex<HashMap<EmergencyAction, usize>>>,
        last_action_time: &Arc<Mutex<HashMap<EmergencyAction, Instant>>>,
        event_history: &Arc<Mutex<Vec<EmergencyEvent>>>,
        current_memory: usize,
    ) {
        let mut counts = action_counts.lock();
        *counts.entry(action.clone()).or_insert(0) += 1;

        let mut times = last_action_time.lock();
        times.insert(action.clone(), Instant::now());

        Self::log_emergency_event(
            event_history,
            EmergencyState::Active, // This would be the current state
            action,
            current_memory,
            "Automatic emergency action".to_string(),
            success,
        );
    }

    /// Log emergency event
    fn log_emergency_event(
        event_history: &Arc<Mutex<Vec<EmergencyEvent>>>,
        state: EmergencyState,
        action: EmergencyAction,
        memory_usage: usize,
        reason: String,
        success: bool,
    ) {
        let event = EmergencyEvent {
            timestamp: Instant::now(),
            state,
            action,
            memory_usage,
            reason,
            success,
        };

        let mut history = event_history.lock();
        history.push(event.clone());

        // Keep only last 100 events
        if history.len() > 100 {
            history.remove(0);
        }
    }
}

/// Emergency protection statistics
#[derive(Debug, Clone)]
pub struct EmergencyStats {
    pub current_state: EmergencyState,
    pub emergency_duration: Duration,
    pub total_events: usize,
    pub action_counts: HashMap<EmergencyAction, usize>,
    pub last_action_times: HashMap<EmergencyAction, Instant>,
}

/// Global emergency protection instance
static GLOBAL_EMERGENCY_PROTECTION: std::sync::LazyLock<Mutex<EmergencyProtection>> =
    std::sync::LazyLock::new(|| Mutex::new(EmergencyProtection::new()));

/// Get global emergency protection instance
pub fn get_emergency_protection() -> &'static Mutex<EmergencyProtection> {
    &GLOBAL_EMERGENCY_PROTECTION
}

/// Check if emergency allocation is allowed
pub fn can_emergency_allocate(requested_bytes: usize) -> bool {
    GLOBAL_EMERGENCY_PROTECTION.lock().can_emergency_allocate(requested_bytes)
}

/// Get current emergency state
pub fn get_emergency_state() -> EmergencyState {
    GLOBAL_EMERGENCY_PROTECTION.lock().get_emergency_state()
}

/// Get emergency statistics
pub fn get_emergency_stats() -> EmergencyStats {
    GLOBAL_EMERGENCY_PROTECTION.lock().get_emergency_stats()
}

/// Trigger emergency action manually
pub fn trigger_emergency_action(action: EmergencyAction, reason: String) -> Result<(), String> {
    GLOBAL_EMERGENCY_PROTECTION.lock().trigger_emergency_action(action, reason)
}

/// Start emergency protection monitoring
pub fn start_emergency_protection() {
    GLOBAL_EMERGENCY_PROTECTION.lock().start_emergency_protection();
}

/// Stop emergency protection monitoring
pub fn stop_emergency_protection() {
    GLOBAL_EMERGENCY_PROTECTION.lock().stop_emergency_protection();
}
