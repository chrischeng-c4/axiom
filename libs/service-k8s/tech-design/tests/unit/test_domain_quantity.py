from __future__ import annotations

from dataclasses import FrozenInstanceError
import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from service_k8s.domain.quantity import (
    SUFFIXES,
    Grow,
    NoOp,
    QuantityError,
    ShrinkUnsupported,
    Unparseable,
    decide,
    parse_storage_bytes,
)


class TestDomainQuantity(unittest.TestCase):
    def test_binary_suffixes(self) -> None:
        self.assertEqual(parse_storage_bytes("1Ki"), 1024)
        self.assertEqual(parse_storage_bytes("1Mi"), 1048576)
        self.assertEqual(parse_storage_bytes("1Gi"), 1073741824)
        self.assertEqual(parse_storage_bytes("1Ti"), 1099511627776)
        self.assertEqual(parse_storage_bytes("1Pi"), 1125899906842624)
        self.assertEqual(parse_storage_bytes("1Ei"), 1152921504606846976)

    def test_decimal_suffixes(self) -> None:
        self.assertEqual(parse_storage_bytes("1k"), 1000)
        self.assertEqual(parse_storage_bytes("1M"), 1000000)
        self.assertEqual(parse_storage_bytes("1G"), 1000000000)
        self.assertEqual(parse_storage_bytes("1T"), 1000000000000)
        self.assertEqual(parse_storage_bytes("1P"), 1000000000000000)
        self.assertEqual(parse_storage_bytes("1E"), 1000000000000000000)

    def test_1G_and_1Gi_differ(self) -> None:
        g = parse_storage_bytes("1G")
        gi = parse_storage_bytes("1Gi")
        self.assertNotEqual(g, gi)
        self.assertEqual(g, 1000000000)
        self.assertEqual(gi, 1073741824)

    def test_20Gi(self) -> None:
        self.assertEqual(parse_storage_bytes("20Gi"), 21474836480)

    def test_bare_byte_counts(self) -> None:
        self.assertEqual(parse_storage_bytes("1024"), 1024)
        self.assertEqual(parse_storage_bytes("0"), 0)

    def test_whitespace_trimmed(self) -> None:
        self.assertEqual(parse_storage_bytes("  20Gi  "), 21474836480)

    def test_fractional_with_suffix(self) -> None:
        self.assertEqual(parse_storage_bytes("1.5Gi"), 1610612736)

    def test_bare_fractional_raises_quantity_error(self) -> None:
        with self.assertRaises(QuantityError):
            parse_storage_bytes("10.5")

    def test_empty_and_whitespace_raise_empty_quantity_error(self) -> None:
        for bad in ("", "   "):
            with self.assertRaises(QuantityError) as cm:
                parse_storage_bytes(bad)
            self.assertEqual(str(cm.exception), "empty storage quantity")

    def test_invalid_inputs_raise_quantity_error(self) -> None:
        for bad in ("abc", "Gi", "1Xi"):
            with self.assertRaises(QuantityError):
                parse_storage_bytes(bad)

    def test_negative_quantity_raises(self) -> None:
        with self.assertRaises(QuantityError) as cm:
            parse_storage_bytes("-1Gi")
        self.assertIn("negative", str(cm.exception))

    def test_suffix_table_structure_and_order(self) -> None:
        expected = ("Ei", "Pi", "Ti", "Gi", "Mi", "Ki", "E", "P", "T", "G", "M", "k")
        actual = tuple(s for s, _ in SUFFIXES)
        self.assertEqual(actual, expected)
        self.assertEqual(len(SUFFIXES), 12)

    def test_decide_grow(self) -> None:
        action = decide("10Gi", "20Gi")
        expected = Grow(current_bytes=10737418240, desired_bytes=21474836480)
        self.assertEqual(action, expected)

    def test_decide_noop(self) -> None:
        action = decide("20Gi", "20Gi")
        self.assertEqual(action, NoOp())

    def test_decide_shrink_unsupported_suffix_change(self) -> None:
        action = decide("20Gi", "20G")
        expected = ShrinkUnsupported(
            current_bytes=21474836480, desired_bytes=20000000000
        )
        self.assertEqual(action, expected)

    # An implementation treating every suffix as a power of 1024 passes eleven of twelve suffix cases and silently under-provisions storage by roughly 7%.
    def test_a_decimal_suffix_is_not_its_binary_neighbour(self) -> None:
        self.assertEqual(parse_storage_bytes("1G"), 1000000000)
        self.assertEqual(parse_storage_bytes("1Gi"), 1073741824)
        self.assertNotEqual(
            parse_storage_bytes("1G"), parse_storage_bytes("1Gi")
        )

    def test_decide_shrink_unsupported_number_change(self) -> None:
        action = decide("20Gi", "10Gi")
        expected = ShrinkUnsupported(
            current_bytes=21474836480, desired_bytes=10737418240
        )
        self.assertEqual(action, expected)

    def test_decide_unparseable_current(self) -> None:
        action = decide("bogus", "20Gi")
        self.assertIsInstance(action, Unparseable)
        assert isinstance(action, Unparseable)
        self.assertTrue(action.detail.startswith("current quantity 'bogus': "))

    def test_decide_unparseable_desired(self) -> None:
        action = decide("20Gi", "bogus")
        self.assertIsInstance(action, Unparseable)
        assert isinstance(action, Unparseable)
        self.assertTrue(action.detail.startswith("desired quantity 'bogus': "))

    def test_decide_unparseable_reports_current_first(self) -> None:
        action = decide("bogus", "alsobogus")
        self.assertIsInstance(action, Unparseable)
        assert isinstance(action, Unparseable)
        self.assertTrue(action.detail.startswith("current quantity 'bogus': "))

    def test_decide_round_trip_noop(self) -> None:
        for q in ("1Ki", "20Gi", "1.5Gi", "1000"):
            self.assertEqual(decide(q, q), NoOp())

    def test_resize_action_dataclasses_frozen(self) -> None:
        g = Grow(100, 200)
        n = NoOp()
        s = ShrinkUnsupported(200, 100)
        u = Unparseable("err")
        for obj in (g, n, s, u):
            with self.assertRaises(FrozenInstanceError):
                obj.foo = "bar"  # type: ignore[attr-defined]


if __name__ == "__main__":
    unittest.main()
