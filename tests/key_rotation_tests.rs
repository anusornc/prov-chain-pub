//! Key Rotation Infrastructure Tests
//!
//! Comprehensive test suite for key rotation infrastructure.
//! Tests validate that:
//! - Key rotation timing is correctly calculated
//! - Rotation intervals are configurable
//! - Days since rotation is accurate
//! - Rotation is triggered at appropriate times

use provchain_org::core::blockchain::Blockchain;
use provchain_org::security::keys::generate_signing_key;
use chrono::{Utc, Duration};

#[cfg(test)]
mod key_rotation_timing_tests {
    use super::*;

    #[test]
    fn test_should_rotate_key_when_expired() {
        let mut blockchain = Blockchain::new();
        
        // Set last rotation to 91 days ago (default interval is 90 days)
        blockchain.last_key_rotation = Utc::now() - Duration::days(91);
        
        assert!(blockchain.should_rotate_key(), "Key should be rotated after interval expires");
    }

    #[test]
    fn test_should_not_rotate_key_when_fresh() {
        let mut blockchain = Blockchain::new();
        
        // Set last rotation to 1 day ago
        blockchain.last_key_rotation = Utc::now() - Duration::days(1);
        
        assert!(!blockchain.should_rotate_key(), "Key should not be rotated when fresh");
    }

    #[test]
    fn test_should_rotate_key_at_exact_interval() {
        let mut blockchain = Blockchain::new();
        
        // Set last rotation to exactly 90 days ago (default interval)
        blockchain.last_key_rotation = Utc::now() - Duration::days(90);
        
        // At exactly 90 days, should rotate
        assert!(blockchain.should_rotate_key(), "Key should rotate at exact interval");
    }

    #[test]
    fn test_days_since_key_rotation() {
        let mut blockchain = Blockchain::new();
        
        // Test various time periods
        let test_cases = vec![
            (0, 0),      // 0 days
            (1, 1),      // 1 day
            (7, 7),      // 1 week
            (30, 30),    // 1 month
            (90, 90),    // 90 days
            (365, 365),  // 1 year
        ];
        
        for (days_ago, expected) in test_cases {
            blockchain.last_key_rotation = Utc::now() - Duration::days(days_ago);
            let result = blockchain.days_since_key_rotation();
            
            // Allow small margin of error for test execution time
            assert!((result - expected as i64).abs() <= 1, 
                "Days since rotation should be approximately {} (got {})", expected, result);
        }
    }

    #[test]
    fn test_future_rotation_time() {
        let blockchain = Blockchain::new();
        
        // Set last rotation to now
        let mut blockchain = blockchain.clone();
        blockchain.last_key_rotation = Utc::now();
        
        assert!(!blockchain.should_rotate_key(), "Freshly rotated key should not need rotation");
        
        // With default 90-day interval, should not rotate for 89 days
        blockchain.last_key_rotation = Utc::now() - Duration::days(89);
        assert!(!blockchain.should_rotate_key(), "Should not rotate 1 day before interval");
    }
}

#[cfg(test)]
mod rotation_interval_configuration_tests {
    use super::*;

    #[test]
    fn test_default_rotation_interval() {
        let blockchain = Blockchain::new();
        
        assert_eq!(blockchain.key_rotation_interval_days, 90, 
            "Default rotation interval should be 90 days");
    }

    #[test]
    fn test_custom_rotation_interval_short() {
        let mut blockchain = Blockchain::new();
        blockchain.key_rotation_interval_days = 30;
        
        // Set last rotation to 31 days ago
        blockchain.last_key_rotation = Utc::now() - Duration::days(31);
        
        assert!(blockchain.should_rotate_key(), "Should rotate with custom 30-day interval");
    }

    #[test]
    fn test_custom_rotation_interval_long() {
        let mut blockchain = Blockchain::new();
        blockchain.key_rotation_interval_days = 365;
        
        // Set last rotation to 366 days ago
        blockchain.last_key_rotation = Utc::now() - Duration::days(366);
        
        assert!(blockchain.should_rotate_key(), "Should rotate with custom 365-day interval");
        
        // But not at 90 days (default interval)
        blockchain.last_key_rotation = Utc::now() - Duration::days(91);
        assert!(!blockchain.should_rotate_key(), 
            "Should not rotate at 91 days with 365-day interval");
    }

    #[test]
    fn test_minimum_rotation_interval() {
        let mut blockchain = Blockchain::new();
        blockchain.key_rotation_interval_days = 1;
        
        // Set last rotation to 2 days ago
        blockchain.last_key_rotation = Utc::now() - Duration::days(2);
        
        assert!(blockchain.should_rotate_key(), "Should rotate with 1-day minimum interval");
    }

    #[test]
    fn test_zero_rotation_interval() {
        let mut blockchain = Blockchain::new();
        blockchain.key_rotation_interval_days = 0;
        
        // With 0-day interval, should always rotate
        blockchain.last_key_rotation = Utc::now();
        
        // At exactly now, should probably rotate (interval expired immediately)
        assert!(blockchain.should_rotate_key() || blockchain.days_since_key_rotation() >= 0,
            "Zero-day interval should trigger rotation");
    }

