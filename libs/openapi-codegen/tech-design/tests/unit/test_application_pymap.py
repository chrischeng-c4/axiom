from __future__ import annotations

import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from openapi_codegen.application.document import parse_ref_or, parse_schema, parse_spec
from openapi_codegen.application.pymap import optional, type_expr, union_expr
from openapi_codegen.application.typemap import build_type_map
from openapi_codegen.domain.target import PythonTarget


class TestApplicationPyMap(unittest.TestCase):
    def setUp(self) -> None:
        spec = parse_spec({"components": {"schemas": {"Pet": {}}}})
        self.tm = build_type_map(spec)
        self.py311 = PythonTarget.PY311

    def test_scalars_py311(self) -> None:
        self.assertEqual(type_expr(parse_ref_or({"type": "integer"}, parse_schema), self.tm, self.py311), "int")
        self.assertEqual(type_expr(parse_ref_or({"type": "number"}, parse_schema), self.tm, self.py311), "float")
        self.assertEqual(type_expr(parse_ref_or({"type": "boolean"}, parse_schema), self.tm, self.py311), "bool")
        self.assertEqual(type_expr(parse_ref_or({"type": "string"}, parse_schema), self.tm, self.py311), "str")

    def test_scalars_legacy(self) -> None:
        self.assertEqual(type_expr(parse_ref_or({"type": "integer"}, parse_schema), self.tm, None), "int")
        self.assertEqual(type_expr(parse_ref_or({"type": "number"}, parse_schema), self.tm, None), "float")
        self.assertEqual(type_expr(parse_ref_or({"type": "boolean"}, parse_schema), self.tm, None), "bool")
        self.assertEqual(type_expr(parse_ref_or({"type": "string"}, parse_schema), self.tm, None), "str")

    def test_binary_format_bytes(self) -> None:
        s = parse_ref_or({"type": "string", "format": "binary"}, parse_schema)
        self.assertEqual(type_expr(s, self.tm, self.py311), "bytes")
        self.assertEqual(type_expr(s, self.tm, None), "bytes")

    def test_empty_schema_any(self) -> None:
        s = parse_ref_or({}, parse_schema)
        self.assertEqual(type_expr(s, self.tm, self.py311), "Any")
        self.assertEqual(type_expr(s, self.tm, None), "Any")

    def test_type_null_only_any(self) -> None:
        s = parse_ref_or({"type": "null"}, parse_schema)
        self.assertEqual(type_expr(s, self.tm, self.py311), "Any")
        self.assertEqual(type_expr(s, self.tm, None), "Any")

    def test_unsupported_ref_any(self) -> None:
        s = parse_ref_or({"$ref": "#/components/parameters/Foo"}, parse_schema)
        self.assertEqual(type_expr(s, self.tm, self.py311), "Any")
        self.assertEqual(type_expr(s, self.tm, None), "Any")

    def test_array_of_ref(self) -> None:
        s = parse_ref_or({"type": "array", "items": {"$ref": "#/components/schemas/Pet"}}, parse_schema)
        self.assertEqual(type_expr(s, self.tm, self.py311), "list[Pet]")
        self.assertEqual(type_expr(s, self.tm, None), "list[Pet]")

    def test_array_no_items(self) -> None:
        s = parse_ref_or({"type": "array"}, parse_schema)
        self.assertEqual(type_expr(s, self.tm, self.py311), "list[Any]")

    def test_nullable_py311(self) -> None:
        s = parse_ref_or({"type": "string", "nullable": True}, parse_schema)
        self.assertEqual(type_expr(s, self.tm, self.py311), "str | None")

    def test_nullable_legacy(self) -> None:
        s = parse_ref_or({"type": "string", "nullable": True}, parse_schema)
        self.assertEqual(type_expr(s, self.tm, None), "Optional[str]")

    def test_type_array_string_null(self) -> None:
        s = parse_ref_or({"type": ["string", "null"]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm, self.py311), "str | None")
        self.assertEqual(type_expr(s, self.tm, None), "Optional[str]")

    def test_type_array_multi_types(self) -> None:
        s = parse_ref_or({"type": ["string", "number"]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm, self.py311), "str | float")
        self.assertEqual(type_expr(s, self.tm, None), "Union[str, float]")

    def test_type_array_multi_types_null(self) -> None:
        s = parse_ref_or({"type": ["string", "number", "null"]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm, self.py311), "str | float | None")
        self.assertEqual(type_expr(s, self.tm, None), "Optional[Union[str, float]]")

    def test_one_of_single_member_short_circuit(self) -> None:
        s = parse_ref_or({"oneOf": [{"type": "string"}]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm, self.py311), "str")
        self.assertEqual(type_expr(s, self.tm, None), "str")

    def test_one_of_multi_members(self) -> None:
        s = parse_ref_or({"oneOf": [{"type": "string"}, {"type": "integer"}]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm, self.py311), "str | int")
        self.assertEqual(type_expr(s, self.tm, None), "Union[str, int]")

    def test_all_of_first_member_only(self) -> None:
        s = parse_ref_or({"allOf": [{"type": "string"}, {"type": "integer"}]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm, self.py311), "str")
        self.assertEqual(type_expr(s, self.tm, None), "str")

    def test_enum_literal_strings(self) -> None:
        s = parse_ref_or({"type": "string", "enum": ["a", "b"]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm, self.py311), 'Literal["a", "b"]')

    def test_enum_literal_booleans(self) -> None:
        s = parse_ref_or({"enum": [True, False]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm, self.py311), "Literal[True, False]")

    def test_enum_literal_none(self) -> None:
        s = parse_ref_or({"enum": [None]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm, self.py311), "Literal[None]")

    def test_enum_literal_backslash(self) -> None:
        s = parse_ref_or({"enum": ["a\\b"]}, parse_schema)
        self.assertEqual(type_expr(s, self.tm, self.py311), 'Literal["a\\b"]')

    def test_object_properties_ignored(self) -> None:
        s = parse_ref_or({"type": "object", "properties": {"a": {"type": "string"}}}, parse_schema)
        self.assertEqual(type_expr(s, self.tm, self.py311), "dict[str, Any]")

    def test_object_additional_properties_schema(self) -> None:
        s = parse_ref_or({"type": "object", "additionalProperties": {"type": "integer"}}, parse_schema)
        self.assertEqual(type_expr(s, self.tm, self.py311), "dict[str, int]")

    def test_object_additional_properties_true(self) -> None:
        s = parse_ref_or({"type": "object", "additionalProperties": True}, parse_schema)
        self.assertEqual(type_expr(s, self.tm, self.py311), "dict[str, Any]")

    def test_optional_idempotence_and_asymmetry(self) -> None:
        self.assertEqual(optional("Any", self.py311), "Any")
        self.assertEqual(optional("None", self.py311), "None")
        self.assertEqual(optional("str", self.py311), "str | None")
        self.assertEqual(optional("str", None), "Optional[str]")
        self.assertEqual(optional("str | None", self.py311), "str | None")
        self.assertEqual(optional("str | None", None), "str | None")
        self.assertEqual(optional("Optional[str]", None), "Optional[str]")

        # Documented asymmetry trap:
        self.assertEqual(optional("Optional[str]", self.py311), "Optional[str] | None")

    def test_union_expr_helper(self) -> None:
        self.assertEqual(union_expr([], self.py311), "Any")
        self.assertEqual(union_expr(["str"], self.py311), "str")
        self.assertEqual(union_expr(["a", "b"], self.py311), "a | b")
        self.assertEqual(union_expr(["a", "b"], None), "Union[a, b]")


if __name__ == "__main__":
    unittest.main()
