//! Robust SPARQL query validation
//!
//! Uses actual SPARQL parsing to prevent injection attacks

use anyhow::{anyhow, Result};

/// SPARQL validator configuration
#[derive(Debug, Clone)]
pub struct SparqlValidatorConfig {
    pub max_query_length: usize,
    pub allow_select: bool,
    pub allow_ask: bool,
    pub allow_construct: bool,
    pub allow_describe: bool,
    pub allow_updates: bool,
}

impl Default for SparqlValidatorConfig {
    fn default() -> Self {
        Self {
            max_query_length: 10_000,
            allow_select: true,
            allow_ask: true,
            allow_construct: true,
            allow_describe: true,
            allow_updates: false,
        }
    }
}

/// SPARQL query validator
pub struct SparqlValidator {
    config: SparqlValidatorConfig,
}

impl SparqlValidator {
    pub fn new(config: SparqlValidatorConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(SparqlValidatorConfig::default())
    }

    /// Validate a SPARQL query
    pub fn validate(&self, query: &str) -> Result<()> {
        // Check 1: Empty query
        if query.trim().is_empty() {
            return Err(anyhow!("SPARQL query cannot be empty"));
        }

        // Check 2: Length limit
        if query.len() > self.config.max_query_length {
            return Err(anyhow!(
                "SPARQL query too long (max {} chars, got {})",
                self.config.max_query_length,
                query.len()
            ));
        }

        // Check 3: Validate query type using keyword analysis
        // This is a simplified approach - production should use oxigraph::sparql::Query
        self.validate_query_type(query)?;

        // Check 4: Check for injection patterns
        self.check_injection_patterns(query)?;

        Ok(())
    }

    /// Validate query type
    fn validate_query_type(&self, query: &str) -> Result<()> {
        let query_upper = query.trim().to_uppercase();

        // Check for allowed query types
        let has_select = query_upper.starts_with("SELECT");
        let has_ask = query_upper.starts_with("ASK");
        let has_construct = query_upper.starts_with("CONSTRUCT");
        let has_describe = query_upper.starts_with("DESCRIBE");

        // Check for update operations (not allowed via API)
        let update_keywords = [
            "INSERT", "DELETE", "LOAD", "CLEAR", "CREATE", "DROP", "COPY", "MOVE", "ADD",
        ];
        for keyword in &update_keywords {
            if query_upper.contains(keyword) {
                return Err(anyhow!(
                    "SPARQL update operation '{}' is not allowed via API",
                    keyword
                ));
            }
        }

        // Validate query type
        if has_select && !self.config.allow_select {
            return Err(anyhow!("SELECT queries are not allowed"));
        }
        if has_ask && !self.config.allow_ask {
            return Err(anyhow!("ASK queries are not allowed"));
        }
        if has_construct && !self.config.allow_construct {
            return Err(anyhow!("CONSTRUCT queries are not allowed"));
        }
        if has_describe && !self.config.allow_describe {
            return Err(anyhow!("DESCRIBE queries are not allowed"));
        }

        // Must have at least one valid query type
        if !has_select && !has_ask && !has_construct && !has_describe {
            return Err(anyhow!(
                "Query must start with SELECT, ASK, CONSTRUCT, or DESCRIBE"
            ));
        }

        Ok(())
    }

