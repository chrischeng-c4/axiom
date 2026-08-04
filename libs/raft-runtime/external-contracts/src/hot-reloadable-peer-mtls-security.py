from __future__ import annotations

from raft_runtime.application.peer_transport import (
    PeerTransport,
    UnusableMaterial,
)
from raft_runtime.domain.peer_tls import (
    HandshakeOutcome,
    PeerCertificate,
    PeerTlsConfig,
    TrustBundle,
    is_accepted,
    is_trusted,
    matches_identity,
    validity_problem,
    verify_peer,
)

MINIMUM_CHECKS = 13

HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX = (
    ("an_untrusted_issuer_is_refused_before_validity_is_consulted",
     ('untrusted-issuer', 'untrusted-issuer')),
    ("validity_is_settled_before_the_name_is_looked_at",
     ('expired', 'not-yet-valid')),
    ("too_early_and_too_late_are_two_different_verdicts",
     (False, 'not-yet-valid', 'expired')),
    ("a_certificate_one_millisecond_early_is_not_yet_usable",
     ('not-yet-valid', 'accepted')),
    ("a_certificate_is_refused_from_the_instant_it_expires",
     ('accepted', 'expired', 'expired')),
    ("identity_matching_is_case_sensitive",
     (True, False, False)),
    ("a_literal_wildcard_entry_does_not_match_a_concrete_name",
     (False, True, False)),
    ("an_empty_trust_bundle_trusts_nobody",
     (False, False, True, 'untrusted-issuer')),
    ("unusable_key_material_stops_the_transport_being_built",
     ('UnusableMaterial', ('no private key',))),
    ("a_reload_that_cannot_build_leaves_the_working_generation_in_place",
     (('UnusableMaterial', ('key rotated away',)), (1, 'client[first]', 'server[first]', (1, 'client[first]', 'server[first]', (('ca',),))))),
    ("a_reload_cannot_switch_peer_mtls_off",
     (('PeerTlsNotRequired', ()), (1, 'client[ca]', 'server[ca]', (1, 'client[ca]', 'server[ca]', (('ca',),))))),
    ("a_reported_new_generation_means_the_material_really_was_replaced",
     ((1, 'client[ca]', 'server[ca]', (1, 'client[ca]', 'server[ca]', (('ca',),))), (2, 'client[rotated]', 'server[rotated]', (2, 'client[rotated]', 'server[rotated]', (('rotated',),))), False, 2)),
    ("every_refusal_verdict_answers_no_to_the_acceptance_predicate",
     (False, False, False, False, True)),
)


def plain(value: object) -> object:
    """A literal-shaped view: records by their fields, enum members by value.

    An expected value has to be a plain literal, and `repr` of a dataclass or
    an enum member is not one. Reading a record as the tuple of its fields
    keeps every field observable while staying transcribable.
    """
    fields = getattr(type(value), "__dataclass_fields__", None)
    if fields is not None:
        return tuple(plain(getattr(value, n)) for n in fields)
    if getattr(type(value), "__members__", None) is not None:
        return plain(value.value)
    if isinstance(value, tuple):
        return tuple(plain(v) for v in value)
    if isinstance(value, list):
        return [plain(v) for v in value]
    if isinstance(value, dict):
        return {k: plain(v) for k, v in value.items()}
    return value


def named(value: object) -> object:
    """A record as (record name, fields); a verdict keeps its wire value.

    `PeerTlsNotRequired` has no fields, so its plain view is the empty tuple
    and it would be indistinguishable from any other field-less refusal.
    Naming the record is what keeps two different refusals apart.
    """
    if isinstance(value, HandshakeOutcome):
        return value.value
    return (type(value).__name__, plain(value))


def cert(issuer: str = "ca", names: tuple[str, ...] = ("peer",),
         start: int = 100, end: int = 200) -> PeerCertificate:
    """A peer certificate with a stated issuer, names and validity window."""
    return PeerCertificate(subject="cn=peer", issuer=issuer, dns_names=names,
                           not_before_ms=start, not_after_ms=end)


def config(required: bool = True,
           issuers: tuple[str, ...] = ("ca",)) -> PeerTlsConfig:
    """A peer TLS configuration over one trust bundle."""
    return PeerTlsConfig(required=required, trust=TrustBundle(issuers=issuers),
                         client_cert=cert(), server_cert=cert())


def tracking() -> object:
    """A builder whose handles name the trust bundle it was handed.

    The handles have to depend on the configuration, or a reload that never
    replaced the material would look exactly like one that did.
    """
    def build(cfg: PeerTlsConfig) -> object:
        joined = "+".join(cfg.trust.issuers)
        return ("client[" + joined + "]", "server[" + joined + "]")
    return build


def refusing(reason: str) -> object:
    """A builder that always refuses, with a stated reason."""
    def build(cfg: PeerTlsConfig) -> object:
        return UnusableMaterial(reason=reason)
    return build


def flaky(reason: str) -> object:
    """A builder that works once and refuses every time after that."""
    calls: list[int] = []

    def build(cfg: PeerTlsConfig) -> object:
        calls.append(1)
        if len(calls) == 1:
            return ("client[first]", "server[first]")
        return UnusableMaterial(reason=reason)
    return build


