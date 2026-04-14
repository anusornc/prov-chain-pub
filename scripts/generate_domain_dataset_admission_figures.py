#!/usr/bin/env python3
"""Generate publication-ready figures for domain dataset admission benchmarks."""

from __future__ import annotations

from pathlib import Path

import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns


REPO_ROOT = Path(__file__).resolve().parent.parent
SUMMARY_CSV = (
    REPO_ROOT
    / "docs"
    / "benchmarking"
    / "data"
    / "domain_dataset_admission_summary_2026-03-11.csv"
)
OUTPUT_ROOT = (
    REPO_ROOT
    / "docs"
    / "benchmarking"
    / "figures"
    / "domain_dataset_admission_2026-03-11"
)

PACKAGE_ORDER = [
    "healthcare_device",
    "pharma_storage",
    "uht",
    "uht_epcis",
]
PACKAGE_LABELS = {
    "healthcare_device": "Healthcare Device",
    "pharma_storage": "Pharma Storage",
    "uht": "UHT",
    "uht_epcis": "UHT + GS1/EPCIS",
    "cross_package": "Cross-Package",
}
PACKAGE_COLORS = {
    "healthcare_device": "#2A9D8F",
    "pharma_storage": "#E76F51",
    "uht": "#8C564B",
    "uht_epcis": "#1D4E89",
    "cross_package": "#264653",
}
CATEGORY_COLORS = {
    "single_record_admission": "#1D4E89",
    "end_to_end": "#E76F51",
    "setup_only": "#6C757D",
}


def configure_style() -> None:
    sns.set_theme(style="whitegrid")
    plt.rcParams.update(
        {
            "font.family": "serif",
            "font.size": 10,
            "axes.labelsize": 11,
            "axes.titlesize": 12,
            "legend.fontsize": 9,
            "figure.dpi": 300,
            "savefig.dpi": 300,
            "axes.linewidth": 0.9,
            "grid.alpha": 0.25,
        }
    )


def load_summary() -> pd.DataFrame:
    frame = pd.read_csv(SUMMARY_CSV)
    frame["mean_ms"] = frame["mean_us"] / 1000.0
    frame["ci_low_ms"] = frame["ci_low_us"] / 1000.0
    frame["ci_high_ms"] = frame["ci_high_us"] / 1000.0
    frame["ci_half_ms"] = (frame["ci_high_ms"] - frame["ci_low_ms"]) / 2.0
    frame["package_label"] = frame["package"].map(PACKAGE_LABELS)
    frame["emitted_count"] = frame["emitted_records"].apply(parse_emitted_count)
    return frame


def parse_emitted_count(value: str) -> int:
    token = str(value).split()[0]
    try:
        return int(token)
    except ValueError:
        return 0


def save_figure(fig: plt.Figure, stem: str) -> None:
    OUTPUT_ROOT.mkdir(parents=True, exist_ok=True)
    png_path = OUTPUT_ROOT / f"{stem}.png"
    svg_path = OUTPUT_ROOT / f"{stem}.svg"
    fig.tight_layout()
    fig.savefig(png_path, bbox_inches="tight", dpi=300)
    fig.savefig(svg_path, bbox_inches="tight")
    plt.close(fig)


def plot_single_record_breakdown(frame: pd.DataFrame) -> None:
    compare = frame[
        frame["category"].isin(["single_record_admission", "end_to_end"])
    ].copy()
    compare["category_label"] = compare["category"].map(
        {
            "single_record_admission": "Add-block only",
            "end_to_end": "Setup + add-block",
        }
    )
    compare["package_rank"] = compare["package"].map(
        {name: idx for idx, name in enumerate(PACKAGE_ORDER)}
    )
    compare = compare.sort_values(["package_rank", "category"])

    setup = frame[frame["category"] == "setup_only"].copy()
    setup["package_rank"] = setup["package"].map(
        {name: idx for idx, name in enumerate(PACKAGE_ORDER)}
    )
    setup = setup.sort_values("package_rank")

    fig, axes = plt.subplots(1, 2, figsize=(12, 5), width_ratios=[1.6, 1.0])

    x_labels = [PACKAGE_LABELS[name] for name in PACKAGE_ORDER]
    x_positions = range(len(PACKAGE_ORDER))
    width = 0.34
    category_order = ["single_record_admission", "end_to_end"]
    offsets = [-width / 2, width / 2]

    for offset, category in zip(offsets, category_order):
        subset = compare[compare["category"] == category].set_index("package")
        means = [subset.loc[name, "mean_ms"] for name in PACKAGE_ORDER]
        errors = [subset.loc[name, "ci_half_ms"] for name in PACKAGE_ORDER]
        axes[0].bar(
            [position + offset for position in x_positions],
            means,
            width=width,
            yerr=errors,
            capsize=4,
            color=CATEGORY_COLORS[category],
            label="Add-block only" if category == "single_record_admission" else "Setup + add-block",
        )

    axes[0].set_xticks(list(x_positions))
    axes[0].set_xticklabels(x_labels, rotation=15, ha="right")
    axes[0].set_ylabel("Latency (ms)")
    axes[0].set_title("Single-record ontology admission")
    axes[0].legend(frameon=True, loc="upper left")

    setup_positions = range(len(setup))
    setup_colors = [PACKAGE_COLORS[name] for name in setup["package"]]
    axes[1].bar(
        list(setup_positions),
        setup["mean_ms"],
        yerr=setup["ci_half_ms"],
        capsize=4,
        color=setup_colors,
    )
    axes[1].set_xticks(list(setup_positions))
    axes[1].set_xticklabels(setup["package_label"], rotation=15, ha="right")
    axes[1].set_ylabel("Latency (ms)")
    axes[1].set_title("Blockchain setup only")

    fig.suptitle(
        "Figure package A: single-record path breakdown with 95% confidence intervals",
        y=1.03,
        fontsize=13,
    )
    save_figure(fig, "single_record_path_breakdown")