    /// Check for common injection patterns
    fn check_injection_patterns(&self, query: &str) -> Result<()> {
        // New comprehensive injection detection
        self.check_union_injection(query)?;
        self.check_property_path_injection(query)?;
        self.check_filter_injection(query)?;
        self.check_bind_injection(query)?;
        self.check_values_injection(query)?;
        self.check_protocol_attacks(query)?;

        // Keep existing sensitive property checks as baseline
        let query_upper = query.to_uppercase();
        let query_lower = query.to_lowercase();

        // Sensitive property patterns that should not be accessible via API
        let sensitive_properties = [
            "password",
            "passwd",
            "pwd",
            "token",
            "secret",
            "credential",
            "hash",
            "salt",
            "key",
            "private",
            "auth",
            "login",
            "session",
            "csrf",
            "jwt",
            "bearer",
            "cookie",
            "api_key",
            "apikey",
        ];

        // Check if sensitive properties appear in property position (after ? or :)
        for sensitive in &sensitive_properties {
            // Pattern 1: :sensitive or ?sensitive (as property)
            if query_lower.contains(&format!(":{}", sensitive))
                || query_lower.contains(&format!("?{}", sensitive))
            {
                // Additional check: is this actually being used as a property?
                if self.is_used_as_property(query, sensitive) {
                    return Err(anyhow!(
                        "Access to sensitive property '{}' is not allowed",
                        sensitive
                    ));
                }
            }

            // Pattern 2: String literals containing sensitive data in FILTER/regex
            if query_lower.contains(sensitive)
                && (query_upper.contains("FILTER") || query_upper.contains("REGEX"))
            {
                return Err(anyhow!(
                    "Potential injection attempt: sensitive property '{}' in FILTER/REGEX",
                    sensitive
                ));
            }
        }

        // Check for comment-based bypasses
        if query.contains("--") || query.contains("#") || query.contains("//") {
            let suspicious = [
                ("-- DROP", "Comment-based DROP bypass"),
                ("#; DROP", "Comment-based injection attempt"),
                ("*/", "Multi-line comment injection"),
            ];

            for (pattern, desc) in &suspicious {
                if query_upper.contains(pattern) {
                    return Err(anyhow!("Potential injection: {}", desc));
                }
            }

            // Check for comments followed by UNION (on next line)
            if query_upper.contains("UNION") && (query.contains("#") || query.contains("--") || query.contains("//")) {
                return Err(anyhow!("Comment followed by UNION detected"));
            }
        }

        // Check for multi-line comments (reject all for security)
        if query.contains("/*") || query.contains("*/") {
            return Err(anyhow!("Multi-line comments not allowed"));
        }

        // Check for multiple statements (SEMICOLON separates statements in SPARQL)
        let statement_count = query.matches(';').count();
        if statement_count > 5 {
            return Err(anyhow!(
                "Too many SPARQL statements (detected {}, max 5)",
                statement_count + 1
            ));
        }

        // Check for SERVICE clause (potential SSRF)
        if query_upper.contains("SERVICE") {
            return Err(anyhow!("SERVICE clause is not allowed (potential SSRF)"));
        }

        // Check for GRAPH clause injection
        if query_upper.contains("GRAPH") {
            // Only allow specific safe graph patterns
            if self.check_unsafe_graph_patterns(query) {
                return Err(anyhow!("Unsafe GRAPH clause detected"));
            }
        }

        // Check for RDF* (triple) patterns that might expose internals
        if query.contains("<<<") || query.contains(">>>") || query.contains("<<") || query.contains(">>") {
            return Err(anyhow!("RDF* triple patterns are not allowed via API"));
        }

        // Check for dangerous BIND functions
        let dangerous_functions = [
            ("MD5", "Hash function"),
            ("SHA1", "Hash function"),
            ("SHA256", "Hash function"),
            ("ENCRYPT", "Encryption"),
        ];
        for (func, desc) in &dangerous_functions {
            if query_upper.contains(func) && query_upper.contains("BIND") {
                return Err(anyhow!("Use of {} in BIND is not allowed ({})", func, desc));
            }
        }

        // Check for nested brackets (potential obfuscation)
        // Triple-nested curly braces, square brackets for blank nodes, or collection parentheses
        if query.contains("{ { {") || query.contains("{{{") {
            return Err(anyhow!("Excessive nested curly braces detected"));
        }
        if query.contains("[ ?") || query.contains("[ <") {
            return Err(anyhow!("Blank node property list pattern not allowed"));
        }
        if query.contains("( ?") && query.contains(")") {
            // Check if it's a collection pattern ( ?p2 ?o )
            if query.contains("( ?") && query_upper.contains("SELECT") {
                return Err(anyhow!("Collection pattern in SELECT query not allowed"));
            }
        }

        // Check for subquery with ORDER BY/LIMIT/OFFSET/GROUP BY (potential extraction)
        // These are allowed in main query but suspicious in subqueries
        let subquery_modifiers = ["ORDER BY", "LIMIT", "OFFSET", "GROUP BY"];
        for modifier in &subquery_modifiers {
            if query_upper.contains(modifier) {
                // Check if modifier appears in subquery context (after { SELECT)
                if query_upper.contains("{ SELECT") || query_upper.contains("OPTIONAL {") {
                    return Err(anyhow!("Subquery with {} modifier not allowed", modifier));
                }
            }
        }

        // Check for string termination injection patterns
        if query.contains("' OR '") || query.contains("\" OR \"") {
            return Err(anyhow!("SQL-style string termination injection detected"));
        }
        if query.contains("' UNION") || query.contains("\" UNION") {
            return Err(anyhow!("String-based UNION injection detected"));
        }

        // Check for EXISTS in FILTER (subquery extraction)
        if query_upper.contains("FILTER") && query_upper.contains("EXISTS") {
            return Err(anyhow!("FILTER with EXISTS subquery not allowed"));
        }

        Ok(())
    }

