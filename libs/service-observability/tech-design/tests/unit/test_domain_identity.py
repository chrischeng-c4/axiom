from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_observability.domain.identity import (
    IdentityError,
    ServiceIdentity,
    make_identity,
)


class TestDomainIdentity(unittest.TestCase):
    def test_make_identity_valid(self) -> None:
        identity = make_identity("svc", "1.2.3")
        self.assertEqual(identity, ServiceIdentity("svc", "1.2.3"))
        self.assertEqual(identity.name, "svc")
        self.assertEqual(identity.version, "1.2.3")

    def test_make_identity_untrimmed(self) -> None:
        identity = make_identity(" svc ", "1.0")
        self.assertEqual(identity, ServiceIdentity(" svc ", "1.0"))

    def test_make_identity_invalid_name(self) -> None:
        with self.assertRaises(IdentityError):
            make_identity("", "1.0")
        with self.assertRaises(IdentityError):
            make_identity("   ", "1.0")
        with self.assertRaises(IdentityError):
            make_identity("\t\n", "1.0")

    def test_make_identity_invalid_version(self) -> None:
        with self.assertRaises(IdentityError):
            make_identity("svc", "")
        with self.assertRaises(IdentityError):
            make_identity("svc", "  ")


if __name__ == "__main__":
    unittest.main()
