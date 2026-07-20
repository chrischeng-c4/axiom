"""Minimal real HTTP target shared by Defer and Cloud Tasks.

The process stores only bounded benchmark receipts and exposes reset/stats
behind a per-run secret. Cloud Run is pinned to one instance, so both backends
observe the same target and the client can verify duplicates and completion.
"""

from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
import threading
import time
from urllib.parse import parse_qs, urlparse


SECRET = os.environ["BENCH_SECRET"]
LOCK = threading.Lock()
RECEIPTS: dict[str, dict[str, object]] = {}


def backend_state(name: str) -> dict[str, object]:
    return RECEIPTS.setdefault(
        name,
        {"requests": 0, "keys": {}, "first_received_ns": None, "last_received_ns": None},
    )


class Handler(BaseHTTPRequestHandler):
    server_version = "axiom-gcp-bench-receiver/1"

    def log_message(self, format: str, *args: object) -> None:
        return

    def authenticated(self) -> bool:
        return self.headers.get("x-axiom-bench-secret") == SECRET

    def json_response(self, status: int, value: object) -> None:
        body = json.dumps(value, separators=(",", ":")).encode()
        self.send_response(status)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self) -> None:
        parsed = urlparse(self.path)
        if parsed.path == "/healthz":
            self.json_response(200, {"status": "ok"})
            return
        if parsed.path != "/stats" or not self.authenticated():
            self.json_response(403, {"error": "forbidden"})
            return
        backend = parse_qs(parsed.query).get("backend", [""])[0]
        with LOCK:
            state = backend_state(backend)
            keys = state["keys"]
            assert isinstance(keys, dict)
            value = {
                "backend": backend,
                "requests": state["requests"],
                "unique": len(keys),
                "duplicates": sum(max(0, count - 1) for count in keys.values()),
                "first_received_ns": state["first_received_ns"],
                "last_received_ns": state["last_received_ns"],
            }
        self.json_response(200, value)

    def do_POST(self) -> None:
        parsed = urlparse(self.path)
        if not self.authenticated():
            self.json_response(403, {"error": "forbidden"})
            return
        if parsed.path == "/reset":
            backend = parse_qs(parsed.query).get("backend", [""])[0]
            with LOCK:
                RECEIPTS.pop(backend, None)
            self.json_response(200, {"backend": backend, "reset": True})
            return
        if parsed.path != "/task":
            self.json_response(404, {"error": "not_found"})
            return

        length = int(self.headers.get("content-length", "0"))
        if length:
            self.rfile.read(length)
        backend = self.headers.get("x-axiom-bench-backend", "unknown")
        key = self.headers.get("x-axiom-bench-key", "missing")
        now = time.time_ns()
        with LOCK:
            state = backend_state(backend)
            keys = state["keys"]
            assert isinstance(keys, dict)
            state["requests"] = int(state["requests"]) + 1
            state["first_received_ns"] = state["first_received_ns"] or now
            state["last_received_ns"] = now
            keys[key] = int(keys.get(key, 0)) + 1
        self.send_response(204)
        self.end_headers()


if __name__ == "__main__":
    port = int(os.environ.get("PORT", "8080"))
    ThreadingHTTPServer(("0.0.0.0", port), Handler).serve_forever()
