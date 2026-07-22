"""HTTP API and static-file server for FocusFlow Todo."""

from __future__ import annotations

import argparse
import json
import mimetypes
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlparse

from .app import NotFoundError, TodoStore, ValidationError

APP_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_DB_PATH = APP_ROOT / "backend" / "data" / "todos.sqlite3"
FRONTEND_ROOT = APP_ROOT / "frontend"


def make_server(
    host: str = "127.0.0.1",
    port: int = 8080,
    db_path: str | Path = DEFAULT_DB_PATH,
    static_root: str | Path = FRONTEND_ROOT,
) -> ThreadingHTTPServer:
    store = TodoStore(db_path)
    static_directory = Path(static_root).resolve()

    class TodoRequestHandler(BaseHTTPRequestHandler):
        server_version = "FocusFlow/1.0"

        def do_GET(self) -> None:  # noqa: N802
            path = urlparse(self.path).path
            if path == "/api/health":
                self._send_json(HTTPStatus.OK, {"status": "ok"})
            elif path == "/api/todos":
                self._send_json(HTTPStatus.OK, {"todos": store.list_todos()})
            else:
                self._serve_static(path)

        def do_POST(self) -> None:  # noqa: N802
            if urlparse(self.path).path != "/api/todos":
                self._send_json(HTTPStatus.NOT_FOUND, {"error": "route not found"})
                return
            self._handle_mutation(lambda body: (HTTPStatus.CREATED, store.create_todo(body)))

        def do_PATCH(self) -> None:  # noqa: N802
            todo_id = self._todo_id()
            if todo_id is None:
                self._send_json(HTTPStatus.NOT_FOUND, {"error": "route not found"})
                return
            self._handle_mutation(lambda body: (HTTPStatus.OK, store.update_todo(todo_id, body)))

        def do_DELETE(self) -> None:  # noqa: N802
            todo_id = self._todo_id()
            if todo_id is None:
                self._send_json(HTTPStatus.NOT_FOUND, {"error": "route not found"})
                return
            try:
                store.delete_todo(todo_id)
            except NotFoundError as error:
                self._send_json(HTTPStatus.NOT_FOUND, {"error": str(error)})
                return
            self.send_response(HTTPStatus.NO_CONTENT)
            self.end_headers()

        def _handle_mutation(self, operation: object) -> None:
            try:
                status, todo = operation(self._read_json())  # type: ignore[operator]
                self._send_json(status, {"todo": todo})
            except ValidationError as error:
                self._send_json(HTTPStatus.UNPROCESSABLE_ENTITY, {"error": str(error)})
            except NotFoundError as error:
                self._send_json(HTTPStatus.NOT_FOUND, {"error": str(error)})
            except ValueError as error:
                self._send_json(HTTPStatus.BAD_REQUEST, {"error": str(error)})

        def _todo_id(self) -> int | None:
            parts = urlparse(self.path).path.strip("/").split("/")
            if len(parts) != 3 or parts[:2] != ["api", "todos"]:
                return None
            try:
                return int(parts[2])
            except ValueError:
                return None

        def _read_json(self) -> dict[str, object]:
            length = int(self.headers.get("Content-Length", "0"))
            if length <= 0:
                raise ValueError("a JSON request body is required")
            try:
                payload = json.loads(self.rfile.read(length))
            except json.JSONDecodeError as error:
                raise ValueError("request body must be valid JSON") from error
            if not isinstance(payload, dict):
                raise ValueError("request body must be a JSON object")
            return payload

        def _serve_static(self, request_path: str) -> None:
            relative_path = "index.html" if request_path in ("", "/") else request_path.lstrip("/")
            candidate = (static_directory / relative_path).resolve()
            if static_directory not in candidate.parents and candidate != static_directory:
                self._send_json(HTTPStatus.NOT_FOUND, {"error": "route not found"})
                return
            if not candidate.is_file():
                self._send_json(HTTPStatus.NOT_FOUND, {"error": "route not found"})
                return
            mime_type = mimetypes.guess_type(candidate.name)[0] or "application/octet-stream"
            content = candidate.read_bytes()
            self.send_response(HTTPStatus.OK)
            self.send_header("Content-Type", f"{mime_type}; charset=utf-8")
            self.send_header("Content-Length", str(len(content)))
            self.end_headers()
            self.wfile.write(content)

        def _send_json(self, status: HTTPStatus, payload: dict[str, object]) -> None:
            content = json.dumps(payload).encode("utf-8")
            self.send_response(status)
            self.send_header("Content-Type", "application/json; charset=utf-8")
            self.send_header("Content-Length", str(len(content)))
            self.end_headers()
            self.wfile.write(content)

        def log_message(self, format: str, *args: object) -> None:
            return

    return ThreadingHTTPServer((host, port), TodoRequestHandler)


def main() -> None:
    parser = argparse.ArgumentParser(description="Run the FocusFlow Todo web app")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8080)
    parser.add_argument("--db", type=Path, default=DEFAULT_DB_PATH)
    args = parser.parse_args()
    server = make_server(args.host, args.port, args.db)
    print(f"FocusFlow is ready at http://{args.host}:{args.port}")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


if __name__ == "__main__":
    main()
