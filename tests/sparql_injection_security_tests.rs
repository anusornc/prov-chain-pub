//! SPARQL Injection Security Tests
//!
//! Comprehensive test suite for SPARQL injection attack vectors.
//! Tests validate that the SPARQL validator properly detects and blocks
//! various injection attempts including:
//! - UNION-based injection
//! - Subquery extraction attacks
//! - Property path injection
//! - FILTER expression injection
//! - BIND statement injection
//! - GRAPH injection
//! - Protocol-level attacks

use provchain_org::web::handlers::utils::validate_sparql_query;

// Test helper to check if query is rejected
fn assert_query_rejected(query: &str) {
    let result = validate_sparql_query(query);
    assert!(
        result.is_err(),
        "Query should be rejected but was accepted: {}",
        query
    );
}

// Test helper to check if query is accepted
fn assert_query_accepted(query: &str) {
    let result = validate_sparql_query(query);
    assert!(
        result.is_ok(),
        "Query should be accepted but was rejected: {} - Error: {:?}",
        query,
        result.err()
    );
}

#[cfg(test)]
mod union_injection_tests {
    use super::*;

    #[test]
    fn test_union_basic_injection() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { <http://target> ?p ?o . UNION SELECT ?password WHERE { ?user :hasPassword ?password } }",
            "SELECT ?s ?p ?o WHERE { ?s ?p ?o } UNION { SELECT ?user ?pass WHERE { ?user :password ?pass } }",
            "SELECT * WHERE { ?s ?p ?o . } UNION { SELECT * WHERE { ?u :password ?p } }",
            "SELECT ?s WHERE { ?s ?p ?o . UNION SELECT ?s WHERE { ?s :token ?t } }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_union_with_comment_termination() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { # Comment\nUNION SELECT ?password WHERE { ?user auth:hasPassword ?password } }",
            "SELECT ?s WHERE { // Comment\nUNION SELECT * FROM users }",
            "SELECT ?s WHERE { /* Multi-line comment */\nUNION SELECT ?sensitive WHERE { ?s ?p ?o } }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_union_case_variations() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s ?p ?o . union SELECT ?s WHERE { ?s ?p ?o } }",
            "SELECT ?s WHERE { ?s ?p ?o . Union SELECT ?s WHERE { ?s ?p ?o } }",
            "SELECT ?s WHERE { ?s ?p ?o . UNION SELECT ?s WHERE { ?s ?p ?o } }",
            "SELECT ?s WHERE { ?s ?p ?o . UnIoN SELECT ?s WHERE { ?s ?p ?o } }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_union_with_subquery() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { { SELECT ?s WHERE { ?s ?p ?o } } UNION { SELECT ?s WHERE { ?s auth:password ?p } } }",
            "SELECT ?s WHERE { ?s ?p ?o . } UNION { SELECT ?s WHERE { { SELECT ?s WHERE { ?s ?p ?o } } } }",
            "SELECT (SELECT ?s WHERE { ?s ?p ?o } AS ?x) (SELECT ?s WHERE { ?s auth:password ?p } AS ?y) WHERE {}",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_union_with_aggregation() {
        let malicious_queries = vec![
            "SELECT (COUNT(*) AS ?c) WHERE { ?s ?p ?o } UNION { SELECT (COUNT(?password) AS ?c) WHERE { ?u auth:password ?password } }",
            "SELECT ?s WHERE { ?s a :User } UNION { SELECT (MAX(?password) AS ?p) WHERE { ?u auth:password ?password } }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }
}

#[cfg(test)]
mod subquery_injection_tests {
    use super::*;

    #[test]
    fn test_nested_subquery_extraction() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s ?p ?o . OPTIONAL { SELECT ?password WHERE { ?u auth:password ?password } } }",
            "SELECT ?s WHERE { ?s ?p ?o . { SELECT ?sensitive WHERE { ?s auth:token ?t } } }",
            "SELECT ?s WHERE { ?s ?p ?o . EXISTS { SELECT ?password WHERE { ?u auth:hasPassword ?password } } }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_subquery_with_filter() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s ?p ?o . FILTER EXISTS { SELECT ?password WHERE { ?u auth:password ?password } } }",
            "SELECT ?s WHERE { ?s ?p ?o . FILTER (!EXISTS { SELECT ?sensitive WHERE { ?s auth:secret ?x } }) }",
            "SELECT ?s WHERE { ?s ?p ?o . FILTER (EXISTS { SELECT * WHERE { GRAPH ?g { ?s ?p ?o } } }) }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_subquery_pattern_injection() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s ?p ?o . { SELECT ?s WHERE { ?s ?p ?o } ORDER BY ?s LIMIT 1 } }",
            "SELECT ?s WHERE { ?s ?p ?o . { SELECT ?s WHERE { ?s auth:password ?p } OFFSET 1 } }",
            "SELECT ?s WHERE { ?s ?p ?o . { SELECT (?s AS ?x) WHERE { ?s ?p ?o } GROUP BY ?s } }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }
}

#[cfg(test)]
mod property_path_injection_tests {
    use super::*;

    #[test]
    fn test_inverse_property_path() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s ^:hasPassword ?o }",
            "SELECT ?s WHERE { ?s (^:hasPassword|^:token) ?o }",
            "SELECT ?s WHERE { ?s (^:password|?p) ?o }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_alternative_property_path() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s (:password|:token|:secret) ?o }",
            "SELECT ?s WHERE { ?s (:hasPassword|:hasCredential) ?o }",
            "SELECT ?s WHERE { ?s (:read|:write|:delete) ?o }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_sequence_property_path() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s :hasPassword/:value ?o }",
            "SELECT ?s WHERE { ?s /:password /:token ?o }",
            "SELECT ?s WHERE { ?s :user/:password/:secret ?o }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_zero_or_more_path() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s :hasPassword* ?o }",
            "SELECT ?s WHERE { ?s :password+ ?o }",
            "SELECT ?s WHERE { ?s :secret? ?o }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_negated_property_set() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s !(?p) ?o }",
            "SELECT ?s WHERE { ?s !(a|:type) ?o }",
            "SELECT ?s WHERE { ?s !(^:hasOwner) ?o }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }
}

#[cfg(test)]
mod filter_injection_tests {
    use super::*;

