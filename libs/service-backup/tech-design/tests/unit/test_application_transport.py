from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_backup.application.transport import (
    admin_request_headers,
    admin_snapshot_url,
    classify_response,
)
from service_backup.domain.errors import RemoteStatus


class TestApplicationTransport(unittest.TestCase):
    def test_admin_snapshot_url(self) -> None:
        self.assertEqual(admin_snapshot_url("http://h:8080"), "http://h:8080/admin/backup")
        self.assertEqual(admin_snapshot_url("http://h:8080/"), "http://h:8080/admin/backup")
        self.assertEqual(admin_snapshot_url("http://h:8080///"), "http://h:8080/admin/backup")
        self.assertEqual(admin_snapshot_url(""), "/admin/backup")
        self.assertEqual(admin_snapshot_url("/"), "/admin/backup")

    def test_admin_request_headers(self) -> None:
        self.assertEqual(admin_request_headers(None), {})
        self.assertEqual(admin_request_headers("t"), {"authorization": "Bearer t"})
        self.assertEqual(admin_request_headers(""), {"authorization": "Bearer "})

    def test_classify_response(self) -> None:
        self.assertIsNone(classify_response(200, ""))
        self.assertIsNone(classify_response(204, ""))
        self.assertIsNone(classify_response(299, ""))
        self.assertEqual(classify_response(199, "b"), RemoteStatus(199, "b"))
        self.assertEqual(classify_response(300, "b"), RemoteStatus(300, "b"))
        self.assertEqual(classify_response(403, "denied"), RemoteStatus(403, "denied"))
        self.assertEqual(classify_response(503, "x"), RemoteStatus(503, "x"))


if __name__ == "__main__":
    unittest.main()
