#!/usr/bin/env python3
"""Generate paper-ready architecture figures for the shared-ontology network model."""

from __future__ import annotations

from pathlib import Path

import matplotlib.pyplot as plt
from matplotlib.patches import FancyArrowPatch, FancyBboxPatch


REPO_ROOT = Path(__file__).resolve().parent.parent
OUTPUT_ROOT = (
    REPO_ROOT
    / "docs"
    / "architecture"
    / "figures"
    / "shared_ontology_network_2026-03-11"
)

LAYER_COLORS = {
    "actors": "#F4EAD5",
    "contract": "#DDECF5",
    "runtime": "#E3F2E1",
    "data": "#ECECEC",
}
BOX_COLORS = {
    "actors": "#E2C48D",
    "contract": "#7CB7D8",
    "runtime": "#6AA876",
    "data": "#7B8794",
    "accent": "#1D4E89",
    "secondary": "#E76F51",
}


def configure_style() -> None:
    plt.rcParams.update(
        {
            "font.family": "serif",
            "font.size": 10,
            "figure.dpi": 300,
            "savefig.dpi": 300,
        }
    )


def rounded_box(
    ax: plt.Axes,
    x: float,
    y: float,
    width: float,
    height: float,
    title: str,
    body: str,
    facecolor: str,
    edgecolor: str = "#2F3E46",
    title_size: int = 11,
    body_size: int = 9,
) -> None:
    ax.add_patch(
        FancyBboxPatch(
            (x, y),
            width,
            height,
            boxstyle="round,pad=0.01,rounding_size=1.2",
            linewidth=1.2,
            facecolor=facecolor,
            edgecolor=edgecolor,
        )
    )
    ax.text(
        x + width / 2,
        y + height - 1.4,
        title,
        ha="center",
        va="top",
        fontsize=title_size,
        fontweight="bold",
        color="#122029",
    )
    ax.text(
        x + width / 2,
        y + height / 2 - 0.5,
        body,
        ha="center",
        va="center",
        fontsize=body_size,
        color="#122029",
    )


def layer_band(ax: plt.Axes, y: float, height: float, title: str, color: str) -> None:
    ax.add_patch(
        FancyBboxPatch(
            (2, y),
            96,
            height,
            boxstyle="round,pad=0.015,rounding_size=1.8",
            linewidth=1.0,
            facecolor=color,
            edgecolor="#A0A8AE",
        )
    )
    ax.text(
        4,
        y + height - 1.6,
        title,
        ha="left",
        va="top",
        fontsize=12,
        fontweight="bold",
        color="#122029",
    )


def connect(
    ax: plt.Axes,
    start: tuple[float, float],
    end: tuple[float, float],
    color: str = "#455A64",
    style: str = "-|>",
    linewidth: float = 1.4,
) -> None:
    ax.add_patch(
        FancyArrowPatch(
            start,
            end,
            arrowstyle=style,
            mutation_scale=12,
            linewidth=linewidth,
            color=color,
        )
    )


def save_figure(fig: plt.Figure, stem: str) -> None:
    OUTPUT_ROOT.mkdir(parents=True, exist_ok=True)
    fig.tight_layout()
    fig.savefig(OUTPUT_ROOT / f"{stem}.png", bbox_inches="tight", dpi=300)
    fig.savefig(OUTPUT_ROOT / f"{stem}.svg", bbox_inches="tight")
    plt.close(fig)


