from __future__ import annotations

import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from service_k8s.infrastructure.resize import (
    SHRINK_DETAIL,
    PvcFacts,
    PvcResizeOutcome,
    QuantityError,
    ResizeAction,
    ResizeKind,
    decide,
    parse_storage_bytes,
    plan_resize,
    storage_patch,
)


class TestInfrastructureResize(unittest.TestCase):
    # --- 5.1 parse_storage_bytes (18) ---
    def test_parse_20Gi(self) -> None:
        self.assertEqual(parse_storage_bytes("20Gi"), 21474836480)

    def test_parse_500Mi(self) -> None:
        self.assertEqual(parse_storage_bytes("500Mi"), 524288000)

    def test_parse_1Ti(self) -> None:
        self.assertEqual(parse_storage_bytes("1Ti"), 1099511627776)

    def test_parse_1Ki(self) -> None:
        self.assertEqual(parse_storage_bytes("1Ki"), 1024)

    def test_parse_2G(self) -> None:
        self.assertEqual(parse_storage_bytes("2G"), 2000000000)

    def test_parse_1k(self) -> None:
        self.assertEqual(parse_storage_bytes("1k"), 1000)

    def test_parse_1G_and_1Gi_differ(self) -> None:
        val_g = parse_storage_bytes("1G")
        val_gi = parse_storage_bytes("1Gi")
        self.assertEqual(val_g, 1000000000)
        self.assertEqual(val_gi, 1073741824)
        self.assertNotEqual(val_g, val_gi)

    def test_parse_bare_integer(self) -> None:
        self.assertEqual(parse_storage_bytes("1024"), 1024)

    def test_parse_fractional_gi(self) -> None:
        self.assertEqual(parse_storage_bytes("1.5Gi"), 1610612736)

    def test_parse_space_between_head_and_suffix(self) -> None:
        self.assertEqual(parse_storage_bytes("20 Gi"), 21474836480)

    def test_parse_outer_whitespace(self) -> None:
        self.assertEqual(parse_storage_bytes("  20Gi  "), 21474836480)

    def test_parse_empty_string_raises(self) -> None:
        with self.assertRaises(QuantityError):
            parse_storage_bytes("")

    def test_parse_banana_raises(self) -> None:
        with self.assertRaises(QuantityError):
            parse_storage_bytes("banana")

    def test_parse_negative_suffixed_raises(self) -> None:
        with self.assertRaises(QuantityError):
            parse_storage_bytes("-5Gi")

    def test_parse_negative_bare_raises(self) -> None:
        with self.assertRaises(QuantityError):
            parse_storage_bytes("-5")

    def test_parse_fractional_bare_raises(self) -> None:
        with self.assertRaises(QuantityError):
            parse_storage_bytes("10.5")

    def test_parse_suffix_only_raises(self) -> None:
        with self.assertRaises(QuantityError):
            parse_storage_bytes("Gi")

    def test_parse_uppercase_k_raises(self) -> None:
        with self.assertRaises(QuantityError):
            parse_storage_bytes("1K")

    # --- 5.2 decide (8) ---
    def test_decide_grow(self) -> None:
        act = decide("20Gi", "30Gi")
        self.assertEqual(act.kind, ResizeKind.GROW)
        self.assertEqual(act.current_bytes, 21474836480)
        self.assertEqual(act.desired_bytes, 32212254720)

    def test_decide_noop(self) -> None:
        act = decide("20Gi", "20Gi")
        self.assertEqual(act.kind, ResizeKind.NOOP)
        self.assertEqual(act.detail, "already at desired size")

    def test_decide_shrink(self) -> None:
        act = decide("20Gi", "10Gi")
        self.assertEqual(act.kind, ResizeKind.SHRINK_UNSUPPORTED)
        self.assertEqual(act.current_bytes, 21474836480)
        self.assertEqual(act.desired_bytes, 10737418240)
        self.assertEqual(act.detail, SHRINK_DETAIL)

    def test_decide_unit_conversion_noop(self) -> None:
        act = decide("20Gi", "20480Mi")
        self.assertEqual(act.kind, ResizeKind.NOOP)

    def test_decide_unparseable_current(self) -> None:
        act = decide("not-a-size", "20Gi")
        self.assertEqual(act.kind, ResizeKind.UNPARSEABLE)
        self.assertIn("current", act.detail)

    def test_decide_unparseable_desired(self) -> None:
        act = decide("20Gi", "not-a-size")
        self.assertEqual(act.kind, ResizeKind.UNPARSEABLE)
        self.assertIn("desired", act.detail)

    def test_decide_both_unparseable_mentions_current(self) -> None:
        act = decide("not-a-size", "also-bad")
        self.assertEqual(act.kind, ResizeKind.UNPARSEABLE)
        self.assertIn("current", act.detail)

    def test_decide_never_raises(self) -> None:
        try:
            act1 = decide("invalid", "invalid")
            act2 = decide("", "")
            self.assertEqual(act1.kind, ResizeKind.UNPARSEABLE)
            self.assertEqual(act2.kind, ResizeKind.UNPARSEABLE)
        except Exception as e:
            self.fail(f"decide raised an exception: {e}")

    # --- 5.3 plan_resize (12) ---
    def test_plan_resize_filtered_out_pvc_no_outcome(self) -> None:
        pvc = PvcFacts("pvc-ignored", "20Gi", "fast")
        outcomes = plan_resize(
            pvcs=(pvc,),
            name_filter=lambda name: False,
            desired_storage=lambda name: "30Gi",
            allow_expansion={"fast": True},
            dry_run=False,
        )
        self.assertEqual(len(outcomes), 0)

    def test_plan_resize_grow_expandable_no_dry_run(self) -> None:
        pvc = PvcFacts("pvc-1", "20Gi", "fast")
        outcomes = plan_resize(
            pvcs=(pvc,),
            name_filter=lambda name: True,
            desired_storage=lambda name: "30Gi",
            allow_expansion={"fast": True},
            dry_run=False,
        )
        self.assertEqual(len(outcomes), 1)
        out = outcomes[0]
        self.assertTrue(out.patched)
        self.assertEqual(out.detail, "patched spec.resources.requests.storage")

    def test_plan_resize_grow_expandable_dry_run(self) -> None:
        pvc = PvcFacts("pvc-1", "20Gi", "fast")
        outcomes = plan_resize(
            pvcs=(pvc,),
            name_filter=lambda name: True,
            desired_storage=lambda name: "30Gi",
            allow_expansion={"fast": True},
            dry_run=True,
        )
        self.assertEqual(len(outcomes), 1)
        out = outcomes[0]
        self.assertFalse(out.patched)
        self.assertEqual(
            out.detail, "dry run: would patch spec.resources.requests.storage"
        )

    def test_plan_resize_grow_non_expandable_class(self) -> None:
        pvc = PvcFacts("pvc-1", "20Gi", "slow")
        outcomes = plan_resize(
            pvcs=(pvc,),
            name_filter=lambda name: True,
            desired_storage=lambda name: "30Gi",
            allow_expansion={"slow": False},
            dry_run=False,
        )
        self.assertEqual(len(outcomes), 1)
        out = outcomes[0]
        self.assertFalse(out.patched)
        self.assertIn("StorageClass 'slow' does not allow", out.detail)

    def test_plan_resize_grow_absent_storage_class(self) -> None:
        pvc = PvcFacts("pvc-1", "20Gi", "unknown")
        outcomes = plan_resize(
            pvcs=(pvc,),
            name_filter=lambda name: True,
            desired_storage=lambda name: "30Gi",
            allow_expansion={},
            dry_run=False,
        )
        self.assertEqual(len(outcomes), 1)
        out = outcomes[0]
        self.assertFalse(out.patched)
        self.assertIn("StorageClass 'unknown' does not allow", out.detail)

    def test_plan_resize_grow_none_storage_class(self) -> None:
        pvc = PvcFacts("pvc-1", "20Gi", None)
        outcomes = plan_resize(
            pvcs=(pvc,),
            name_filter=lambda name: True,
            desired_storage=lambda name: "30Gi",
            allow_expansion={},
            dry_run=False,
        )
        self.assertEqual(len(outcomes), 1)
        out = outcomes[0]
        self.assertFalse(out.patched)
        self.assertIn("StorageClass '<none>' does not allow", out.detail)

    def test_plan_resize_non_expandable_class_dry_run_shows_blocker(
        self,
    ) -> None:
        pvc = PvcFacts("pvc-1", "20Gi", "slow")
        outcomes = plan_resize(
            pvcs=(pvc,),
            name_filter=lambda name: True,
            desired_storage=lambda name: "30Gi",
            allow_expansion={"slow": False},
            dry_run=True,
        )
        self.assertEqual(len(outcomes), 1)
        out = outcomes[0]
        self.assertFalse(out.patched)
        self.assertIn("StorageClass 'slow' does not allow", out.detail)
        self.assertNotIn("would patch", out.detail)

    def test_plan_resize_shrink(self) -> None:
        pvc = PvcFacts("pvc-1", "20Gi", "fast")
        outcomes = plan_resize(
            pvcs=(pvc,),
            name_filter=lambda name: True,
            desired_storage=lambda name: "10Gi",
            allow_expansion={"fast": True},
            dry_run=False,
        )
        self.assertEqual(len(outcomes), 1)
        out = outcomes[0]
        self.assertFalse(out.patched)
        self.assertEqual(out.detail, SHRINK_DETAIL)

    def test_plan_resize_noop(self) -> None:
        pvc = PvcFacts("pvc-1", "20Gi", "fast")
        outcomes = plan_resize(
            pvcs=(pvc,),
            name_filter=lambda name: True,
            desired_storage=lambda name: "20Gi",
            allow_expansion={"fast": True},
            dry_run=False,
        )
        self.assertEqual(len(outcomes), 1)
        out = outcomes[0]
        self.assertFalse(out.patched)
        self.assertEqual(out.detail, "already at desired size")

    def test_plan_resize_unparseable_current_continues(self) -> None:
        pvc1 = PvcFacts("pvc-bad", "invalid", "fast")
        pvc2 = PvcFacts("pvc-good", "20Gi", "fast")
        outcomes = plan_resize(
            pvcs=(pvc1, pvc2),
            name_filter=lambda name: True,
            desired_storage=lambda name: "30Gi",
            allow_expansion={"fast": True},
            dry_run=False,
        )
        self.assertEqual(len(outcomes), 2)
        self.assertFalse(outcomes[0].patched)
        self.assertIn("unrecognized storage quantity", outcomes[0].detail)
        self.assertTrue(outcomes[1].patched)

    def test_plan_resize_ordering_and_desired(self) -> None:
        pvcs = (
            PvcFacts("pvc-a", "10Gi", "fast"),
            PvcFacts("pvc-b", "20Gi", "fast"),
        )
        des_map = {"pvc-a": "15Gi", "pvc-b": "25Gi"}
        outcomes = plan_resize(
            pvcs=pvcs,
            name_filter=lambda name: True,
            desired_storage=lambda name: des_map[name],
            allow_expansion={"fast": True},
            dry_run=False,
        )
        self.assertEqual(len(outcomes), 2)
        self.assertEqual(outcomes[0].pvc_name, "pvc-a")
        self.assertEqual(outcomes[0].desired, "15Gi")
        self.assertEqual(outcomes[1].pvc_name, "pvc-b")
        self.assertEqual(outcomes[1].desired, "25Gi")

    def test_storage_patch_shape(self) -> None:
        patch = storage_patch("30Gi")
        expected = {"spec": {"resources": {"requests": {"storage": "30Gi"}}}}
        self.assertEqual(patch, expected)
        self.assertEqual(len(patch), 1)


if __name__ == "__main__":
    unittest.main()
