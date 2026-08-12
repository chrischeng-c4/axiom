from __future__ import annotations

import pathlib
import sys
import unittest
from types import MappingProxyType

SRC_DIR = pathlib.Path(__file__).parents[2] / "src"
if str(SRC_DIR) not in sys.path:
    sys.path.insert(0, str(SRC_DIR))

from lumen.capability_ownership import (
    FailureSlices,
    GateSequencePlan,
    OwnershipVerdict,
    Reason,
    classify_failure_slices,
    decide_terminal_result,
    ownership_inventory,
    required_gate_sequence,
    validate_ownership_inventory,
)


class TestCapability2324(unittest.TestCase):
    def test_required_gate_sequence_returns_plan(self) -> None:
        plan = required_gate_sequence()
        self.assertIsInstance(plan, GateSequencePlan)
        self.assertEqual(len(plan.commands), 2)
        self.assertTrue(plan.commands[0].startswith("aw capability check"))
        self.assertTrue(plan.commands[1].startswith("aw health"))
        self.assertEqual(
            plan.missing_live_command_policy,
            "require_shared_or_thin_app_wrapper_before_passed",
        )

    def test_ownership_inventory_is_immutable_mapping(self) -> None:
        inv = ownership_inventory()
        self.assertIsInstance(inv, MappingProxyType)
        with self.assertRaises(TypeError):
            inv["test_mutation"] = {"owner": "Lumen-domain"}  # type: ignore[index]
        self.assertIn("auth", inv)
        self.assertIn("search-planner", inv)

    def test_validate_ownership_inventory_admitted(self) -> None:
        verdict = validate_ownership_inventory(ownership_inventory())
        self.assertIsInstance(verdict, OwnershipVerdict)
        self.assertEqual(verdict.reason, Reason.ADMITTED)
        self.assertEqual(verdict.field_path, "")

    def test_validate_ownership_inventory_missing_owner_field_path(self) -> None:
        custom_inv = dict(ownership_inventory())
        custom_inv["observability"] = {
            "owner": "",
            "capability_id": "operations-observability",
            "integration_seam": "service_observability",
        }
        verdict = validate_ownership_inventory(custom_inv)
        self.assertEqual(verdict.reason, Reason.MISSING_CANONICAL_OWNER)
        self.assertEqual(verdict.field_path, "observability.owner")

    def test_validate_ownership_inventory_multiple_canonical_owners_field_path(self) -> None:
        custom_inv = dict(ownership_inventory())
        custom_inv["cli"] = {
            "owner": ("cli-std", "service-auth"),
            "capability_id": "api-cli-agent-integration",
            "integration_seam": "cli_std",
        }
        verdict = validate_ownership_inventory(custom_inv)
        self.assertEqual(verdict.reason, Reason.MULTIPLE_CANONICAL_OWNERS)
        self.assertEqual(verdict.field_path, "cli.owner")

    def test_validate_ownership_inventory_conflicting_owners_field_path(self) -> None:
        custom_inv = dict(ownership_inventory())
        custom_inv["auth"] = {
            "owner": ("service-auth", "Lumen-domain"),
            "capability_id": "security-hardening",
            "integration_seam": "service_auth",
        }
        verdict = validate_ownership_inventory(custom_inv)
        self.assertEqual(verdict.reason, Reason.CONFLICTING_FEATURE_OWNERSHIP)
        self.assertEqual(verdict.field_path, "auth.owner")

    def test_validate_ownership_inventory_missing_integration_seam(self) -> None:
        custom_inv = dict(ownership_inventory())
        custom_inv["peer-identity"] = {
            "owner": "peer-tls",
            "capability_id": "security-hardening",
            "integration_seam": "",
        }
        verdict = validate_ownership_inventory(custom_inv)
        self.assertEqual(verdict.reason, Reason.MISSING_INTEGRATION_SEAM)
        self.assertEqual(verdict.field_path, "peer-identity.integration_seam")

    def test_validate_ownership_inventory_missing_required_concern(self) -> None:
        custom_inv = dict(ownership_inventory())
        del custom_inv["raft-host"]
        verdict = validate_ownership_inventory(custom_inv)
        self.assertEqual(verdict.reason, Reason.MISSING_CANONICAL_OWNER)
        self.assertEqual(verdict.field_path, "raft-host.owner")

    def test_classify_failure_slices_partitions_correctly(self) -> None:
        failures = (
            {"concern": "kubernetes-render", "owner": "service-k8s"},
            {"concern": "index-storage-policy", "owner": "Lumen-domain"},
        )
        res = classify_failure_slices(failures)
        self.assertIsInstance(res, FailureSlices)
        if isinstance(res, FailureSlices):
            self.assertEqual(res.shared_non_domain, ("kubernetes-render",))
            self.assertEqual(res.lumen_domain, ("index-storage-policy",))
            self.assertEqual(res.shared_action, "repair_and_rerun")

    def test_classify_failure_slices_unknown_owner_indexed(self) -> None:
        failures = (
            {"concern": "http", "owner": "service-http"},
            {"concern": "auth", "owner": "service-auth"},
            {"concern": "custom", "owner": "unknown-owner-component"},
        )
        verdict = classify_failure_slices(failures)
        self.assertIsInstance(verdict, OwnershipVerdict)
        if isinstance(verdict, OwnershipVerdict):
            self.assertEqual(verdict.reason, Reason.UNKNOWN_FAILURE_OWNER)
            self.assertEqual(verdict.field_path, "failures[2].owner")

    def test_decide_terminal_result_passed(self) -> None:
        res = decide_terminal_result((), None)
        self.assertEqual(res, "passed")

    def test_decide_terminal_result_tracked_skip_custom_issues(self) -> None:
        res_int = decide_terminal_result(("Lumen-domain",), 9999)
        self.assertEqual(res_int, "tracked_skip(#9999)")

        res_str = decide_terminal_result(("Lumen-domain",), "#7777")
        self.assertEqual(res_str, "tracked_skip(#7777)")

    def test_decide_terminal_result_refuses_shared_failure(self) -> None:
        verdict = decide_terminal_result(("service-k8s",), 9999)
        self.assertIsInstance(verdict, OwnershipVerdict)
        if isinstance(verdict, OwnershipVerdict):
            self.assertEqual(verdict.reason, Reason.SHARED_NON_DOMAIN_FAILURE)
            self.assertEqual(verdict.field_path, "failure_owners")

    def test_decide_terminal_result_refuses_missing_issue(self) -> None:
        verdict = decide_terminal_result(("Lumen-domain",), None)
        self.assertIsInstance(verdict, OwnershipVerdict)
        if isinstance(verdict, OwnershipVerdict):
            self.assertEqual(verdict.reason, Reason.MISSING_BOUNDED_ISSUE)
            self.assertEqual(verdict.field_path, "issue")

    def test_decide_terminal_result_refuses_invalid_issue(self) -> None:
        for malformed in ("unbounded", "issue filed soon", "#abc", "-50"):
            verdict = decide_terminal_result(("Lumen-domain",), malformed)
            self.assertIsInstance(verdict, OwnershipVerdict)
            if isinstance(verdict, OwnershipVerdict):
                self.assertEqual(verdict.reason, Reason.INVALID_BOUNDED_ISSUE)
                self.assertEqual(verdict.field_path, "issue")


if __name__ == "__main__":
    unittest.main()

