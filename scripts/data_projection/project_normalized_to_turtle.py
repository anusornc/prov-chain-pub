#!/usr/bin/env python3
"""Project normalized records into simple Turtle transaction payloads."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
NORMALIZATION_DIR = SCRIPT_DIR.parent / "data_normalization"
if str(NORMALIZATION_DIR) not in sys.path:
    sys.path.insert(0, str(NORMALIZATION_DIR))

from common import load_json


RESOURCE_PREFIX = "http://provchain.org/resource/"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, help="Path to normalized JSON records.")
    parser.add_argument("--output", required=True, help="Path to Turtle output.")
    return parser


def sanitize_identifier(value: str | None) -> str:
    text = (value or "unknown").strip()
    return re.sub(r"[^A-Za-z0-9_-]+", "-", text)


def resource_uri(identifier: str | None) -> str:
    return f"<{RESOURCE_PREFIX}{sanitize_identifier(identifier)}>"


def literal(value: str | None) -> str:
    text = (value or "").replace("\\", "\\\\").replace('"', '\\"')
    return f"\"{text}\""


def datetime_literal(value: str | None) -> str | None:
    if not value:
        return None
    return f"\"{value}\"^^xsd:dateTime"


def domain_classes(record: dict) -> str:
    domain = record.get("domain")
    entity_type = record.get("core", {}).get("entity_type")
    if domain == "device" or entity_type == "MedicalDevice":
        return "core:Product , healthcare:MedicalDevice"
    return "core:Product"


def transaction_type(record: dict) -> str:
    record_type = record.get("record_type")
    if record_type == "incident":
        return "Compliance"
    if record_type == "activity":
        return "Processing"
    return "Production"


def emit_record(record: dict) -> str:
    core = record.get("core", {})
    attributes = record.get("attributes", {})
    source = record.get("source_record", {})

    entity_uri = resource_uri(core.get("entity_id") or record.get("record_id"))
    participant_uri = resource_uri(core.get("agent_id") or f"{record.get('record_id')}-participant")
    activity_uri = resource_uri(core.get("activity_id") or f"{record.get('record_id')}-activity")

    published_at = datetime_literal(core.get("published_at"))
    occurred_at = datetime_literal(core.get("occurred_at"))
    tx_type = transaction_type(record)
    participant_name = core.get("agent_id") or attributes.get("name") or "unknown-participant"
    participant_role = core.get("agent_type") or "Participant"

    lines = [
        f"{participant_uri} a core:Participant ;",
        f"    trace:name {literal(participant_name)} ;",
        f"    trace:role {literal(participant_role)} .",
        "",
        f"{entity_uri} a {domain_classes(record)} ;",
        f"    trace:name {literal(attributes.get('name'))} ;",
        f"    trace:participant {literal(core.get('agent_id'))} ;",
        f"    trace:status {literal(attributes.get('status'))} ;",
    ]
    if attributes.get("catalog_code"):
        lines.append(f"    core:hasIdentifier {literal(attributes.get('catalog_code'))} ;")
    if source.get("source_url"):
        lines.append(f"    rdfs:seeAlso {literal(source.get('source_url'))} ;")
    lines[-1] = lines[-1].rstrip(" ;")
    lines[-1] += " ."
    lines.append("")

    lines.extend(
        [
            f"{activity_uri} a core:Transaction , prov:Activity ;",
            f"    trace:transactionType {literal(tx_type)} ;",
            f"    trace:participant {literal(core.get('agent_id'))} ;",
            f"    prov:wasAssociatedWith {participant_uri} ;",
            f"    prov:used {entity_uri} ;",
        ]
    )
    if occurred_at:
        lines.append(f"    prov:startedAtTime {occurred_at} ;")
    if published_at:
        lines.append(f"    trace:timestamp {published_at} ;")
        lines.append(f"    prov:endedAtTime {published_at} ;")
    else:
        lines.append("    trace:timestamp \"2026-03-10T00:00:00Z\"^^xsd:dateTime ;")
    lines[-1] = lines[-1].rstrip(" ;")
    lines[-1] += " ."
    lines.append("")
    return "\n".join(lines)


def main() -> None:
    args = build_parser().parse_args()
    records = load_json(Path(args.input))
    parts = [
        "@prefix prov: <http://www.w3.org/ns/prov#> .",
        "@prefix core: <http://provchain.org/core#> .",
        "@prefix trace: <http://provchain.org/trace#> .",
        "@prefix healthcare: <http://provchain.org/healthcare#> .",
        "@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .",
        "@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .",
        "",
    ]
    for record in records:
        parts.append(emit_record(record))
    Path(args.output).write_text("\n".join(parts).rstrip() + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
