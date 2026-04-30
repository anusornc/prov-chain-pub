#!/usr/bin/env python3
"""Summarize ProvChain benchmark profiling metadata from a campaign directory."""

from __future__ import annotations

import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from statistics import mean
from typing import Any


PROFILE_TEST_NAMES = {
    "Single-threaded Write (100 tx)",
    "Steady-state Append After Cold Load (100 tx)",
}
LOAD_TEST_NAME = "Turtle RDF Import"


def percentile(values: list[float], quantile: float) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    if len(ordered) == 1:
        return ordered[0]
    position = (len(ordered) - 1) * quantile
    lower = int(position)
    upper = min(lower + 1, len(ordered) - 1)
    weight = position - lower
    return ordered[lower] * (1.0 - weight) + ordered[upper] * weight


def summary(values: list[float]) -> dict[str, float | int]:
    if not values:
        return {
            "samples": 0,
            "mean": 0.0,
            "p95": 0.0,
            "p99": 0.0,
            "min": 0.0,
            "max": 0.0,
        }
    return {
        "samples": len(values),
        "mean": mean(values),
        "p95": percentile(values, 0.95),
        "p99": percentile(values, 0.99),
        "min": min(values),
        "max": max(values),
    }


def load_campaign_manifest(campaign_dir: Path) -> dict[str, Any]:
    manifest_path = campaign_dir / "campaign_manifest.json"
    if not manifest_path.is_file():
        return {}
    return json.loads(manifest_path.read_text(encoding="utf-8"))


