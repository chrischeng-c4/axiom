from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_http.infrastructure.routes import (
    DOCS_PATH,
    HEALTHZ_PATH,
    METRICS_PATH,
    OPENAPI_PATH,
    PROBE_PATHS,
    READYZ_PATH,
    OpenapiSource,
    access_log_level,
    is_probe_path,
    metrics_response,
    openapi_content_type,
    probe_routes,
    readiness_response,
)


class TestInfrastructureRoutes(unittest.TestCase):
    def test_is_probe_path_exact_matches(self) -> None:
        self.assertTrue(is_probe_path(HEALTHZ_PATH))
        self.assertTrue(is_probe_path(READYZ_PATH))
        self.assertTrue(is_probe_path(METRICS_PATH))
        self.assertTrue(is_probe_path(OPENAPI_PATH))
        self.assertTrue(is_probe_path(DOCS_PATH))

        self.assertFalse(is_probe_path("/healthz/"))
        self.assertFalse(is_probe_path("/healthzz"))
        self.assertFalse(is_probe_path("/metrics/foo"))
        self.assertFalse(is_probe_path("/v1/query"))

    def test_access_log_level(self) -> None:
        self.assertEqual(access_log_level("/readyz"), "debug")
        self.assertEqual(access_log_level("/v1/query"), "info")

    def test_probe_routes_specs(self) -> None:
        routes = probe_routes()
        self.assertEqual(len(routes), 5)
        for r in routes:
            self.assertFalse(r.requires_auth)
            self.assertFalse(r.enforces_body_limit)

    def test_probe_routes_path_tuple(self) -> None:
        expected = (
            "/healthz",
            "/readyz",
            "/metrics",
            "/openapi.json",
            "/docs",
        )
        self.assertEqual(PROBE_PATHS, expected)
        self.assertEqual(tuple(r.path for r in probe_routes()), expected)

    def test_readiness_response(self) -> None:
        self.assertEqual(readiness_response(False), (200, "ok"))
        self.assertEqual(readiness_response(True), (503, "draining"))

    def test_metrics_response(self) -> None:
        self.assertEqual(
            metrics_response(None), (200, "text/plain; version=0.0.4", "")
        )
        self.assertEqual(
            metrics_response("x 1\n"),
            (200, "text/plain; version=0.0.4", "x 1\n"),
        )

    def test_openapi_content_type(self) -> None:
        self.assertEqual(
            openapi_content_type(OpenapiSource.TYPED), "application/json"
        )
        self.assertEqual(
            openapi_content_type(OpenapiSource.CANONICAL_JSON),
            "application/json",
        )
