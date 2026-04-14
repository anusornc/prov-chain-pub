#!/usr/bin/env python3
"""Normalize FoodData Central sample records into the shared intermediate schema."""

from __future__ import annotations

import argparse
from pathlib import Path

from common import build_source_record, first_non_empty, iso_date_to_rfc3339, load_json, write_json


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, help="Path to raw FoodData Central JSON.")
    parser.add_argument("--output", required=True, help="Path to normalized JSON output.")
    parser.add_argument("--snapshot-id", required=True, help="Raw snapshot identifier.")
    return parser


def normalize_record(record: dict, snapshot_id: str) -> dict:
    fdc_id = str(record.get("fdcId"))
    description = first_non_empty(record.get("description"), f"food-{fdc_id}")
    brand_owner = first_non_empty(record.get("brandOwner"), "unknown-food-owner")
    entity_id = f"food:fdc:{fdc_id}"
    agent_id = f"agent:brand:{brand_owner.lower().replace(' ', '-').replace('.', '')}"
    category = first_non_empty(
        record.get("foodCategory", {}).get("description") if isinstance(record.get("foodCategory"), dict) else None,
        record.get("foodClass"),
        record.get("dataType"),
        "FoodProduct",
    )
    secondary = []
    gtin = first_non_empty(record.get("gtinUpc"))
    if gtin:
        secondary.append(f"gtin:{gtin}")

    return {
        "schema_version": "1.0",
        "record_id": entity_id,
        "source_id": "fooddata_central",
        "domain": "food",
        "record_type": "entity",
        "core": {
            "entity_id": entity_id,
            "entity_type": "FoodProduct",
            "agent_id": agent_id,
            "agent_type": "Manufacturer",
            "activity_id": None,
            "activity_type": None,
            "occurred_at": None,
            "published_at": iso_date_to_rfc3339(first_non_empty(record.get("publicationDate"))),
        },
        "identifiers": {
            "primary": f"fdc:{fdc_id}",
            "secondary": secondary,
        },
        "attributes": {
            "name": description,
            "status": "active",
            "classification": category,
            "lot_or_batch": None,
            "catalog_code": fdc_id,
        },
        "relationships": [
            {
                "predicate": "manufactured_by",
                "target_id": agent_id,
                "target_type": "Agent",
            }
        ],
        "source_record": build_source_record(
            "FoodData Central",
            "https://fdc.nal.usda.gov/download-datasets/",
            snapshot_id,
        ),
    }


def main() -> None:
    args = build_parser().parse_args()
    payload = load_json(Path(args.input))
    results = []
    if isinstance(payload, dict):
        for key in ("FoundationFoods", "BrandedFoods", "SurveyFoods", "SRLegacyFoods"):
            values = payload.get(key, [])
            if isinstance(values, list):
                results.extend(values)
    elif isinstance(payload, list):
        results = payload
    normalized = [normalize_record(record, args.snapshot_id) for record in results]
    write_json(Path(args.output), normalized)


if __name__ == "__main__":
    main()

