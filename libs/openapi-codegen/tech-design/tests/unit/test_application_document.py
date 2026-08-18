from __future__ import annotations

import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from openapi_codegen.application.document import (
    AdditionalPropertiesBool,
    AdditionalPropertiesSchema,
    Info,
    Item,
    Ref,
    RefObj,
    Schema,
    Spec,
    parse_additional_properties,
    parse_info,
    parse_operation,
    parse_parameter,
    parse_path_item,
    parse_ref_or,
    parse_response,
    parse_schema,
    parse_spec,
    parse_type_field,
)
from openapi_codegen.application.operations import METHOD_FIELDS, METHODS


class TestApplicationDocument(unittest.TestCase):
    def test_parse_ref_or_ref(self) -> None:
        res = parse_ref_or({"$ref": "#/components/schemas/Pet"}, parse_schema)
        self.assertEqual(res, Ref(RefObj("#/components/schemas/Pet")))

    def test_parse_ref_or_ref_discards_siblings(self) -> None:
        res = parse_ref_or({"$ref": "X", "type": "string"}, parse_schema)
        self.assertEqual(res, Ref(RefObj("X")))

    def test_parse_ref_or_non_string_ref(self) -> None:
        res = parse_ref_or({"$ref": 7}, parse_schema)
        self.assertIsInstance(res, Item)
        assert isinstance(res, Item)
        self.assertIsInstance(res.value, Schema)

    def test_parse_type_field_single(self) -> None:
        self.assertEqual(parse_type_field("string"), ("string",))

    def test_parse_type_field_array(self) -> None:
        self.assertEqual(
            parse_type_field(["string", "null"]), ("string", "null")
        )

    def test_parse_type_field_absent_and_none(self) -> None:
        self.assertEqual(parse_type_field(None), ())

    def test_parse_type_field_invalid(self) -> None:
        self.assertEqual(parse_type_field(7), ())

    def test_parse_type_field_filters_non_string(self) -> None:
        self.assertEqual(
            parse_type_field(["string", 7, "null"]), ("string", "null")
        )

    def test_schema_type_names(self) -> None:
        s = Schema(ty=("string", "null"))
        self.assertEqual(s.type_names(), ("string",))

    def test_schema_type_names_null_only(self) -> None:
        s = Schema(ty=("null",))
        self.assertEqual(s.type_names(), ())

    def test_schema_is_nullable(self) -> None:
        s = Schema(ty=("null",))
        self.assertTrue(s.is_nullable())

    def test_schema_is_nullable_false_with_null_type(self) -> None:
        s = Schema(nullable=False, ty=("null",))
        self.assertTrue(s.is_nullable())

    def test_schema_is_nullable_true_flag(self) -> None:
        s = Schema(nullable=True, ty=())
        self.assertTrue(s.is_nullable())

    def test_schema_is_nullable_default(self) -> None:
        s = Schema()
        self.assertFalse(s.is_nullable())

    def test_parse_additional_properties_bool_false(self) -> None:
        res = parse_additional_properties(False)
        self.assertEqual(res, AdditionalPropertiesBool(False))

    def test_parse_additional_properties_bool_true(self) -> None:
        res = parse_additional_properties(True)
        self.assertEqual(res, AdditionalPropertiesBool(True))

    def test_parse_additional_properties_none(self) -> None:
        self.assertIsNone(parse_additional_properties(None))

    def test_parse_additional_properties_schema(self) -> None:
        res = parse_additional_properties({"type": "string"})
        self.assertIsInstance(res, AdditionalPropertiesSchema)
        assert isinstance(res, AdditionalPropertiesSchema)
        self.assertEqual(res.schema, Item(Schema(ty=("string",))))

    def test_parse_spec_openapi_version(self) -> None:
        spec = parse_spec({"openapi": "4.0.0"})
        self.assertEqual(spec.openapi, "4.0.0")

    def test_parse_spec_degenerate(self) -> None:
        self.assertEqual(parse_spec(None), Spec())
        self.assertEqual(parse_spec(None).openapi, "")

    def test_parse_parameter_degenerate(self) -> None:
        param = parse_parameter({})
        self.assertEqual(param.name, "")
        self.assertEqual(param.location, "")
        self.assertFalse(param.required)

    def test_parse_response_no_description(self) -> None:
        resp = parse_response({"description": "ok"})
        self.assertEqual(resp.content, ())

    def test_operation_extensions_sorted_and_non_x(self) -> None:
        op = parse_operation(
            {"x-b": 1, "custom": 2, "operationId": "op_test"}
        )
        self.assertEqual(op.operation_id, "op_test")
        self.assertEqual(op.extensions, (("custom", 2), ("x-b", 1)))

    def test_schema_properties_sorted(self) -> None:
        s = parse_schema({"properties": {"z": {}, "a": {}}})
        self.assertEqual(
            s.properties,
            (("a", Item(Schema())), ("z", Item(Schema()))),
        )

    def test_parse_path_item_and_components_sorted(self) -> None:
        spec = parse_spec({
            "paths": {"/b": {}, "/a": {}},
            "components": {"schemas": {"Z": {}, "A": {}}},
        })
        path_keys = tuple(k for k, _ in spec.paths)
        self.assertEqual(path_keys, ("/a", "/b"))
        schema_keys = tuple(k for k, _ in spec.components.schemas)
        self.assertEqual(schema_keys, ("A", "Z"))

    def test_operation_summary_valid(self) -> None:
        op = parse_operation({"summary": "List all pets"})
        self.assertEqual(op.summary, "List all pets")
        ext_keys = tuple(k for k, _ in op.extensions)
        self.assertNotIn("summary", ext_keys)

    def test_operation_summary_non_string(self) -> None:
        op = parse_operation({"summary": 7})
        self.assertIsNone(op.summary)
        ext_keys = tuple(k for k, _ in op.extensions)
        self.assertNotIn("summary", ext_keys)

    def test_operation_summary_absent(self) -> None:
        op = parse_operation({})
        self.assertIsNone(op.summary)

    def test_spec_info_valid(self) -> None:
        spec = parse_spec({"info": {"title": "Pet API", "version": "1.0.0"}})
        self.assertEqual(spec.info, Info(title="Pet API", version="1.0.0"))

    def test_spec_info_empty(self) -> None:
        spec = parse_spec({})
        self.assertEqual(spec.info, Info())
        self.assertEqual(spec.info.title, "")

    def test_spec_info_non_dict(self) -> None:
        self.assertEqual(parse_spec({"info": "junk"}).info, Info())
        self.assertEqual(parse_spec({"info": None}).info, Info())

    def test_spec_info_title_non_string(self) -> None:
        self.assertEqual(parse_spec({"info": {"title": 7}}).info.title, "")

    def test_methods_derived_from_method_fields(self) -> None:
        self.assertEqual(
            METHODS, ("get", "post", "put", "patch", "delete", "query")
        )
        self.assertEqual(len(METHOD_FIELDS), 6)
        self.assertEqual(tuple(name for name, _ in METHOD_FIELDS), METHODS)

    def test_spec_info_dataclass_defaults(self) -> None:
        info = parse_info(None)
        self.assertEqual(info.title, "")
        self.assertEqual(info.version, "")


if __name__ == "__main__":
    unittest.main()
