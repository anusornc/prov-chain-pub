#!/usr/bin/env python3
"""Fetch a deterministic openFDA snapshot and emit an acquisition manifest."""

from __future__ import annotations

import argparse
import json
import urllib.parse
import urllib.request
from pathlib import Path

from common import utc_now_iso, write_json, write_toml_like_manifest


ENDPOINTS = {
    "drug_ndc": {
        "api_url": "https://api.fda.gov/drug/ndc.json",
        "documentation_url": "https://open.fda.gov/apis/drug/ndc/",
        "domain": "pharma",
    },
    "drug_shortages": {
        "api_url": "https://api.fda.gov/drug/drugshortages.json",
        "documentation_url": "https://open.fda.gov/apis/drug/drugshortages/",
        "domain": "pharma",
    },
    "device_udi": {
        "api_url": "https://api.fda.gov/device/udi.json",
        "documentation_url": "https://open.fda.gov/apis/device/udi/",
        "domain": "device",
    },
    "device_enforcement": {
        "api_url": "https://api.fda.gov/device/enforcement.json",
        "documentation_url": "https://open.fda.gov/apis/device/enforcement/",
        "domain": "device",
    },
    "food_enforcement": {
        "api_url": "https://api.fda.gov/food/enforcement.json",
        "documentation_url": "https://open.fda.gov/apis/food/enforcement/",
        "domain": "food",
    },
    "drug_enforcement": {
        "api_url": "https://api.fda.gov/drug/enforcement.json",
        "documentation_url": "https://open.fda.gov/apis/drug/enforcement/",
        "domain": "pharma",
    },
}


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", choices=sorted(ENDPOINTS), required=True)
    parser.add_argument("--snapshot-id", required=True)
    parser.add_argument("--limit", type=int, default=25)
    parser.add_argument("--search", default="")
    parser.add_argument("--skip", type=int, default=0)
    parser.add_argument("--output-root", default="data/external")
    return parser


def main() -> None:
    args = build_parser().parse_args()
    endpoint = ENDPOINTS[args.source]
    output_root = Path(args.output_root)
    retrieved_at = utc_now_iso()

    params = {"limit": args.limit, "skip": args.skip}
    if args.search:
        params["search"] = args.search
    query_string = urllib.parse.urlencode(params)
    url = f"{endpoint['api_url']}?{query_string}"

    with urllib.request.urlopen(url) as response:
        payload = json.load(response)

    raw_path = output_root / "raw" / args.source / f"{args.snapshot_id}.json"
    sha256 = write_json(raw_path, payload)

    record_count = len(payload.get("results", [])) if isinstance(payload, dict) else 0
    manifest = {
        "manifest_version": "1.0",
        "snapshot_id": args.snapshot_id,
        "source_id": args.source,
        "retrieved_at": retrieved_at,
        "retrieval_tool": "scripts/data_acquisition/fetch_openfda_snapshot.py",
        "output_root": str(output_root),
        "source": {
            "provider": "openFDA",
            "documentation_url": endpoint["documentation_url"],
            "data_url": endpoint["api_url"],
            "license": "U.S. Government public data",
        },
        "query": {
            "kind": "api_snapshot",
            "query_string": query_string,
            "page_size": args.limit,
            "record_limit": args.limit,
        },
        "release": {
            "release_label": "api_snapshot",
            "release_date": "",
        },
        "artifacts": {
            "raw_file": str(raw_path),
            "sha256": sha256,
            "content_type": "application/json",
            "record_count": record_count,
        },
        "normalization": {
            "target_domain": endpoint["domain"],
            "ontology_package_id": "pending",
            "ontology_package_version": "pending",
            "normalizer_version": "pending",
        },
        "notes": {
            "purpose": "benchmark",
            "comments": "Deterministic openFDA snapshot for ontology-package evaluation.",
        },
    }
    manifest_path = output_root / "raw" / args.source / f"{args.snapshot_id}.manifest.toml"
    write_toml_like_manifest(manifest_path, manifest)


if __name__ == "__main__":
    main()

