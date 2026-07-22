from __future__ import annotations

import json
import tempfile
import threading
import unittest
from pathlib import Path
from urllib.error import HTTPError
from urllib.request import Request, urlopen

from backend.server import FRONTEND_ROOT, make_server


class TodoApiTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tempdir = tempfile.TemporaryDirectory()
        self.server = make_server("127.0.0.1", 0, Path(self.tempdir.name) / "todos.sqlite3", FRONTEND_ROOT)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        host, port = self.server.server_address
        self.base_url = f"http://{host}:{port}"

    def tearDown(self) -> None:
        self.server.shutdown()
        self.thread.join()
        self.server.server_close()
        self.tempdir.cleanup()

    def request(self, path: str, method: str = "GET", payload: dict | None = None):
        data = json.dumps(payload).encode() if payload is not None else None
        request = Request(
            f"{self.base_url}{path}",
            method=method,
            data=data,
            headers={"Content-Type": "application/json"} if data else {},
        )
        try:
            with urlopen(request) as response:
                content = response.read()
                return response.status, json.loads(content) if content else None
        except HTTPError as error:
            return error.code, json.loads(error.read())

    def test_create_update_list_and_delete_todo(self) -> None:
        status, body = self.request(
            "/api/todos",
            "POST",
            {"title": "Ship FocusFlow", "priority": "high", "due_date": "2026-07-30"},
        )
        self.assertEqual(status, 201)
        todo = body["todo"]
        self.assertEqual(todo["title"], "Ship FocusFlow")
        self.assertFalse(todo["completed"])
        self.assertEqual(todo["priority"], "high")

        status, body = self.request(f"/api/todos/{todo['id']}", "PATCH", {"completed": True})
        self.assertEqual(status, 200)
        self.assertTrue(body["todo"]["completed"])

        status, body = self.request("/api/todos")
        self.assertEqual(status, 200)
        self.assertEqual(len(body["todos"]), 1)
        self.assertTrue(body["todos"][0]["completed"])

        status, body = self.request(f"/api/todos/{todo['id']}", "DELETE")
        self.assertEqual(status, 204)
        self.assertIsNone(body)
        self.assertEqual(self.request("/api/todos")[1]["todos"], [])

    def test_rejects_invalid_payload_and_unknown_task(self) -> None:
        status, body = self.request("/api/todos", "POST", {"title": "  ", "priority": "urgent"})
        self.assertEqual(status, 422)
        self.assertIn("title", body["error"])

        status, body = self.request("/api/todos/999", "PATCH", {"completed": True})
        self.assertEqual(status, 404)
        self.assertIn("not found", body["error"])

    def test_health_and_frontend_are_served(self) -> None:
        status, body = self.request("/api/health")
        self.assertEqual((status, body), (200, {"status": "ok"}))
        with urlopen(f"{self.base_url}/") as response:
            document = response.read().decode()
        self.assertIn("FocusFlow", document)


if __name__ == "__main__":
    unittest.main()
