#!/usr/bin/env python3
"""Export publication-ready summary tables from Criterion domain dataset benchmarks."""

from __future__ import annotations

import csv
import json
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
CRITERION_ROOT = REPO_ROOT / "target" / "criterion" / "domain_dataset_admission"
OUTPUT_ROOT = REPO_ROOT / "docs" / "benchmarking" / "data"
CSV_PATH = OUTPUT_ROOT / "domain_dataset_admission_summary_2026-03-11.csv"
PLOT_CSV_PATH = OUTPUT_ROOT / "domain_dataset_admission_plot_data_2026-03-11.csv"
MARKDOWN_PATH = (
    REPO_ROOT / "docs" / "benchmarking" / "DOMAIN_DATASET_ADMISSION_SUMMARY_TABLES_2026-03-11.md"
)


@dataclass
class Metric:
    benchmark: str
    category: str
    package: str
    workload: str
    scale_factor: str
    emitted_records: str
    mean_us: float
    ci_low_us: float
    ci_high_us: float


def parse_package(name: str) -> str:
    if name.startswith("uht_product"):
        return "uht"
    if name.startswith("uht_epcis"):
        return "uht_epcis"
    if name.startswith("healthcare_device"):
        return "healthcare_device"
    if name.startswith("pharma_storage"):
        return "pharma_storage"
    return "cross_package"


def parse_category(name: str) -> str:
    if name.startswith("cross_package"):
        return "cross_package"
    if "_event_setup_only" in name:
        return "setup_only"
    if "_event_add_block_only" in name:
        return "single_record_admission"
    if "_event_add_block" in name:
        return "end_to_end"
    if "_batch_x" in name:
        return "batch_scaling"
    if "_batch_add_block_only" in name:
        return "batch_admission"
    return "other"


def parse_workload(name: str) -> str:
    if name.startswith("cross_package_round_robin_single_record"):
        return "round_robin_single_record"
    if name.startswith("cross_package_round_robin_batch"):
        return "round_robin_batch"
    if "_batch_" in name or name.endswith("_batch_add_block_only"):
        return "batch"
    if "_event_" in name:
        return "single_record"
    return "other"


def parse_scale_factor(name: str) -> str:
    if "_batch_x2_" in name:
        return "x2"
    if "_batch_x4_" in name:
        return "x4"
    if "_batch_" in name:
        return "x1"
    return "x1"


def emitted_records(name: str) -> str:
    if name.startswith("uht_product_batch"):
        base = 3
    elif name.startswith("uht_epcis_batch"):
        base = 3
    elif name.startswith("healthcare_device_batch"):
        base = 2
    elif name.startswith("pharma_storage_batch"):
        base = 2
    elif name.startswith("cross_package_round_robin_single_record"):
        return "4 admissions"
    elif name.startswith("cross_package_round_robin_batch"):
        return "10 admissions"
    else:
        return "1"

    factor = 1
    if "_batch_x2_" in name:
        factor = 2
    elif "_batch_x4_" in name:
        factor = 4
    return str(base * factor)


def load_metric(path: Path) -> Metric:
    benchmark = path.parent.parent.name
    with path.open(encoding="utf-8") as handle:
        payload = json.load(handle)
    mean = payload["mean"]
    return Metric(
        benchmark=benchmark,
        category=parse_category(benchmark),
        package=parse_package(benchmark),
        workload=parse_workload(benchmark),
        scale_factor=parse_scale_factor(benchmark),
        emitted_records=emitted_records(benchmark),
        mean_us=mean["point_estimate"] / 1000.0,
        ci_low_us=mean["confidence_interval"]["lower_bound"] / 1000.0,
        ci_high_us=mean["confidence_interval"]["upper_bound"] / 1000.0,
    )


def fmt_us(value: float) -> str:
    if value >= 1000.0:
        return f"{value / 1000.0:.3f} ms"
    return f"{value:.2f} us"


