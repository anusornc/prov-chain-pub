#!/usr/bin/env python3
"""Synthesize healthcare device traceability events from normalized device records."""

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
from ontology_package_emitters import emit_healthcare_device


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, help="Path to normalized device JSON records.")
    parser.add_argument("--output", required=True, help="Path to synthesized Turtle output.")
    parser.add_argument(
        "--default-location",
        default="Registry Catalog",
        help="Synthetic location label for generated healthcare device events.",
    )
    return parser


def main() -> None:
    args = build_parser().parse_args()
    records = load_json(Path(args.input))
    payload = "\n".join(
        emit_healthcare_device(record, default_location=args.default_location)
        for record in records
    )
    Path(args.output).write_text(payload, encoding="utf-8")


if __name__ == "__main__":
    main()
