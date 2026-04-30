#!/usr/bin/env python3
"""Summarize benchmark campaign results across epochs."""

from __future__ import annotations

import argparse
import csv
import json
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path
from statistics import mean, median, pstdev


def display_value(value) -> str:
    if value is None:
        return ""
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, list):
        return ", ".join(str(item) for item in value)
    return str(value)


def consistent(values: list[str]) -> str:
    normalized = [value for value in values if value != ""]
    if not normalized:
        return ""
    first = normalized[0]
    if all(value == first for value in normalized):
        return first
    return "mixed"


def percentile(values: list[float], rank: float) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    if len(ordered) == 1:
        return ordered[0]
    position = (len(ordered) - 1) * rank
    lower = int(position)
    upper = min(lower + 1, len(ordered) - 1)
    fraction = position - lower
    return ordered[lower] + (ordered[upper] - ordered[lower]) * fraction


def load_epoch_statuses(campaign_dir: Path) -> dict[str, str]:
    statuses: dict[str, str] = {}
    for manifest_path in sorted(campaign_dir.glob("epochs/epoch-*/epoch_manifest.json")):
        with manifest_path.open("r", encoding="utf-8") as handle:
            manifest = json.load(handle)
        statuses[manifest.get("run_id", "")] = manifest.get("status", "unknown")
    return statuses


def iter_result_rows(campaign_dir: Path):
    for csv_path in sorted(campaign_dir.glob("epochs/epoch-*/runs/*/benchmark_results.csv")):
        run_id = csv_path.parent.name
        epoch_id = csv_path.parents[2].name
        with csv_path.open("r", encoding="utf-8", newline="") as handle:
            reader = csv.DictReader(handle)
            for row in reader:
                row["_run_id"] = run_id
                row["_epoch_id"] = epoch_id
                yield row


def metadata_value(row: dict, key: str) -> str:
    try:
        metadata = json.loads(row.get("metadata_json", "{}") or "{}")
    except json.JSONDecodeError:
        metadata = {}
    return display_value(metadata.get(key))


def summarize(campaign_dir: Path) -> tuple[list[dict], list[dict]]:
    epoch_statuses = load_epoch_statuses(campaign_dir)
    groups: dict[tuple[str, str, str, str, str, str, str, str], list[dict]] = defaultdict(list)

    for row in iter_result_rows(campaign_dir):
        key = (
            row.get("family", ""),
            row.get("scenario", ""),
            row.get("test_name", ""),
            row.get("system", ""),
            row.get("fairness_label", ""),
            row.get("capability_path", ""),
            row.get("metric_type", ""),
            row.get("unit", ""),
        )
        groups[key].append(row)

    summaries: list[dict] = []
    for (
        family,
        scenario,
        test_name,
        system,
        fairness_label,
        capability_path,
        metric_type,
        unit,
    ), rows in sorted(groups.items()):
        durations: list[float] = []
        ops: list[float] = []
        success_count = 0
        failed_count = 0
        run_ids = set()
        epoch_ids = set()

        for row in rows:
            run_ids.add(row.get("_run_id", ""))
            epoch_ids.add(row.get("_epoch_id", ""))
            success = row.get("success", "").lower() == "true"
            if success:
                success_count += 1
                try:
                    durations.append(float(row.get("duration_ms", "0") or 0))
                except ValueError:
                    pass
                try:
                    ops.append(float(row.get("operations_per_second", "0") or 0))
                except ValueError:
                    pass
            else:
                failed_count += 1

        total = success_count + failed_count
        success_rate = (success_count / total * 100.0) if total else 0.0
        summary = {
            "family": family,
            "scenario": scenario,
            "test_name": test_name,
            "system": system,
            "fairness_label": fairness_label,
            "capability_path": capability_path,
            "metric_type": metric_type,
            "unit": unit,
            "native_semantic_support": consistent(
                [metadata_value(row, "native_semantic_support") for row in rows]
            ),
            "external_semantic_stages": consistent(
                [metadata_value(row, "external_semantic_stages") for row in rows]
            ),
            "explanation_support": consistent(
                [metadata_value(row, "explanation_support") for row in rows]
            ),
            "epochs_observed": len(epoch_ids),
            "runs_observed": len(run_ids),
            "samples": total,
            "success_count": success_count,
            "failed_count": failed_count,
            "success_rate": success_rate,
            "mean_ms": mean(durations) if durations else 0.0,
            "median_ms": median(durations) if durations else 0.0,
            "p95_ms": percentile(durations, 0.95),
            "p99_ms": percentile(durations, 0.99),
            "stddev_ms": pstdev(durations) if len(durations) > 1 else 0.0,
            "mean_ops_per_sec": mean(ops) if ops else 0.0,
            "epoch_statuses": {
                run_id: epoch_statuses.get(run_id, "unknown")
                for run_id in sorted(run_ids)
                if run_id
            },
        }
        summaries.append(summary)

    rows = list(iter_result_rows(campaign_dir))
    return summaries, rows


