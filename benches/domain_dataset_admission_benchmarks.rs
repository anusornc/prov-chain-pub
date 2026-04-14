use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use provchain_org::config::Config;
use provchain_org::core::blockchain::Blockchain;
use provchain_org::ontology::OntologyConfig;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy)]
struct DatasetCase {
    normalizer: &'static str,
    raw_input: &'static str,
    package: &'static str,
}

fn healthcare_ontology_config() -> OntologyConfig {
    let config = Config::default();
    OntologyConfig::new(
        Some("src/semantic/ontologies/healthcare.owl".to_string()),
        &config,
    )
    .expect("healthcare ontology config")
}

fn uht_ontology_config() -> OntologyConfig {
    let config = Config::default();
    OntologyConfig::new(
        Some("src/semantic/ontologies/uht_manufacturing.owl".to_string()),
        &config,
    )
    .expect("uht ontology config")
}

fn pharmaceutical_ontology_config() -> OntologyConfig {
    let config = Config::default();
    OntologyConfig::new(
        Some("src/semantic/ontologies/pharmaceutical.owl".to_string()),
        &config,
    )
    .expect("pharmaceutical ontology config")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn temp_path(name: &str, extension: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{name}_{unique}.{extension}"))
}

fn run_python_script(script: &Path, args: &[&str]) {
    let output = Command::new("python3")
        .arg(script)
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("python script should run");

    assert!(
        output.status.success(),
        "script {:?} failed: stdout={} stderr={}",
        script,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn normalizer_snapshot_id(script_name: &str) -> &'static str {
    match script_name {
        "normalize_fooddata_central.py" => "fooddata_central_sample_2026_03_10",
        "normalize_accessgudid.py" => "accessgudid_sample_2026_03_10",
        "normalize_openfda_drug_shortages.py" => "openfda_drug_shortages_sample_2026_03_10",
        "normalize_openfda_drug_ndc.py" => "openfda_drug_ndc_sample_2026_03_10",
        _ => "dataset_snapshot_2026_03_10",
    }
}

fn emit_payload(case: DatasetCase) -> String {
    let repo = repo_root();
    let normalized_path = temp_path(case.package, "normalized.json");
    let turtle_path = temp_path(case.package, "ttl");
    let normalizer = repo.join("scripts/data_normalization").join(case.normalizer);
    let raw_input = repo.join(case.raw_input);
    let emitter = repo.join("scripts/data_projection/emit_ontology_package_turtle.py");

    run_python_script(
        &normalizer,
        &[
            "--input",
            raw_input.to_str().expect("utf-8 path"),
            "--output",
            normalized_path.to_str().expect("utf-8 path"),
            "--snapshot-id",
            normalizer_snapshot_id(case.normalizer),
        ],
    );
    run_python_script(
        &emitter,
        &[
            "--package",
            case.package,
            "--input",
            normalized_path.to_str().expect("utf-8 path"),
            "--output",
            turtle_path.to_str().expect("utf-8 path"),
            "--limit",
            "1",
        ],
    );

    let payload = fs::read_to_string(&turtle_path).expect("emitted turtle should exist");
    let _ = fs::remove_file(normalized_path);
    let _ = fs::remove_file(turtle_path);
    payload
}

fn emit_batch_payload(case: DatasetCase) -> String {
    let repo = repo_root();
    let normalized_path = temp_path(case.package, "normalized.json");
    let turtle_path = temp_path(case.package, "ttl");
    let normalizer = repo.join("scripts/data_normalization").join(case.normalizer);
    let raw_input = repo.join(case.raw_input);
    let emitter = repo.join("scripts/data_projection/emit_ontology_package_turtle.py");

    run_python_script(
        &normalizer,
        &[
            "--input",
            raw_input.to_str().expect("utf-8 path"),
            "--output",
            normalized_path.to_str().expect("utf-8 path"),
            "--snapshot-id",
            normalizer_snapshot_id(case.normalizer),
        ],
    );
    run_python_script(
        &emitter,
        &[
            "--package",
            case.package,
            "--input",
            normalized_path.to_str().expect("utf-8 path"),
            "--output",
            turtle_path.to_str().expect("utf-8 path"),
        ],
    );

    let payload = fs::read_to_string(&turtle_path).expect("emitted turtle should exist");
    let _ = fs::remove_file(normalized_path);
    let _ = fs::remove_file(turtle_path);
    payload
}

fn bench_setup_only<F>(b: &mut criterion::Bencher<'_>, factory: F)
where
    F: Fn() -> Blockchain,
{
    b.iter_custom(|iters| {
        let mut total = Duration::ZERO;
        for _ in 0..iters {
            let start = std::time::Instant::now();
            let blockchain = factory();
            black_box(blockchain.chain.len());
            total += start.elapsed();
        }
        total
    });
}

fn bench_add_block_only<F>(b: &mut criterion::Bencher<'_>, factory: F, payload: &str)
where
    F: Fn() -> Blockchain,
{
    b.iter_custom(|iters| {
        let mut total = Duration::ZERO;
        for _ in 0..iters {
            let mut blockchain = factory();
            let start = std::time::Instant::now();
            let block = blockchain
                .add_block(payload.to_string())
                .expect("valid ontology event");
            black_box(block);
            total += start.elapsed();
        }
        total
    });
}

fn bench_cross_package_round_robin_single_record(
    b: &mut criterion::Bencher<'_>,
    uht_payload: &str,
    uht_epcis_payload: &str,
    healthcare_payload: &str,
    pharma_payload: &str,
) {
    b.iter_custom(|iters| {
        let mut total = Duration::ZERO;
        for _ in 0..iters {
            let mut uht = Blockchain::new_with_ontology(uht_ontology_config()).expect("blockchain");
            let mut uht_epcis =
                Blockchain::new_with_ontology(uht_ontology_config()).expect("blockchain");
            let mut healthcare =
                Blockchain::new_with_ontology(healthcare_ontology_config()).expect("blockchain");
            let mut pharma =
                Blockchain::new_with_ontology(pharmaceutical_ontology_config()).expect("blockchain");

            let start = std::time::Instant::now();
            let uht_block = uht
                .add_block(uht_payload.to_string())
                .expect("valid uht event");
            let uht_epcis_block = uht_epcis
                .add_block(uht_epcis_payload.to_string())
                .expect("valid uht epcis event");
            let healthcare_block = healthcare
                .add_block(healthcare_payload.to_string())
                .expect("valid healthcare event");
            let pharma_block = pharma
                .add_block(pharma_payload.to_string())
                .expect("valid pharma event");
            total += start.elapsed();

            black_box((uht_block, uht_epcis_block, healthcare_block, pharma_block));
        }
        total
    });
}

fn bench_cross_package_round_robin_batch(
    b: &mut criterion::Bencher<'_>,
    uht_batch_payload: &str,
    uht_epcis_batch_payload: &str,
    healthcare_batch_payload: &str,
    pharma_batch_payload: &str,
) {
    b.iter_custom(|iters| {
        let mut total = Duration::ZERO;
        for _ in 0..iters {
            let mut uht = Blockchain::new_with_ontology(uht_ontology_config()).expect("blockchain");
            let mut uht_epcis =
                Blockchain::new_with_ontology(uht_ontology_config()).expect("blockchain");
            let mut healthcare =
                Blockchain::new_with_ontology(healthcare_ontology_config()).expect("blockchain");
            let mut pharma =
                Blockchain::new_with_ontology(pharmaceutical_ontology_config()).expect("blockchain");

            let start = std::time::Instant::now();
            let uht_block = uht
                .add_block(uht_batch_payload.to_string())
                .expect("valid uht batch");
            let uht_epcis_block = uht_epcis
                .add_block(uht_epcis_batch_payload.to_string())
                .expect("valid uht epcis batch");
            let healthcare_block = healthcare
                .add_block(healthcare_batch_payload.to_string())
                .expect("valid healthcare batch");
            let pharma_block = pharma
                .add_block(pharma_batch_payload.to_string())
                .expect("valid pharma batch");
            total += start.elapsed();

            black_box((uht_block, uht_epcis_block, healthcare_block, pharma_block));
        }
        total
    });
}

fn scale_payload_instances(payload: &str, multiplier: usize) -> String {
    let mut expanded = String::new();
    for index in 0..multiplier {
        let tag = format!("scaled{}_", index + 1);
        for line in payload.lines() {
            if line.starts_with("@prefix ex:") {
                expanded.push_str(line);
            } else {
                expanded.push_str(&line.replace("ex:", &format!("ex:{tag}")));
            }
            expanded.push('\n');
        }
        expanded.push('\n');
    }
    expanded
}

fn bench_domain_dataset_admission(c: &mut Criterion) {
    let uht_case = DatasetCase {
        normalizer: "normalize_fooddata_central.py",
        raw_input: "config/datasets/raw_examples/fooddata_central_sample.json",
        package: "uht",
    };
    let uht_epcis_case = DatasetCase {
        normalizer: "normalize_fooddata_central.py",
        raw_input: "config/datasets/raw_examples/fooddata_central_sample.json",
        package: "uht_epcis",
    };
    let healthcare_case = DatasetCase {
        normalizer: "normalize_accessgudid.py",
        raw_input: "config/datasets/raw_examples/accessgudid_sample.csv",
        package: "healthcare_device",
    };
    let pharma_case = DatasetCase {
        normalizer: "normalize_openfda_drug_shortages.py",
        raw_input: "config/datasets/raw_examples/openfda_drug_shortages_sample.json",
        package: "pharma_storage",
    };

    let uht_payload = emit_payload(uht_case);
    let uht_epcis_payload = emit_payload(uht_epcis_case);
    let healthcare_payload = emit_payload(healthcare_case);
    let pharma_payload = emit_payload(pharma_case);
    let uht_batch_payload = emit_batch_payload(uht_case);
    let uht_epcis_batch_payload = emit_batch_payload(uht_epcis_case);
    let healthcare_batch_payload = emit_batch_payload(healthcare_case);
    let pharma_batch_payload = emit_batch_payload(pharma_case);
    let uht_batch_payload_x2 = scale_payload_instances(&uht_batch_payload, 2);
    let uht_batch_payload_x4 = scale_payload_instances(&uht_batch_payload, 4);
    let uht_epcis_batch_payload_x2 = scale_payload_instances(&uht_epcis_batch_payload, 2);
    let uht_epcis_batch_payload_x4 = scale_payload_instances(&uht_epcis_batch_payload, 4);
    let healthcare_batch_payload_x2 = scale_payload_instances(&healthcare_batch_payload, 2);
    let healthcare_batch_payload_x4 = scale_payload_instances(&healthcare_batch_payload, 4);
    let pharma_batch_payload_x2 = scale_payload_instances(&pharma_batch_payload, 2);
    let pharma_batch_payload_x4 = scale_payload_instances(&pharma_batch_payload, 4);

    let mut group = c.benchmark_group("domain_dataset_admission");

    group.bench_function("uht_product_event_add_block", |b| {
        let payload = uht_payload.clone();
        b.iter_batched(
            || Blockchain::new_with_ontology(uht_ontology_config()).expect("blockchain"),
            |mut blockchain| {
                let block = blockchain
                    .add_block(black_box(payload.clone()))
                    .expect("valid uht event");
                black_box(block);
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("uht_product_event_setup_only", |b| {
        bench_setup_only(b, || Blockchain::new_with_ontology(uht_ontology_config()).expect("blockchain"));
    });

    group.bench_function("uht_product_event_add_block_only", |b| {
        bench_add_block_only(
            b,
            || Blockchain::new_with_ontology(uht_ontology_config()).expect("blockchain"),
            &uht_payload,
        );
    });

    group.bench_function("uht_product_batch_add_block_only", |b| {
        bench_add_block_only(
            b,
            || Blockchain::new_with_ontology(uht_ontology_config()).expect("blockchain"),
            &uht_batch_payload,
        );
    });

    group.bench_function("uht_product_batch_x2_add_block_only", |b| {
        bench_add_block_only(
            b,
            || Blockchain::new_with_ontology(uht_ontology_config()).expect("blockchain"),
            &uht_batch_payload_x2,
        );
    });

    group.bench_function("uht_product_batch_x4_add_block_only", |b| {
        bench_add_block_only(
            b,
            || Blockchain::new_with_ontology(uht_ontology_config()).expect("blockchain"),
            &uht_batch_payload_x4,
        );
    });

    group.bench_function("uht_epcis_event_add_block", |b| {
        let payload = uht_epcis_payload.clone();
        b.iter_batched(
            || Blockchain::new_with_ontology(uht_ontology_config()).expect("blockchain"),
            |mut blockchain| {
                let block = blockchain
                    .add_block(black_box(payload.clone()))
                    .expect("valid uht epcis event");
                black_box(block);
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("uht_epcis_event_add_block_only", |b| {
        bench_add_block_only(
            b,
            || Blockchain::new_with_ontology(uht_ontology_config()).expect("blockchain"),
            &uht_epcis_payload,
        );
    });

    group.bench_function("uht_epcis_batch_add_block_only", |b| {
        bench_add_block_only(
            b,
            || Blockchain::new_with_ontology(uht_ontology_config()).expect("blockchain"),
            &uht_epcis_batch_payload,
        );
    });

    group.bench_function("uht_epcis_batch_x2_add_block_only", |b| {
        bench_add_block_only(
            b,
            || Blockchain::new_with_ontology(uht_ontology_config()).expect("blockchain"),
            &uht_epcis_batch_payload_x2,
        );
    });

    group.bench_function("uht_epcis_batch_x4_add_block_only", |b| {
        bench_add_block_only(
            b,
            || Blockchain::new_with_ontology(uht_ontology_config()).expect("blockchain"),
            &uht_epcis_batch_payload_x4,
        );
    });

    group.bench_function("healthcare_device_event_add_block", |b| {
        let payload = healthcare_payload.clone();
        b.iter_batched(
            || Blockchain::new_with_ontology(healthcare_ontology_config()).expect("blockchain"),
            |mut blockchain| {
                let block = blockchain
                    .add_block(black_box(payload.clone()))
                    .expect("valid healthcare event");
                black_box(block);
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("healthcare_device_event_setup_only", |b| {
        bench_setup_only(b, || {
            Blockchain::new_with_ontology(healthcare_ontology_config()).expect("blockchain")
        });
    });

    group.bench_function("healthcare_device_event_add_block_only", |b| {
        bench_add_block_only(
            b,
            || Blockchain::new_with_ontology(healthcare_ontology_config()).expect("blockchain"),
            &healthcare_payload,
        );
    });

    group.bench_function("healthcare_device_batch_add_block_only", |b| {
        bench_add_block_only(
            b,
            || Blockchain::new_with_ontology(healthcare_ontology_config()).expect("blockchain"),
            &healthcare_batch_payload,
        );
    });

    group.bench_function("healthcare_device_batch_x2_add_block_only", |b| {
        bench_add_block_only(
            b,
            || Blockchain::new_with_ontology(healthcare_ontology_config()).expect("blockchain"),
            &healthcare_batch_payload_x2,
        );
    });

    group.bench_function("healthcare_device_batch_x4_add_block_only", |b| {
        bench_add_block_only(
            b,
            || Blockchain::new_with_ontology(healthcare_ontology_config()).expect("blockchain"),
            &healthcare_batch_payload_x4,
        );
    });

    group.bench_function("pharma_storage_event_add_block", |b| {
        let payload = pharma_payload.clone();
        b.iter_batched(
            || Blockchain::new_with_ontology(pharmaceutical_ontology_config()).expect("blockchain"),
            |mut blockchain| {
                let block = blockchain
                    .add_block(black_box(payload.clone()))
                    .expect("valid pharma event");
                black_box(block);
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("pharma_storage_event_setup_only", |b| {
        bench_setup_only(b, || {
            Blockchain::new_with_ontology(pharmaceutical_ontology_config()).expect("blockchain")
        });
    });

    group.bench_function("pharma_storage_event_add_block_only", |b| {
        bench_add_block_only(
            b,
            || Blockchain::new_with_ontology(pharmaceutical_ontology_config()).expect("blockchain"),
            &pharma_payload,
        );
    });

    group.bench_function("pharma_storage_batch_add_block_only", |b| {
        bench_add_block_only(
            b,
            || Blockchain::new_with_ontology(pharmaceutical_ontology_config()).expect("blockchain"),
            &pharma_batch_payload,
        );
    });

    group.bench_function("pharma_storage_batch_x2_add_block_only", |b| {
        bench_add_block_only(
            b,
            || Blockchain::new_with_ontology(pharmaceutical_ontology_config()).expect("blockchain"),
            &pharma_batch_payload_x2,
        );
    });

    group.bench_function("pharma_storage_batch_x4_add_block_only", |b| {
        bench_add_block_only(
            b,
            || Blockchain::new_with_ontology(pharmaceutical_ontology_config()).expect("blockchain"),
            &pharma_batch_payload_x4,
        );
    });

    group.bench_function("cross_package_round_robin_single_record_add_block_only", |b| {
        bench_cross_package_round_robin_single_record(
            b,
            &uht_payload,
            &uht_epcis_payload,
            &healthcare_payload,
            &pharma_payload,
        );
    });

    group.bench_function("cross_package_round_robin_batch_add_block_only", |b| {
        bench_cross_package_round_robin_batch(
            b,
            &uht_batch_payload,
            &uht_epcis_batch_payload,
            &healthcare_batch_payload,
            &pharma_batch_payload,
        );
    });

    group.finish();
}

criterion_group!(benches, bench_domain_dataset_admission);
criterion_main!(benches);
