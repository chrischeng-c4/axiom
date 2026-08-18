from __future__ import annotations

import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from service_k8s.infrastructure.crd import (
    BOOLEAN_LIKE,
    CelRuleError,
    add_spec_validation_rule,
    normalize_unsigned_integer_formats,
    quote_yaml_1_1_boolean_like_strings,
)


class TestInfrastructureCrd(unittest.TestCase):
    def test_normalize_uint32_removes_format_adds_minimum_zero(self) -> None:
        schema = {"type": "integer", "format": "uint32"}
        normalize_unsigned_integer_formats(schema)
        self.assertNotIn("format", schema)
        self.assertEqual(schema.get("minimum"), 0)

    def test_normalize_uint64_removes_format_adds_minimum_zero(self) -> None:
        schema = {"type": "integer", "format": "uint64"}
        normalize_unsigned_integer_formats(schema)
        self.assertNotIn("format", schema)
        self.assertEqual(schema.get("minimum"), 0)

    def test_normalize_preserves_existing_minimum(self) -> None:
        schema = {"type": "integer", "format": "uint64", "minimum": 1}
        normalize_unsigned_integer_formats(schema)
        self.assertNotIn("format", schema)
        self.assertEqual(schema.get("minimum"), 1)

    def test_normalize_int32_int64_datetime_untouched(self) -> None:
        schema1 = {"type": "integer", "format": "int32"}
        schema2 = {"type": "integer", "format": "int64"}
        schema3 = {"type": "string", "format": "date-time"}
        normalize_unsigned_integer_formats(schema1)
        normalize_unsigned_integer_formats(schema2)
        normalize_unsigned_integer_formats(schema3)
        self.assertEqual(schema1.get("format"), "int32")
        self.assertEqual(schema2.get("format"), "int64")
        self.assertEqual(schema3.get("format"), "date-time")

    def test_normalize_recursively_dict_properties(self) -> None:
        schema = {
            "type": "object",
            "properties": {
                "count": {"type": "integer", "format": "uint32"},
            },
        }
        normalize_unsigned_integer_formats(schema)
        count_schema = schema["properties"]["count"]
        self.assertNotIn("format", count_schema)
        self.assertEqual(count_schema.get("minimum"), 0)

    def test_normalize_recursively_list_elements(self) -> None:
        schema = {
            "allOf": [
                {"type": "integer", "format": "uint32"},
                {"type": "string"},
            ]
        }
        normalize_unsigned_integer_formats(schema)
        sub = schema["allOf"][0]
        self.assertNotIn("format", sub)
        self.assertEqual(sub.get("minimum"), 0)

    def test_normalize_schema_no_formats_unchanged(self) -> None:
        schema = {"type": "string", "minimum": 5}
        normalize_unsigned_integer_formats(schema)
        self.assertEqual(schema, {"type": "string", "minimum": 5})

    def test_add_rule_attaches_to_all_versions(self) -> None:
        crd = {
            "spec": {
                "versions": [
                    {
                        "name": "v1",
                        "schema": {
                            "openAPIV3Schema": {
                                "properties": {"spec": {"type": "object"}}
                            }
                        },
                    },
                    {
                        "name": "v2",
                        "schema": {
                            "openAPIV3Schema": {
                                "properties": {"spec": {"type": "object"}}
                            }
                        },
                    },
                ]
            }
        }
        cnt = add_spec_validation_rule(crd, "has(self.a)", "msg a")
        self.assertEqual(cnt, 2)

    def test_add_rule_appends_multiple_rules(self) -> None:
        crd = {
            "spec": {
                "versions": [
                    {
                        "name": "v1",
                        "schema": {
                            "openAPIV3Schema": {
                                "properties": {"spec": {"type": "object"}}
                            }
                        },
                    }
                ]
            }
        }
        add_spec_validation_rule(crd, "has(self.a)", "msg a")
        add_spec_validation_rule(crd, "has(self.b)", "msg b")
        v1_spec = crd["spec"]["versions"][0]["schema"]["openAPIV3Schema"][
            "properties"
        ]["spec"]
        rules = v1_spec["x-kubernetes-validations"]
        self.assertEqual(len(rules), 2)
        self.assertEqual(rules[0]["rule"], "has(self.a)")
        self.assertEqual(rules[1]["rule"], "has(self.b)")

    def test_add_rule_entry_structure(self) -> None:
        crd = {
            "spec": {
                "versions": [
                    {
                        "name": "v1",
                        "schema": {
                            "openAPIV3Schema": {
                                "properties": {"spec": {"type": "object"}}
                            }
                        },
                    }
                ]
            }
        }
        add_spec_validation_rule(crd, "has(self.x)", "message x")
        v1_spec = crd["spec"]["versions"][0]["schema"]["openAPIV3Schema"][
            "properties"
        ]["spec"]
        self.assertEqual(
            v1_spec["x-kubernetes-validations"][0],
            {"rule": "has(self.x)", "message": "message x"},
        )

    def test_add_rule_missing_spec_schema_returns_zero(self) -> None:
        crd = {"spec": {"versions": [{"name": "v1"}]}}
        self.assertEqual(add_spec_validation_rule(crd, "has(self.a)", "m"), 0)

    def test_add_rule_empty_dict_returns_zero(self) -> None:
        crd: dict[str, object] = {}
        self.assertEqual(add_spec_validation_rule(crd, "has(self.a)", "m"), 0)

    def test_add_rule_rejects_ne_null_spaces(self) -> None:
        crd = {"spec": {"versions": []}}
        with self.assertRaises(CelRuleError):
            add_spec_validation_rule(crd, "self.x != null", "m")

    def test_add_rule_rejects_ne_null_no_spaces(self) -> None:
        crd = {"spec": {"versions": []}}
        with self.assertRaises(CelRuleError):
            add_spec_validation_rule(crd, "self.x !=null", "m")

    def test_add_rule_rejects_eq_null(self) -> None:
        crd = {"spec": {"versions": []}}
        with self.assertRaises(CelRuleError):
            add_spec_validation_rule(crd, "self.x == null", "m")

    def test_add_rule_accepts_has_self(self) -> None:
        crd = {
            "spec": {
                "versions": [
                    {
                        "name": "v1",
                        "schema": {
                            "openAPIV3Schema": {
                                "properties": {"spec": {"type": "object"}}
                            }
                        },
                    }
                ]
            }
        }
        cnt = add_spec_validation_rule(
            crd, "has(self.a) != has(self.b)", "msg"
        )
        self.assertEqual(cnt, 1)

    def test_add_rule_error_does_not_mutate_crd(self) -> None:
        crd = {
            "spec": {
                "versions": [
                    {
                        "name": "v1",
                        "schema": {
                            "openAPIV3Schema": {
                                "properties": {"spec": {"type": "object"}}
                            }
                        },
                    }
                ]
            }
        }
        with self.assertRaises(CelRuleError):
            add_spec_validation_rule(crd, "self.a != null", "m")

        v1_spec = crd["spec"]["versions"][0]["schema"]["openAPIV3Schema"][
            "properties"
        ]["spec"]
        self.assertNotIn("x-kubernetes-validations", v1_spec)

    def test_quote_yaml_default_off(self) -> None:
        inp = "    default: off"
        self.assertEqual(
            quote_yaml_1_1_boolean_like_strings(inp), '    default: "off"'
        )

    def test_quote_yaml_indentation_preserved(self) -> None:
        inp = "    - off"
        self.assertEqual(
            quote_yaml_1_1_boolean_like_strings(inp), '    - "off"'
        )

    def test_quote_yaml_description_first_colon_only(self) -> None:
        inp = "    description: mode defaults to: off"
        self.assertEqual(quote_yaml_1_1_boolean_like_strings(inp), inp)

    def test_quote_yaml_default_false_unchanged(self) -> None:
        inp = "    default: false"
        self.assertEqual(quote_yaml_1_1_boolean_like_strings(inp), inp)

    def test_quote_yaml_preserves_casing(self) -> None:
        inp = "    default: OFF"
        self.assertEqual(
            quote_yaml_1_1_boolean_like_strings(inp), '    default: "OFF"'
        )

    def test_quote_yaml_trailing_newline_preserved(self) -> None:
        inp_nl = "default: off\n"
        self.assertEqual(
            quote_yaml_1_1_boolean_like_strings(inp_nl), 'default: "off"\n'
        )

        inp_no_nl = "default: off"
        self.assertEqual(
            quote_yaml_1_1_boolean_like_strings(inp_no_nl), 'default: "off"'
        )


if __name__ == "__main__":
    unittest.main()
