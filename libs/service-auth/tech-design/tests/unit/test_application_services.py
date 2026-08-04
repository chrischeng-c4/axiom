"""Unit tests for application layer authorization, reload, and delegation services."""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

SRC_ROOT = Path(__file__).resolve().parents[2] / "src"
if str(SRC_ROOT) not in sys.path:
    sys.path.insert(0, str(SRC_ROOT))

from service_auth.application.authorize_request import (
    AuthorizeRequest,
    authorize,
    principal_for_bearer,
)
from service_auth.application.delegate_authorization import (
    AuthRejection,
    DelegatedCache,
    DelegatedOutcome,
    MissingAudienceError,
    authorize_delegated,
    fingerprint,
    judge,
    make_config,
)
from service_auth.application.reload_registry import (
    ReloadableRegistry,
    reload_documents,
)
from service_auth.domain.audit import (
    RegistryReloadEvent,
)
from service_auth.domain.cache_policy import CachePolicy
from service_auth.domain.claims import TokenClaims
from service_auth.domain.principal import (
    AuthorizationOutcome,
    DenialReason,
    TokenPrincipal,
)
from service_auth.domain.registry import Registry
from service_auth.domain.review import (
    AccessReviewOutcome,
    ResourceAttributes,
    TokenReviewOutcome,
)
from service_auth.domain.role import Role
from service_auth.domain.service_account import (
    PrincipalRejection,
    ReviewedIdentity,
    ServiceAccountRef,
)
from service_auth.infrastructure.manual_clock import ManualClock
from service_auth.infrastructure.memory_registry_source import MemoryRegistrySource
from service_auth.infrastructure.memory_review_backend import MemoryReviewBackend
from service_auth.infrastructure.recording_event_sink import RecordingEventSink


class TestApplicationServices(unittest.TestCase):
    def test_authorize_request_flow(self) -> None:
        reg = Registry(
            tokens={"secret1": TokenClaims("svc1", {"res1": Role.READ})},
            identities={},
        )
        svc = AuthorizeRequest(registry=reg, auth_required=True)
        sink = RecordingEventSink()

        princ = principal_for_bearer(svc, "unknown")
        self.assertEqual(princ, DenialReason.UNKNOWN_BEARER)

        princ = principal_for_bearer(svc, "secret1")
        self.assertIsInstance(princ, TokenPrincipal)
        assert isinstance(princ, TokenPrincipal)
        outcome = authorize(svc, princ, "res1", Role.READ, sink)
        self.assertEqual(outcome, AuthorizationOutcome.ALLOW)
        self.assertEqual(len(sink.events), 1)

    def test_reload_documents_flow(self) -> None:
        state = ReloadableRegistry(auth_required=True)
        sink = RecordingEventSink()
        doc1 = json.dumps(
            {"tokens": {"sec1": {"subject": "sub1", "roles": {"*": "read"}}}}
        )
        sources = [MemoryRegistrySource("s1", doc1)]

        err = reload_documents(state, sources, sink)
        self.assertIsNone(err)
        self.assertEqual(state.revision, 1)
        self.assertEqual(state.registry.len(), 1)
        self.assertEqual(len(sink.events), 1)
        event = sink.events[0]
        self.assertIsInstance(event, RegistryReloadEvent)
        assert isinstance(event, RegistryReloadEvent)
        self.assertTrue(event.applied)
        self.assertEqual(event.entries, 1)

    def test_delegated_authorization_judge_order(self) -> None:
        policy = CachePolicy()
        config = make_config(["aud1"], policy)

        identity = ReviewedIdentity("system:serviceaccount:ns:name", "1", (), ())
        review_unauth = TokenReviewOutcome(False, identity, ("aud1",))
        self.assertEqual(
            judge(config, review_unauth), PrincipalRejection.NOT_AUTHENTICATED
        )

        review_bad_aud = TokenReviewOutcome(True, identity, ("wrong_aud",))
        self.assertEqual(judge(config, review_bad_aud), AuthRejection.AUDIENCE_MISMATCH)

        review_ok = TokenReviewOutcome(True, identity, ("aud1",))
        verdict = judge(config, review_ok)
        self.assertIsInstance(verdict, ServiceAccountRef)

    def test_make_config_refuses_empty_audiences(self) -> None:
        with self.assertRaises(MissingAudienceError):
            make_config([], CachePolicy())
        with self.assertRaises(MissingAudienceError):
            make_config(["  "], CachePolicy())

    def test_authorize_delegated_end_to_end(self) -> None:
        policy = CachePolicy()
        config = make_config(["aud1"], policy)
        clock = ManualClock(now=1000)
        cache = DelegatedCache(policy=policy)
        backend = MemoryReviewBackend()

        identity = ReviewedIdentity("system:serviceaccount:ns:name", "1", (), ())
        attributes = ResourceAttributes("apps", "ns", "deployments", "web", "get")
        backend.tokens["tok1"] = TokenReviewOutcome(True, identity, ("aud1",))
        backend.access[(identity, attributes)] = AccessReviewOutcome(True, False)

        outcome = authorize_delegated(
            config, backend, clock, cache, "tok1", attributes
        )
        self.assertEqual(outcome, DelegatedOutcome.AUTHENTICATED)
        self.assertEqual(backend.review_calls, 2)

        outcome2 = authorize_delegated(
            config, backend, clock, cache, "tok1", attributes
        )
        self.assertEqual(outcome2, DelegatedOutcome.AUTHENTICATED)
        self.assertEqual(backend.review_calls, 2)

    def test_fingerprint(self) -> None:
        fp = fingerprint("tok1")
        self.assertEqual(len(fp), 12)


if __name__ == "__main__":
    unittest.main()