def category_title(category: str) -> str:
    return {
        "end_to_end": "End-to-End Admission",
        "setup_only": "Setup Only",
        "single_record_admission": "Single-Record Admission Only",
        "batch_admission": "Batch Admission Only",
        "batch_scaling": "Batch Scaling Curves",
        "cross_package": "Cross-Package Round-Robin Workloads",
    }.get(category, category.replace("_", " ").title())


def write_csv(metrics: list[Metric]) -> None:
    OUTPUT_ROOT.mkdir(parents=True, exist_ok=True)
    with CSV_PATH.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle)
        writer.writerow(
            [
                "benchmark",
                "category",
                "package",
                "workload",
                "scale_factor",
                "emitted_records",
                "mean_us",
                "ci_low_us",
                "ci_high_us",
            ]
        )
        for metric in metrics:
            writer.writerow(
                [
                    metric.benchmark,
                    metric.category,
                    metric.package,
                    metric.workload,
                    metric.scale_factor,
                    metric.emitted_records,
                    f"{metric.mean_us:.6f}",
                    f"{metric.ci_low_us:.6f}",
                    f"{metric.ci_high_us:.6f}",
                ]
            )


def write_plot_csv(metrics: list[Metric]) -> None:
    with PLOT_CSV_PATH.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle)
        writer.writerow(["package", "series", "scale_factor", "mean_us"])
        for metric in metrics:
            if metric.category not in {"single_record_admission", "batch_admission", "batch_scaling"}:
                continue
            writer.writerow(
                [
                    metric.package,
                    metric.category,
                    metric.scale_factor,
                    f"{metric.mean_us:.6f}",
                ]
            )


def write_markdown(metrics: list[Metric]) -> None:
    sections = [
        "end_to_end",
        "setup_only",
        "single_record_admission",
        "batch_admission",
        "batch_scaling",
        "cross_package",
    ]
    lines: list[str] = [
        "# Domain Dataset Admission Summary Tables - 2026-03-11",
        "",
        "This file converts the latest Criterion results into paper-ready summary tables.",
        "",
        "Source benchmark:",
        "",
        "- `target/criterion/domain_dataset_admission/*/new/estimates.json`",
        "",
        "Exported artifacts:",
        "",
        "- `docs/benchmarking/data/domain_dataset_admission_summary_2026-03-11.csv`",
        "- `docs/benchmarking/data/domain_dataset_admission_plot_data_2026-03-11.csv`",
        "",
    ]

    for category in sections:
        rows = [metric for metric in metrics if metric.category == category]
        if not rows:
            continue
        lines.extend(
            [
                f"## {category_title(category)}",
                "",
                "| Benchmark | Package | Emitted Records | Mean | 95% CI |",
                "|---|---|---:|---:|---:|",
            ]
        )
        for metric in rows:
            lines.append(
                "| "
                + " | ".join(
                    [
                        metric.benchmark,
                        metric.package,
                        metric.emitted_records,
                        fmt_us(metric.mean_us),
                        f"{fmt_us(metric.ci_low_us)} - {fmt_us(metric.ci_high_us)}",
                    ]
                )
                + " |"
            )
        lines.append("")

    lines.extend(
        [
            "## Figure Notes",
            "",
            "- Use `domain_dataset_admission_plot_data_2026-03-11.csv` for line plots or grouped bar charts.",
            "- Recommended figures:",
            "  - single-record admission by package",
            "  - batch admission vs `x2/x4` scaling by package",
            "  - cross-package round-robin single-record vs batch workload",
            "",
        ]
    )

    MARKDOWN_PATH.write_text("\n".join(lines), encoding="utf-8")


def main() -> None:
    metrics = [
        load_metric(path)
        for path in sorted(CRITERION_ROOT.glob("*/new/estimates.json"))
    ]
    write_csv(metrics)
    write_plot_csv(metrics)
    write_markdown(metrics)


if __name__ == "__main__":
    main()