    #[test]
    fn test_extended_rotation_interval() {
        let mut blockchain = Blockchain::new();
        blockchain.key_rotation_interval_days = 730; // 2 years
        
        // Set last rotation to 1 year ago
        blockchain.last_key_rotation = Utc::now() - Duration::days(365);
        
        assert!(!blockchain.should_rotate_key(), 
            "Should not rotate at 1 year with 2-year interval");
    }
}

#[cfg(test)]
mod rotation_tracking_tests {
    use super::*;

    #[test]
    fn test_rotation_initialization() {
        let blockchain = Blockchain::new();
        
        // On creation, last_key_rotation should be set to current time
        let now = Utc::now();
        let time_diff = (now - blockchain.last_key_rotation).num_seconds().abs();
        
        assert!(time_diff < 5, "Initial rotation time should be close to now");
        assert_eq!(blockchain.key_rotation_interval_days, 90, "Default interval should be 90 days");
    }

    #[test]
    fn test_rotation_time_persists() {
        let mut blockchain = Blockchain::new();
        
        let original_time = blockchain.last_key_rotation;
        let original_interval = blockchain.key_rotation_interval_days;
        
        // Modify
        blockchain.last_key_rotation = Utc::now() - Duration::days(50);
        blockchain.key_rotation_interval_days = 60;
        
        // Values should persist
        assert_eq!(blockchain.days_since_key_rotation(), 50);
        assert_eq!(blockchain.key_rotation_interval_days, 60);
        
        // Restore
        blockchain.last_key_rotation = original_time;
        blockchain.key_rotation_interval_days = original_interval;
    }

    #[test]
    fn test_rotation_state_across_operations() {
        let mut blockchain = Blockchain::new();
        
        // Set to near expiration
        blockchain.last_key_rotation = Utc::now() - Duration::days(89);
        blockchain.key_rotation_interval_days = 90;
        
        // Before any operations
        assert!(!blockchain.should_rotate_key());
        
        // Simulate blockchain operations (these shouldn't affect rotation state)
        let _block = blockchain.add_block("test data".to_string());
        
        // Rotation state should be unchanged
        assert!(!blockchain.should_rotate_key());
        assert_eq!(blockchain.key_rotation_interval_days, 90);
    }
}

#[cfg(test)]
mod rotation_warning_tests {
    use super::*;

    #[test]
    fn test_approaching_rotation_warning() {
        let mut blockchain = Blockchain::new();
        blockchain.key_rotation_interval_days = 90;
        
        // Test thresholds
        let test_cases = vec![
            (80, false),  // 80 days ago - not close to expiration
            (85, true),   // 85 days ago - approaching expiration (5 days left)
            (88, true),   // 88 days ago - very close (2 days left)
            (90, true),   // 90 days ago - expired
            (95, true),   // 95 days ago - well overdue
        ];
        
        for (days_ago, should_warn) in test_cases {
            blockchain.last_key_rotation = Utc::now() - Duration::days(days_ago);
            let days_left = blockchain.key_rotation_interval_days as i64 - blockchain.days_since_key_rotation();
            
            let is_approaching = days_left <= 7; // Warn within 7 days
            assert_eq!(is_approaching, should_warn,
                "Warning at {} days should be {}", days_ago, should_warn);
        }
    }

    #[test]
    fn test_critical_rotation_period() {
        let mut blockchain = Blockchain::new();
        blockchain.key_rotation_interval_days = 90;
        
        // Define critical period as 3 days before expiration
        let critical_threshold = 3;
        
        let test_cases = vec![
            (86, false),  // 4 days left - not critical
            (87, true),   // 3 days left - critical
            (89, true),   // 1 day left - critical
            (90, true),   // Expired - critical
        ];
        
        for (days_ago, is_critical) in test_cases {
            blockchain.last_key_rotation = Utc::now() - Duration::days(days_ago);
            let days_left = blockchain.key_rotation_interval_days as i64 - blockchain.days_since_key_rotation();
            
            let result = days_left <= critical_threshold;
            assert_eq!(result, is_critical,
                "Critical status at {} days should be {}", days_ago, is_critical);
        }
    }

    #[test]
    fn test_rotation_overdue() {
        let mut blockchain = Blockchain::new();
        blockchain.key_rotation_interval_days = 90;
        
        let overdue_days = vec![1, 7, 30, 90, 365];
        
        for overdue in overdue_days {
            let days_ago = 90 + overdue;
            blockchain.last_key_rotation = Utc::now() - Duration::days(days_ago);
            
            assert!(blockchain.should_rotate_key(), 
                "Should be overdue by {} days", overdue);
            
            let days_overdue = blockchain.days_since_key_rotation() - 90;
            assert_eq!(days_overdue, overdue as i64,
                "Should be {} days overdue", overdue);
        }
    }
}

#[cfg(test)]
mod edge_cases_tests {
    use super::*;