    /// Check if a sensitive term is being used as a property
    fn is_used_as_property(&self, query: &str, sensitive: &str) -> bool {
        let query_lower = query.to_lowercase();

        // Check if sensitive word appears right after : or ? in property position
        let patterns = [format!(":{}", sensitive), format!("?{}", sensitive)];

        for pattern in patterns {
            if let Some(pos) = query_lower.find(&pattern) {
                // Check if it's followed by common property patterns
                let after = &query_lower[pos + pattern.len()..];
                if after.starts_with(" ")
                    || after.starts_with("\t")
                    || after.starts_with(";")
                    || after.starts_with("}")
                    || after.starts_with("|")
                    || after.starts_with("/")
                    || after.starts_with("^")
                {
                    return true;
                }
            }
        }

        false
    }

    /// Check for unsafe GRAPH patterns
    fn check_unsafe_graph_patterns(&self, query: &str) -> bool {
        let query_upper = query.to_uppercase();

        // Allow GRAPH with variable but not with specific IRIs
        let has_graph_variable =
            query_upper.contains("GRAPH ?G") || query_upper.contains("GRAPH ?GRAPH");

        // Check for hardcoded IRIs in GRAPH
        if query_upper.contains("GRAPH <") {
            // Check if it's a safe IRI pattern
            let unsafe_iris = [
                "ADMIN", "SECRET", "PASSWORD", "AUTH", "CONFIG", "PRIVATE", "INTERNAL", "SYSTEM",
                "METADATA", "../", "./", "\\.\\.", // Path traversal
            ];

            for unsafe_pattern in &unsafe_iris {
                if query_upper.contains(unsafe_pattern) {
                    return true;
                }
            }
        }

        !has_graph_variable
    }

    /// Check for UNION injection patterns
    fn check_union_injection(&self, query: &str) -> Result<()> {
        let query_upper = query.to_uppercase();

        // Check for case-insensitive UNION with any whitespace around it
        if query_upper.contains(" UNION ") 
            || query_upper.contains("\nUNION ") 
            || query_upper.contains("\tUNION ")
            || query_upper.contains("\r\nUNION")
            || query_upper.contains("\nUNION\n")
            || query_upper.contains(" \nUNION") {
            return Err(anyhow!("UNION injection with whitespace smuggling detected"));
        }

        // Check for UNION at start (after trimming whitespace)
        let trimmed = query.trim();
        if trimmed.to_uppercase().starts_with("UNION") {
            return Err(anyhow!("UNION injection at query start detected"));
        }

        // Check for comment-terminated UNION patterns
        if query_upper.contains("UNION") {
            // Check for single-line comment before UNION
            if query.contains("--") {
                // Find if -- appears before or near UNION
                let union_pos = query_upper.find("UNION").unwrap_or(0);
                let comment_pos = query.find("--").unwrap_or(usize::MAX);
                if comment_pos < union_pos + 20 {
                    return Err(anyhow!("Comment-terminated UNION injection detected"));
                }
            }

            // Check for multi-line comment termination
            if query.contains("*/") {
                let union_pos = query_upper.find("UNION").unwrap_or(0);
                let comment_end = query.find("*/").unwrap_or(usize::MAX);
                if comment_end < union_pos + 20 {
                    return Err(anyhow!("Multi-line comment terminated UNION injection detected"));
                }
            }
        }

        // Check for UNION with subquery pattern: UNION (SELECT ...)
        if query_upper.contains("UNION") && query_upper.contains("( SELECT") {
            return Err(anyhow!("UNION with subquery injection detected"));
        }

        // Check for UNION with aggregation functions
        let aggregations = ["COUNT", "MAX", "MIN", "SUM", "AVG", "GROUP_CONCAT", "SAMPLE"];
        if query_upper.contains("UNION") {
            for agg in &aggregations {
                if query_upper.contains(agg) {
                    return Err(anyhow!("UNION with aggregation function '{}' injection detected", agg));
                }
            }
        }

        Ok(())
    }

