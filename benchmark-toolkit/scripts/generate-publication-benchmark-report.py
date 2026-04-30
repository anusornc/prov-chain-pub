#!/usr/bin/env python3
"""Generate a family-scoped publication benchmark report from evidence exports."""

from __future__ import annotations

import argparse
import json
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path


REFERENCE_FAIRNESS_LABELS = {
    "externalized-semantic-pipeline",
    "not-comparable",
    "public-chain-baseline",
    "secondary-baseline",
}


def load_json(path: Path) -> dict:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def evidence_rows(evidence_dir: Path) -> list[dict]:
    results_path = evidence_dir / "campaign_results.json"
    manifest_path = evidence_dir / "campaign_manifest.json"
    status_path = evidence_dir / "campaign_status.json"

    if not results_path.exists():
        raise SystemExit(f"missing campaign_results.json: {evidence_dir}")
    if not manifest_path.exists():
        raise SystemExit(f"missing campaign_manifest.json: {evidence_dir}")
    if not status_path.exists():
        raise SystemExit(f"missing campaign_status.json: {evidence_dir}")

    results = load_json(results_path)
    manifest = load_json(manifest_path)
    status = load_json(status_path)
    rows = []
    for row in results.get("summaries", []):
        enriched = dict(row)
        enriched["campaign_id"] = manifest.get("campaign_id", evidence_dir.name)
        enriched["dataset_slice"] = manifest.get("dataset_slice", "")
        enriched["campaign_status"] = status.get("status", "")
        rows.append(enriched)
    return rows


def evidence_source(evidence_dir: Path) -> dict:
    manifest_path = evidence_dir / "campaign_manifest.json"
    status_path = evidence_dir / "campaign_status.json"

    if not manifest_path.exists():
        raise SystemExit(f"missing campaign_manifest.json: {evidence_dir}")
    if not status_path.exists():
        raise SystemExit(f"missing campaign_status.json: {evidence_dir}")

    manifest = load_json(manifest_path)
    status = load_json(status_path)
    products = manifest.get("products", [])
    if isinstance(products, list):
        products_value = ", ".join(str(product) for product in products)
    else:
        products_value = str(products)

    return {
        "directory": evidence_dir.as_posix(),
        "campaign_id": manifest.get("campaign_id", evidence_dir.name),
        "status": status.get("status", ""),
        "family": manifest.get("benchmark_family", ""),
        "dataset": manifest.get("dataset_slice", ""),
        "workload": manifest.get("workload", ""),
        "products": products_value,
    }


def fmt_number(value, digits: int = 3) -> str:
    try:
        return f"{float(value):.{digits}f}"
    except (TypeError, ValueError):
        return ""


def field(row: dict, name: str, default: str) -> str:
    value = row.get(name)
    if value is None or value == "":
        return default
    return str(value)


def family_title(family: str) -> str:
    return family.replace("_", "-")


def is_load_or_import(row: dict) -> bool:
    test_name = str(row.get("test_name", "")).lower()
    metric_type = str(row.get("metric_type", "")).lower()
    return any(token in test_name for token in ("load", "import")) or any(
        token in metric_type for token in ("load", "import")
    )


def is_reference_metric(row: dict) -> bool:
    fairness_label = field(row, "fairness_label", "legacy-not-recorded")
    metric_type = field(row, "metric_type", "legacy-latency-ms")
    unit = field(row, "unit", "ms")
    return (
        fairness_label in REFERENCE_FAIRNESS_LABELS
        or metric_type == "gas-used"
        or unit == "gas"
    )


def row_caveat(row: dict) -> str:
    caveats = []
    family = field(row, "family", "")
    scenario_name = field(row, "scenario", "")
    fairness_label = field(row, "fairness_label", "legacy-not-recorded")
    metric_type = field(row, "metric_type", "legacy-latency-ms")

    if family == "governance-policy" and scenario_name == "Policy Enforcement":
        caveats.append("server-reported policy decision latency; not full API round-trip")
    if family == "governance-policy" and scenario_name == "Policy Enforcement E2E":
        caveats.append("client-observed policy API round-trip")
    if is_load_or_import(row):
        caveats.append("load/import setup metric; not primary ledger-write evidence")
    if fairness_label == "legacy-not-recorded":
        caveats.append("legacy export lacks capability/fairness metadata")
    if fairness_label == "public-chain-baseline":
        caveats.append("public-chain reference; not permissioned-ledger evidence")
    if fairness_label == "secondary-baseline":
        caveats.append("secondary reference path")
    if fairness_label == "externalized-semantic-pipeline":
        caveats.append("externalized semantic pipeline reference")
    if fairness_label == "cross-model-with-caveat":
        caveats.append("cross-model parity row; compare within the scenario but preserve endpoint/model caveat")
    if fairness_label == "not-comparable":
        caveats.append("diagnostic/reference metric; not a primary comparison row")
    if metric_type == "gas-used":
        caveats.append("resource-cost metric; not latency")

    return "; ".join(caveats) if caveats else "primary scoped metric"


def split_primary_and_reference(rows: list[dict]) -> tuple[list[dict], list[dict]]:
    primary_rows = []
    reference_rows = []
    for row in rows:
        if is_load_or_import(row) or is_reference_metric(row):
            reference_rows.append(row)
        else:
            primary_rows.append(row)
    return primary_rows, reference_rows


