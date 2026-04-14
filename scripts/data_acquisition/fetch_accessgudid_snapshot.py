#!/usr/bin/env python3
"""Download an AccessGUDID release archive and emit an acquisition manifest."""

from __future__ import annotations

import argparse
import urllib.request
from pathlib import Path

from common import utc_now_iso, write_bytes, write_toml_like_manifest


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--url", required=True, help="Direct AccessGUDID archive URL.")
    parser.add_argument("--snapshot-id", required=True)
    parser.add_argument("--release-label", default="accessgudid_release")
    parser.add_argument("--release-date", default="")
    parser.add_argument("--output-root", default="data/external")
    return parser


def main() -> None:
    args = build_parser().parse_args()
    output_root = Path(args.output_root)
    retrieved_at = utc_now_iso()
    filename = args.url.rstrip("/").split("/")[-1] or "accessgudid_release.zip"
    raw_path = output_root / "raw" / "accessgudid" / args.snapshot_id / filename

    with urllib.request.urlopen(args.url) as response:
        payload = response.read()

    sha256 = write_bytes(raw_path, payload)
    manifest = {
        "manifest_version": "1.0",
        "snapshot_id": args.snapshot_id,
        "source_id": "accessgudid",
        "retrieved_at": retrieved_at,
        "retrieval_tool": "scripts/data_acquisition/fetch_accessgudid_snapshot.py",
        "output_root": str(output_root),
        "source": {
            "provider": "NLM / FDA GUDID",
            "documentation_url": "https://accessgudid.nlm.nih.gov/download",
            "data_url": args.url,
            "license": "U.S. Government public data",
        },
        "query": {
            "kind": "release_archive_download",
            "query_string": args.url,
            "page_size": 0,
            "record_limit": 1,
        },
        "release": {
            "release_label": args.release_label,
            "release_date": args.release_date,
        },
        "artifacts": {
            "raw_file": str(raw_path),
            "sha256": sha256,
            "content_type": "application/zip",
            "record_count": 1,
        },
        "normalization": {
            "target_domain": "device",
            "ontology_package_id": "pending",
            "ontology_package_version": "pending",
            "normalizer_version": "pending",
        },
        "notes": {
            "purpose": "benchmark",
            "comments": "Stable AccessGUDID release archive for device ontology mapping.",
        },
    }
    manifest_path = output_root / "raw" / "accessgudid" / args.snapshot_id / "acquisition_manifest.toml"
    write_toml_like_manifest(manifest_path, manifest)


if __name__ == "__main__":
    main()