def draw_layered_architecture() -> None:
    fig, ax = plt.subplots(figsize=(14, 9))
    ax.set_xlim(0, 100)
    ax.set_ylim(0, 100)
    ax.axis("off")

    layer_band(ax, 77, 19, "Layer 1. Consortium and Organization Actors", LAYER_COLORS["actors"])
    layer_band(ax, 56, 18, "Layer 2. Shared Network Contract", LAYER_COLORS["contract"])
    layer_band(ax, 24, 29, "Layer 3. Node Runtime and Semantic Enforcement", LAYER_COLORS["runtime"])
    layer_band(ax, 4, 17, "Layer 4. Provenance Data, Queries, and Audit Evidence", LAYER_COLORS["data"])

    rounded_box(
        ax,
        7,
        81,
        18,
        10,
        "Consortium Administrator",
        "Defines network profile\nApproves validators\nApproves ontology package",
        BOX_COLORS["actors"],
    )
    rounded_box(
        ax,
        29,
        81,
        18,
        10,
        "Validator Organizations",
        "Run validator nodes\nEnforce network contract\nParticipate in PoA/PBFT",
        BOX_COLORS["actors"],
    )
    rounded_box(
        ax,
        51,
        81,
        18,
        10,
        "Participant Organizations",
        "Map local data\nSubmit traceability events\nQuery provenance",
        BOX_COLORS["actors"],
    )
    rounded_box(
        ax,
        73,
        81,
        18,
        10,
        "Auditor / Regulator",
        "Inspect conformance\nReview immutable history\nRequest audit evidence",
        BOX_COLORS["actors"],
    )

    rounded_box(
        ax,
        10,
        60,
        24,
        10,
        "Network Profile",
        "network_id\nconsensus type\nvalidator set\nblock timing\nvalidation policy",
        BOX_COLORS["contract"],
    )
    rounded_box(
        ax,
        39,
        58.5,
        51,
        13,
        "Shared Ontology Package",
        "PROV-O core\nNetwork-specific ontology extensions\nSHACL shapes\nPackage id/version/hash\nOptional GS1/EPCIS mappings",
        BOX_COLORS["contract"],
    )
    ax.text(
        64.5,
        57.1,
        "One permissioned network = one shared ontology package",
        ha="center",
        va="top",
        fontsize=10,
        color=BOX_COLORS["accent"],
        fontweight="bold",
    )

    ax.add_patch(
        FancyBboxPatch(
            (8, 28),
            84,
            21,
            boxstyle="round,pad=0.015,rounding_size=1.5",
            linewidth=1.2,
            facecolor="#F8FFF7",
            edgecolor="#5B8C5A",
        )
    )
    ax.text(
        10,
        47.3,
        "Representative validator / participant node",
        ha="left",
        va="top",
        fontsize=11,
        fontweight="bold",
        color="#204128",
    )

    rounded_box(
        ax,
        11,
        37,
        16,
        8,
        "API and Event Mapping",
        "REST / WebSocket\nLocal data mapping\nPayload preparation",
        "#A9D6A2",
    )
    rounded_box(
        ax,
        31,
        37,
        16,
        8,
        "Peer Discovery + Consensus",
        "Semantic contract checks\nPoA / PBFT\nBlock agreement",
        "#A9D6A2",
    )
    rounded_box(
        ax,
        51,
        37,
        18,
        8,
        "Ontology Manager",
        "Package loading\nHash checks\nValidation orchestration",
        "#A9D6A2",
    )
    rounded_box(
        ax,
        73,
        37,
        16,
        8,
        "Query and Audit API",
        "SPARQL queries\nTrace reconstruction\nAudit responses",
        "#A9D6A2",
    )
    rounded_box(
        ax,
        18,
        28.5,
        18,
        7,
        "Blockchain Core",
        "add_block\nTransaction admission\nChain state",
        "#D7F0D2",
    )
    rounded_box(
        ax,
        40,
        28.5,
        22,
        7,
        "SHACL + SPACL Reasoning",
        "Constraint checking\nSubclass support\nViolation summaries",
        "#D7F0D2",
    )
    rounded_box(
        ax,
        66,
        28.5,
        18,
        7,
        "RDF Store + Persistence",
        "Oxigraph\nWAL / persistent store\nNamed graphs",
        "#D7F0D2",
    )

    rounded_box(
        ax,
        12,
        8,
        22,
        8,
        "Immutable RDF Blocks",
        "Committed graph payloads\nDeterministic provenance records",
        BOX_COLORS["data"],
        edgecolor="#54606B",
    )
    rounded_box(
        ax,
        39,
        8,
        22,
        8,
        "Provenance Graph",
        "Traceability relations\nSPARQL / semantic queries\nCross-organization history",
        BOX_COLORS["data"],
        edgecolor="#54606B",
    )
    rounded_box(
        ax,
        66,
        8,
        22,
        8,
        "Audit Evidence",
        "Conformance reports\nLineage evidence\nRegulatory review outputs",
        BOX_COLORS["data"],
        edgecolor="#54606B",
    )

    connect(ax, (16, 81), (22, 70), color=BOX_COLORS["accent"])
    connect(ax, (38, 81), (22, 70), color=BOX_COLORS["accent"])
    connect(ax, (60, 81), (64.5, 71.5), color=BOX_COLORS["accent"])
    connect(ax, (82, 81), (80, 45), color=BOX_COLORS["secondary"])

    connect(ax, (22, 60), (39, 45), color=BOX_COLORS["accent"])
    connect(ax, (64.5, 58.5), (60, 45), color=BOX_COLORS["accent"])
    connect(ax, (64.5, 58.5), (51, 32.5), color=BOX_COLORS["accent"])

    connect(ax, (27, 41), (31, 41), color="#3C6E71")
    connect(ax, (47, 41), (51, 41), color="#3C6E71")
    connect(ax, (60, 37), (51, 35.5), color="#3C6E71")
    connect(ax, (36, 32), (40, 32), color="#3C6E71")
    connect(ax, (62, 32), (66, 32), color="#3C6E71")
    connect(ax, (75, 37), (75, 35.5), color="#3C6E71")
    connect(ax, (27, 41), (27, 32), color="#3C6E71")
    connect(ax, (80, 37), (75, 16), color="#3C6E71")
    connect(ax, (75, 28.5), (75, 16), color="#3C6E71")
    connect(ax, (27, 28.5), (23, 16), color="#3C6E71")
    connect(ax, (51, 28.5), (50, 16), color="#3C6E71")

    ax.text(
        50,
        1.5,
        "Production semantic path: src/ontology/* + SPACL. Legacy src/semantic/* modules are not part of this architecture figure.",
        ha="center",
        va="bottom",
        fontsize=9,
        color="#39434D",
    )

    save_figure(fig, "shared_ontology_layered_architecture")


