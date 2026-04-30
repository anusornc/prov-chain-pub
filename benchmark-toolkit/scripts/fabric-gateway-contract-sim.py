#!/usr/bin/env python3
"""Minimal Fabric gateway contract simulator for local adapter/probe checks.

This is not a Fabric benchmark target. It only implements the REST contract
defined for the future Fabric gateway so local tooling can be validated before
a real Fabric network is available.
"""

from __future__ import annotations

import json
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def _read_json(self) -> dict:
        length = int(self.headers.get("Content-Length", "0"))
        if length == 0:
            return {}
        raw = self.rfile.read(length)
        return json.loads(raw.decode("utf-8"))

    def _write_json(self, status: int, payload: dict) -> None:
        body = json.dumps(payload).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, format: str, *args) -> None:
        return

    def do_GET(self) -> None:
        if self.path == "/health":
            self._write_json(
                200,
                {
                    "status": "ok",
                    "channel": "provchain",
                    "chaincode": "traceability",
                },
            )
            return
        self._write_json(404, {"error": "not found"})

    def do_POST(self) -> None:
        started = time.monotonic()
        try:
            payload = self._read_json()
        except Exception as exc:
            self._write_json(400, {"error": f"invalid json: {exc}"})
            return

        if self.path == "/ledger/records":
            if "record_id" not in payload:
                self._write_json(400, {"error": "record_id is required"})
                return
            submit_ms = (time.monotonic() - started) * 1000.0
            self._write_json(
                200,
                {
                    "success": True,
                    "tx_id": f"tx-{payload['record_id']}",
                    "submit_latency_ms": round(submit_ms, 3),
                    "commit_latency_ms": round(submit_ms + 10.0, 3),
                    "block_number": 1,
                },
            )
            return

        if self.path == "/ledger/records/batch":
            records = payload.get("records", [])
            submit_ms = (time.monotonic() - started) * 1000.0
            self._write_json(
                200,
                {
                    "success": True,
                    "submitted": len(records),
                    "committed": len(records),
                    "submit_latency_ms": round(submit_ms, 3),
                    "commit_latency_ms": round(submit_ms + 10.0, 3),
                },
            )
            return

        if self.path == "/policy/check":
            authorized = payload.get("actor_org") in {"Org1MSP", "AuditorMSP"}
            self._write_json(
                200,
                {
                    "authorized": authorized,
                    "policy_latency_ms": 1.0,
                },
            )
            return

        self._write_json(404, {"error": "not found"})


def main() -> None:
    server = ThreadingHTTPServer(("0.0.0.0", 18800), Handler)
    print("Fabric gateway contract simulator listening on :18800", flush=True)
    server.serve_forever()


if __name__ == "__main__":
    main()