def write_csv(path: Path, rows: list[dict]) -> None:
    fields = [
        "family",
        "scenario",
        "test_name",
        "system",
        "fairness_label",
        "capability_path",
        "metric_type",
        "unit",
        "native_semantic_support",
        "external_semantic_stages",
        "explanation_support",
        "epochs_observed",
        "runs_observed",
        "samples",
        "success_count",
        "failed_count",
        "success_rate",
        "mean_ms",
        "median_ms",
        "p95_ms",
        "p99_ms",
        "stddev_ms",
        "mean_ops_per_sec",
    ]
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fields)
        writer.writeheader()
        for row in rows:
            writer.writerow({field: row.get(field, "") for field in fields})


def write_markdown(path: Path, campaign_dir: Path, summaries: list[dict]) -> None:
    generated_at = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    with path.open("w", encoding="utf-8") as handle:
        handle.write("# Campaign Aggregate Summary\n\n")
        handle.write(f"- Campaign: `{campaign_dir.name}`\n")
        handle.write(f"- Generated at: `{generated_at}`\n\n")
        handle.write("| Family | Test | System | Path | Metric | Unit | Samples | Success Rate | Mean | p95 | p99 |\n")
        handle.write("|---|---|---|---|---|---|---:|---:|---:|---:|---:|\n")
        for row in summaries:
            handle.write(
                "| `{family}` | `{test}` | `{system}` | `{path}` | `{metric}` | `{unit}` | {samples} | {success:.2f}% | {mean_ms:.3f} | {p95_ms:.3f} | {p99_ms:.3f} |\n".format(
                    family=row["family"],
                    test=row["test_name"],
                    system=row["system"],
                    path=row["capability_path"],
                    metric=row["metric_type"],
                    unit=row["unit"],
                    samples=row["samples"],
                    success=row["success_rate"],
                    mean_ms=row["mean_ms"],
                    p95_ms=row["p95_ms"],
                    p99_ms=row["p99_ms"],
                )
            )
        semantic_rows = [row for row in summaries if row.get("family") == "semantic"]
        if semantic_rows:
            handle.write("\n## Semantic Capability Notes\n\n")
            handle.write("| System | Native Semantic Support | External Semantic Stages | Explanation Support |\n")
            handle.write("|---|---:|---|---:|\n")
            seen = set()
            for row in semantic_rows:
                key = (row["system"], row["capability_path"])
                if key in seen:
                    continue
                seen.add(key)
                handle.write(
                    "| `{system}` | `{native}` | `{stages}` | `{explain}` |\n".format(
                        system=row["system"],
                        native=row.get("native_semantic_support", ""),
                        stages=row.get("external_semantic_stages", ""),
                        explain=row.get("explanation_support", ""),
                    )
                )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("campaign_dir", type=Path, help="Path to benchmark-toolkit/results/campaigns/<campaign_id>")
    args = parser.parse_args()

    campaign_dir = args.campaign_dir.resolve()
    if not campaign_dir.exists():
        raise SystemExit(f"campaign directory does not exist: {campaign_dir}")

    summaries, raw_rows = summarize(campaign_dir)
    output = {
        "campaign_id": campaign_dir.name,
        "generated_at_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "raw_row_count": len(raw_rows),
        "summary_row_count": len(summaries),
        "summaries": summaries,
    }

    with (campaign_dir / "campaign_results.json").open("w", encoding="utf-8") as handle:
        json.dump(output, handle, indent=2)
        handle.write("\n")

    write_csv(campaign_dir / "campaign_results.csv", summaries)
    write_markdown(campaign_dir / "campaign_aggregate_summary.md", campaign_dir, summaries)

    print(f"wrote campaign summary: {campaign_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
