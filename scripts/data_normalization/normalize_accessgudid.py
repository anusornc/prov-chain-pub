#!/usr/bin/env python3
"""Normalize AccessGUDID device records into the shared intermediate schema."""

from __future__ import annotations

import argparse
import csv
from pathlib import Path

from common import build_source_record, first_non_empty, iso_date_to_rfc3339, write_json


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, help="Path to raw AccessGUDID CSV export.")
    parser.add_argument("--output", required=True, help="Path to normalized JSON output.")
    parser.add_argument("--snapshot-id", required=True, help="Raw snapshot identifier.")
    return parser


def normalize_row(row: dict, snapshot_id: str, index: int) -> dict:
    primary_di = first_non_empty(row.get("PrimaryDI"), row.get("DI"), f"device-{index}")
    company_name = first_non_empty(row.get("CompanyName"), "unknown-company")
    device_description = first_non_empty(row.get("DeviceDescription"), row.get("BrandName"), f"device-{index}")
    public_record_key = first_non_empty(row.get("PublicDeviceRecordKey"), f"record-key-{index}")
    agent_id = f"agent:manufacturer:{company_name.lower().replace(' ', '-').replace('.', '')}"
    entity_id = f"device:udi-di:{primary_di}"

    return {
        "schema_version": "1.0",
        "record_id": entity_id,
        "source_id": "accessgudid",
        "domain": "device",
        "record_type": "entity",
        "core": {
            "entity_id": entity_id,
            "entity_type": "MedicalDevice",
            "agent_id": agent_id,
            "agent_type": "Manufacturer",
            "activity_id": None,
            "activity_type": None,
            "occurred_at": None,
            "published_at": iso_date_to_rfc3339(first_non_empty(row.get("PublishDate"))),
        },
        "identifiers": {
            "primary": f"udi-di:{primary_di}",
            "secondary": [public_record_key],
        },
        "attributes": {
            "name": device_description,
            "status": "listed",
            "classification": "MedicalDevice",
            "lot_or_batch": None,
            "catalog_code": first_non_empty(row.get("VersionModelNumber"), primary_di),
        },
        "relationships": [
            {
                "predicate": "manufactured_by",
                "target_id": agent_id,
                "target_type": "Agent",
            }
        ],
        "source_record": build_source_record(
            "AccessGUDID",
            "https://accessgudid.nlm.nih.gov/download",
            snapshot_id,
        ),
    }


def main() -> None:
    args = build_parser().parse_args()
    with Path(args.input).open("r", encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle)
        normalized = [normalize_row(row, args.snapshot_id, index) for index, row in enumerate(reader, start=1)]
    write_json(Path(args.output), normalized)


if __name__ == "__main__":
    main()
