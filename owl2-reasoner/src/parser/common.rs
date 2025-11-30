//! Common parsing utilities and helpers

use crate::error::{OwlError, OwlResult};
use crate::iri::IRI;
use hashbrown::HashMap;

/// Common RDF/OWL vocabulary terms
pub static RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
pub static RDFS_SUBCLASSOF: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";
pub static RDFS_DOMAIN: &str = "http://www.w3.org/2000/01/rdf-schema#domain";
pub static RDFS_RANGE: &str = "http://www.w3.org/2000/01/rdf-schema#range";
pub static OWL_ONTOLOGY: &str = "http://www.w3.org/2002/07/owl#Ontology";
pub static OWL_IMPORTS: &str = "http://www.w3.org/2002/07/owl#imports";
pub static OWL_CLASS: &str = "http://www.w3.org/2002/07/owl#Class";
pub static OWL_OBJECT_PROPERTY: &str = "http://www.w3.org/2002/07/owl#ObjectProperty";
pub static OWL_DATA_PROPERTY: &str = "http://www.w3.org/2002/07/owl#DataProperty";
pub static OWL_NAMED_INDIVIDUAL: &str = "http://www.w3.org/2002/07/owl#NamedIndividual";
pub static OWL_ANNOTATION_PROPERTY: &str = "http://www.w3.org/2002/07/owl#AnnotationProperty";
pub static OWL_EQUIVALENT_CLASS: &str = "http://www.w3.org/2002/07/owl#equivalentClass";
pub static OWL_DISJOINT_WITH: &str = "http://www.w3.org/2002/07/owl#disjointWith";
pub static OWL_SAME_AS: &str = "http://www.w3.org/2002/07/owl#sameAs";
pub static OWL_DIFFERENT_FROM: &str = "http://www.w3.org/2002/07/owl#differentFrom";

/// Parse a literal value with optional datatype or language tag
pub fn parse_literal(
    value: &str,
    datatype: Option<&str>,
    language: Option<&str>,
) -> OwlResult<crate::entities::Literal> {
    match (datatype, language) {
        (Some(dt), None) => {
            let iri = IRI::new(dt)?;
            Ok(crate::entities::Literal::typed(value, iri))
        }
        (None, Some(lang)) => Ok(crate::entities::Literal::lang_tagged(value, lang)),
        (None, None) => Ok(crate::entities::Literal::simple(value)),
        (Some(_), Some(_)) => Err(OwlError::ParseError(
            "Literal cannot have both datatype and language tag".to_string(),
        )),
    }
}

/// Parse a CURIE (Compact URI) like "owl:Class"
pub fn parse_curie(curie: &str, prefixes: &HashMap<String, String>) -> OwlResult<IRI> {
    if let Some(colon_pos) = curie.find(':') {
        let prefix = &curie[..colon_pos];
        let local = &curie[colon_pos + 1..];

        if let Some(namespace) = prefixes.get(prefix) {
            let iri_str = format!("{namespace}{local}");
            return IRI::new(iri_str);
        }
    }

    // If no colon or prefix not found, treat as full IRI
    IRI::new(curie)
}

/// Normalize a whitespace string
pub fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Parse a boolean string
pub fn parse_bool(s: &str) -> OwlResult<bool> {
    match s.to_lowercase().as_str() {
        "true" | "1" | "yes" => Ok(true),
        "false" | "0" | "no" => Ok(false),
        _ => Err(OwlError::ParseError(format!("Invalid boolean value: {s}"))),
    }
}

/// Parse an integer string
pub fn parse_int(s: &str) -> OwlResult<i64> {
    s.parse()
        .map_err(|_| OwlError::ParseError(format!("Invalid integer value: {s}")))
}

/// Parse a float string
pub fn parse_float(s: &str) -> OwlResult<f64> {
    s.parse()
        .map_err(|_| OwlError::ParseError(format!("Invalid float value: {s}")))
}

/// Validate IRI syntax
pub fn validate_iri(iri: &str) -> OwlResult<()> {
    if iri.is_empty() {
        return Err(OwlError::InvalidIRI("Empty IRI".to_string()));
    }

    // Basic validation - check for invalid characters
    if iri.contains(' ') || iri.contains('<') || iri.contains('>') {
        return Err(OwlError::InvalidIRI(format!(
            "Invalid characters in IRI: {iri}"
        )));
    }

    // Check for valid scheme
    if !iri.contains(':') {
        return Err(OwlError::InvalidIRI(format!(
            "Missing scheme in IRI: {iri}"
        )));
    }

    // Enhanced IRI validation according to RFC 3987
    validate_iri_scheme(iri)?;
    validate_iri_path(iri)?;
    validate_ri_components(iri)?;

    Ok(())
}

