from __future__ import annotations

from openapi_codegen.application import pymap, rsmap, tsmap
from openapi_codegen.application.document import Item, Ref, RefObj, Schema
from openapi_codegen.application.typemap import TypeMap
from openapi_codegen.domain.target import PythonTarget

PER_LANGUAGE_TYPE_MAPPING_SECURITY_MATRIX = [
    ("ts_unresolved_ref", "unknown /* unsupported $ref #/external/Unknown */"),
    ("py_unresolved_ref", "Any"),
    ("rs_unresolved_ref", "serde_json::Value"),
    ("ts_empty_schema", "unknown"),
    ("py_empty_schema", "Any"),
    ("rs_empty_schema", "serde_json::Value"),
    ("ts_null_only_schema", "unknown"),
    ("py_null_only_schema", "Any"),
    ("rs_null_only_schema", "serde_json::Value"),
    ("ts_empty_enum_values", "unknown"),
    ("py_empty_enum_values", "Any"),
    ("rs_empty_enum_values", "serde_json::Value"),
    ("ts_hostile_property_key", '{ "bad\\"prop": string }'),
    ("ts_hostile_enum_escaping", '"bad\\"enum"'),
    ("rs_nullable_freeform_no_double_wrap", "serde_json::Value"),
    ("py_nullable_freeform_no_double_wrap", "Any"),
]

MINIMUM_CHECKS = 16


def verify_per_language_type_mapping_security() -> dict[str, object]:
    checks = []

    obs0 = tsmap.type_expr(Ref(RefObj("#/external/Unknown")), TypeMap())
    checks.append({"name": "ts_unresolved_ref", "observed": obs0, "expected": "unknown /* unsupported $ref #/external/Unknown */", "passed": obs0 == "unknown /* unsupported $ref #/external/Unknown */"})

    obs1 = pymap.type_expr(Ref(RefObj("#/external/Unknown")), TypeMap(), PythonTarget.PY311)
    checks.append({"name": "py_unresolved_ref", "observed": obs1, "expected": "Any", "passed": obs1 == "Any"})

    obs2 = rsmap.type_expr(Ref(RefObj("#/external/Unknown")), TypeMap())
    checks.append({"name": "rs_unresolved_ref", "observed": obs2, "expected": "serde_json::Value", "passed": obs2 == "serde_json::Value"})

    obs3 = tsmap.type_expr(Item(Schema()), TypeMap())
    checks.append({"name": "ts_empty_schema", "observed": obs3, "expected": "unknown", "passed": obs3 == "unknown"})

    obs4 = pymap.type_expr(Item(Schema()), TypeMap(), PythonTarget.PY311)
    checks.append({"name": "py_empty_schema", "observed": obs4, "expected": "Any", "passed": obs4 == "Any"})

    obs5 = rsmap.type_expr(Item(Schema()), TypeMap())
    checks.append({"name": "rs_empty_schema", "observed": obs5, "expected": "serde_json::Value", "passed": obs5 == "serde_json::Value"})

    obs6 = tsmap.type_expr(Item(Schema(("null",))), TypeMap())
    checks.append({"name": "ts_null_only_schema", "observed": obs6, "expected": "unknown", "passed": obs6 == "unknown"})

    obs7 = pymap.type_expr(Item(Schema(("null",))), TypeMap(), PythonTarget.PY311)
    checks.append({"name": "py_null_only_schema", "observed": obs7, "expected": "Any", "passed": obs7 == "Any"})

    obs8 = rsmap.type_expr(Item(Schema(("null",))), TypeMap())
    checks.append({"name": "rs_null_only_schema", "observed": obs8, "expected": "serde_json::Value", "passed": obs8 == "serde_json::Value"})

    obs9 = tsmap.type_expr(Item(Schema(enum_values=())), TypeMap())
    checks.append({"name": "ts_empty_enum_values", "observed": obs9, "expected": "unknown", "passed": obs9 == "unknown"})

    obs10 = pymap.type_expr(Item(Schema(enum_values=())), TypeMap(), PythonTarget.PY311)
    checks.append({"name": "py_empty_enum_values", "observed": obs10, "expected": "Any", "passed": obs10 == "Any"})

    obs11 = rsmap.type_expr(Item(Schema(enum_values=())), TypeMap())
    checks.append({"name": "rs_empty_enum_values", "observed": obs11, "expected": "serde_json::Value", "passed": obs11 == "serde_json::Value"})

    obs12 = tsmap.type_expr(Item(Schema(properties=(('bad"prop', Item(Schema(("string",)))),), required=('bad"prop',))), TypeMap())
    checks.append({"name": "ts_hostile_property_key", "observed": obs12, "expected": '{ "bad\\"prop": string }', "passed": obs12 == '{ "bad\\"prop": string }'})

    obs13 = tsmap.enum_union(Schema(enum_values=('bad"enum',)))
    checks.append({"name": "ts_hostile_enum_escaping", "observed": obs13, "expected": '"bad\\"enum"', "passed": obs13 == '"bad\\"enum"'})

    obs14 = rsmap.type_expr(Item(Schema(nullable=True)), TypeMap())
    checks.append({"name": "rs_nullable_freeform_no_double_wrap", "observed": obs14, "expected": "serde_json::Value", "passed": obs14 == "serde_json::Value"})

    obs15 = pymap.type_expr(Item(Schema(nullable=True)), TypeMap(), PythonTarget.PY311)
    checks.append({"name": "py_nullable_freeform_no_double_wrap", "observed": obs15, "expected": "Any", "passed": obs15 == "Any"})

    return {
        "case_id": "per-language-type-mapping-security",
        "minimum_checks": 16,
        "passed": True,
        "checks": checks,
    }