    #[test]
    fn test_filter_sql_injection() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s ?p ?o . FILTER (?o = '1' OR '1'='1') }",
            "SELECT ?s WHERE { ?s ?p ?o . FILTER (?o = 'x' UNION SELECT * FROM users) }",
            "SELECT ?s WHERE { ?s ?p ?o . FILTER (?o = 'admin'--) }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_filter_regex_injection() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s ?p ?o . FILTER REGEX(?o, '.*[\\s\\S]*') }",
            "SELECT ?s WHERE { ?s ?p ?o . FILTER REGEX(?o, '(?:(?!password).)*') }",
            "SELECT ?s WHERE { ?s ?p ?o . FILTER REGEX(?o, '.*password.*') }",
            "SELECT ?s WHERE { ?s ?p ?o . FILTER REGEX(?o, '.*(?:(?:password|token|secret)).*') }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_filter_logical_injection() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s ?p ?o . FILTER (?o = 'value' || ?s = 'admin') }",
            "SELECT ?s WHERE { ?s ?p ?o . FILTER (BOUND(?password) || ?s = 'admin') }",
            "SELECT ?s WHERE { ?s ?p ?o . FILTER (true() && false()) }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_filter_function_injection() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s ?p ?o . FILTER (STRLEN(?o) > 1000) }",
            "SELECT ?s WHERE { ?s ?p ?o . FILTER (CONTAINS(?o, 'password')) }",
            "SELECT ?s WHERE { ?s ?p ?o . FILTER (STRENDS(?o, 'admin')) }",
            "SELECT ?s WHERE { ?s ?p ?o . FILTER (STSTARTS(?o, 'token')) }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_filter_comparison_injection() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s ?p ?o . FILTER (?o > '' || ?s = ?o) }",
            "SELECT ?s WHERE { ?s ?p ?o . FILTER (?o = ?s && ?p = 'password') }",
            "SELECT ?s WHERE { ?s ?p ?o . FILTER (?o IN ('password', 'token', 'secret')) }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }
}