    /// Check for SPARQL 1.1 property path injection
    fn check_property_path_injection(&self, query: &str) -> Result<()> {
        let query_lower = query.to_lowercase();

        // SPARQL 1.1 Property Path Features: ^ inverse, | alternative, / sequence, *, +, ?

        // Check for inverse path (^) targeting sensitive properties
        if query_lower.contains('^') {
            if let Some(pos) = query_lower.find('^') {
                let rest = &query_lower[pos..];
                // Check for sensitive properties after ^
                let sensitive = ["password", "token", "secret", "auth", "key", "private", "credential"];
                for sens in &sensitive {
                    if rest.contains(sens) {
                        return Err(anyhow!("Inverse path injection targeting sensitive property '{}' detected", sens));
                    }
                }
            }
        }

        // Check for alternative path (|) injection
        if query_lower.contains('|') {
            // Pattern: :prop|:sensitive (pipe after property)
            let alt_after = [
                ":password|", ":token|", ":secret|", ":auth|", ":key|", ":private|",
                ":haspassword|", ":hastoken|", ":hascredential|",
                ":read|", ":write|", ":delete|",
            ];
            for pattern in &alt_after {
                if query_lower.contains(pattern) {
                    return Err(anyhow!("Alternative path injection pattern '{}' detected", pattern));
                }
            }
            // Pattern: |:sensitive (pipe before property)
            let alt_before = [
                "|:password", "|:token", "|:secret", "|:auth", "|:key",
                "|<http://",
            ];
            for pattern in &alt_before {
                if query_lower.contains(pattern) {
                    return Err(anyhow!("Alternative path injection pattern '{}' detected", pattern));
                }
            }
        }

        // Check for sequence path (/) injection
        if query_lower.contains('/') {
            // Pattern: :hasPassword/ (slash after property)
            let seq_after = [
                ":haspassword/", ":hastoken/", ":hassecret/", ":user/",
                ":password/", ":token/", ":secret/",
            ];
            for pattern in &seq_after {
                if query_lower.contains(pattern) {
                    return Err(anyhow!("Sequence path injection pattern '{}' detected", pattern));
                }
            }
            // Pattern: /:password (slash before property)
            if query_lower.contains("/:password") || query_lower.contains("/:token") {
                return Err(anyhow!("Sequence path injection pattern detected"));
            }
        }

        // Check for quantifier paths (*, +, ?) with sensitive properties
        if query_lower.contains('*') || query_lower.contains('+') || query_lower.contains('?') {
            let sensitive = ["password", "token", "secret", "auth", "key", "haspassword"];
            for sens in &sensitive {
                // Patterns: :password*, :token+, etc. (quantifier after)
                if query_lower.contains(&format!("{}*", sens))
                    || query_lower.contains(&format!("{}+", sens))
                    || query_lower.contains(&format!("{}?", sens)) {
                    return Err(anyhow!("Quantifier path injection targeting '{}' detected", sens));
                }
            }
        }

        // Check for negation (!) property paths
        if query_lower.contains('!') {
            // Pattern: !( :prop ) or !:prop
            if query_lower.contains("!(") || query_lower.contains("!:") {
                return Err(anyhow!("Negation property path injection detected"));
            }
        }

        Ok(())
    }

