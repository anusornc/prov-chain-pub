#!/usr/bin/env python3
"""Fetch one or more GS1 EPCIS example files and emit an acquisition manifest."""

from __future__ import annotations

import argparse
import urllib.request
from pathlib import Path

from common import utc_now_iso, write_bytes, write_toml_like_manifest


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--url",
        action="append",
        required=True,
        help="Example file URL to download. Repeat for multiple files.",
    )
    parser.add_argument(
        "--output-root",
        default="data/external",
        help="Root directory for raw snapshot files and manifests.",
    )
    parser.add_argument(
        "--snapshot-id",
        required=True,
        help="Logical snapshot identifier, e.g. gs1_epcis_examples_2026_03_10.",
    )
    return parser


def main() -> None:
    args = build_parser().parse_args()
    output_root = Path(args.output_root)
    retrieved_at = utc_now_iso()

    downloaded_files: list[str] = []
    last_sha256 = ""
    for url in args.url:
        filename = url.rstrip("/").split("/")[-1] or "index.html"
        raw_path = output_root / "raw" / "gs1_epcis_examples" / args.snapshot_id / filename
        with urllib.request.urlopen(url) as response:
            payload = response.read()
        last_sha256 = write_bytes(raw_path, payload)
        downloaded_files.append(str(raw_path))

    manifest = {
        "manifest_version": "1.0",
        "snapshot_id": args.snapshot_id,
        "source_id": "gs1_epcis_examples",
        "retrieved_at": retrieved_at,
        "retrieval_tool": "scripts/data_acquisition/fetch_gs1_examples.py",
        "output_root": str(output_root),
        "source": {
            "provider": "GS1",
            "documentation_url": "https://ref.gs1.org/docs/epcis/examples",
            "data_url": ",".join(args.url),
            "license": "GS1 website terms",
        },
        "query": {
            "kind": "static_file_download",
            "query_string": ",".join(args.url),
            "page_size": 0,
            "record_limit": len(args.url),
        },
        "release": {
            "release_label": "gs1_examples_snapshot",
            "release_date": "",
        },
        "artifacts": {
            "raw_file": ";".join(downloaded_files),
            "sha256": last_sha256,
            "content_type": "mixed",
            "record_count": len(args.url),
        },
        "normalization": {
            "target_domain": "food",
            "ontology_package_id": "pending",
            "ontology_package_version": "pending",
            "normalizer_version": "pending",
        },
        "notes": {
            "purpose": "benchmark",
            "comments": "Standards-aligned EPCIS example snapshot.",
        },
    }
    manifest_path = output_root / "raw" / "gs1_epcis_examples" / args.snapshot_id / "acquisition_manifest.toml"
    write_toml_like_manifest(manifest_path, manifest)


if __name__ == "__main__":
    main()