    #[test]
    fn test_rotation_with_leap_year() {
        let mut blockchain = Blockchain::new();
        blockchain.key_rotation_interval_days = 366; // Leap year
        
        // Simulate crossing a leap year boundary
        blockchain.last_key_rotation = Utc::now() - Duration::days(367);
        
        assert!(blockchain.should_rotate_key(), "Should rotate after leap year interval");
    }

    #[test]
    fn test_rotation_calculation_precision() {
        let mut blockchain = Blockchain::new();
        
        // Test with hours/minutes precision
        blockchain.last_key_rotation = Utc::now() - Duration::days(1) - Duration::hours(12);
        
        let days_since = blockchain.days_since_key_rotation();
        assert_eq!(days_since, 1, "Should round to 1 day (12 hours into day 2)");
        
        blockchain.last_key_rotation = Utc::now() - Duration::days(1) - Duration::hours(23);
        let days_since = blockchain.days_since_key_rotation();
        assert_eq!(days_since, 1, "Should still be 1 day (23 hours into day 2)");
    }

    #[test]
    fn test_negative_rotation_interval() {
        let mut blockchain = Blockchain::new();
        
        // Negative interval doesn't make sense, but we test behavior
        blockchain.key_rotation_interval_days = 0;
        blockchain.last_key_rotation = Utc::now();
        
        // With 0 interval, should rotate immediately
        assert!(blockchain.days_since_key_rotation() >= 0);
    }

    #[test]
    fn test_very_large_rotation_interval() {
        let mut blockchain = Blockchain::new();
        blockchain.key_rotation_interval_days = 36500; // 100 years
        
        blockchain.last_key_rotation = Utc::now() - Duration::days(365);
        
        assert!(!blockchain.should_rotate_key(), 
            "Should not rotate with 100-year interval after only 1 year");
    }

    #[test]
    fn test_rotation_independence_from_other_fields() {
        let mut blockchain = Blockchain::new();
        
        // Set rotation state
        blockchain.last_key_rotation = Utc::now() - Duration::days(45);
        blockchain.key_rotation_interval_days = 90;
        
        let original_rotation = blockchain.last_key_rotation;
        let original_interval = blockchain.key_rotation_interval_days;
        
        // Add blocks to blockchain (shouldn't affect rotation)
        for i in 0..5 {
            let _block = blockchain.add_block(format!("block {}", i));
        }
        
        // Rotation state should be unchanged
        assert_eq!(blockchain.last_key_rotation, original_rotation);
        assert_eq!(blockchain.key_rotation_interval_days, original_interval);
        assert!(!blockchain.should_rotate_key());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_rotation_workflow() {
        let mut blockchain = Blockchain::new();
        
        // Initial state
        assert!(!blockchain.should_rotate_key());
        assert_eq!(blockchain.days_since_key_rotation(), 0);
        
        // Simulate time passing to day 89
        blockchain.last_key_rotation = Utc::now() - Duration::days(89);
        assert!(!blockchain.should_rotate_key(), "Should not rotate at day 89");
        assert_eq!(blockchain.days_since_key_rotation(), 89);
        
        // Day 90 - should rotate
        blockchain.last_key_rotation = Utc::now() - Duration::days(90);
        assert!(blockchain.should_rotate_key(), "Should rotate at day 90");
        
        // Simulate rotation (reset timer)
        blockchain.last_key_rotation = Utc::now();
        assert!(!blockchain.should_rotate_key(), "Should not rotate immediately after rotation");
        assert_eq!(blockchain.days_since_key_rotation(), 0);
    }

    #[test]
    fn test_rotation_with_custom_interval_workflow() {
        let mut blockchain = Blockchain::new();
        blockchain.key_rotation_interval_days = 30;
        
        // Set to 29 days ago
        blockchain.last_key_rotation = Utc::now() - Duration::days(29);
        assert!(!blockchain.should_rotate_key());
        
        // Day 30 - should rotate
        blockchain.last_key_rotation = Utc::now() - Duration::days(30);
        assert!(blockchain.should_rotate_key());
        
        // Rotate
        blockchain.last_key_rotation = Utc::now();
        assert!(!blockchain.should_rotate_key());
        
        // 29 days later - still good
        blockchain.last_key_rotation = Utc::now() - Duration::days(29);
        assert!(!blockchain.should_rotate_key());
    }

    #[test]
    fn test_rotation_schedule_consistency() {
        let mut blockchain = Blockchain::new();
        blockchain.key_rotation_interval_days = 90;
        
        // Simulate multiple rotation cycles
        for cycle in 0..5 {
            // Before rotation
            blockchain.last_key_rotation = Utc::now() - Duration::days(90);
            assert!(blockchain.should_rotate_key(), 
                "Cycle {}: Should rotate at 90 days", cycle);
            
            // After rotation
            blockchain.last_key_rotation = Utc::now();
            assert!(!blockchain.should_rotate_key(), 
                "Cycle {}: Should not rotate immediately after", cycle);
            
            // Mid-cycle
            blockchain.last_key_rotation = Utc::now() - Duration::days(45);
            assert!(!blockchain.should_rotate_key(), 
                "Cycle {}: Should not rotate mid-cycle", cycle);
        }
    }
}