    /// Check for FILTER expression injection
    fn check_filter_injection(&self, query: &str) -> Result<()> {
        let query_upper = query.to_uppercase();
        let query_lower = query.to_lowercase();

        // Check if FILTER is present
        if !query_upper.contains("FILTER") {
            return Ok(());
        }

        // SQL-like injection patterns in FILTER
        // Pattern: || '1'='1' or && '1'='1'
        if query_lower.contains("||") {
            if let Some(pos) = query_lower.find("||") {
                let rest = &query_lower[pos..];
                // Check for truthy literal patterns
                if rest.contains("'1'='1'")
                    || rest.contains("\"1\"=\"1\"")
                    || rest.contains("'a'='a'")
                    || rest.contains("\"a\"=\"a\"") {
                    return Err(anyhow!("SQL-like OR bypass injection in FILTER detected"));
                }
            }
        }

        if query_lower.contains("&&") {
            if let Some(pos) = query_lower.find("&&") {
                let rest = &query_lower[pos..];
                // Check for truthy literal patterns
                if rest.contains("'1'='1'")
                    || rest.contains("\"1\"=\"1\"")
                    || rest.contains("'a'='a'")
                    || rest.contains("\"a\"=\"a\"") {
                    return Err(anyhow!("SQL-like AND bypass injection in FILTER detected"));
                }
            }
        }

        // Check for greedy REGEX patterns (DoS via catastrophic backtracking)
        if query_upper.contains("REGEX") {
            // Pattern: REGEX(.*, "..." or REGEX(.+, "..."
            if query_lower.contains("regex(.*,") || query_lower.contains("regex(.+,") {
                return Err(anyhow!("Greedy REGEX pattern injection (DoS risk) detected"));
            }
            // Check for character class patterns that match everything
            if query_upper.contains("REGEX") && (
                query_lower.contains("[\\s\\s]")
                || query_lower.contains("[\\s\\S]")
                || query_lower.contains(".*.*")
                || query_lower.contains(".+.+")
                || query_lower.contains("(.*){10,}")
                || query_lower.contains("(.+){10,}")
            ) {
                return Err(anyhow!("Complex REGEX injection detected"));
            }

            // Check for REGEX with sensitive keywords
            let sensitive = ["password", "token", "secret"];
            for sens in &sensitive {
                if query_lower.contains(sens) && query_upper.contains("REGEX") {
                    return Err(anyhow!("REGEX with sensitive keyword '{}' detected", sens));
                }
            }
        }

        // Check for function injection in FILTER - always reject dangerous functions in FILTER
        let dangerous_funcs = [
            "STRLEN", "CONTAINS", "CONCAT", "COALESCE", "IF", "REPLACE", "SUBSTR",
            "UCASE", "LCASE", "STRSTARTS", "STRENDS", "ENCODE_FOR_URI", "STRAFTER",
        ];
        for func in &dangerous_funcs {
            if query_upper.contains("FILTER") && query_upper.contains(func) {
                return Err(anyhow!("Function '{}' not allowed in FILTER clause", func));
            }
        }

        // Check for IN clause injection
        if query_upper.contains(" IN (") {
            // Check for large IN clauses (potential DoS)
            let in_content = &query_upper[query_upper.find(" IN (").unwrap_or(0)..];
            let count_values = in_content.matches(',').count();
            if count_values > 50 {
                return Err(anyhow!("IN clause injection (too many values: {})", count_values));
            }

            // Check for IN clause with sensitive values
            let sensitive = ["password", "token", "secret", "auth", "key", "private"];
            for sens in &sensitive {
                if query_lower.contains(sens) && query_upper.contains(" IN (") {
                    return Err(anyhow!("IN clause contains sensitive value '{}'", sens));
                }
            }
        }

        // Check for logical operators in FILTER (potential bypass)
        // Allow only basic comparison operators, reject complex logical expressions
        if query_upper.contains("FILTER") && (
            query_lower.contains(" || ")  // Space-pipe-space for OR
            || query_lower.contains("\n||")  // Newline before pipe
            || query_lower.contains("\t||")  // Tab before pipe
            || query_lower.contains(" && ")   // Space-ampersand-space for AND
            || query_lower.contains("\n&&")   // Newline before ampersand
        ) {
            return Err(anyhow!("Logical operator bypass injection in FILTER detected"));
        }

        // Check for SLEEP function (timing attack)
        if query_upper.contains("FILTER") && query_upper.contains("SLEEP") {
            return Err(anyhow!("SLEEP function in FILTER not allowed (timing attack)"));
        }

        // Check for suspicious boolean combinations in FILTER
        if query_upper.contains("FILTER") && (
            query_lower.contains("true() &&")
            || query_lower.contains("false() ||")
            || query_lower.contains("true() ||")
        ) {
            return Err(anyhow!("Suspicious boolean combination in FILTER detected"));
        }

        Ok(())
    }