def plot_batch_scaling(frame: pd.DataFrame) -> None:
    subset = frame[
        frame["category"].isin(["batch_admission", "batch_scaling"])
        & frame["package"].isin(PACKAGE_ORDER)
    ].copy()
    subset["package_rank"] = subset["package"].map(
        {name: idx for idx, name in enumerate(PACKAGE_ORDER)}
    )
    subset = subset.sort_values(["package_rank", "emitted_count"])

    fig, ax = plt.subplots(figsize=(10, 6))

    for package in PACKAGE_ORDER:
        package_rows = subset[subset["package"] == package]
        ax.errorbar(
            package_rows["emitted_count"],
            package_rows["mean_ms"],
            yerr=package_rows["ci_half_ms"],
            color=PACKAGE_COLORS[package],
            marker="o",
            linewidth=2.0,
            capsize=4,
            label=PACKAGE_LABELS[package],
        )

    ax.set_xlabel("Emitted records in admission payload")
    ax.set_ylabel("Latency (ms)")
    ax.set_title("Figure package B: batch-size scaling for ontology-backed add_block admission")
    ax.set_xticks(sorted(subset["emitted_count"].unique()))
    ax.legend(frameon=True, ncol=2)
    save_figure(fig, "batch_scaling_curves")


def plot_cross_package_workloads(frame: pd.DataFrame) -> None:
    subset = frame[frame["category"] == "cross_package"].copy()
    subset = subset.sort_values("emitted_count")
    subset["per_admission_us"] = subset["mean_us"] / subset["emitted_count"]
    subset["ci_half_total_ms"] = subset["ci_half_ms"]
    subset["ci_half_per_admission_us"] = (
        (subset["ci_high_us"] - subset["ci_low_us"]) / 2.0
    ) / subset["emitted_count"]
    subset["workload_label"] = subset["benchmark"].map(
        {
            "cross_package_round_robin_single_record_add_block_only": "Round-robin single-record",
            "cross_package_round_robin_batch_add_block_only": "Round-robin batch",
        }
    )

    fig, axes = plt.subplots(1, 2, figsize=(11, 4.8))
    positions = range(len(subset))
    colors = [PACKAGE_COLORS["cross_package"], "#6C757D"]

    axes[0].bar(
        list(positions),
        subset["mean_ms"],
        yerr=subset["ci_half_total_ms"],
        capsize=4,
        color=colors,
    )
    axes[0].set_xticks(list(positions))
    axes[0].set_xticklabels(subset["workload_label"], rotation=15, ha="right")
    axes[0].set_ylabel("Total latency (ms)")
    axes[0].set_title("Cross-package workload latency")

    axes[1].bar(
        list(positions),
        subset["per_admission_us"],
        yerr=subset["ci_half_per_admission_us"],
        capsize=4,
        color=colors,
    )
    axes[1].set_xticks(list(positions))
    axes[1].set_xticklabels(subset["workload_label"], rotation=15, ha="right")
    axes[1].set_ylabel("Latency per admission (us)")
    axes[1].set_title("Normalized latency per admission")

    fig.suptitle(
        "Figure package C: cross-package round-robin workloads with 95% confidence intervals",
        y=1.03,
        fontsize=13,
    )
    save_figure(fig, "cross_package_workloads")


def main() -> None:
    configure_style()
    frame = load_summary()
    plot_single_record_breakdown(frame)
    plot_batch_scaling(frame)
    plot_cross_package_workloads(frame)
    print(f"Generated figures in {OUTPUT_ROOT}")


if __name__ == "__main__":
    main()
