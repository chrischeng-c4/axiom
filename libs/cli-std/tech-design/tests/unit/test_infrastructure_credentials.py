from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from cli_std.infrastructure.credentials import (
    COURIER_TOKEN_KEY,
    COURIER_URL_KEY,
    GH_TOKEN_KEY,
    GITHUB_TOKEN_KEY,
    resolve_courier_token,
    resolve_courier_url,
    resolve_github_token,
)


class RecordingGhFallback:
    def __init__(self, return_val: str | None = None) -> None:
        self.calls: int = 0
        self.return_val = return_val

    def __call__(self) -> str | None:
        self.calls += 1
        return self.return_val


class TestInfrastructureCredentials(unittest.TestCase):
    def test_resolve_github_token_blank_gh_token(self) -> None:
        env_map = {"GH_TOKEN": "  ", "GITHUB_TOKEN": "g"}
        gh = RecordingGhFallback("fallback")
        res = resolve_github_token(env_map.get, gh)
        self.assertEqual(res, "g")
        self.assertEqual(gh.calls, 0)

    def test_resolve_github_token_absent_fallback(self) -> None:
        env_absent = {}
        gh_absent = RecordingGhFallback("gh_val")
        res_absent = resolve_github_token(env_absent.get, gh_absent)
        self.assertEqual(res_absent, "gh_val")
        self.assertEqual(gh_absent.calls, 1)

        env_blank = {"GH_TOKEN": "  ", "GITHUB_TOKEN": "  "}
        gh_blank = RecordingGhFallback("gh_val")
        res_blank = resolve_github_token(env_blank.get, gh_blank)
        self.assertEqual(res_blank, "gh_val")
        self.assertEqual(gh_blank.calls, 1)

    def test_resolve_github_token_stripping(self) -> None:
        env_map = {"GH_TOKEN": "  abc  "}
        gh = RecordingGhFallback("fallback")
        res = resolve_github_token(env_map.get, gh)
        self.assertEqual(res, "abc")
        self.assertEqual(gh.calls, 0)

    def test_resolve_courier_url_and_token(self) -> None:
        self.assertIsNone(resolve_courier_url({}.get))
        self.assertIsNone(resolve_courier_url({COURIER_URL_KEY: ""}.get))
        self.assertIsNone(resolve_courier_url({COURIER_URL_KEY: "   "}.get))

        valid_env = {
            COURIER_URL_KEY: "  https://courier.internal  ",
            COURIER_TOKEN_KEY: "  secret_token  ",
        }
        self.assertEqual(
            resolve_courier_url(valid_env.get), "https://courier.internal"
        )
        self.assertEqual(
            resolve_courier_token(valid_env.get), "secret_token"
        )

    def test_resolve_courier_token_stripping(self) -> None:
        self.assertIsNone(resolve_courier_token({}.get))
        self.assertIsNone(resolve_courier_token({COURIER_TOKEN_KEY: " "}.get))

    def test_resolve_github_token_both_present(self) -> None:
        env_map = {GH_TOKEN_KEY: "tok1", GITHUB_TOKEN_KEY: "tok2"}
        gh = RecordingGhFallback("fallback")
        res = resolve_github_token(env_map.get, gh)
        self.assertEqual(res, "tok1")

    def test_resolve_courier_url_whitespace_handling(self) -> None:
        env_map = {COURIER_URL_KEY: "\thttps://example.com\n"}
        self.assertEqual(resolve_courier_url(env_map.get), "https://example.com")

    def test_resolve_github_token_gh_fallback_returns_none(self) -> None:
        gh = RecordingGhFallback(None)
        res = resolve_github_token({}.get, gh)
        self.assertIsNone(res)
        self.assertEqual(gh.calls, 1)


if __name__ == "__main__":
    unittest.main()
