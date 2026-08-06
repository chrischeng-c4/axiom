from __future__ import annotations

import sys
import unittest
from dataclasses import fields
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_http.application.config import (
    HttpConfig,
    ObservabilityConfig,
    ServiceIdentity,
    bind_addr,
    identity_problem,
    observability_config,
)


class TestApplicationConfig(unittest.TestCase):
    def test_bind_addr(self) -> None:
        cfg = HttpConfig(
            host="0.0.0.0",
            port=8080,
            log_level="info",
            log_format="json",
            grace_secs=30,
            body_limit_bytes=8388608,
        )
        self.assertEqual(bind_addr(cfg), "0.0.0.0:8080")

    def test_observability_config_fields(self) -> None:
        names = tuple(f.name for f in fields(ObservabilityConfig))
        self.assertEqual(
            names, ("log_level", "log_format", "otlp_endpoint")
        )

    def test_observability_config_projection(self) -> None:
        cfg = HttpConfig(
            host="0.0.0.0",
            port=8080,
            log_level="debug",
            log_format="text",
            grace_secs=10,
            body_limit_bytes=1000,
            otlp_endpoint="http://localhost:4317",
        )
        obs = observability_config(cfg)
        self.assertEqual(obs.log_level, "debug")
        self.assertEqual(obs.log_format, "text")
        self.assertEqual(obs.otlp_endpoint, "http://localhost:4317")

    def test_identity_problem_valid(self) -> None:
        id_val = ServiceIdentity("svc", "1.0")
        self.assertIsNone(identity_problem(id_val))

    def test_identity_problem_blank_name(self) -> None:
        id_val = ServiceIdentity("   ", "1.0")
        self.assertEqual(
            identity_problem(id_val), "service name must not be blank"
        )

    def test_identity_problem_blank_version(self) -> None:
        id_val = ServiceIdentity("svc", "\t")
        self.assertEqual(
            identity_problem(id_val), "service version must not be blank"
        )

    def test_identity_problem_blank_both(self) -> None:
        id_val = ServiceIdentity("", "")
        self.assertEqual(
            identity_problem(id_val), "service name must not be blank"
        )

    def test_http_config_optional_otlp(self) -> None:
        cfg = HttpConfig(
            host="127.0.0.1",
            port=3000,
            log_level="info",
            log_format="json",
            grace_secs=5,
            body_limit_bytes=1024,
        )
        self.assertIsNone(cfg.otlp_endpoint)
