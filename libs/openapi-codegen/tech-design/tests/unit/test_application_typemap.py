from __future__ import annotations

import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from openapi_codegen.application.document import parse_spec
from openapi_codegen.application.typemap import TypeMap, build_type_map


class TestApplicationTypeMap(unittest.TestCase):
    def test_resolve_ref_mapped(self) -> None:
        spec = parse_spec({"components": {"schemas": {"Pet": {}}}})
        tmap = build_type_map(spec)
        self.assertEqual(tmap.resolve_ref("#/components/schemas/Pet"), "Pet")

    def test_resolve_ref_unmapped_pascal(self) -> None:
        spec = parse_spec({"components": {"schemas": {}}})
        tmap = build_type_map(spec)
        self.assertEqual(
            tmap.resolve_ref("#/components/schemas/ghost_type"), "GhostType"
        )

    def test_resolve_ref_other_prefix(self) -> None:
        spec = parse_spec({})
        tmap = build_type_map(spec)
        self.assertIsNone(tmap.resolve_ref("#/components/parameters/PetId"))

    def test_resolve_ref_no_prefix(self) -> None:
        spec = parse_spec({})
        tmap = build_type_map(spec)
        self.assertIsNone(tmap.resolve_ref("Pet"))

    def test_resolve_ref_empty_name(self) -> None:
        spec = parse_spec({})
        tmap = build_type_map(spec)
        self.assertEqual(tmap.resolve_ref("#/components/schemas/"), "Anonymous")

    def test_collision_sorting_order_independent(self) -> None:
        spec1 = parse_spec({
            "components": {
                "schemas": {
                    "PetCategory": {},
                    "pet_category": {},
                }
            }
        })
        spec2 = parse_spec({
            "components": {
                "schemas": {
                    "pet_category": {},
                    "PetCategory": {},
                }
            }
        })
        tmap1 = build_type_map(spec1)
        tmap2 = build_type_map(spec2)

        self.assertEqual(tmap1.get("PetCategory"), "PetCategory")
        self.assertEqual(tmap1.get("pet_category"), "PetCategory_2")

        self.assertEqual(tmap2.get("PetCategory"), "PetCategory")
        self.assertEqual(tmap2.get("pet_category"), "PetCategory_2")

    def test_typemap_get_valid(self) -> None:
        spec = parse_spec({"components": {"schemas": {"Pet": {}}}})
        tmap = build_type_map(spec)
        self.assertEqual(tmap.get("Pet"), "Pet")

    def test_typemap_get_missing(self) -> None:
        spec = parse_spec({})
        tmap = build_type_map(spec)
        self.assertIsNone(tmap.get("Missing"))

    def test_build_type_map_empty(self) -> None:
        tmap = build_type_map(parse_spec({}))
        self.assertEqual(tmap.names, ())

    def test_build_type_map_single_schema(self) -> None:
        spec = parse_spec({"components": {"schemas": {"User": {}}}})
        tmap = build_type_map(spec)
        self.assertEqual(tmap.names, (("User", "User"),))

    def test_build_type_map_multiple_schemas_sorted(self) -> None:
        spec = parse_spec({"components": {"schemas": {"B": {}, "A": {}}}})
        tmap = build_type_map(spec)
        self.assertEqual(tmap.names, (("A", "A"), ("B", "B")))

    def test_build_type_map_snake_case_conversion(self) -> None:
        spec = parse_spec({"components": {"schemas": {"pet_owner": {}}}})
        tmap = build_type_map(spec)
        self.assertEqual(tmap.get("pet_owner"), "PetOwner")

    def test_build_type_map_duplicate_pascal_names(self) -> None:
        spec = parse_spec({"components": {"schemas": {"pet": {}, "PET": {}}}})
        tmap = build_type_map(spec)
        self.assertEqual(tmap.get("PET"), "Pet")
        self.assertEqual(tmap.get("pet"), "Pet_2")

    def test_build_type_map_underscore_digit_key_does_not_collide(self) -> None:
        spec = parse_spec({"components": {"schemas": {"Pet": {}, "Pet_2": {}}}})
        tmap = build_type_map(spec)
        # to_pascal("Pet_2") is "Pet2", so the keys pascalize differently and do not collide
        self.assertEqual(tmap.get("Pet"), "Pet")
        self.assertEqual(tmap.get("Pet_2"), "Pet2")

    def test_resolve_ref_subpath(self) -> None:
        spec = parse_spec({})
        tmap = build_type_map(spec)
        self.assertEqual(
            tmap.resolve_ref("#/components/schemas/user_profile"), "UserProfile"
        )

    def test_resolve_ref_with_digits(self) -> None:
        spec = parse_spec({})
        tmap = build_type_map(spec)
        self.assertEqual(
            tmap.resolve_ref("#/components/schemas/api_v2"), "ApiV2"
        )

    def test_resolve_ref_with_hyphen(self) -> None:
        spec = parse_spec({})
        tmap = build_type_map(spec)
        self.assertEqual(
            tmap.resolve_ref("#/components/schemas/x-request"), "XRequest"
        )

    def test_typemap_names_is_tuple(self) -> None:
        tmap = build_type_map(parse_spec({}))
        self.assertIsInstance(tmap.names, tuple)

    def test_typemap_immutable(self) -> None:
        tmap = TypeMap()
        self.assertEqual(tmap.names, ())

    def test_resolve_ref_case_sensitive_prefix(self) -> None:
        spec = parse_spec({})
        tmap = build_type_map(spec)
        self.assertIsNone(tmap.resolve_ref("#/Components/Schemas/Pet"))

    def test_resolve_ref_relative_path(self) -> None:
        spec = parse_spec({})
        tmap = build_type_map(spec)
        self.assertIsNone(tmap.resolve_ref("schemas/Pet"))

    def test_build_type_map_anonymous(self) -> None:
        spec = parse_spec({"components": {"schemas": {"": {}}}})
        tmap = build_type_map(spec)
        self.assertEqual(tmap.get(""), "Anonymous")

    def test_build_type_map_three_collisions(self) -> None:
        spec = parse_spec(
            {"components": {"schemas": {"pet": {}, "Pet": {}, "PET": {}}}}
        )
        tmap = build_type_map(spec)
        self.assertEqual(tmap.get("PET"), "Pet")
        self.assertEqual(tmap.get("Pet"), "Pet_2")
        self.assertEqual(tmap.get("pet"), "Pet_3")

    def test_typemap_get_after_collisions(self) -> None:
        spec = parse_spec(
            {"components": {"schemas": {"pet_item": {}, "PetItem": {}}}}
        )
        tmap = build_type_map(spec)
        self.assertEqual(tmap.get("PetItem"), "PetItem")
        self.assertEqual(tmap.get("pet_item"), "PetItem_2")

    def test_resolve_ref_after_collisions(self) -> None:
        spec = parse_spec(
            {"components": {"schemas": {"pet_item": {}, "PetItem": {}}}}
        )
        tmap = build_type_map(spec)
        self.assertEqual(
            tmap.resolve_ref("#/components/schemas/pet_item"), "PetItem_2"
        )


if __name__ == "__main__":
    unittest.main()