def state(transport: object) -> object:
    """Everything a caller can observe about a live transport."""
    return (transport.generation(), transport.client_handle(),
            transport.server_handle(), plain(transport.snapshot()))


def opened(cfg: PeerTlsConfig, make: object) -> object:
    """A transport's observable state, or the refusal that stopped it."""
    result = PeerTransport.from_config(cfg, make)
    if isinstance(result, PeerTransport):
        return state(result)
    return named(result)


def live(cfg: PeerTlsConfig, make: object) -> object:
    """A transport that was built successfully; only for rows that need one."""
    result = PeerTransport.from_config(cfg, make)
    return result


def verify_hot_reloadable_peer_mtls_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. an untrusted issuer is refused before validity is consulted
    exp1 = HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[0][1]
    obs1 = plain((verify_peer(cert(issuer="rogue", start=1000, end=2000),
        TrustBundle(issuers=("ca",)), "peer", 150),
        verify_peer(cert(issuer="rogue"), TrustBundle(issuers=("ca",)),
        "peer", 9999)))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. validity is settled before the name is looked at
    exp2 = HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[1][1]
    obs2 = plain((verify_peer(cert(names=("other",)),
        TrustBundle(issuers=("ca",)), "peer", 9999),
        verify_peer(cert(names=("other",)), TrustBundle(issuers=("ca",)),
        "peer", 1)))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. too early and too late are two different verdicts
    exp3 = HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[2][1]
    obs3 = plain((HandshakeOutcome.NOT_YET_VALID ==
        HandshakeOutcome.EXPIRED, named(validity_problem(cert(), 1)),
        named(validity_problem(cert(), 999))))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. a certificate one millisecond early is not yet usable
    exp4 = HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[3][1]
    obs4 = plain((verify_peer(cert(), TrustBundle(issuers=("ca",)), None,
        99), verify_peer(cert(), TrustBundle(issuers=("ca",)), None,
        100)))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. a certificate is refused from the instant it expires
    exp5 = HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[4][1]
    obs5 = plain((verify_peer(cert(), TrustBundle(issuers=("ca",)), None,
        199), verify_peer(cert(), TrustBundle(issuers=("ca",)), None,
        200), verify_peer(cert(), TrustBundle(issuers=("ca",)), None,
        201)))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. identity matching is case sensitive
    exp6 = HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[5][1]
    obs6 = plain((matches_identity(cert(names=("peer",)), "peer"),
        matches_identity(cert(names=("peer",)), "Peer"),
        matches_identity(cert(names=("PEER",)), "peer")))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. a literal wildcard entry does not match a concrete name
    exp7 = HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[6][1]
    obs7 = plain((matches_identity(cert(names=("*.svc",)), "raft-0.svc"),
        matches_identity(cert(names=("*.svc",)), "*.svc"),
        matches_identity(cert(names=("peer",)), "")))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. an empty trust bundle trusts nobody
    exp8 = HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[7][1]
    obs8 = plain((is_trusted(cert(), TrustBundle(issuers=())),
        is_trusted(cert(), TrustBundle(issuers=("other",))),
        is_trusted(cert(), TrustBundle(issuers=("other", "ca"))),
        verify_peer(cert(), TrustBundle(issuers=()), None, 150)))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. unusable key material stops the transport being built
    exp9 = HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[8][1]
    obs9 = plain(opened(config(), refusing("no private key")))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. a reload that cannot build leaves the working generation in place
    exp10 = HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[9][1]
    stubborn = live(config(issuers=("ca",)), flaky("key rotated away"))
    rejected = stubborn.reload(config(issuers=("ca2",)))
    obs10 = plain((named(rejected), state(stubborn)))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. a reload cannot switch peer mtls off
    exp11 = HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[10][1]
    pinned = live(config(issuers=("ca",)), tracking())
    refused = pinned.reload(config(required=False, issuers=("ca2",)))
    obs11 = plain((named(refused), state(pinned)))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. a reported new generation means the material really was replaced
    exp12 = HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[11][1]
    swapped = live(config(issuers=("ca",)), tracking())
    before = state(swapped)
    generation = swapped.reload(config(issuers=("rotated",)))
    after = state(swapped)
    obs12 = plain((before, after, before == after, generation))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. every refusal verdict answers no to the acceptance predicate
    exp13 = HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[12][1]
    obs13 = plain((is_accepted(verify_peer(cert(issuer="rogue"),
        TrustBundle(issuers=("ca",)), None, 150)),
        is_accepted(verify_peer(cert(), TrustBundle(issuers=("ca",)),
        None, 1)), is_accepted(verify_peer(cert(),
        TrustBundle(issuers=("ca",)), None, 999)),
        is_accepted(verify_peer(cert(), TrustBundle(issuers=("ca",)),
        "nope", 150)), is_accepted(verify_peer(cert(),
        TrustBundle(issuers=("ca",)), "peer", 150))))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_SECURITY_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    return {
        "case_id": "hot-reloadable-peer-mtls-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