/// Validate IRI scheme according to RFC 3987
fn validate_iri_scheme(iri: &str) -> OwlResult<()> {
    let scheme_end = iri
        .find(':')
        .ok_or_else(|| OwlError::InvalidIRI("Missing scheme in IRI".to_string()))?;

    let scheme = &iri[..scheme_end];

    if scheme.is_empty() {
        return Err(OwlError::InvalidIRI("Empty scheme in IRI".to_string()));
    }

    // Scheme must start with a letter
    if let Some(first_char) = scheme.chars().next() {
        if !first_char.is_ascii_alphabetic() {
            return Err(OwlError::InvalidIRI(
                "Scheme must start with a letter".to_string(),
            ));
        }
    } else {
        return Err(OwlError::InvalidIRI("Empty scheme".to_string()));
    }

    // Scheme can contain letters, digits, '+', '-', '.'
    for c in scheme.chars() {
        if !c.is_ascii_alphanumeric() && c != '+' && c != '-' && c != '.' {
            return Err(OwlError::InvalidIRI(format!(
                "Invalid character '{}' in scheme",
                c
            )));
        }
    }

    Ok(())
}

/// Validate IRI path according to RFC 3987
fn validate_iri_path(iri: &str) -> OwlResult<()> {
    let scheme_end = iri
        .find(':')
        .ok_or_else(|| OwlError::InvalidIRI("Missing scheme in IRI".to_string()))?;

    let after_scheme = &iri[scheme_end + 1..];

    if after_scheme.is_empty() {
        return Err(OwlError::InvalidIRI(
            "IRI must have content after scheme".to_string(),
        ));
    }

    // Check for invalid characters in path
    let invalid_chars = ['<', '>', '"', ' ', '{', '}', '|', '\\', '^', '`'];
    for c in invalid_chars {
        if after_scheme.contains(c) {
            return Err(OwlError::InvalidIRI(format!(
                "Invalid character '{}' in IRI path",
                c
            )));
        }
    }

    Ok(())
}

/// Validate IRI components according to RFC 3987 (Internationalized Resource Identifiers)
fn validate_ri_components(iri: &str) -> OwlResult<()> {
    // Check for valid UTF-8 encoding (Rust strings are always valid UTF-8)

    // Check for fragment identifier
    if let Some(fragment_start) = iri.find('#') {
        let fragment = &iri[fragment_start + 1..];
        if fragment.is_empty() {
            return Err(OwlError::InvalidIRI(
                "Empty fragment identifier".to_string(),
            ));
        }
    }

    // Check for query string
    if let Some(query_start) = iri.find('?') {
        let query = &iri[query_start + 1..];
        if query.is_empty() {
            return Err(OwlError::InvalidIRI("Empty query string".to_string()));
        }
    }

    // Basic check for IP literal format (simplified)
    if let (Some(ip_start), Some(ip_end)) = (iri.find('['), iri.find(']')) {
        if ip_start >= ip_end {
            return Err(OwlError::InvalidIRI(
                "Invalid IP literal format".to_string(),
            ));
        }
    }

    Ok(())
}

/// Escape XML special characters
pub fn escape_xml(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&apos;")
}

/// Unescape XML special characters
pub fn unescape_xml(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

/// Parse a list of IRIs from a string (space or comma separated)
pub fn parse_iri_list(s: &str, prefixes: &HashMap<String, String>) -> OwlResult<Vec<IRI>> {
    let items: Vec<&str> = s
        .split([',', ' '])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut iris = Vec::new();
    for item in items {
        iris.push(parse_curie(item, prefixes)?);
    }

    Ok(iris)
}

/// Get the local name from an IRI
pub fn get_local_name(iri: &str) -> &str {
    if let Some(hash_pos) = iri.rfind('#') {
        &iri[hash_pos + 1..]
    } else if let Some(slash_pos) = iri.rfind('/') {
        &iri[slash_pos + 1..]
    } else {
        iri
    }
}

/// Get the namespace from an IRI
pub fn get_namespace(iri: &str) -> &str {
    if let Some(hash_pos) = iri.rfind('#') {
        &iri[..hash_pos + 1]
    } else if let Some(slash_pos) = iri.rfind('/') {
        &iri[..slash_pos + 1]
    } else {
        ""
    }
}
