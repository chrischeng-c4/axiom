from __future__ import annotations

import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from openapi_codegen.application.document import parse_ref_or, parse_schema, parse_spec
from openapi_codegen.application.rsmap import ANY, optional, type_expr
from openapi_codegen.application.typemap import build_type_map


class TestApplicationRsMap(unittest.TestCase):
    def setUp(self) -> None:
        spec = parse_spec({"components": {"schemas": {"Pet": {}}}})
        self.tm = build_type_map(spec)

    def test_scalars(self) -> None:
        self.assertEqual(type_expr(parse_ref_or({"type": "integer"}, parse_schema), self.tm), "i64")
        self.assertEqual(type_expr(parse_ref_or({"type": "number"}, parse_schema), self.tm), "f64")
        self.assertEqual(type_expr(parse_ref_or({"type": "string"}, parse_schema), self.tm), "String")
        self.assertEqual(type_expr(parse_ref_or({"type": "boolean"}, parse_schema), self.tm), "bool")

    def test_binary_format_vec_u8(self) -> None:
        s = parse_ref_or({"type": "string", "format": "binary"}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "Vec<u8>")

    def test_empty_schema_any(self) -> None:
        s = parse_ref_or({}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), ANY)

    def test_type_null_only_any(self) -> None:
        s = parse_ref_or({"type": "null"}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), ANY)

    def test_resolved_ref(self) -> None:
        s = parse_ref_or({"$ref": "#/components/schemas/Pet"}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "Pet")

    def test_unsupported_ref_any(self) -> None:
        s = parse_ref_or({"$ref": "#/components/parameters/Foo"}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), ANY)

    def test_array_of_ref(self) -> None:
        s = parse_ref_or({"type": "array", "items": {"$ref": "#/components/schemas/Pet"}}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "Vec<Pet>")

    def test_array_no_items(self) -> None:
        s = parse_ref_or({"type": "array"}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "Vec<serde_json::Value>")

    def test_nullable_string(self) -> None:
        s = parse_ref_or({"type": "string", "nullable": True}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "Option<String>")

    def test_type_array_string_null(self) -> None:
        s = parse_ref_or({"type": ["string", "null"]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "Option<String>")

    def test_type_array_multi_types(self) -> None:
        s = parse_ref_or({"type": ["string", "integer"]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), ANY)

    def test_type_array_multi_types_null(self) -> None:
        s = parse_ref_or({"type": ["string", "integer", "null"]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), ANY)

    def test_one_of_collapse_any(self) -> None:
        s = parse_ref_or({"oneOf": [{"type": "string"}, {"type": "integer"}]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), ANY)

    def test_any_of_collapse_any(self) -> None:
        s = parse_ref_or({"anyOf": [{"type": "string"}]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), ANY)

    def test_all_of_first_member(self) -> None:
        s = parse_ref_or({"allOf": [{"type": "string"}, {"type": "integer"}]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "String")

    def test_enum_string(self) -> None:
        s = parse_ref_or({"type": "string", "enum": ["a", "b"]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "String")

    def test_enum_numeric(self) -> None:
        s = parse_ref_or({"enum": [1, 2]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "String")

    def test_object_additional_properties_schema(self) -> None:
        s = parse_ref_or({"type": "object", "additionalProperties": {"type": "integer"}}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), "std::collections::HashMap<String, i64>")

    def test_object_additional_properties_true(self) -> None:
        s = parse_ref_or({"type": "object", "additionalProperties": True}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), ANY)

    def test_object_properties_any(self) -> None:
        s = parse_ref_or({"type": "object", "properties": {"a": {"type": "string"}}}, parse_schema)
        self.assertEqual(type_expr(s, self.tm), ANY)

    def test_optional_helper(self) -> None:
        self.assertEqual(optional(ANY), ANY)
        self.assertEqual(optional("Option<String>"), "Option<String>")
        self.assertEqual(optional("String"), "Option<String>")


if __name__ == "__main__":
    unittest.main()
