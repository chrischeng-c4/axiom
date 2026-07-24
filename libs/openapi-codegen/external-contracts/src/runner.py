"""Execute the independent openapi-codegen external-contract oracle."""

from __future__ import annotations

import json
import sys
from pathlib import Path

from active_reference_boundary import EXPECTED_CHECK_IDS as REFERENCE_CHECKS
from consumer_boundary import EXPECTED_CHECK_IDS as CONSUMER_CHECKS
from evidence_schema import build_evidence, validate_evidence, write_evidence
from git_version_boundary import EXPECTED_CHECK_IDS as IDENTITY_CHECKS
from repository_oracle import (
    check_active_references,
    check_consumers,
    check_identity,
    check_target_matrix,
)
from target_matrix_boundary import EXPECTED_CHECK_IDS as MATRIX_CHECKS


EC_ROOT = Path(__file__).resolve().parent.parent
REPO_ROOT = EC_ROOT.parents[2]

MODES = {
    "identity": (IDENTITY_CHECKS, check_identity, EC_ROOT / "evidence/identity.json"),
    "references": (
        REFERENCE_CHECKS,
        check_active_references,
        EC_ROOT / "evidence/references.json",
    ),
    "matrix": (MATRIX_CHECKS, check_target_matrix, EC_ROOT / "evidence/matrix.json"),
    "consumers": (
        CONSUMER_CHECKS,
        check_consumers,
        EC_ROOT / "evidence/consumers.json",
    ),
}


def main(argv: list[str]) -> int:
    if len(argv) != 2 or argv[1] not in MODES:
        print(f"usage: {argv[0]} <{'|'.join(MODES)}>", file=sys.stderr)
        return 2

    mode = argv[1]
    expected_ids, verifier, evidence_path = MODES[mode]
    checks = verifier(REPO_ROOT)
    evidence = build_evidence(mode, checks)
    write_evidence(evidence_path, evidence)
    findings = validate_evidence(evidence, expected_ids)
    print(json.dumps(evidence, indent=2, sort_keys=True))
    if findings:
        for finding in findings:
            print(f"oracle: {finding}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
