#!/usr/bin/env python3
"""Emit ontology-package-specific Turtle payloads from normalized records."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
NORMALIZATION_DIR = SCRIPT_DIR.parent / "data_normalization"
if str(NORMALIZATION_DIR) not in sys.path:
    sys.path.insert(0, str(NORMALIZATION_DIR))
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from common import load_json
from ontology_package_emitters import EMITTERS, emit_healthcare_device


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package", required=True, choices=sorted(EMITTERS))
    parser.add_argument("--input", required=True, help="Path to normalized JSON records.")
    parser.add_argument("--output", required=True, help="Path to emitted Turtle output.")
    parser.add_argument(
        "--limit",
        type=int,
        default=None,
        help="Optional maximum number of normalized records to emit.",
    )
    parser.add_argument(
        "--default-location",
        default="Registry Catalog",
        help="Healthcare-only fallback location.",
    )
    return parser


def main() -> None:
    args = build_parser().parse_args()
    records = load_json(Path(args.input))
    if args.limit is not None:
        records = records[: args.limit]
    emitter = EMITTERS[args.package]

    if emitter is emit_healthcare_device:
        payload = "\n".join(
            emitter(record, default_location=args.default_location) for record in records
        )
    else:
        payload = "\n".join(emitter(record) for record in records)

    Path(args.output).write_text(payload, encoding="utf-8")


if __name__ == "__main__":
    main()
