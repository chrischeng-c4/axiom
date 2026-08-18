from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from transport_h2c.domain.endpoint import authority_of


class TestDomainEndpoint(unittest.TestCase):
    def test_bare_authority_unchanged(self) -> None:
        self.assertEqual(authority_of("keep:7117"), "keep:7117")

    def test_http_prefix_removed(self) -> None:
        self.assertEqual(authority_of("http://keep:7117"), "keep:7117")

    def test_single_trailing_slash_trimmed(self) -> None:
        self.assertEqual(authority_of("http://keep:7117/"), "keep:7117")

    def test_multiple_trailing_slashes_trimmed(self) -> None:
        self.assertEqual(authority_of("http://keep:7117///"), "keep:7117")

    def test_https_prefix_not_removed(self) -> None:
        self.assertEqual(authority_of("https://keep:7117"), "https://keep:7117")

    def test_uppercase_http_not_removed(self) -> None:
        self.assertEqual(authority_of("HTTP://keep:7117"), "HTTP://keep:7117")

    def test_prefix_removed_only_once(self) -> None:
        self.assertEqual(authority_of("http://http://a"), "http://a")

    def test_leading_slashes_preserved(self) -> None:
        self.assertEqual(authority_of("http:///x"), "/x")

    def test_empty_http_endpoint(self) -> None:
        self.assertEqual(authority_of("http://"), "")

    def test_slash_only_endpoint(self) -> None:
        self.assertEqual(authority_of("/"), "")

    def test_empty_string_endpoint(self) -> None:
        self.assertEqual(authority_of(""), "")


if __name__ == "__main__":
    unittest.main()
