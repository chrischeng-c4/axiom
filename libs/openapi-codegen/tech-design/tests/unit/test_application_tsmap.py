from __future__ import annotations

import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from openapi_codegen.application.document import parse_ref_or, parse_schema, parse_spec
from openapi_codegen.application.tsmap import type_expr
from openapi_codegen.application.typemap import build_type_map


class TestApplicationTsMap(unittest.TestCase):
    def setUp(self) -> None:
        spec = parse_spec({"components": {"schemas": {"Pet": {}}}})
        self.tm = build_type_map(spec)

    def test_scalars(self) -> None:
        self.assertEqual(type_expr(parse_ref_or({"type": "string"}, parse_schema), self.tm), "string")
        self.assertEqual(type_expr(parse_ref_or({"type": "integer"}, parse_schema), self.tm), "number")
        self.assertEqual(type_expr(parse_ref_or({"type": "number"}, parse_schema), self.tm), "number")
        self.assertEqual(type_expr(parse_ref_or({"type": "boolean"}, parse_schema), self.tm), "boolean")

    def test_binary_format_blob(self) -> None:
        s = parse_ref_or({"type": "string", "format": "binary"}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "Blob")

    def test_unresolved_ref(self) -> None:
        s = parse_ref_or({"$ref": "#/components/parameters/Foo"}, parse_schema)
        self.assertEqual(
            type_expr(s, self.tm),
            "unknown /* unsupported $ref #/components/parameters/Foo */",
        )

    def test_resolved_ref(self) -> None:
        s = parse_ref_or({"$ref": "#/components/schemas/Pet"}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "Pet")

    def test_resolved_ref_unmapped(self) -> None:
        s = parse_ref_or({"$ref": "#/components/schemas/Ghost"}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "Ghost")

    def test_array_of_ref(self) -> None:
        s = parse_ref_or(
            {"type": "array", "items": {"$ref": "#/components/schemas/Pet"}},
            parse_schema,
        )
        self.assertEqual(type_expr(s, self.tm), "Pet[]")

    def test_array_of_union_parenthesized(self) -> None:
        s = parse_ref_or(
            {"type": "array", "items": {"type": "string", "nullable": True}},
            parse_schema,
        )
        self.assertEqual(type_expr(s, self.tm), "(string | null)[]")

    def test_array_of_intersection_parenthesized(self) -> None:
        s = parse_ref_or(
            {
                "type": "array",
                "items": {"allOf": [{"type": "string"}, {"type": "integer"}]},
            },
            parse_schema,
        )
        self.assertEqual(type_expr(s, self.tm), "(string & number)[]")

    def test_array_no_items(self) -> None:
        s = parse_ref_or({"type": "array"}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "unknown[]")

    def test_nullable_true(self) -> None:
        s = parse_ref_or({"type": "string", "nullable": True}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "string | null")

    def test_type_array_string_null(self) -> None:
        s = parse_ref_or({"type": ["string", "null"]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "string | null")

    def test_type_array_multi_types(self) -> None:
        s = parse_ref_or({"type": ["string", "integer"]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "string | number")

    def test_type_array_multi_types_null(self) -> None:
        s = parse_ref_or({"type": ["string", "integer", "null"]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "string | number | null")

    def test_type_null_only(self) -> None:
        s = parse_ref_or({"type": "null"}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "unknown")

    def test_enum_strings(self) -> None:
        s = parse_ref_or({"type": "string", "enum": ["a", "b"]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), '"a" | "b"')

    def test_enum_numbers(self) -> None:
        s = parse_ref_or({"enum": [1, 2.5]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "1 | 2.5")

    def test_enum_booleans(self) -> None:
        s = parse_ref_or({"enum": [True, False]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "true | false")

    def test_enum_null(self) -> None:
        s = parse_ref_or({"enum": [None]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "null")

    def test_enum_empty(self) -> None:
        s = parse_ref_or({"enum": []}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "unknown")

    def test_enum_string_with_double_quote(self) -> None:
        s = parse_ref_or({"enum": ['a"b']}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), '"a\\"b"')

    def test_enum_string_with_backslash(self) -> None:
        s = parse_ref_or({"enum": ["a\\b"]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), '"a\\b"')

    def test_one_of(self) -> None:
        s = parse_ref_or(
            {"oneOf": [{"type": "string"}, {"type": "integer"}]}, parse_schema
        )
        self.assertEqual(type_expr(s, self.tm), "string | number")

    def test_any_of(self) -> None:
        s = parse_ref_or(
            {"anyOf": [{"type": "string"}, {"type": "integer"}]}, parse_schema
        )
        self.assertEqual(type_expr(s, self.tm), "string | number")

    def test_all_of_intersection(self) -> None:
        s = parse_ref_or(
            {
                "allOf": [
                    {"$ref": "#/components/schemas/Pet"},
                    {"type": "object", "properties": {"x": {"type": "integer"}}},
                ]
            },
            parse_schema,
        )
        self.assertEqual(type_expr(s, self.tm), "Pet & { x?: number }")

    def test_object_properties_required_optional(self) -> None:
        s = parse_ref_or(
            {
                "type": "object",
                "properties": {"id": {"type": "integer"}, "tag": {"type": "string"}},
                "required": ["id"],
            },
            parse_schema,
        )
        self.assertEqual(type_expr(s, self.tm), "{ id: number; tag?: string }")

    def test_object_property_key_quoting(self) -> None:
        s = parse_ref_or(
            {"type": "object", "properties": {"a-b": {"type": "string"}}},
            parse_schema,
        )
        self.assertEqual(type_expr(s, self.tm), '{ "a-b"?: string }')

    def test_additional_properties_true(self) -> None:
        s = parse_ref_or({"type": "object", "additionalProperties": True}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "{ [key: string]: unknown }")

    def test_additional_properties_schema(self) -> None:
        s = parse_ref_or(
            {"type": "object", "additionalProperties": {"type": "integer"}},
            parse_schema,
        )
        self.assertEqual(type_expr(s, self.tm), "{ [key: string]: number }")

    def test_additional_properties_false(self) -> None:
        s = parse_ref_or({"type": "object", "additionalProperties": False}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "Record<string, unknown>")

    def test_empty_object(self) -> None:
        s = parse_ref_or({"type": "object"}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "Record<string, unknown>")


if __name__ == "__main__":
    unittest.main()
