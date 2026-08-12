"""EC behavior case for #3085 -- release compatibility facts and admission.

Every expected value below is an EC-owned literal transcribed from #3085:
R1 exposes binary, build, release/protocol, public-API, peer, and public
contract facts; R2 inventories every durable and Raft format family; R3/R4
keep the authorized descriptor and observed image digest distinct; R5 admits
only the settled adjacent window; R6 serves the member-contract intersection;
and R7 admits one recorded immutable digest reported by every member.
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
from lumen.release_compatibility.status import (
    project_operator_descriptor,
    project_release_status,
)
from lumen.release_compatibility.verdict import Rejection

MINIMUM_CHECKS = 19

RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX = (
    ("descriptor_preserves_binary_version", "2.1.0"),
    ("descriptor_preserves_build_sha", "a1b2c3d4"),
    ("descriptor_preserves_release_protocol_epoch", 2),
    ("descriptor_preserves_public_api_read_epoch", 1),
    ("descriptor_preserves_public_api_serve_epoch", 2),
    ("descriptor_preserves_peer_release_window", ("2.0.0", "2.1.0")),
    ("descriptor_preserves_public_contract_release_window", ("2.0.0", "2.1.0")),
    ("inventory_preserves_wal_read_write_versions", ((1,), (1,))),
    ("inventory_preserves_segment_read_write_versions", ((1,), (1,))),
    ("inventory_preserves_raft_log_snapshot_command_and_peer_versions", ((1,), (1,), (1,), (1,))),
    ("authorized_projection_contains_only_descriptor_fields", ("binary_version", "build_sha", "format_inventory", "max_peer_release", "max_public_contract_release", "min_peer_release", "min_public_contract_release", "public_api_read_epoch", "public_api_serve_epoch", "release_protocol_epoch")),
    ("status_retains_observed_image_digest_separately", "registry.example/lumen@sha256:1111"),
    ("status_retains_release_fact", "2.1.0"),
    ("status_retains_public_api_facts", (1, 2)),
    ("status_retains_format_fact", ((1,), (1,))),
    ("adjacent_overlapping_releases_are_admitted", "admitted"),
    ("mixed_release_operations_are_the_intersection", ("search",)),
    ("mixed_release_response_fields_are_the_intersection", ("hits",)),
    ("one_recorded_digest_for_every_member_is_admitted", "admitted"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def _descriptor(
    *,
    binary_version: str,
    release_protocol_epoch: int,
    public_api_read_epoch: int,
    public_api_serve_epoch: int,
    min_peer_release: str,
    max_peer_release: str,
    min_public_contract_release: str,
    max_public_contract_release: str,
    format_inventory: FormatCompatibilityInventory,
    public_operations: tuple[str, ...],
    public_response_fields: tuple[str, ...],
) -> ReleaseCompatibilityDescriptor:
    return ReleaseCompatibilityDescriptor.from_spec(
        ReleaseCompatibilitySpec(
            binary_version=binary_version,
            build_sha="a1b2c3d4",
            release_protocol_epoch=release_protocol_epoch,
            public_api_read_epoch=public_api_read_epoch,
            public_api_serve_epoch=public_api_serve_epoch,
            min_peer_release=min_peer_release,
            max_peer_release=max_peer_release,
            min_public_contract_release=min_public_contract_release,
            max_public_contract_release=max_public_contract_release,
            format_inventory=format_inventory,
            public_operations=public_operations,
            public_response_fields=public_response_fields,
        )
    )


def verify_release_compatibility_3085_behavior() -> dict:
    checks = []
    inventory = FormatCompatibilityInventory(
        readable_wal_versions=(1,), writable_wal_versions=(1,),
        readable_segment_versions=(1,), writable_segment_versions=(1,),
        raft_log_versions=(1,), raft_snapshot_versions=(1,),
        raft_command_versions=(1,), peer_protocol_versions=(1,),
    )
    descriptor = _descriptor(
        binary_version="2.1.0", release_protocol_epoch=2,
        public_api_read_epoch=1, public_api_serve_epoch=2,
        min_peer_release="2.0.0", max_peer_release="2.1.0",
        min_public_contract_release="2.0.0", max_public_contract_release="2.1.0",
        format_inventory=inventory, public_operations=("search", "index-v2"),
        public_response_fields=("hits", "ranking-v2"),
    )

    # 1. R1 -- binary provenance is an independently readable runtime fact.
    obs1 = descriptor.binary_version; exp1 = RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    # 2. R1 -- build identity is not inferred from a mutable image tag.
    obs2 = descriptor.build_sha; exp2 = RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    # 3. R1 -- release/protocol compatibility has its own epoch.
    obs3 = descriptor.release_protocol_epoch; exp3 = RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    # 4. R1 -- clients can read the previous public API epoch during rollout.
    obs4 = descriptor.public_api_read_epoch; exp4 = RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    # 5. R1 -- serving epoch remains separately visible from readable epoch.
    obs5 = descriptor.public_api_serve_epoch; exp5 = RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    # 6. R1 -- peer releases have explicit lower and upper compatibility bounds.
    obs6 = (descriptor.min_peer_release, descriptor.max_peer_release); exp6 = RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    # 7. R1 -- public contracts have bounds independent of peer releases.
    obs7 = (descriptor.min_public_contract_release, descriptor.max_public_contract_release); exp7 = RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    # 8. R2 -- WAL read and write versions are both declared.
    obs8 = (inventory.readable_wal_versions, inventory.writable_wal_versions); exp8 = RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    # 9. R2 -- segment read and write versions are also independent facts.
    obs9 = (inventory.readable_segment_versions, inventory.writable_segment_versions); exp9 = RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    # 10. R2 -- every Raft compatibility family is present in the inventory.
    obs10 = (inventory.raft_log_versions, inventory.raft_snapshot_versions, inventory.raft_command_versions, inventory.peer_protocol_versions); exp10 = RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    # 11. R3 -- an authorized projection contains descriptor facts, not credentials.
    projection = project_operator_descriptor(descriptor, authorization="operator")
    obs11 = tuple(sorted(projection)); exp11 = RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    # 12. R4 -- status retains the separately observed Kubernetes digest.
    status = project_release_status("registry.example/lumen@sha256:1111", descriptor)
    obs12 = status.observed_image_digest; exp12 = RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    # 13. R4 -- status also retains a descriptor-derived release fact.
    obs13 = status.binary_version; exp13 = RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    # 14. R4 -- public API facts do not collapse into the release string.
    obs14 = (status.public_api_read_epoch, status.public_api_serve_epoch); exp14 = RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    # 15. R4 -- status preserves durable-format facts separately as well.
    obs15 = (status.format_inventory.readable_wal_versions, status.format_inventory.writable_wal_versions); exp15 = RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})
    # 16. R5 -- the settled N-1/N release window is admitted before cutover.
    previous = _descriptor(binary_version="2.0.0", release_protocol_epoch=1, public_api_read_epoch=1, public_api_serve_epoch=1, min_peer_release="2.0.0", max_peer_release="2.1.0", min_public_contract_release="2.0.0", max_public_contract_release="2.1.0", format_inventory=inventory, public_operations=("search",), public_response_fields=("hits",))
    adjacent = decide_adjacent_release(previous, descriptor)
    obs16 = _outcome(adjacent); exp16 = RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[15][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})
    # 17. R6 -- every admitted member limits the advertised public operations.
    mixed = decide_mixed_release_contract((previous, descriptor))
    obs17 = mixed.advertised_operations; exp17 = RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[16][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})
    # 18. R6 -- every admitted member limits advertised response fields too.
    obs18 = mixed.advertised_response_fields; exp18 = RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[17][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})
    # 19. R7 -- one immutable digest across staged/replacement members admits.
    digest = decide_target_digest("sha256:1111", ("sha256:1111", "sha256:1111"))
    obs19 = _outcome(digest); exp19 = RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[18][1]
    checks.append({"name": RELEASE_COMPATIBILITY_3085_BEHAVIOR_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})

    return {"case_id": "release-compatibility-3085-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
