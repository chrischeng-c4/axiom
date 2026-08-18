from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from transport_h2c.application.request import (
    Admitted,
    Delivered,
    Failed,
    Refused,
    admit,
    resolve_request,
    should_retry,
)
from transport_h2c.domain.errors import (
    Connect,
    H2Protocol,
    InvalidRequest,
    NoConnection,
    Shutdown,
    Timeout,
)
from transport_h2c.infrastructure.config import default_config


class TestApplicationRequest(unittest.TestCase):
    def setUp(self) -> None:
        self.cfg = default_config(8)

    def test_admit_shutdown_first(self) -> None:
        self.assertEqual(
            admit(
                self.cfg,
                shut_down=True,
                admission_closed=False,
                waited_seconds=0.0,
            ),
            Refused(Shutdown()),
        )
        self.assertEqual(
            admit(
                self.cfg,
                shut_down=True,
                admission_closed=False,
                waited_seconds=60.0,
            ),
            Refused(Shutdown()),
        )
        self.assertEqual(
            admit(
                self.cfg,
                shut_down=True,
                admission_closed=True,
                waited_seconds=60.0,
            ),
            Refused(Shutdown()),
        )

    def test_admit_timeout(self) -> None:
        self.assertEqual(
            admit(
                self.cfg,
                shut_down=False,
                admission_closed=False,
                waited_seconds=5.001,
            ),
            Refused(Timeout(5.0)),
        )
        self.assertEqual(
            admit(
                self.cfg,
                shut_down=False,
                admission_closed=False,
                waited_seconds=60.0,
            ),
            Refused(Timeout(5.0)),
        )
        self.assertEqual(
            admit(
                self.cfg,
                shut_down=False,
                admission_closed=True,
                waited_seconds=60.0,
            ),
            Refused(Timeout(5.0)),
        )

    def test_admit_exact_timeout_admitted(self) -> None:
        self.assertEqual(
            admit(
                self.cfg,
                shut_down=False,
                admission_closed=False,
                waited_seconds=5.0,
            ),
            Admitted(),
        )

    def test_admit_closed(self) -> None:
        self.assertEqual(
            admit(
                self.cfg,
                shut_down=False,
                admission_closed=True,
                waited_seconds=0.0,
            ),
            Refused(Shutdown()),
        )

    def test_admit_success(self) -> None:
        self.assertEqual(
            admit(
                self.cfg,
                shut_down=False,
                admission_closed=False,
                waited_seconds=0.0,
            ),
            Admitted(),
        )
        self.assertEqual(
            admit(
                self.cfg,
                shut_down=False,
                admission_closed=False,
                waited_seconds=4.999,
            ),
            Admitted(),
        )

    def test_should_retry_rules(self) -> None:
        self.assertTrue(should_retry(0, Connect("a", "refused")))
        self.assertTrue(should_retry(0, NoConnection("a")))
        self.assertTrue(should_retry(0, H2Protocol(go_away=True)))
        self.assertTrue(should_retry(0, H2Protocol(io=True)))
        self.assertTrue(should_retry(0, H2Protocol(reset=True)))

        self.assertFalse(should_retry(0, H2Protocol()))
        self.assertFalse(should_retry(0, Timeout(5.0)))
        self.assertFalse(should_retry(0, Shutdown()))
        self.assertFalse(should_retry(0, InvalidRequest("x")))

        self.assertFalse(should_retry(1, Connect("a", "refused")))
        self.assertFalse(should_retry(1, H2Protocol(io=True)))
        self.assertFalse(should_retry(2, Connect("a", "refused")))

    def test_resolve_request_outcomes(self) -> None:
        c = Connect("a", "refused")
        l = H2Protocol(io=True)
        g = H2Protocol(go_away=True)
        p = H2Protocol()
        t = Timeout(5.0)

        self.assertEqual(
            resolve_request("keep:7117", [None]), Delivered(1)
        )
        self.assertEqual(
            resolve_request("keep:7117", [None, None]), Delivered(1)
        )
        self.assertEqual(
            resolve_request("keep:7117", [None, c]), Delivered(1)
        )
        self.assertEqual(
            resolve_request("keep:7117", [c, None]), Delivered(2)
        )
        self.assertEqual(
            resolve_request("keep:7117", [l, None]), Delivered(2)
        )
        self.assertEqual(
            resolve_request("keep:7117", [l, g]), Failed(g, 2)
        )
        self.assertEqual(
            resolve_request("keep:7117", [c, c]), Failed(c, 2)
        )
        self.assertEqual(
            resolve_request("keep:7117", [c, c, None]), Failed(c, 2)
        )
        self.assertEqual(
            resolve_request("keep:7117", [t, None]), Failed(t, 1)
        )
        self.assertEqual(
            resolve_request("keep:7117", [p]), Failed(p, 1)
        )
        self.assertEqual(
            resolve_request("keep:7117", [p, None]), Failed(p, 1)
        )
        self.assertEqual(
            resolve_request("keep:7117", [c]), Failed(c, 1)
        )
        self.assertEqual(
            resolve_request("keep:7117", []),
            Failed(NoConnection("keep:7117"), 0),
        )


if __name__ == "__main__":
    unittest.main()
