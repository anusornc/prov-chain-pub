//! Input sanitization for security
//!
//! This module provides input sanitization to clean and normalize
//! user inputs before processing.

use std::collections::HashMap;

/// Sanitization configuration
#[derive(Debug, Clone)]
pub struct SanitizationConfig {
    /// Remove HTML tags
    pub strip_html: bool,
    /// Normalize whitespace
    pub normalize_whitespace: bool,
    /// Convert to lowercase
    pub to_lowercase: bool,
    /// Remove control characters
    pub remove_control_chars: bool,
    /// Maximum length after sanitization
    pub max_length: Option<usize>,
    /// Custom character replacements
    pub char_replacements: HashMap<char, String>,
}

impl Default for SanitizationConfig {
    fn default() -> Self {
        Self {
            strip_html: true,
            normalize_whitespace: true,
            to_lowercase: false,
            remove_control_chars: true,
            max_length: None,
            char_replacements: HashMap::new(),
        }
    }
}

impl SanitizationConfig {
    /// Create a strict sanitization config for security-sensitive inputs
    pub fn strict() -> Self {
        let mut config = Self::default();
        config.strip_html = true;
        config.normalize_whitespace = true;
        config.remove_control_chars = true;
        config.max_length = Some(1000);
        config
    }

    /// Create a lenient config for content that may contain formatting
    pub fn lenient() -> Self {
        let mut config = Self::default();
        config.strip_html = false;
        config.normalize_whitespace = false;
        config.remove_control_chars = false;
        config
    }

    /// Create config for usernames
    pub fn username() -> Self {
        let mut config = Self::default();
        config.strip_html = true;
        config.normalize_whitespace = true;
        config.to_lowercase = true;
        config.remove_control_chars = true;
        config.max_length = Some(32);
        config
    }

    /// Create config for batch IDs
    pub fn batch_id() -> Self {
        let mut config = Self::default();
        config.strip_html = true;
        config.normalize_whitespace = true;
        config.remove_control_chars = true;
        config.max_length = Some(20);
        
        // Convert common characters to uppercase equivalents
        config.char_replacements.insert('a', "A".to_string());
        config.char_replacements.insert('b', "B".to_string());
        config.char_replacements.insert('c', "C".to_string());
        config.char_replacements.insert('d', "D".to_string());
        config.char_replacements.insert('e', "E".to_string());
        config.char_replacements.insert('f', "F".to_string());
        config.char_replacements.insert('g', "G".to_string());
        config.char_replacements.insert('h', "H".to_string());
        config.char_replacements.insert('i', "I".to_string());
        config.char_replacements.insert('j', "J".to_string());
        config.char_replacements.insert('k', "K".to_string());
        config.char_replacements.insert('l', "L".to_string());
        config.char_replacements.insert('m', "M".to_string());
        config.char_replacements.insert('n', "N".to_string());
        config.char_replacements.insert('o', "O".to_string());
        config.char_replacements.insert('p', "P".to_string());
        config.char_replacements.insert('q', "Q".to_string());
        config.char_replacements.insert('r', "R".to_string());
        config.char_replacements.insert('s', "S".to_string());
        config.char_replacements.insert('t', "T".to_string());
        config.char_replacements.insert('u', "U".to_string());
        config.char_replacements.insert('v', "V".to_string());
        config.char_replacements.insert('w', "W".to_string());
        config.char_replacements.insert('x', "X".to_string());
        config.char_replacements.insert('y', "Y".to_string());
        config.char_replacements.insert('z', "Z".to_string());
        
        config
    }
}

/// Input sanitizer
pub struct InputSanitizer {
    config: SanitizationConfig,
}

impl InputSanitizer {
    pub fn new(config: SanitizationConfig) -> Self {
        Self { config }
    }

    /// Sanitize a string input
    pub fn sanitize(&self, input: &str) -> String {
        let mut result = input.to_string();

        // Strip HTML tags if configured
        if self.config.strip_html {
            result = self.strip_html_tags(&result);
        }

        // Remove control characters if configured
        if self.config.remove_control_chars {
            result = self.remove_control_characters(&result);
        }

        // Apply character replacements
        for (from_char, to_string) in &self.config.char_replacements {
            result = result.replace(*from_char, to_string);
        }

        // Convert to lowercase if configured
        if self.config.to_lowercase {
            result = result.to_lowercase();
        }

        // Normalize whitespace if configured
        if self.config.normalize_whitespace {
            result = self.normalize_whitespace(&result);
        }

        // Truncate to max length if configured
        if let Some(max_len) = self.config.max_length {
            if result.len() > max_len {
                result.truncate(max_len);
            }
        }

        result
    }

    /// Strip HTML tags from input
    fn strip_html_tags(&self, input: &str) -> String {
        // Simple HTML tag removal - in production, use a proper HTML sanitizer library
        let mut result = String::new();
        let mut in_tag = false;
        
        for ch in input.chars() {
            match ch {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ => {
                    if !in_tag {
                        result.push(ch);
                    }
                }
            }
        }
        
        result
    }

    /// Remove control characters
    fn remove_control_characters(&self, input: &str) -> String {
        input.chars()
            .filter(|&ch| !ch.is_control() || ch == '\n' || ch == '\r' || ch == '\t')
            .collect()
    }

    /// Normalize whitespace
    fn normalize_whitespace(&self, input: &str) -> String {
        // Replace multiple whitespace with single space and trim
        input.split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
            .trim()
            .to_string()
    }

    /// Sanitize multiple fields
    pub fn sanitize_fields(&self, fields: &HashMap<String, String>) -> HashMap<String, String> {
        fields.iter()
            .map(|(key, value)| (key.clone(), self.sanitize(value)))
            .collect()
    }
}

impl Default for InputSanitizer {
    fn default() -> Self {
        Self::new(SanitizationConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_stripping() {
        let sanitizer = InputSanitizer::new(SanitizationConfig::strict());
        let input = "<script>alert('xss')</script>Hello World<br>";
        let result = sanitizer.sanitize(input);
        
        assert!(!result.contains("<script>"));
        assert!(!result.contains("</script>"));
        assert!(result.contains("Hello World"));
    }

    #[test]
    fn test_whitespace_normalization() {
        let sanitizer = InputSanitizer::new(SanitizationConfig::default());
        let input = "  Hello    World  \n\t  ";
        let result = sanitizer.sanitize(input);
        
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_control_character_removal() {
        let sanitizer = InputSanitizer::new(SanitizationConfig::strict());
        let input = "Hello\x00World\x01Test";
        let result = sanitizer.sanitize(input);
        
        assert!(!result.contains('\x00'));
        assert!(!result.contains('\x01'));
        assert!(result.contains("HelloWorldTest"));
    }

    #[test]
    fn test_length_truncation() {
        let mut config = SanitizationConfig::default();
        config.max_length = Some(5);
        let sanitizer = InputSanitizer::new(config);
        
        let input = "Hello World";
        let result = sanitizer.sanitize(input);
        
        assert_eq!(result.len(), 5);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_batch_id_sanitization() {
        let sanitizer = InputSanitizer::new(SanitizationConfig::batch_id());
        let input = "batch123";
        let result = sanitizer.sanitize(input);
        
        assert_eq!(result, "BATCH123");
    }

    #[test]
    fn test_username_sanitization() {
        let sanitizer = InputSanitizer::new(SanitizationConfig::username());
        let input = "  User_Name123  ";
        let result = sanitizer.sanitize(input);
        
        assert_eq!(result, "user_name123");
    }
}
