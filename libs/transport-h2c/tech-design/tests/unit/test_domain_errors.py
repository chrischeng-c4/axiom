from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from transport_h2c.domain.errors import (
    Connect,
    H2Protocol,
    InvalidRequest,
    NoConnection,
    Shutdown,
    Timeout,
    is_connection_lost,
)


class TestDomainErrors(unittest.TestCase):
    def test_connect_error_is_lost(self) -> None:
        err = Connect("keep:7117", "refused")
        self.assertTrue(is_connection_lost(err))

    def test_no_connection_error_is_lost(self) -> None:
        err = NoConnection("keep:7117")
        self.assertTrue(is_connection_lost(err))

    def test_h2_protocol_go_away_is_lost(self) -> None:
        err = H2Protocol(go_away=True)
        self.assertTrue(is_connection_lost(err))

    def test_h2_protocol_io_is_lost(self) -> None:
        err = H2Protocol(io=True)
        self.assertTrue(is_connection_lost(err))

    def test_h2_protocol_reset_is_lost(self) -> None:
        err = H2Protocol(reset=True)
        self.assertTrue(is_connection_lost(err))

    def test_h2_protocol_all_flags_is_lost(self) -> None:
        err = H2Protocol(go_away=True, io=True, reset=True)
        self.assertTrue(is_connection_lost(err))

    def test_h2_protocol_no_flags_is_not_lost(self) -> None:
        err = H2Protocol()
        self.assertFalse(is_connection_lost(err))

    def test_timeout_error_is_not_lost(self) -> None:
        err = Timeout(5.0)
        self.assertFalse(is_connection_lost(err))

    def test_shutdown_error_is_not_lost(self) -> None:
        err = Shutdown()
        self.assertFalse(is_connection_lost(err))

    def test_invalid_request_error_is_not_lost(self) -> None:
        err = InvalidRequest("bad uri")
        self.assertFalse(is_connection_lost(err))


if __name__ == "__main__":
    unittest.main()
