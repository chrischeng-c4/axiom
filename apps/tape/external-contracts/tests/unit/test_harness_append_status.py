"""Unit tests for `harness.append`'s status reporting.

This file exists because of a specific false green. The durability contract
partitions every key into acknowledged / refused / unknown, and the partition
is only as good as the function that reports the status. `urllib.request.urlopen`
raises `HTTPError` on every non-2xx, so an `append` that simply returns
`response.status` can only ever return a 2xx: every refusal arrives at the
caller as an exception, indistinguishable from a dead socket. The contract's
`refused` set was therefore permanently empty and the two assertions reading it
were vacuous -- while the contract reported green.

The distinction that was being lost is not cosmetic. A 507 from
`enforce_storage_writable` is the server promising it will *not* keep the
write, which is an obligation recovery must honour. A dropped connection is a
genuinely unknown outcome that may legally go either way. Collapsing the first
into the second discards an enforceable invariant.

Both directions are asserted here, against a real socket rather than a mocked
`urlopen`: a mock would encode my belief about when urllib raises, which is the
exact belief that was wrong.

Stdlib `unittest` and `http.server`, no third-party deps: see the note in
`test_recovery_contract.py`.
"""

from __future__ import annotations

import http.server
import importlib.util
import sys
import threading
import unittest
from pathlib import Path

SRC = Path(__file__).resolve().parents[2] / "src"

_spec = importlib.util.spec_from_file_location("tape_ec_harness", SRC / "harness.py")
assert _spec and _spec.loader
harness = importlib.util.module_from_spec(_spec)
sys.modules[_spec.name] = harness
_spec.loader.exec_module(harness)


class _Handler(http.server.BaseHTTPRequestHandler):
    status = 200

    def do_POST(self) -> None:  # noqa: N802 - BaseHTTPRequestHandler's spelling
        self.rfile.read(int(self.headers.get("content-length", 0)))
        self.send_response(type(self).status)
        self.send_header("content-length", "0")
        self.end_headers()

    def log_message(self, *args: object) -> None:
        pass


class AppendStatusTest(unittest.TestCase):
    def setUp(self) -> None:
        self.server = http.server.HTTPServer(("127.0.0.1", 0), _Handler)
        thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        thread.start()
        self.addCleanup(self.server.shutdown)
        self.base = f"http://127.0.0.1:{self.server.server_address[1]}"

    def respond_with(self, status: int) -> None:
        _Handler.status = status
        self.addCleanup(setattr, _Handler, "status", 200)

    def test_returns_the_status_on_success(self):
        self.respond_with(200)

        self.assertEqual(harness.append(self.base, "t", "k", {"n": 1}), 200)

    def test_returns_507_rather_than_raising_it(self):
        """The degraded-mode refusal tape actually emits (#2573/#2516)."""
        self.respond_with(507)

        self.assertEqual(harness.append(self.base, "t", "k", {"n": 1}), 507)

    def test_returns_500_rather_than_raising_it(self):
        """A failed persist. Also a refusal, also must not read as unknown."""
        self.respond_with(500)

        self.assertEqual(harness.append(self.base, "t", "k", {"n": 1}), 500)

    def test_append_raw_reports_the_status_of_a_body_it_did_not_build(self):
        """The refusal probe's path. Same contract, arbitrary bytes.

        `append` delegates here, so this is not merely a second spelling: the
        durability contract's probe depends on a malformed body producing a
        returned status rather than a raised one, exactly as a well-formed
        rejected body does.
        """
        self.respond_with(400)

        self.assertEqual(harness.append_raw(self.base, "t", b'{"key": "m0'), 400)

    def test_still_raises_when_there_is_no_server_to_answer(self):
        """The post-kill path. An unknown outcome must stay distinguishable.

        Shutting the listener down first means the port refuses connections,
        which is what a SIGKILLed tape leaves behind, and it must not be
        reported as some status code.
        """
        self.server.shutdown()
        self.server.server_close()

        with self.assertRaises(OSError):
            harness.append(self.base, "t", "k", {"n": 1}, timeout=2.0)


if __name__ == "__main__":
    unittest.main()