def load_result_rows(campaign_dir: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for results_path in sorted(campaign_dir.glob("epochs/*/runs/*/benchmark_results.json")):
        with results_path.open("r", encoding="utf-8") as handle:
            data = json.load(handle)
        if not isinstance(data, list):
            continue
        for row in data:
            if row.get("system") == "ProvChain-Org":
                row = dict(row)
                row["_results_path"] = str(results_path)
                rows.append(row)
    return rows


def collect_metric(rows: list[dict[str, Any]], key: str) -> list[float]:
    values: list[float] = []
    for row in rows:
        value = row.get(key)
        if isinstance(value, (int, float)):
            values.append(float(value))
    return values


def collect_metadata_metric(rows: list[dict[str, Any]], key: str) -> list[float]:
    values: list[float] = []
    for row in rows:
        metadata = row.get("metadata") or {}
        value = metadata.get(key)
        if isinstance(value, (int, float)):
            values.append(float(value))
    return values


def collect_stage_metric(rows: list[dict[str, Any]], stage: str) -> list[float]:
    values: list[float] = []
    for row in rows:
        metadata = row.get("metadata") or {}
        timings = metadata.get("server_timing_avg_ms") or {}
        value = timings.get(stage)
        if isinstance(value, (int, float)):
            values.append(float(value))
    return values


def write_markdown(report: dict[str, Any], path: Path) -> None:
    metrics = report["metrics_ms"]
    cold_load = report["cold_load_ms"]
    stages = report["server_stage_avg_ms"]
    cold_load_stages = report["cold_load_server_stage_avg_ms"]
    dominant = report.get("dominant_server_stage", {})

    lines = [
        "# ProvChain Ledger Write Profiling Summary",
        "",
        f"- Campaign: `{report['campaign_id']}`",
        f"- Generated at: `{report['generated_at_utc']}`",
        f"- Append samples: `{report['append_samples']}`",
        f"- Cold-load samples: `{report['cold_load_samples']}`",
        f"- Transactions per sample: `{report['transaction_count_per_sample']}`",
        f"- Evidence role: `{report['evidence_role']}`",
        f"- Append test names observed: `{', '.join(report['append_test_names_observed'])}`",
        "",
        "## Cold-Load Metrics",
        "",
        "| Metric | Samples | Mean ms | p95 ms | p99 ms | Min ms | Max ms |",
        "|---|---:|---:|---:|---:|---:|---:|",
    ]
    for key, label in [
        ("total", "Cold Turtle RDF import total"),
        ("dataset_read", "Dataset read"),
        ("dataset_normalize", "Turtle normalize"),
        ("dataset_parse", "Turtle parse"),
        ("auth", "Authentication"),
        ("client_submit_loop", "Client submit loop"),
    ]:
        item = cold_load[key]
        lines.append(
            f"| {label} | {item['samples']} | {item['mean']:.3f} | {item['p95']:.3f} | "
            f"{item['p99']:.3f} | {item['min']:.3f} | {item['max']:.3f} |"
        )

    if report["cold_load_samples"]:
        lines.extend(
            [
                "",
                "## Cold-Load Server Handler Stage Averages",
                "",
                "| Stage | Samples | Mean ms/tx | p95 ms/tx | p99 ms/tx | Min ms/tx | Max ms/tx |",
                "|---|---:|---:|---:|---:|---:|---:|",
            ]
        )
        for stage in [
            "handler_total",
            "block_admission",
            "request_validation",
            "blockchain_lock_wait",
            "turtle_materialization",
        ]:
            item = cold_load_stages[stage]
            lines.append(
                f"| `{stage}` | {item['samples']} | {item['mean']:.3f} | {item['p95']:.3f} | "
                f"{item['p99']:.3f} | {item['min']:.3f} | {item['max']:.3f} |"
            )

    lines.extend(
        [
            "",
            "## Steady-State Append Metrics",
        "",
        "| Metric | Samples | Mean ms | p95 ms | p99 ms | Min ms | Max ms |",
        "|---|---:|---:|---:|---:|---:|---:|",
        ]
    )
    for key, label in [
        ("batch_total", "Batch total"),
        ("auth", "Authentication"),
        ("client_submit_loop", "Client submit loop"),
    ]:
        item = metrics[key]
        lines.append(
            f"| {label} | {item['samples']} | {item['mean']:.3f} | {item['p95']:.3f} | "
            f"{item['p99']:.3f} | {item['min']:.3f} | {item['max']:.3f} |"
        )

    lines.extend(
        [
            "",
            "## Steady-State Append Server Handler Stage Averages",
            "",
            "| Stage | Samples | Mean ms/tx | p95 ms/tx | p99 ms/tx | Min ms/tx | Max ms/tx |",
            "|---|---:|---:|---:|---:|---:|---:|",
        ]
    )
    for stage in [
        "handler_total",
        "block_admission",
        "request_validation",
        "blockchain_lock_wait",
        "turtle_materialization",
    ]:
        item = stages[stage]
        lines.append(
            f"| `{stage}` | {item['samples']} | {item['mean']:.3f} | {item['p95']:.3f} | "
            f"{item['p99']:.3f} | {item['min']:.3f} | {item['max']:.3f} |"
        )

    if dominant:
        lines.extend(
            [
                "",
                "## Interpretation",
                "",
                (
                    f"- Dominant measured server stage: `{dominant['stage']}` "
                    f"at `{dominant['percent_of_handler_total']:.2f}%` of mean handler time."
                ),
                "- This is profiling/remediation evidence, not a primary cross-system comparison table.",
            ]
        )

    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    if len(sys.argv) not in (2, 3):
        print(
            "Usage: summarize-provchain-profile.py <campaign_dir> [output_dir]",
            file=sys.stderr,
        )
        return 2

    campaign_dir = Path(sys.argv[1]).resolve()
    output_dir = Path(sys.argv[2]).resolve() if len(sys.argv) == 3 else campaign_dir
    output_dir.mkdir(parents=True, exist_ok=True)

    rows = load_result_rows(campaign_dir)
    append_rows = [
        row for row in rows if row.get("test_name") in PROFILE_TEST_NAMES
    ]
    cold_load_rows = [row for row in rows if row.get("test_name") == LOAD_TEST_NAME]
    if not append_rows and not cold_load_rows:
        print(f"error: no ProvChain load or append profile rows found in {campaign_dir}", file=sys.stderr)
        return 1

    manifest = load_campaign_manifest(campaign_dir)
    campaign_id = manifest.get("campaign_id", campaign_dir.name)
    transaction_counts = {
        int(row.get("metadata", {}).get("transaction_count", 0))
        for row in append_rows
        if isinstance(row.get("metadata", {}).get("transaction_count"), int)
    }
    transaction_count_per_sample = (
        next(iter(transaction_counts)) if len(transaction_counts) == 1 else "mixed"
    )
    append_test_names_observed = sorted(
        {
            row.get("test_name")
            for row in append_rows
            if isinstance(row.get("test_name"), str)
        }
    )

    stages = {
        stage: summary(collect_stage_metric(append_rows, stage))
        for stage in [
            "handler_total",
            "block_admission",
            "request_validation",
            "blockchain_lock_wait",
            "turtle_materialization",
        ]
    }
    cold_load_stages = {
        stage: summary(collect_stage_metric(cold_load_rows, stage))
        for stage in [
            "handler_total",
            "block_admission",
            "request_validation",
            "blockchain_lock_wait",
            "turtle_materialization",
        ]
    }

    handler_mean = stages["handler_total"]["mean"]
    block_mean = stages["block_admission"]["mean"]
    dominant = {
        "stage": "block_admission",
        "percent_of_handler_total": (block_mean / handler_mean * 100.0)
        if handler_mean
        else 0.0,
    }

    report = {
        "campaign_id": campaign_id,
        "generated_at_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "source_campaign_dir": str(campaign_dir),
        "append_samples": len(append_rows),
        "cold_load_samples": len(cold_load_rows),
        "transaction_count_per_sample": transaction_count_per_sample,
        "append_test_names_observed": append_test_names_observed,
        "evidence_role": "profiling_reference_not_primary_paper_comparison",
        "cold_load_ms": {
            "total": summary(collect_metric(cold_load_rows, "duration_ms")),
            "dataset_read": summary(
                collect_metadata_metric(cold_load_rows, "dataset_read_latency_ms")
            ),
            "dataset_normalize": summary(
                collect_metadata_metric(cold_load_rows, "dataset_normalize_latency_ms")
            ),
            "dataset_parse": summary(
                collect_metadata_metric(cold_load_rows, "dataset_parse_latency_ms")
            ),
            "auth": summary(collect_metadata_metric(cold_load_rows, "auth_latency_ms")),
            "client_submit_loop": summary(
                collect_metadata_metric(cold_load_rows, "client_submit_loop_latency_ms")
            ),
        },
        "metrics_ms": {
            "batch_total": summary(collect_metric(append_rows, "duration_ms")),
            "auth": summary(collect_metadata_metric(append_rows, "auth_latency_ms")),
            "client_submit_loop": summary(
                collect_metadata_metric(append_rows, "client_submit_loop_latency_ms")
            ),
        },
        "server_stage_avg_ms": stages,
        "cold_load_server_stage_avg_ms": cold_load_stages,
        "dominant_server_stage": dominant,
    }

    json_path = output_dir / "provchain_profile_summary.json"
    md_path = output_dir / "provchain_profile_summary.md"
    json_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    write_markdown(report, md_path)
    print(f"wrote {json_path}")
    print(f"wrote {md_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
