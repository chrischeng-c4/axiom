from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_http.application.probes import ProbeState, handle_probe
from service_http.infrastructure.routes import OpenapiSource, is_probe_path


class TestApplicationProbes(unittest.TestCase):
    def test_handle_probe_agreement_with_is_probe_path(self) -> None:
        state = ProbeState(
            draining=False,
            metrics_text=None,
            openapi_document="{}",
            openapi_source=OpenapiSource.TYPED,
            docs_html="<h1>Docs</h1>",
        )
        paths = [
            "/healthz",
            "/readyz",
            "/metrics",
            "/openapi.json",
            "/docs",
            "/healthz/",
            "/v1/query",
            "",
        ]
        for path in paths:
            resp = handle_probe(state, path)
            self.assertEqual(resp is not None, is_probe_path(path))

    def test_healthz(self) -> None:
        state = ProbeState(
            draining=False,
            metrics_text=None,
            openapi_document="{}",
            openapi_source=OpenapiSource.TYPED,
            docs_html="<h1>Docs</h1>",
        )
        resp = handle_probe(state, "/healthz")
        self.assertIsNotNone(resp)
        self.assertEqual(resp.status, 200)
        self.assertEqual(resp.body, "ok")
        self.assertEqual(resp.content_type, "text/plain; charset=utf-8")

    def test_readyz_both_states(self) -> None:
        s_ok = ProbeState(
            draining=False,
            metrics_text=None,
            openapi_document="{}",
            openapi_source=OpenapiSource.TYPED,
            docs_html="<h1>Docs</h1>",
        )
        r_ok = handle_probe(s_ok, "/readyz")
        self.assertIsNotNone(r_ok)
        self.assertEqual(r_ok.status, 200)
        self.assertEqual(r_ok.body, "ok")

        s_drain = ProbeState(
            draining=True,
            metrics_text=None,
            openapi_document="{}",
            openapi_source=OpenapiSource.TYPED,
            docs_html="<h1>Docs</h1>",
        )
        r_drain = handle_probe(s_drain, "/readyz")
        self.assertIsNotNone(r_drain)
        self.assertEqual(r_drain.status, 503)
        self.assertEqual(r_drain.body, "draining")

    def test_metrics_both_states(self) -> None:
        s_none = ProbeState(
            draining=False,
            metrics_text=None,
            openapi_document="{}",
            openapi_source=OpenapiSource.TYPED,
            docs_html="<h1>Docs</h1>",
        )
        r_none = handle_probe(s_none, "/metrics")
        self.assertIsNotNone(r_none)
        self.assertEqual(r_none.status, 200)
        self.assertEqual(r_none.body, "")
        self.assertEqual(r_none.content_type, "text/plain; version=0.0.4")

        s_text = ProbeState(
            draining=False,
            metrics_text="m 1\n",
            openapi_document="{}",
            openapi_source=OpenapiSource.TYPED,
            docs_html="<h1>Docs</h1>",
        )
        r_text = handle_probe(s_text, "/metrics")
        self.assertIsNotNone(r_text)
        self.assertEqual(r_text.status, 200)
        self.assertEqual(r_text.body, "m 1\n")

    def test_openapi_verbatim(self) -> None:
        doc = '{"openapi": "3.0.0"}'
        for src in (OpenapiSource.TYPED, OpenapiSource.CANONICAL_JSON):
            state = ProbeState(
                draining=False,
                metrics_text=None,
                openapi_document=doc,
                openapi_source=src,
                docs_html="<h1>Docs</h1>",
            )
            resp = handle_probe(state, "/openapi.json")
            self.assertIsNotNone(resp)
            self.assertEqual(resp.status, 200)
            self.assertEqual(resp.body, doc)
            self.assertEqual(resp.content_type, "application/json")

    def test_docs(self) -> None:
        state = ProbeState(
            draining=False,
            metrics_text=None,
            openapi_document="{}",
            openapi_source=OpenapiSource.TYPED,
            docs_html="<h1>Docs</h1>",
        )
        resp = handle_probe(state, "/docs")
        self.assertIsNotNone(resp)
        self.assertEqual(resp.status, 200)
        self.assertEqual(resp.body, "<h1>Docs</h1>")
        self.assertEqual(resp.content_type, "text/html; charset=utf-8")
