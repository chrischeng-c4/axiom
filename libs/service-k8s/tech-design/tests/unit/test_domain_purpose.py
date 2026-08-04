from __future__ import annotations

import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from service_k8s.domain.purpose import ExtendedUsage, Purpose


class TestDomainPurpose(unittest.TestCase):
    def test_purpose_tokens(self) -> None:
        self.assertEqual(Purpose.SERVING.token, "serving")
        self.assertEqual(Purpose.PEER.token, "peer")

    def test_extended_usage_tokens(self) -> None:
        self.assertEqual(ExtendedUsage.SERVER_AUTH.token, "serverAuth")
        self.assertEqual(ExtendedUsage.CLIENT_AUTH.token, "clientAuth")

    def test_serving_extended_key_usages_exact_tuple(self) -> None:
        self.assertEqual(
            Purpose.SERVING.extended_key_usages(),
            (ExtendedUsage.SERVER_AUTH,),
        )

    def test_serving_does_not_contain_client_auth(self) -> None:
        self.assertNotIn(
            ExtendedUsage.CLIENT_AUTH,
            Purpose.SERVING.extended_key_usages(),
        )

    def test_peer_extended_key_usages_exact_tuple_and_order(self) -> None:
        self.assertEqual(
            Purpose.PEER.extended_key_usages(),
            (ExtendedUsage.SERVER_AUTH, ExtendedUsage.CLIENT_AUTH),
        )

    def test_extended_usage_closed_enumeration(self) -> None:
        self.assertEqual(len(tuple(ExtendedUsage)), 2)

    def test_purpose_closed_enumeration(self) -> None:
        self.assertEqual(len(tuple(Purpose)), 2)

    def test_purposes_usage_sets_not_equal(self) -> None:
        self.assertNotEqual(
            Purpose.SERVING.extended_key_usages(),
            Purpose.PEER.extended_key_usages(),
        )


if __name__ == "__main__":
    unittest.main()