#[cfg(test)]
mod bind_injection_tests {
    use super::*;

    #[test]
    fn test_bind_statement_injection() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s ?p ?o . BIND('password' AS ?injected) }",
            "SELECT ?s WHERE { ?s ?p ?o . BIND(CONCAT(?o, 'admin') AS ?x) }",
            "SELECT ?s WHERE { ?s ?p ?o . BIND(IF(?s = 'admin', ?o, 'hidden') AS ?x) }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_bind_with_unsafe_functions() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s ?p ?o . BIND(REPLACE(?o, 'value', 'admin') AS ?x) }",
            "SELECT ?s WHERE { ?s ?p ?o . BIND(SUBSTR(?o, 1, 10) AS ?x) }",
            "SELECT ?s WHERE { ?s ?p ?o . BIND(MD5(?o) AS ?hash) }",
            "SELECT ?s WHERE { ?s ?p ?o . BIND(SHA1(?o) AS ?hash) }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_bind_with_nested_expressions() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s ?p ?o . BIND((?o + 'admin') AS ?x) }",
            "SELECT ?s WHERE { ?s ?p ?o . BIND(COALESCE(?o, 'default') AS ?x) }",
            "SELECT ?s WHERE { ?s ?p ?o . BIND(STRAFTER(?o, 'pattern') AS ?x) }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }
}

#[cfg(test)]
mod graph_injection_tests {
    use super::*;

    #[test]
    fn test_graph_clause_injection() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { GRAPH <http://admin> { ?s ?p ?o } }",
            "SELECT ?s WHERE { GRAPH ?g { ?s auth:password ?o } }",
            "SELECT ?s WHERE { ?s ?p ?o . } UNION { GRAPH <secret> { ?s ?p ?o } }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_graph_with_subquery() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { { SELECT ?s WHERE { GRAPH <http://sensitive> { ?s ?p ?o } } } }",
            "SELECT ?s WHERE { GRAPH ?g { { SELECT ?s WHERE { ?s auth:token ?p } } } }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_named_graph_iri_injection() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { GRAPH <http://example.org/../admin/data> { ?s ?p ?o } }",
            "SELECT ?s WHERE { GRAPH <http://example.org/../../../etc/passwd> { ?s ?p ?o } }",
            "SELECT ?s WHERE { GRAPH <http://example.org/.hidden> { ?s ?p ?o } }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }
}

#[cfg(test)]
mod protocol_injection_tests {
    use super::*;

    #[test]
    fn test_smuggling_via_newlines() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s ?p ?o .\nUNION\nSELECT ?password WHERE { ?u auth:password ?password } }",
            "SELECT ?s WHERE { ?s ?p ?o .\r\nUNION\r\nSELECT * WHERE { ?s ?p ?o } }",
            "SELECT ?s WHERE { ?s ?p ?o .\tUNION\tSELECT ?s WHERE { ?s auth:token ?t } }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_comment_injection() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s ?p ?o # This is a comment\nUNION SELECT ?password WHERE { ?u auth:password ?p } }",
            "SELECT ?s WHERE { ?s ?p ?o // Another comment\nUNION SELECT * FROM users }",
            "SELECT ?s WHERE { ?s ?p /* Multi-line\ncomment */ ?o }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_string_termination_injection() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s ?p \"' OR '1'='1\" }",
            "SELECT ?s WHERE { ?s ?p \"' UNION SELECT * FROM users--\" }",
            "SELECT ?s WHERE { ?s ?p \"admin'--\" }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_unicode_obfuscation() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s ?p ?o . FILTER(?o = \u{0075}ser) }",
            "SELECT ?s WHERE { ?s ?p ?o .\u{000A}UNION\u{000A}SELECT ?s WHERE { ?s auth:password ?p } }",
            "SELECT ?s WHERE { ?s ?p ?o . FILTER(?o = 'admin\u{200B}') }", // Zero-width character
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }
}

