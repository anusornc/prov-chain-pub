//! RDF/XML parser performance benchmarks using rio_xml-backed parser

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use owl2_reasoner::parser::rdf_xml::RdfXmlParser;
use owl2_reasoner::parser::OntologyParser;

#[cfg(feature = "rio-xml")]
const BACKEND: &str = "streaming";
#[cfg(not(feature = "rio-xml"))]
const BACKEND: &str = "legacy";

pub fn bench_rdfxml_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("rdfxml_parsing_{}", BACKEND));

    let small = small_rdfxml();
    let medium = generate_rdfxml(50, 100);
    let large = generate_rdfxml(300, 600);

    let cases = vec![("small", small), ("medium", &medium), ("large", &large)];

    for (name, content) in cases {
        group.bench_with_input(
            BenchmarkId::new("parse_rdfxml", name),
            &content,
            |b, content| {
                b.iter(|| {
                    let mut parser = RdfXmlParser::new();
                    // Enable streaming path in non-strict mode when feature is enabled
                    parser.config.strict_validation = false;
                    let res = parser.parse_str(black_box(content));
                    black_box(res).ok();
                })
            },
        );
    }

    group.finish();
}

fn small_rdfxml() -> &'static str {
    r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
         xmlns:owl="http://www.w3.org/2002/07/owl#"
         xmlns:ex="http://example.org/">
  <owl:Class rdf:about="http://example.org/Person" />
  <owl:Class rdf:about="http://example.org/Student" />
  <rdfs:subClassOf rdf:resource="http://example.org/Person" rdf:about="http://example.org/Student"/>
</rdf:RDF>"#
}

fn generate_rdfxml(classes: usize, individuals: usize) -> String {
    let mut s = String::new();
    s.push_str("<?xml version=\"1.0\"?>\n");
    s.push_str("<rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\" ");
    s.push_str("xmlns:rdfs=\"http://www.w3.org/2000/01/rdf-schema#\" ");
    s.push_str("xmlns:owl=\"http://www.w3.org/2002/07/owl#\" ");
    s.push_str("xmlns:ex=\"http://example.org/\">\n");
    for i in 0..classes {
        s.push_str(&format!(
            "  <owl:Class rdf:about=\"http://example.org/C{}\"/>\n",
            i
        ));
    }
    for i in 0..individuals {
        s.push_str(&format!(
            "  <owl:NamedIndividual rdf:about=\"http://example.org/I{}\"/>\n",
            i
        ));
    }
    s.push_str("</rdf:RDF>\n");
    s
}

criterion_group!(benches, bench_rdfxml_parsing);
criterion_main!(benches);