def write_metric_table(handle, rows: list[dict]) -> None:
    handle.write(
        "| Campaign | Dataset | Test | System | Path | Fairness | Metric | Unit | Samples | Success | Mean | p95 | p99 | Caveat |\n"
    )
    handle.write("|---|---|---|---|---|---|---|---|---:|---:|---:|---:|---:|---|\n")
    for row in rows:
        handle.write(
            "| `{campaign}` | `{dataset}` | `{test}` | `{system}` | `{path}` | `{fairness}` | `{metric}` | `{unit}` | {samples} | {success}% | {mean} | {p95} | {p99} | {caveat} |\n".format(
                campaign=row.get("campaign_id", ""),
                dataset=row.get("dataset_slice", ""),
                test=row.get("test_name", ""),
                system=row.get("system", ""),
                path=field(row, "capability_path", "legacy-not-recorded"),
                fairness=field(row, "fairness_label", "legacy-not-recorded"),
                metric=field(row, "metric_type", "legacy-latency-ms"),
                unit=field(row, "unit", "ms"),
                samples=row.get("samples", ""),
                success=fmt_number(row.get("success_rate"), 2),
                mean=fmt_number(row.get("mean_ms")),
                p95=fmt_number(row.get("p95_ms")),
                p99=fmt_number(row.get("p99_ms")),
                caveat=row_caveat(row),
            )
        )


def write_report(output_path: Path, evidence_dirs: list[Path]) -> None:
    all_rows = []
    sources = []
    for evidence_dir in evidence_dirs:
        sources.append(evidence_source(evidence_dir))
        all_rows.extend(evidence_rows(evidence_dir))

    grouped: dict[str, list[dict]] = defaultdict(list)
    for row in all_rows:
        grouped[row.get("family", "unknown")].append(row)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    generated_at = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    with output_path.open("w", encoding="utf-8") as handle:
        handle.write("# Publication Benchmark Report Bundle\n\n")
        handle.write(f"- Generated at: `{generated_at}`\n")
        handle.write("- Scope: family-specific benchmark evidence only\n")
        handle.write("- Rule: no single global winner table is generated\n\n")
        handle.write(
            "This report separates primary benchmark metrics from load, import, and reference metrics. "
            "Load/import rows describe setup or data-ingestion paths and must not be used as primary ledger-write evidence.\n\n"
        )

        handle.write("## Evidence Sources\n\n")
        handle.write(
            "| Evidence Directory | Campaign | Status | Family | Dataset | Workload | Products |\n"
        )
        handle.write("|---|---|---|---|---|---|---|\n")
        for source in sources:
            handle.write(
                "| `{directory}` | `{campaign}` | `{status}` | `{family}` | `{dataset}` | `{workload}` | `{products}` |\n".format(
                    directory=source["directory"],
                    campaign=source["campaign_id"],
                    status=source["status"],
                    family=source["family"],
                    dataset=source["dataset"],
                    workload=source["workload"],
                    products=source["products"],
                )
            )
        handle.write("\n")

        for family in sorted(grouped):
            rows = sorted(
                grouped[family],
                key=lambda row: (
                    row.get("campaign_id", ""),
                    row.get("test_name", ""),
                    row.get("system", ""),
                    row.get("metric_type", ""),
                ),
            )
            primary_rows, reference_rows = split_primary_and_reference(rows)
            handle.write(f"## {family_title(family)}\n\n")

            handle.write("### Primary Benchmark Metrics\n\n")
            if primary_rows:
                write_metric_table(handle, primary_rows)
            else:
                handle.write(
                    "No primary metrics are emitted for this family from the supplied evidence set.\n"
                )
            handle.write("\n")

            if reference_rows:
                handle.write("### Load, Import, And Reference Metrics\n\n")
                handle.write(
                    "These rows are retained as evidence context, but they are not primary within-family winner evidence.\n\n"
                )
                write_metric_table(handle, reference_rows)
                handle.write("\n")

            semantic_rows = [row for row in rows if family == "semantic"]
            if semantic_rows:
                handle.write("\nSemantic capability fields:\n\n")
                handle.write(
                    "| System | Native Semantic Support | External Semantic Stages | Explanation Support |\n"
                )
                handle.write("|---|---:|---|---:|\n")
                seen = set()
                for row in semantic_rows:
                    key = (row.get("system", ""), row.get("capability_path", ""))
                    if key in seen:
                        continue
                    seen.add(key)
                    handle.write(
                        "| `{system}` | `{native}` | `{stages}` | `{explain}` |\n".format(
                            system=row.get("system", ""),
                            native=row.get("native_semantic_support", ""),
                            stages=row.get("external_semantic_stages", ""),
                            explain=row.get("explanation_support", ""),
                        )
                    )
            handle.write("\n")

        handle.write("## Fairness And Limitations\n\n")
        handle.write(
            "- Interpret every result only within its benchmark family, workload, dataset slice, and capability path.\n"
        )
        handle.write(
            "- Treat load/import rows as data-ingestion or setup-path evidence, not as primary ledger-write results.\n"
        )
        handle.write(
            "- Do not compare native RDF trace-query latency directly with ledger finality, public-chain gas, or policy checks.\n"
        )
        handle.write(
            "- `legacy-not-recorded` rows come from older exports that did not capture explicit capability and fairness metadata.\n"
        )
        handle.write(
            "- `externalized-semantic-pipeline` rows include a different semantic capability path from native ProvChain validation.\n"
        )
        handle.write(
            "- `public-chain-baseline` rows are public-chain execution evidence, not permissioned-enterprise ledger evidence.\n"
        )
        handle.write(
            "- `cross-model-with-caveat` rows are scenario-level parity evidence and must preserve endpoint/model differences in any claim.\n"
        )
        handle.write(
            "- Failed or partial campaigns must remain excluded from publication claims unless explicitly discussed as negative evidence.\n"
        )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path, help="Markdown report path")
    parser.add_argument("evidence_dirs", nargs="+", type=Path, help="Curated evidence directories")
    args = parser.parse_args()

    write_report(args.output, args.evidence_dirs)
    print(f"wrote publication benchmark report: {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