def draw_admission_pipeline() -> None:
    fig, ax = plt.subplots(figsize=(14, 4.8))
    ax.set_xlim(0, 100)
    ax.set_ylim(0, 30)
    ax.axis("off")

    steps = [
        (3, "Local Source Data", "ERP / MES / device\nlogs / batch records", "#F4EAD5"),
        (18, "Event Mapping", "PROV-O entities\nactivities / agents\nrecord normalization", "#F4EAD5"),
        (33, "Shared Ontology Contract", "PROV-O core\nnetwork extension\nSHACL + package hash", "#DDECF5"),
        (52, "Semantic Admission", "OntologyManager\nSHACL validation\nSPACL reasoning", "#E3F2E1"),
        (69, "Consensus + Commit", "Blockchain Core\nPoA / PBFT\nimmutable block commit", "#E3F2E1"),
        (86, "Traceability Output", "RDF provenance graph\nSPARQL query\naudit evidence", "#ECECEC"),
    ]

    for x, title, body, color in steps:
        rounded_box(ax, x, 11, 11, 9, title, body, color)

    for current, nxt in zip(steps, steps[1:]):
        connect(ax, (current[0] + 11, 15.5), (nxt[0], 15.5), color=BOX_COLORS["accent"])

    rounded_box(
        ax,
        34,
        22,
        20,
        5,
        "Pre-join Network Checks",
        "network profile match\nontology package match\nsemantic contract compatibility",
        "#E9F3FB",
        title_size=10,
        body_size=8,
    )
    connect(ax, (44, 22), (44, 20), color=BOX_COLORS["secondary"])

    ax.text(
        50,
        4.5,
        "Optional GS1/EPCIS mappings are consumed inside the shared ontology contract layer when a network requires them.",
        ha="center",
        va="center",
        fontsize=10,
        color="#39434D",
    )

    save_figure(fig, "shared_ontology_admission_pipeline")


def main() -> None:
    configure_style()
    draw_layered_architecture()
    draw_admission_pipeline()
    print(f"Generated architecture figures in {OUTPUT_ROOT}")


if __name__ == "__main__":
    main()
