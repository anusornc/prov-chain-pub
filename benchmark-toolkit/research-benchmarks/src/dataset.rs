use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

const STANDARD_PREFIXES: [(&str, &str); 2] = [
    ("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#"),
    ("rdfs", "http://www.w3.org/2000/01/rdf-schema#"),
];

fn prefix_declaration_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"@prefix\s+([A-Za-z][\w-]*):\s*<([^>]+)>\s*\.").expect("valid prefix regex")
    })
}

fn slashed_curie_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"\b([A-Za-z][\w-]*):([A-Za-z0-9._~-]+(?:/[A-Za-z0-9._~-]+)+)\b")
            .expect("valid slashed curie regex")
    })
}

pub fn normalize_turtle(content: &str) -> String {
    let with_prefixes = ensure_required_prefixes(content);
    expand_slashed_curie_tokens(&with_prefixes)
}

fn ensure_required_prefixes(content: &str) -> String {
    let mut missing = Vec::new();
    for (prefix, uri) in STANDARD_PREFIXES {
        let marker = format!("@prefix {prefix}:");
        if !content.contains(&marker) {
            missing.push(format!("@prefix {prefix}: <{uri}> ."));
        }
    }

    if missing.is_empty() {
        return content.to_string();
    }

    format!("{}\n{}", missing.join("\n"), content)
}

fn expand_slashed_curie_tokens(content: &str) -> String {
    let prefix_map: HashMap<String, String> = prefix_declaration_regex()
        .captures_iter(content)
        .map(|captures| (captures[1].to_string(), captures[2].to_string()))
        .collect();

    slashed_curie_regex()
        .replace_all(content, |captures: &regex::Captures<'_>| {
            let prefix = &captures[1];
            let local_part = &captures[2];
            match prefix_map.get(prefix) {
                Some(namespace) => format!("<{namespace}{local_part}>"),
                None => captures[0].to_string(),
            }
        })
        .into_owned()
}
