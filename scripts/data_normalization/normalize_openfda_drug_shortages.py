#!/usr/bin/env python3
"""Normalize openFDA Drug Shortages snapshot records into the shared intermediate schema."""

from __future__ import annotations

import argparse
from pathlib import Path

from common import build_source_record, first_non_empty, load_json, write_json, yyyymmdd_to_rfc3339


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, help="Path to raw openFDA Drug Shortages JSON.")
    parser.add_argument("--output", required=True, help="Path to normalized JSON output.")
    parser.add_argument("--snapshot-id", required=True, help="Raw snapshot identifier.")
    return parser


def normalize_record(record: dict, snapshot_id: str, index: int) -> dict:
    package_ndc = first_non_empty(record.get("package_ndc"))
    proprietary_name = first_non_empty(record.get("proprietary_name"), record.get("generic_name"), f"shortage-{index}")
    company_name = first_non_empty(record.get("company_name"), "unknown-company")
    activity_id = f"incident:drug-shortage:{package_ndc or index}"
    entity_id = f"drug:ndc:{package_ndc}" if package_ndc else f"drug:shortage:{index}"
    agent_id = f"agent:labeler:{company_name.lower().replace(' ', '-').replace('.', '')}"

    return {
        "schema_version": "1.0",
        "record_id": activity_id,
        "source_id": "openfda_drug_shortages",
        "domain": "pharma",
        "record_type": "incident",
        "core": {
            "entity_id": entity_id,
            "entity_type": "DrugProduct",
            "agent_id": agent_id,
            "agent_type": "Manufacturer",
            "activity_id": activity_id,
            "activity_type": "DrugShortage",
            "occurred_at": yyyymmdd_to_rfc3339(record.get("initial_posting_date")),
            "published_at": yyyymmdd_to_rfc3339(record.get("update_date")),
        },
        "identifiers": {
            "primary": f"ndc:{package_ndc}" if package_ndc else proprietary_name,
            "secondary": [activity_id],
        },
        "attributes": {
            "name": proprietary_name,
            "status": first_non_empty(record.get("status"), "unknown"),
            "classification": first_non_empty(record.get("dosage_form"), "DrugShortage"),
            "lot_or_batch": None,
            "catalog_code": package_ndc,
        },
        "relationships": [
            {
                "predicate": "affects_product",
                "target_id": entity_id,
                "target_type": "Entity",
            }
        ],
        "source_record": build_source_record(
            "openFDA Drug Shortages",
            "https://open.fda.gov/apis/drug/drugshortages/",
            snapshot_id,
        ),
    }


def main() -> None:
    args = build_parser().parse_args()
    payload = load_json(Path(args.input))
    results = payload.get("results", []) if isinstance(payload, dict) else []
    normalized = [normalize_record(record, args.snapshot_id, index) for index, record in enumerate(results, start=1)]
    write_json(Path(args.output), normalized)


if __name__ == "__main__":
    main()