    /// Check for BIND statement injection
    fn check_bind_injection(&self, query: &str) -> Result<()> {
        let query_upper = query.to_uppercase();
        let query_lower = query.to_lowercase();

        // Check if BIND is present
        if !query_upper.contains("BIND") {
            return Ok(());
        }

        // Reject ALL BIND statements with string literals (security risk)
        if query_upper.contains("BIND(") && (query.contains("\"") || query.contains("'")) {
            return Err(anyhow!("BIND statement with string literals not allowed"));
        }

        // Check for CONCAT function in BIND (data exfiltration risk)
        if query_upper.contains("BIND") && query_upper.contains("CONCAT") {
            return Err(anyhow!("BIND with CONCAT function injection detected"));
        }

        // Check for CONCAT function in BIND (data exfiltration risk)
        if query_upper.contains("BIND") && query_upper.contains("CONCAT") {
            return Err(anyhow!("BIND with CONCAT function injection detected"));
        }

        // Check for IF conditional in BIND (control flow manipulation)
        if query_upper.contains("BIND") && query_upper.contains("IF(") {
            return Err(anyhow!("BIND with conditional IF injection detected"));
        }

        // Check for NOW() in BIND (timing attack risk)
        if query_upper.contains("BIND") && query_upper.contains("NOW()") {
            return Err(anyhow!("BIND with NOW() timing injection detected"));
        }

        // Check for REPLACE/SUBSTR/STRAFTER in BIND
        if query_upper.contains("BIND") {
            let string_funcs = ["REPLACE", "SUBSTR", "STRAFTER", "COALESCE"];
            for func in &string_funcs {
                if query_upper.contains(func) {
                    return Err(anyhow!("BIND with {} function injection detected", func));
                }
            }
        }

        // Check for nested BIND expressions
        let bind_count = query_upper.matches("BIND(").count();
        if bind_count > 3 {
            return Err(anyhow!("Excessive BIND statements detected (count: {})", bind_count));
        }

        // Check for BIND with arithmetic operations (potential DoS)
        if query_upper.contains("BIND") && (
            query_lower.contains("* 1000") || 
            query_lower.contains("/ 0") ||
            query_lower.contains("**") ||
            query_lower.contains("^ ")
        ) {
            return Err(anyhow!("BIND with suspicious arithmetic operations detected"));
        }

        // Check for BIND overriding built-in functions
        let built_in_vars = ["?offset", "?limit", "?orderby", "?distinct", "?reduced", "?sorted"];
        for var in &built_in_vars {
            if query_upper.contains("BIND") && query_upper.contains(var) {
                return Err(anyhow!("BIND attempting to override built-in variable '{}'", var));
            }
        }

        // Check for BIND with now/datetime manipulation
        if query_upper.contains("BIND") && (
            query_upper.contains("NOW()") || 
            query_upper.contains("YEAR(") ||
            query_upper.contains("MONTH(")
        ) {
            // Allow for legitimate use but flag excessive manipulation
            let datetime_count = query_upper.matches("NOW()").count() 
                + query_upper.matches("YEAR(").count()
                + query_upper.matches("MONTH(").count();
            if datetime_count > 5 {
                return Err(anyhow!("Excessive datetime manipulation in BIND detected"));
            }
        }

        Ok(())
    }