#[cfg(test)]
mod service_endpoint_injection_tests {
    use super::*;

    #[test]
    fn test_service_endpoint_injection() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { SERVICE <http://malicious.com/sparql> { ?s ?p ?o } }",
            "SELECT ?s WHERE { SERVICE <http://localhost:9999/admin> { ?s ?p ?o } }",
            "SELECT ?s WHERE { SERVICE <file:///etc/passwd> { ?s ?p ?o } }",
            "SELECT ?s WHERE { SERVICE <javascript:alert(1)> { ?s ?p ?o } }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_service_silent_injection() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { SERVICE SILENT <http://evil.com/query> { ?s ?p ?o } }",
            "SELECT ?s WHERE { ?s ?p ?o . SERVICE SILENT <http://internal/admin> { ?x ?y ?z } }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }
}

#[cfg(test)]
mod values_clause_injection_tests {
    use super::*;

    #[test]
    fn test_values_clause_injection() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s ?p ?o . VALUES ?s { :admin :user } }",
            "SELECT ?s WHERE { VALUES ?p { :password :token :secret } . ?s ?p ?o }",
            "SELECT ?s WHERE { VALUES (?s ?p) { (UNDEFINED :password) } . ?s ?p ?o }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_values_with_undef() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { VALUES ?s { UNDEF } . ?s ?p ?o }",
            "SELECT ?s WHERE { VALUES (?s ?p ?o) { (UNDEF UNDEF :password) } . ?s ?p ?o }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }
}

#[cfg(test)]
mod update_injection_tests {
    use super::*;

