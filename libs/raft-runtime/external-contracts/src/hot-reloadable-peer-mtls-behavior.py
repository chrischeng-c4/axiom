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

MINIMUM_CHECKS = 11

HOT_RELOADABLE_PEER_MTLS_BEHAVIOR_MATRIX = (
    ("a_transport_is_built_only_when_peer_mtls_is_required",
     ((1, 'client[ca]', 'server[ca]', (1, 'client[ca]', 'server[ca]', (('ca',),))), ('PeerTlsNotRequired', ()))),
    ("a_freshly_built_transport_starts_at_the_first_generation",
     (1, 'client[ca]', 'server[ca]', (1, 'client[ca]', 'server[ca]', (('ca',),)))),
    ("a_reload_advances_the_generation_by_exactly_one",
     (2, (2, 'client[ca2]', 'server[ca2]', (2, 'client[ca2]', 'server[ca2]', (('ca2',),))))),
    ("each_further_reload_advances_the_generation_again",
     (2, 3, 4, (4, 'client[r3]', 'server[r3]', (4, 'client[r3]', 'server[r3]', (('r3',),))))),
    ("the_snapshot_carries_the_trust_bundle_the_reload_supplied",
     ((2, 'client[new-ca+spare-ca]', 'server[new-ca+spare-ca]', (('new-ca', 'spare-ca'),)), ('new-ca', 'spare-ca'))),
    ("a_trusted_in_window_correctly_named_server_is_accepted",
     'accepted'),
    ("an_inbound_peer_is_not_checked_against_any_pinned_name",
     ('accepted', 'accepted')),
    ("an_outbound_connection_is_checked_against_the_name_it_asked_for",
     ('accepted', 'hostname-mismatch')),
    ("the_acceptance_predicate_agrees_with_the_verdict_it_is_given",
     (True, False, False, False, False)),
    ("every_verdict_has_the_wire_spelling_an_operator_reads",
     ('accepted', 'untrusted-issuer', 'not-yet-valid', 'expired', 'hostname-mismatch')),
    ("the_validity_window_is_closed_at_the_start_and_open_at_the_end",
     ('not-yet-valid', ('NoneType', None), ('NoneType', None), 'expired')),
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


def verify_hot_reloadable_peer_mtls_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a transport is built only when peer mtls is required
    exp1 = HOT_RELOADABLE_PEER_MTLS_BEHAVIOR_MATRIX[0][1]
    obs1 = plain((opened(config(required=True), tracking()),
        opened(config(required=False), tracking())))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a freshly built transport starts at the first generation
    exp2 = HOT_RELOADABLE_PEER_MTLS_BEHAVIOR_MATRIX[1][1]
    obs2 = plain(opened(config(issuers=("ca",)), tracking()))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. a reload advances the generation by exactly one
    exp3 = HOT_RELOADABLE_PEER_MTLS_BEHAVIOR_MATRIX[2][1]
    hot = live(config(issuers=("ca",)), tracking())
    moved = hot.reload(config(issuers=("ca2",)))
    obs3 = plain((moved, state(hot)))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. each further reload advances the generation again
    exp4 = HOT_RELOADABLE_PEER_MTLS_BEHAVIOR_MATRIX[3][1]
    rolling = live(config(issuers=("ca",)), tracking())
    first = rolling.reload(config(issuers=("r1",)))
    second = rolling.reload(config(issuers=("r2",)))
    third = rolling.reload(config(issuers=("r3",)))
    obs4 = plain((first, second, third, state(rolling)))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the snapshot carries the trust bundle the reload supplied
    exp5 = HOT_RELOADABLE_PEER_MTLS_BEHAVIOR_MATRIX[4][1]
    rotated = live(config(issuers=("old-ca",)), tracking())
    rotated.reload(config(issuers=("new-ca", "spare-ca")))
    obs5 = plain((plain(rotated.snapshot()),
        rotated.snapshot().trust.issuers))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. a trusted in window correctly named server is accepted
    exp6 = HOT_RELOADABLE_PEER_MTLS_BEHAVIOR_MATRIX[5][1]
    dialled = live(config(issuers=("ca",)), tracking())
    obs6 = plain(dialled.connect(cert(), "peer", 150))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. an inbound peer is not checked against any pinned name
    exp7 = HOT_RELOADABLE_PEER_MTLS_BEHAVIOR_MATRIX[6][1]
    inbound = live(config(issuers=("ca",)), tracking())
    obs7 = plain((inbound.accept(cert(names=("something-else",)), 150),
        inbound.accept(cert(names=()), 150)))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. an outbound connection is checked against the name it asked for
    exp8 = HOT_RELOADABLE_PEER_MTLS_BEHAVIOR_MATRIX[7][1]
    outbound = live(config(issuers=("ca",)), tracking())
    obs8 = plain((outbound.connect(cert(names=("peer", "alt")), "alt",
        150), outbound.connect(cert(names=("peer",)), "alt", 150)))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the acceptance predicate agrees with the verdict it is given
    exp9 = HOT_RELOADABLE_PEER_MTLS_BEHAVIOR_MATRIX[8][1]
    obs9 = plain((is_accepted(HandshakeOutcome.ACCEPTED),
        is_accepted(HandshakeOutcome.UNTRUSTED_ISSUER),
        is_accepted(HandshakeOutcome.NOT_YET_VALID),
        is_accepted(HandshakeOutcome.EXPIRED),
        is_accepted(HandshakeOutcome.HOSTNAME_MISMATCH)))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. every verdict has the wire spelling an operator reads
    exp10 = HOT_RELOADABLE_PEER_MTLS_BEHAVIOR_MATRIX[9][1]
    obs10 = plain((HandshakeOutcome.ACCEPTED,
        HandshakeOutcome.UNTRUSTED_ISSUER, HandshakeOutcome.NOT_YET_VALID,
        HandshakeOutcome.EXPIRED, HandshakeOutcome.HOSTNAME_MISMATCH))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. the validity window is closed at the start and open at the end
    exp11 = HOT_RELOADABLE_PEER_MTLS_BEHAVIOR_MATRIX[10][1]
    obs11 = plain((named(validity_problem(cert(), 99)),
        named(validity_problem(cert(), 100)),
        named(validity_problem(cert(), 199)),
        named(validity_problem(cert(), 200))))
    checks.append({"name": HOT_RELOADABLE_PEER_MTLS_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    return {
        "case_id": "hot-reloadable-peer-mtls-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
