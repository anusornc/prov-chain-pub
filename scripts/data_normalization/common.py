#!/usr/bin/env python3
"""Common helpers for external-dataset normalization."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


def utc_now_iso() -> str:
    """Return the current UTC time in RFC3339-like format."""
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def ensure_parent(path: Path) -> None:
    """Create the parent directory for a file path."""
    path.parent.mkdir(parents=True, exist_ok=True)


def write_json(path: Path, payload: Any) -> None:
    """Write JSON to disk using deterministic formatting."""
    ensure_parent(path)
    path.write_text(json.dumps(payload, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def load_json(path: Path) -> Any:
    """Load JSON from disk."""
    return json.loads(path.read_text(encoding="utf-8"))


def as_list(value: Any) -> list[Any]:
    """Normalize optional scalar-or-list values into a list."""
    if value is None:
        return []
    if isinstance(value, list):
        return value
    return [value]


def first_non_empty(*values: Any) -> str | None:
    """Return the first non-empty string-like value."""
    for value in values:
        if isinstance(value, list):
            for item in value:
                result = first_non_empty(item)
                if result:
                    return result
        elif value is not None:
            text = str(value).strip()
            if text:
                return text
    return None


def yyyymmdd_to_rfc3339(value: str | None) -> str | None:
    """Convert a YYYYMMDD string into a midnight UTC timestamp."""
    if not value:
        return None
    text = str(value).strip()
    if len(text) != 8 or not text.isdigit():
        return None
    return f"{text[:4]}-{text[4:6]}-{text[6:8]}T00:00:00Z"


def iso_date_to_rfc3339(value: str | None) -> str | None:
    """Convert a YYYY-MM-DD string into a midnight UTC timestamp."""
    if not value:
        return None
    text = str(value).strip()
    if len(text) != 10:
        return None
    try:
        datetime.strptime(text, "%Y-%m-%d")
    except ValueError:
        return None
    return f"{text}T00:00:00Z"


def build_source_record(dataset_name: str, source_url: str, raw_snapshot_id: str) -> dict[str, str]:
    """Create the shared source-record block."""
    return {
        "dataset_name": dataset_name,
        "source_url": source_url,
        "retrieved_at": utc_now_iso(),
        "raw_snapshot_id": raw_snapshot_id,
    }