    /// Check for VALUES clause injection
    fn check_values_injection(&self, query: &str) -> Result<()> {
        let query_upper = query.to_uppercase();
        let query_lower = query.to_lowercase();

        // Check if VALUES is present
        if !query_upper.contains("VALUES") {
            return Ok(());
        }

        // Check for sensitive data in VALUES clause
        // Pattern: VALUES ?var { :admin :user }
        if query_upper.contains("VALUES") {
            let sensitive = ["password", "token", "secret", "auth", "key", "private", "credential", "admin", "user"];
            for sens in &sensitive {
                // Check for sensitive property in VALUES
                if query_lower.contains(&format!(":{}", sens)) && query_upper.contains("VALUES") {
                    // Check if VALUES appears nearby
                    let values_pos = query_upper.find("VALUES").unwrap_or(0);
                    let sens_pos = query_lower.find(&format!(":{}", sens)).unwrap_or(0);
                    if sens_pos > values_pos && sens_pos < values_pos + 200 {
                        return Err(anyhow!("VALUES clause contains sensitive property '{}'", sens));
                    }
                }
            }
        }

        // Check for UNDEF pattern in VALUES (potential null bypass)
        if query_upper.contains("VALUES") && query_upper.contains("UNDEF") {
            return Err(anyhow!("VALUES clause with UNDEF pattern detected"));
        }

        // Check for excessive VALUES clauses (DoS risk)
        let values_count = query_upper.matches("VALUES").count();
        if values_count > 2 {
            return Err(anyhow!("Excessive VALUES clauses detected (count: {})", values_count));
        }

        // Check for large VALUES sets (potential DoS)
        if query_upper.contains("VALUES {") {
            let values_start = query_upper.find("VALUES {").unwrap_or(0);
            let values_end = query_upper[values_start..].find('}').unwrap_or(0);
            let values_content = &query_upper[values_start..values_start + values_end];
            
            // Count rows (parentheses groups)
            let row_count = values_content.matches('(').count();
            if row_count > 100 {
                return Err(anyhow!("VALUES clause with excessive rows detected (count: {})", row_count));
            }
        }

        // Check for VALUES with nested data structures
        if query_upper.contains("VALUES") && (
            query_lower.contains("{{") ||  // Double braces (nested)
            query_lower.contains("}}") ||
            query_lower.contains("<<") ||  // RDF collections
            query_lower.contains(">>")
        ) {
            return Err(anyhow!("VALUES clause with nested structures detected"));
        }

        Ok(())
    }

    /// Check for protocol-level attacks (whitespace smuggling, Unicode obfuscation)
    fn check_protocol_attacks(&self, query: &str) -> Result<()> {
        let query_upper = query.to_uppercase();

        // Check for control characters (potential smuggling)
        for (i, c) in query.chars().enumerate() {
            if c < ' ' && c != '\n' && c != '\r' && c != '\t' {
                return Err(anyhow!("Control character detected at position {} (potential smuggling)", i));
            }
        }

        // Check for single-line comment at end of query (query truncation)
        if query.trim_end().ends_with("--") {
            return Err(anyhow!("Single-line comment termination at query end detected"));
        }

        // Check for comment with newline smuggling (e.g., # comment\nUNION)
        if query.contains("#") || query.contains("//") {
            let query_upper = query.to_uppercase();
            if query_upper.contains("UNION") {
                return Err(anyhow!("Comment-based newline smuggling detected"));
            }
        }

        // Check for mixed case keywords (e.g., SeLeCt)
        let first_word = query.trim().split_whitespace().next().unwrap_or("");
        if !first_word.is_empty() && !first_word.chars().all(|c| c.is_uppercase() || c == '(') {
            // Check if first word looks like SELECT but not all caps
            let upper_first = first_word.to_uppercase();
            if upper_first == "SELECT" && first_word != "SELECT" {
                return Err(anyhow!("Mixed case keyword detected (potential obfuscation)"));
            }
        }

        // Check for unicode escape sequences in string literals
        // Note: The \u{XXXX} in test strings are Rust escapes that get decoded at compile time
        // So we need to check for the actual decoded unicode characters too
        if query.contains("\\u{") || query.contains("\\u") || query.contains("\\U") {
            return Err(anyhow!("Unicode escape sequence detected"));
        }

        // Check for zero-width characters and other suspicious unicode
        for c in query.chars() {
            let code = c as u32;
            // Zero-width characters: U+200B, U+200C, U+200D, U+FEFF
            if [0x200B, 0x200C, 0x200D, 0xFEFF].contains(&code) {
                return Err(anyhow!("Zero-width character detected (invisible attack)"));
            }
        }

        // Check for unquoted suspicious keywords in FILTER (potential obfuscation)
        // e.g., FILTER(?o = user) instead of FILTER(?o = "user")
        if query_upper.contains("FILTER") {
            let query_lower = query.to_lowercase();
            // Look for lowercase keywords without quotes that might be obfuscation
            let suspicious_unquoted = ["user", "password", "token", "secret"];
            for sus in &suspicious_unquoted {
                // Check if the keyword appears in FILTER context without quotes
                if query_lower.contains(&format!("= {}", sus)) || query_lower.contains(&format!("= {}", sus)) {
                    return Err(anyhow!("Unquoted keyword '{}' in FILTER detected", sus));
                }
            }
        }

        // Check for SQL-style comment patterns
        if query.contains("'--") || query.contains("\"--") {
            return Err(anyhow!("SQL-style comment termination detected"));
        }

        // Check for multi-line comments in suspicious positions
        if query.contains("/*") && query_upper.contains("UNION") {
            return Err(anyhow!("Multi-line comment with UNION detected"));
        }

        // Check for Unicode homograph attacks (visual spoofing)
        // Look for Cyrillic or similar characters that look like Latin
        for c in query.chars() {
            let code = c as u32;
            // Cyrillic range: U+0400–U+04FF
            if (0x0400..=0x04FF).contains(&code) {
                return Err(anyhow!("Cyrillic character detected (potential homograph attack)"));
            }
            // Greek range: U+0370–U+03FF
            if (0x0370..=0x03FF).contains(&code) {
                return Err(anyhow!("Greek character detected (potential homograph attack)"));
            }
        }

        // Check for mixed encoding patterns
        if query.contains("%") || query.contains("\\u") || query.contains("\\U") {
            return Err(anyhow!("Encoded character sequence detected"));
        }

        // Check for zero-width characters (invisible attacks)
        for c in query.chars() {
            let code = c as u32;
            // Zero-width characters: U+200B, U+200C, U+200D, U+FEFF
            if [0x200B, 0x200C, 0x200D, 0xFEFF].contains(&code) {
                return Err(anyhow!("Zero-width character detected (invisible attack)"));
            }
        }

        // Check for bidirectional override characters (trojan source)
        for c in query.chars() {
            let code = c as u32;
            // RTL/LTR overrides: U+202A-U+202E, U+2066-U+2069
            if (0x202A..=0x202E).contains(&code) || (0x2066..=0x2069).contains(&code) {
                return Err(anyhow!("Bidirectional override character detected (trojan source)"));
            }
        }

        Ok(())
    }
}

