#!/usr/bin/env python3
"""Translate the trace benchmark Turtle dataset into JSON-LD for Fluree.

This script is intentionally deterministic:
- fixed input path
- fixed output path
- sorted JSON keys
- stable indentation
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

from rdflib import Graph

STANDARD_PREFIXES = {
    "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
    "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
}
PREFIX_DECLARATION_PATTERN = re.compile(r"@prefix\s+([A-Za-z][\w-]*):\s*<([^>]+)>\s*\.")
SLASHED_CURIE_PATTERN = re.compile(r"(?<!<)\b([A-Za-z][\w-]*):([A-Za-z0-9._~-]+(?:/[A-Za-z0-9._~-]+)+)\b")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Translate a Turtle trace benchmark dataset into JSON-LD."
    )
    parser.add_argument("--input", required=True, help="Path to the Turtle dataset")
    parser.add_argument("--output", required=True, help="Path to the JSON-LD output file")
    return parser.parse_args()


def ensure_required_prefixes(turtle_text: str) -> str:
    prefix_block: list[str] = []
    for prefix, uri in STANDARD_PREFIXES.items():
        marker = f"@prefix {prefix}:"
        if marker not in turtle_text:
            prefix_block.append(f"@prefix {prefix}: <{uri}> .")

    if not prefix_block:
        return turtle_text

    return "\n".join(prefix_block) + "\n" + turtle_text


def expand_slashed_curie_tokens(turtle_text: str) -> str:
    prefix_map = {
        prefix: uri for prefix, uri in PREFIX_DECLARATION_PATTERN.findall(turtle_text)
    }

    def replace(match: re.Match[str]) -> str:
        prefix, local_part = match.groups()
        namespace = prefix_map.get(prefix)
        if namespace is None:
            return match.group(0)
        return f"<{namespace}{local_part}>"

    return SLASHED_CURIE_PATTERN.sub(replace, turtle_text)


def main() -> int:
    args = parse_args()
    input_path = Path(args.input)
    output_path = Path(args.output)

    turtle_text = input_path.read_text(encoding="utf-8")
    turtle_text = ensure_required_prefixes(turtle_text)
    turtle_text = expand_slashed_curie_tokens(turtle_text)

    graph = Graph()
    graph.parse(data=turtle_text, format="turtle")

    output_path.parent.mkdir(parents=True, exist_ok=True)
    jsonld_text = graph.serialize(format="json-ld", indent=2, auto_compact=False)
    jsonld_data = json.loads(jsonld_text)

    with output_path.open("w", encoding="utf-8") as fh:
        json.dump(jsonld_data, fh, indent=2, sort_keys=True, ensure_ascii=False)
        fh.write("\n")

    print(f"Translated {input_path} -> {output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
