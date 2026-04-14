#!/usr/bin/env python3
"""Normalize openFDA Drug NDC snapshot records into the shared intermediate schema."""

from __future__ import annotations

import argparse
from pathlib import Path

from common import as_list, build_source_record, first_non_empty, load_json, write_json, yyyymmdd_to_rfc3339


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, help="Path to raw openFDA Drug NDC JSON.")
    parser.add_argument("--output", required=True, help="Path to normalized JSON output.")
    parser.add_argument("--snapshot-id", required=True, help="Raw snapshot identifier.")
    return parser


def normalize_record(record: dict, snapshot_id: str) -> dict:
    product_ndc = first_non_empty(record.get("product_ndc"), record.get("openfda", {}).get("product_ndc"))
    brand_name = first_non_empty(record.get("brand_name"), record.get("generic_name"), "unknown-drug")
    labeler_name = first_non_empty(
        record.get("labeler_name"),
        record.get("openfda", {}).get("manufacturer_name"),
        "unknown-labeler",
    )
    entity_id = f"drug:ndc:{product_ndc or brand_name.lower().replace(' ', '-')}"
    agent_id = f"agent:labeler:{labeler_name.lower().replace(' ', '-').replace('.', '')}"

    return {
        "schema_version": "1.0",
        "record_id": entity_id,
        "source_id": "openfda_drug_ndc",
        "domain": "pharma",
        "record_type": "entity",
        "core": {
            "entity_id": entity_id,
            "entity_type": "DrugProduct",
            "agent_id": agent_id,
            "agent_type": "Manufacturer",
            "activity_id": None,
            "activity_type": None,
            "occurred_at": yyyymmdd_to_rfc3339(record.get("marketing_start_date")),
            "published_at": yyyymmdd_to_rfc3339(record.get("listing_expiration_date")),
        },
        "identifiers": {
            "primary": f"ndc:{product_ndc}" if product_ndc else brand_name,
            "secondary": [f"route:{route}" for route in as_list(record.get("route"))],
        },
        "attributes": {
            "name": brand_name,
            "status": first_non_empty(record.get("finished"), "listed"),
            "classification": first_non_empty(record.get("product_type"), "DrugProduct"),
            "lot_or_batch": None,
            "catalog_code": product_ndc,
        },
        "relationships": [
            {
                "predicate": "manufactured_by",
                "target_id": agent_id,
                "target_type": "Agent",
            }
        ],
        "source_record": build_source_record(
            "openFDA Drug NDC",
            "https://open.fda.gov/apis/drug/ndc/",
            snapshot_id,
        ),
    }


def main() -> None:
    args = build_parser().parse_args()
    payload = load_json(Path(args.input))
    results = payload.get("results", []) if isinstance(payload, dict) else []
    normalized = [normalize_record(record, args.snapshot_id) for record in results]
    write_json(Path(args.output), normalized)


if __name__ == "__main__":
    main()