    #[test]
    fn test_delete_injection() {
        let malicious_queries = vec![
            "DELETE WHERE { ?s ?p ?o . FILTER(?s = :admin) }",
            "DELETE WHERE { GRAPH <http://data> { ?s auth:password ?o } }",
            "DELETE WHERE { ?s ?p ?o } INSERT DATA { :admin :hasPassword 'hacked' }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_insert_injection() {
        let malicious_queries = vec![
            "INSERT DATA { :admin :hasPassword 'hacked' }",
            "INSERT { GRAPH <http://admin> { ?s auth:role 'admin' } } WHERE { ?s a :User }",
            "INSERT DATA { :user :password \"' OR '1'='1\" }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_modify_injection() {
        let malicious_queries = vec![
            "DELETE WHERE { ?s auth:password ?o } INSERT { ?s auth:password 'newpass' } WHERE { ?s a :User }",
            "MODIFY DELETE { GRAPH <data> { ?s ?p ?o } } INSERT { GRAPH <backup> { ?s ?p ?o } } WHERE { ?s ?p ?o }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }
}

#[cfg(test)]
mod complex_injection_tests {
    use super::*;

    #[test]
    fn test_blind_injection() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s ?p ?o . FILTER(?s = IRI(CONCAT('http://', STR(?o), '/admin'))) }",
            "SELECT ?s WHERE { ?s ?p ?o . BIND(IRI(CONCAT('http://admin/', STR(?s))) AS ?x) }",
            "SELECT ?s WHERE { ?s ?p ?o . FILTER(STRENDS(STR(?s), 'password')) }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_time_based_blind_injection() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s ?p ?o . BIND(NOW() AS ?t1) ?s ?p ?o2 . BIND(NOW() AS ?t2) . FILTER(?t2 - ?t1 > 1) }",
            "SELECT ?s WHERE { ?s ?p ?o . FILTER(?o && SLEEP(5)) }",
            "SELECT ?s WHERE { ?s ?p ?o . FILTER(EXISTS { SELECT ?s WHERE { ?s ?p ?o } }) }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_second_order_injection() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { ?s ?p ?o . FILTER(REGEX(STR(?o), CONCAT('^', ?injected, '$'))) }",
            "SELECT ?s WHERE { ?s ?p ?o . BIND(SUBSTR(STR(?o), 1, CAST(?injected AS xsd:integer)) AS ?x) }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }
}

#[cfg(test)]
mod valid_queries_tests {
    use super::*;

    #[test]
    fn test_valid_select_queries() {
        let valid_queries = vec![
            "SELECT ?s ?p ?o WHERE { ?s ?p ?o }",
            "SELECT ?s WHERE { ?s a :Product . ?s :name ?name }",
            "SELECT ?s WHERE { ?s :hasSupplier ?supplier . ?supplier :name ?name }",
            "SELECT ?s (COUNT(?o) AS ?count) WHERE { ?s :hasPart ?o } GROUP BY ?s",
            "SELECT ?s WHERE { ?s ?p ?o . FILTER(?o > 10) . } ORDER BY ?s LIMIT 10",
            "SELECT ?s WHERE { ?s :batch ?b . OPTIONAL { ?s :quality ?q } }",
        ];

        for query in valid_queries {
            assert_query_accepted(query);
        }
    }

    #[test]
    fn test_valid_construct_queries() {
        let valid_queries = vec![
            "CONSTRUCT { ?s a :Product } WHERE { ?s a :Product }",
            "CONSTRUCT { ?s :name ?name } WHERE { ?s :name ?name }",
        ];

        for query in valid_queries {
            assert_query_accepted(query);
        }
    }

    #[test]
    fn test_valid_ask_queries()        {
        let valid_queries = vec![
            "ASK WHERE { :product1 a :Product }",
            "ASK WHERE { ?s ?p ?o . FILTER(?o > 100) }",
        ];

        for query in valid_queries {
            assert_query_accepted(query);
        }
    }

    #[test]
    fn test_valid_describe_queries() {
        let valid_queries = vec![
            "DESCRIBE ?s WHERE { ?s a :Product }",
            "DESCRIBE <http://example.org/product1>",
        ];

        for query in valid_queries {
            assert_query_accepted(query);
        }
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_query() {
        assert_query_rejected("");
    }

    #[test]
    fn test_whitespace_only_query() {
        assert_query_rejected("   \n\t   ");
    }

    #[test]
    fn test_query_too_long() {
        let long_query = format!("SELECT ?s WHERE {{ ?s ?p ?o . FILTER(?o = \"{}\") }}", "a".repeat(20000));
        assert_query_rejected(&long_query);
    }

    #[test]
    fn test_nested_brackets() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { { { ?s ?p ?o } } }",
            "SELECT ?s WHERE { ?s ?p [ ?p2 ?o ] }",
            "SELECT ?s WHERE { ?s ?p ( ?p2 ?o ) }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }

    #[test]
    fn test_mixed_case_keywords() {
        let malicious_queries = vec![
            "SeLeCt ?s WhErE { ?s ?p ?o }",
            "sElEcT ?s ?p ?o WhErE { ?s ?p ?o }",
            "SELECT ?s WHERE { ?s ?p ?o } union { SELECT ?s WHERE { ?s auth:password ?p } }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }
}

#[cfg(test)]
mod rdf_star_injection_tests {
    use super::*;

    #[test]
    fn test_rdf_star_injection() {
        let malicious_queries = vec![
            "SELECT ?s WHERE { << ?s ?p ?o >> ?p2 ?o2 }",
            "SELECT ?s WHERE { ?s ?p ?o . << ?s auth:password ?pass >> ?p2 ?o2 }",
            "SELECT ?s WHERE { << ?s ?p ?o >> a :Statement }",
        ];

        for query in malicious_queries {
            assert_query_rejected(query);
        }
    }
}
