from __future__ import annotations

import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from service_k8s.application.rotation import IssuerId
from service_k8s.application.trust_bundle import (
    TrustBundle,
    split_pem_blocks,
)


def anchor(tag: str) -> str:
    return "-----BEGIN CERTIFICATE-----\n" + tag + "\n-----END CERTIFICATE-----"


class TestApplicationTrustBundle(unittest.TestCase):
    def test_split_pem_blocks_two_well_formed_anchors(self) -> None:
        pem = f"{anchor('A')}\n{anchor('B')}"
        blocks = split_pem_blocks(pem)
        self.assertEqual(len(blocks), 2)
        self.assertEqual(blocks[0], anchor("A"))
        self.assertEqual(blocks[1], anchor("B"))

    def test_split_pem_blocks_drops_surrounding_text(self) -> None:
        pem = f"before\n{anchor('A')}\nbetween\n{anchor('B')}\nafter"
        blocks = split_pem_blocks(pem)
        self.assertEqual(len(blocks), 2)
        self.assertEqual(blocks[0], anchor("A"))
        self.assertEqual(blocks[1], anchor("B"))

    def test_split_pem_blocks_drops_unclosed_block(self) -> None:
        pem = f"{anchor('A')}\n-----BEGIN CERTIFICATE-----\nunclosed"
        blocks = split_pem_blocks(pem)
        self.assertEqual(len(blocks), 1)
        self.assertEqual(blocks[0], anchor("A"))

    def test_split_pem_blocks_begin_discards_partial_block(self) -> None:
        pem = "-----BEGIN CERTIFICATE-----\npartial\n-----BEGIN CERTIFICATE-----\nB\n-----END CERTIFICATE-----"
        blocks = split_pem_blocks(pem)
        self.assertEqual(len(blocks), 1)
        self.assertEqual(blocks[0], anchor("B"))

    def test_split_pem_blocks_stray_end_ignored(self) -> None:
        pem = f"-----END CERTIFICATE-----\n{anchor('A')}"
        blocks = split_pem_blocks(pem)
        self.assertEqual(len(blocks), 1)
        self.assertEqual(blocks[0], anchor("A"))

    def test_split_pem_blocks_empty_string(self) -> None:
        self.assertEqual(split_pem_blocks(""), ())

    def test_split_pem_blocks_returns_tuple(self) -> None:
        res = split_pem_blocks(anchor("A"))
        self.assertIsInstance(res, tuple)

    def test_trust_bundle_new_is_empty(self) -> None:
        b = TrustBundle()
        self.assertTrue(b.is_empty())
        self.assertEqual(b.issuers(), ())
        self.assertEqual(b.to_pem(), "")
        self.assertEqual(b.annotation(), "")

    def test_with_anchor_empty_bundle(self) -> None:
        b = TrustBundle().with_anchor(IssuerId("pool-a"), anchor("A"))
        self.assertFalse(b.is_empty())
        self.assertEqual(b.issuers(), (IssuerId("pool-a"),))

    def test_with_anchor_sorts(self) -> None:
        b = (
            TrustBundle()
            .with_anchor(IssuerId("pool-z"), anchor("Z"))
            .with_anchor(IssuerId("pool-a"), anchor("A"))
        )
        self.assertEqual(
            b.issuers(), (IssuerId("pool-a"), IssuerId("pool-z"))
        )

    def test_with_anchor_replaces_existing(self) -> None:
        b1 = TrustBundle().with_anchor(IssuerId("pool-a"), anchor("A1"))
        b2 = b1.with_anchor(IssuerId("pool-a"), anchor("A2"))
        self.assertEqual(len(b2.entries), 1)
        self.assertIn("A2", b2.to_pem())
        self.assertNotIn("A1", b2.to_pem())

    def test_with_anchor_immutable(self) -> None:
        b1 = TrustBundle()
        b2 = b1.with_anchor(IssuerId("pool-a"), anchor("A"))
        self.assertTrue(b1.is_empty())
        self.assertFalse(b2.is_empty())

    def test_retaining_filters_issuers(self) -> None:
        b = (
            TrustBundle()
            .with_anchor(IssuerId("a"), anchor("A"))
            .with_anchor(IssuerId("b"), anchor("B"))
            .with_anchor(IssuerId("c"), anchor("C"))
        )
        b_sub = b.retaining((IssuerId("a"), IssuerId("c")))
        self.assertEqual(b_sub.issuers(), (IssuerId("a"), IssuerId("c")))

    def test_retaining_empty(self) -> None:
        b = TrustBundle().with_anchor(IssuerId("a"), anchor("A"))
        b_empty = b.retaining(())
        self.assertTrue(b_empty.is_empty())

    def test_retaining_absent_issuer(self) -> None:
        b = TrustBundle().with_anchor(IssuerId("a"), anchor("A"))
        b_sub = b.retaining((IssuerId("a"), IssuerId("absent")))
        self.assertEqual(b_sub.issuers(), (IssuerId("a"),))

    def test_contains(self) -> None:
        b = TrustBundle().with_anchor(IssuerId("a"), anchor("A"))
        self.assertTrue(b.contains(IssuerId("a")))
        self.assertFalse(b.contains(IssuerId("b")))

    def test_to_pem_formatting(self) -> None:
        b = (
            TrustBundle()
            .with_anchor(IssuerId("a"), anchor("A") + "\n\n")
            .with_anchor(IssuerId("b"), anchor("B"))
        )
        expected = f"{anchor('A')}\n{anchor('B')}\n"
        self.assertEqual(b.to_pem(), expected)

    def test_annotation_formatting(self) -> None:
        b = (
            TrustBundle()
            .with_anchor(IssuerId("c"), anchor("C"))
            .with_anchor(IssuerId("a"), anchor("A"))
            .with_anchor(IssuerId("b"), anchor("B"))
        )
        self.assertEqual(b.annotation(), "a,b,c")

    def test_parse_round_trip(self) -> None:
        b_orig = (
            TrustBundle()
            .with_anchor(IssuerId("pool-a"), anchor("A"))
            .with_anchor(IssuerId("pool-b"), anchor("B"))
        )
        b_parsed = TrustBundle.parse(b_orig.to_pem(), b_orig.annotation())
        self.assertEqual(b_parsed, b_orig)

    def test_parse_count_mismatch_too_many_blocks(self) -> None:
        pem = f"{anchor('A')}\n{anchor('B')}"
        b = TrustBundle.parse(pem, "pool-a")
        self.assertTrue(b.is_empty())

    def test_parse_count_mismatch_too_few_blocks(self) -> None:
        pem = anchor("A")
        b = TrustBundle.parse(pem, "pool-a,pool-b")
        self.assertTrue(b.is_empty())

    def test_parse_whitespace_and_empty_annotation(self) -> None:
        self.assertTrue(TrustBundle.parse("", None).is_empty())
        pem = f"{anchor('A')}\n{anchor('B')}"
        b = TrustBundle.parse(pem, " pool-a , , pool-b ")
        self.assertEqual(b.issuers(), (IssuerId("pool-a"), IssuerId("pool-b")))


if __name__ == "__main__":
    unittest.main()
