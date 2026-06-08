"""Minimal HTTP server for the temporary Cue backend bridge profile."""

from __future__ import annotations

import asyncio
import inspect
import json
import re
from dataclasses import dataclass
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Any, Callable
from urllib.parse import urlparse


@dataclass(frozen=True)
class Route:
    method: str
    path: str
    pattern: re.Pattern[str]
    handler: Callable[..., Any]


def _compile_path(path: str) -> re.Pattern[str]:
    parts = path.strip("/").split("/") if path != "/" else []
    compiled: list[str] = []
    for part in parts:
        if part.startswith("{") and part.endswith("}"):
            compiled.append(f"(?P<{part[1:-1]}>[^/]+)")
        else:
            compiled.append(re.escape(part))
    body = "/".join(compiled)
    return re.compile(f"^/{body}/?$" if body else r"^/$")


def _routes(app: Any) -> list[Route]:
    handlers = getattr(app, "_handlers", {})
    return [
        Route(method=method, path=path, pattern=_compile_path(path), handler=handler)
        for (method, path), handler in handlers.items()
    ]


async def _invoke(handler: Callable[..., Any], path_params: dict[str, str], body: Any) -> Any:
    signature = inspect.signature(handler)
    kwargs: dict[str, Any] = {}
    for name, parameter in signature.parameters.items():
        if name in path_params:
            kwargs[name] = path_params[name]
        elif name == "payload":
            kwargs[name] = body
        elif parameter.default is inspect.Parameter.empty:
            kwargs[name] = body

    result = handler(**kwargs)
    if inspect.isawaitable(result):
        result = await result
    return result


class CueDevRequestHandler(BaseHTTPRequestHandler):
    server_version = "CueBackendDev/0.1"
    routes: list[Route] = []

    def do_OPTIONS(self) -> None:
        self._send_empty(204)

    def do_GET(self) -> None:
        self._handle("GET")

    def do_POST(self) -> None:
        self._handle("POST")

    def do_PUT(self) -> None:
        self._handle("PUT")

    def do_PATCH(self) -> None:
        self._handle("PATCH")

    def do_DELETE(self) -> None:
        self._handle("DELETE")

    def log_message(self, format: str, *args: Any) -> None:
        print(f"{self.address_string()} - {format % args}")

    def _handle(self, method: str) -> None:
        pathname = urlparse(self.path).path
        for route in self.routes:
            if route.method != method:
                continue
            match = route.pattern.match(pathname)
            if not match:
                continue

            try:
                body = self._read_json_body()
                payload = asyncio.run(_invoke(route.handler, match.groupdict(), body))
                self._send_json(200, payload)
            except Exception as error:  # pragma: no cover - dev diagnostics path
                self._send_json(
                    500,
                    {
                        "ok": False,
                        "error": {
                            "code": "dev_server_error",
                            "message": str(error),
                        },
                    },
                )
            return

        self._send_json(
            404,
            {
                "ok": False,
                "error": {
                    "code": "not_found",
                    "message": f"No {method} route for {pathname}",
                },
            },
        )

    def _read_json_body(self) -> Any:
        length = int(self.headers.get("content-length") or "0")
        if length == 0:
            return {}
        raw = self.rfile.read(length)
        if not raw:
            return {}
        return json.loads(raw.decode("utf-8"))

    def _send_empty(self, status: int) -> None:
        self.send_response(status)
        self._send_cors_headers()
        self.end_headers()

    def _send_json(self, status: int, payload: Any) -> None:
        body = json.dumps(payload, ensure_ascii=False).encode("utf-8")
        self.send_response(status)
        self._send_cors_headers()
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _send_cors_headers(self) -> None:
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Headers", "content-type")
        self.send_header("Access-Control-Allow-Methods", "GET,POST,PUT,PATCH,DELETE,OPTIONS")


def run_dev_server(app: Any, host: str = "127.0.0.1", port: int = 43219) -> None:
    routes = _routes(app)
    if not routes:
        raise RuntimeError("Cue backend app has no registered routes")

    handler = type("CueDevRequestHandler", (CueDevRequestHandler,), {"routes": routes})
    server = ThreadingHTTPServer((host, port), handler)
    print(f"Cue backend dev API: http://{host}:{port}")
    print(f"Registered {len(routes)} routes")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
