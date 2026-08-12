"""EC security case for #3085 -- fail-closed release compatibility decisions.

Expected literals are transcribed from #3085: R3 refuses unauthorized descriptor
access; R5 names and locates skipped binary, public-API, peer, and durable
format incompatibility; R6 keeps N-only public surface inactive; and R7 treats
a different member digest as a compatibility failure, never another target.
"""

from __future__ import annotations

from lumen.release_compatibility.admission import (
    decide_adjacent_release,
    decide_mixed_release_contract,
    decide_target_digest,
)
from lumen.release_compatibility.spec import (
    FormatCompatibilityInventory,
    ReleaseCompatibilityDescriptor,
    ReleaseCompatibilitySpec,
)
from lumen.release_compatibility.status import project_operator_descriptor
from lumen.release_compatibility.verdict import Rejection

MINIMUM_CHECKS = 15

RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX = (
    ("unauthorized_descriptor_projection_is_refused", "unauthorized"),
    ("unauthorized_descriptor_projection_names_authorization", "authorization"),
    ("skipped_binary_epoch_is_rejected", "skipped_binary_epoch"),
    ("skipped_binary_epoch_names_release_protocol_epoch", "release_protocol_epoch"),
    ("non_overlapping_public_api_epoch_is_rejected", "non_overlapping_public_api_epoch"),
    ("non_overlapping_public_api_epoch_names_public_api", "public_api"),
    ("non_overlapping_peer_epoch_is_rejected", "non_overlapping_peer_epoch"),
    ("non_overlapping_peer_epoch_names_peer_release", "peer_release"),
    ("non_overlapping_durable_format_epoch_is_rejected", "non_overlapping_durable_format_epoch"),
    ("non_overlapping_durable_format_epoch_names_format_inventory", "format_inventory"),
    ("adjacent_overlapping_neighbour_remains_admitted", "admitted"),
    ("n_only_operations_remain_inactive_with_an_n_minus_one_member", ("index-v2",)),
    ("n_only_response_fields_remain_inactive_with_an_n_minus_one_member", ("ranking-v2",)),
    ("different_member_digest_is_a_compatibility_failure", "digest_mismatch"),
    ("different_member_digest_names_reported_member_digests", "reported_member_digests"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def _descriptor(*, epoch: int = 2, api_read: int = 1, api_serve: int = 2, peer_window: tuple[str, str] = ("2.0.0", "2.1.0"), format_versions: tuple[int, ...] = (1,)) -> ReleaseCompatibilityDescriptor:
    inventory = FormatCompatibilityInventory(
        readable_wal_versions=format_versions, writable_wal_versions=format_versions,
        readable_segment_versions=format_versions, writable_segment_versions=format_versions,
        raft_log_versions=format_versions, raft_snapshot_versions=format_versions,
        raft_command_versions=format_versions, peer_protocol_versions=format_versions,
    )
    return ReleaseCompatibilityDescriptor.from_spec(ReleaseCompatibilitySpec(
        binary_version="2.1.0", build_sha="a1b2c3d4", release_protocol_epoch=epoch,
        public_api_read_epoch=api_read, public_api_serve_epoch=api_serve,
        min_peer_release=peer_window[0], max_peer_release=peer_window[1],
        min_public_contract_release="2.0.0", max_public_contract_release="2.1.0",
        format_inventory=inventory, public_operations=("search", "index-v2"),
        public_response_fields=("hits", "ranking-v2"),
    ))


def verify_release_compatibility_3085_security() -> dict:
    checks = []
    current = _descriptor(epoch=1, api_read=1, api_serve=1)
    target = _descriptor()

    # 1-2. R3 -- descriptor access preserves authorization and fails closed.
    unauthorized = project_operator_descriptor(target, authorization="anonymous")
    obs1 = _outcome(unauthorized); exp1 = RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[0][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = unauthorized.field_path if isinstance(unauthorized, Rejection) else ""; exp2 = RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[1][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    # 3-4. R5 -- skipping from N-1 to N+2 is not an adjacent release.
    skipped_binary = decide_adjacent_release(current, _descriptor(epoch=3))
    obs3 = _outcome(skipped_binary); exp3 = RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[2][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    obs4 = skipped_binary.field_path if isinstance(skipped_binary, Rejection) else ""; exp4 = RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[3][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    # 5-6. R5 -- a public API window with no shared epoch must be refused.
    no_public_api = decide_adjacent_release(current, _descriptor(api_read=3, api_serve=3))
    obs5 = _outcome(no_public_api); exp5 = RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[4][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = no_public_api.field_path if isinstance(no_public_api, Rejection) else ""; exp6 = RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[5][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    # 7-8. R5 -- peer release bounds must overlap as well.
    no_peer = decide_adjacent_release(current, _descriptor(peer_window=("3.0.0", "3.1.0")))
    obs7 = _outcome(no_peer); exp7 = RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[6][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    obs8 = no_peer.field_path if isinstance(no_peer, Rejection) else ""; exp8 = RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[7][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    # 9-10. R5 -- a durable-format overlap is independently required.
    no_format = decide_adjacent_release(current, _descriptor(format_versions=(2,)))
    obs9 = _outcome(no_format); exp9 = RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[8][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    obs10 = no_format.field_path if isinstance(no_format, Rejection) else ""; exp10 = RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[9][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    # 11. R5 -- the neighbouring overlapping N-1/N input remains admitted.
    adjacent = decide_adjacent_release(current, target)
    obs11 = _outcome(adjacent); exp11 = RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[10][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    # 12-13. R6 -- N-only surface is neither advertised nor active while N-1 serves.
    previous = _descriptor(epoch=1, api_read=1, api_serve=1)
    mixed = decide_mixed_release_contract((previous, target))
    obs12 = mixed.inactive_operations; exp12 = RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[11][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    obs13 = mixed.inactive_response_fields; exp13 = RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[12][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    # 14-15. R7 -- a different tag resolution is a named compatibility failure.
    mismatch = decide_target_digest("sha256:1111", ("sha256:1111", "sha256:2222"))
    obs14 = _outcome(mismatch); exp14 = RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[13][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    obs15 = mismatch.field_path if isinstance(mismatch, Rejection) else ""; exp15 = RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[14][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    return {"case_id": "release-compatibility-3085-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
