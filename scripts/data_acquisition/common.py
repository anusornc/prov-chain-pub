#!/usr/bin/env python3
"""Common helpers for dataset snapshot acquisition scripts."""

from __future__ import annotations

import hashlib
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


def write_bytes(path: Path, data: bytes) -> str:
    """Write bytes to disk and return their SHA-256 hash."""
    ensure_parent(path)
    path.write_bytes(data)
    return hashlib.sha256(data).hexdigest()


def write_json(path: Path, payload: Any) -> str:
    """Serialize JSON with stable formatting and return the SHA-256 hash."""
    encoded = json.dumps(payload, indent=2, sort_keys=True).encode("utf-8")
    return write_bytes(path, encoded)


def write_toml_like_manifest(path: Path, manifest: dict[str, Any]) -> None:
    """Write a simple TOML-shaped manifest without external dependencies."""
    ensure_parent(path)
    lines: list[str] = []
    scalar_keys = [
        "manifest_version",
        "snapshot_id",
        "source_id",
        "retrieved_at",
        "retrieval_tool",
        "output_root",
    ]
    for key in scalar_keys:
        if key in manifest:
            lines.append(f'{key} = "{manifest[key]}"')
    lines.append("")
    for section in ("source", "query", "release", "artifacts", "normalization", "notes"):
        values = manifest.get(section)
        if not values:
            continue
        lines.append(f"[{section}]")
        for key, value in values.items():
            if isinstance(value, bool):
                rendered = "true" if value else "false"
            elif isinstance(value, int):
                rendered = str(value)
            else:
                rendered = f'"{value}"'
            lines.append(f"{key} = {rendered}")
        lines.append("")
    path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")