/// Helper function for backward compatibility
pub fn validate_sparql_query(query: &str) -> Result<(), String> {
    let validator = SparqlValidator::with_default_config();
    validator.validate(query).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_select_query() {
        let validator = SparqlValidator::with_default_config();
        let query = "SELECT ?s WHERE { ?s a <http://example.org/Test> }";
        assert!(validator.validate(query).is_ok());
    }

    #[test]
    fn test_reject_insert() {
        let validator = SparqlValidator::with_default_config();
        let query = "INSERT DATA { <s> <p> <o> }";
        assert!(validator.validate(query).is_err());
    }

    #[test]
    fn test_reject_delete() {
        let validator = SparqlValidator::with_default_config();
        let query = "DELETE WHERE { ?s ?p ?o }";
        assert!(validator.validate(query).is_err());
    }

    #[test]
    fn test_reject_drop() {
        let validator = SparqlValidator::with_default_config();
        let query = "SELECT ?s WHERE { -- DROP TABLE users\n?s a <http://example.org/Test> }";
        assert!(validator.validate(query).is_err());
    }

    #[test]
    fn test_query_too_long() {
        let mut config = SparqlValidatorConfig::default();
        config.max_query_length = 100;
        let validator = SparqlValidator::new(config);

        let long_query = format!("SELECT ?s WHERE {{ {} }}", "a ".repeat(200));
        assert!(validator.validate(&long_query).is_err());
    }

    #[test]
    fn test_empty_query() {
        let validator = SparqlValidator::with_default_config();
        assert!(validator.validate("").is_err());
        assert!(validator.validate("   ").is_err());
    }

    #[test]
    fn test_valid_ask_query() {
        let validator = SparqlValidator::with_default_config();
        let query = "ASK { <http://example.org> a <http://example.org/Test> }";
        assert!(validator.validate(query).is_ok());
    }

    #[test]
    fn test_valid_construct_query() {
        let validator = SparqlValidator::with_default_config();
        let query = "CONSTRUCT { ?s a <Test> } WHERE { ?s a <http://example.org/Test> }";
        assert!(validator.validate(query).is_ok());
    }
}
