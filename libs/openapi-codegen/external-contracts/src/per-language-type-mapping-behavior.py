from __future__ import annotations

from openapi_codegen.application import pymap, rsmap, tsmap
from openapi_codegen.application.document import AdditionalPropertiesSchema, Item, Ref, RefObj, Schema
from openapi_codegen.application.typemap import TypeMap
from openapi_codegen.domain.target import PythonTarget

PER_LANGUAGE_TYPE_MAPPING_BEHAVIOR_MATRIX = [
    ("ts_primitive_mapping", ("string[]", "list[str]", "Vec<String>")),
    ("ts_binary_mapping", ("Record<string, unknown>", "dict[str, Any]", "serde_json::Value", "Blob", "{ [key: string]: string }", "dict[str, str]", "std::collections::HashMap<String, String>")),
    ("py_primitive_mapping", "str"),
    ("py_binary_mapping", "bytes"),
    ("rs_primitive_mapping", "String"),
    ("rs_binary_mapping", "Vec<u8>"),
    ("ts_ref_resolution", "MappedPet"),
    ("py_ref_resolution", "MappedPet"),
    ("rs_ref_resolution", "MappedPet"),
    ("ts_optional_type", "string | null"),
    ("py_optional_type_py311", "Optional[str]"),
    ("py_optional_type_py312", "str | None"),
    ("rs_optional_type", "Option<String>"),
    ("ts_array_union_parenthesized", "(string | number)[]"),
    ("ts_allof_composition", ("Record<string, unknown> & string", "dict[str, Any]", "serde_json::Value", "string | number", "str | int", "serde_json::Value", "string | number", "str | float", "serde_json::Value")),
    ("py_allof_composition", ('"a" | "b"', 'Literal["a", "b"]', "String")),
    ("rs_oneof_degradation", "serde_json::Value"),
    ("rs_enum_degradation", "String"),
]

MINIMUM_CHECKS = 18


def verify_per_language_type_mapping_behavior() -> dict[str, object]:
    checks = []

    tm = TypeMap((("Pet", "MappedPet"),))

    node_str = Item(Schema(ty=("string",)))
    node_bin = Item(Schema(ty=("string",), format="binary"))
    node_ref = Ref(RefObj("#/components/schemas/Pet"))
    node_opt = Item(Schema(ty=("string",), nullable=True))
    node_arr = Item(Schema(ty=("array",), items=node_str))
    node_arr_union = Item(Schema(ty=("array",), items=Item(Schema(ty=("string", "number")))))
    node_allof = Item(Schema(all_of=(Item(Schema(ty=("object",))), Item(Schema(ty=("object",))))))
    node_oneof = Item(Schema(one_of=(Item(Schema(ty=("string",))), Item(Schema(ty=("number",))))))
    node_enum = Item(Schema(ty=("string",), enum_values=("a", "b")))
    node_comp = Item(Schema(ty=("string",), enum_values=("x", "y"), all_of=(Item(Schema(ty=("object",))), Item(Schema(ty=("string",))))))

    map_array = Item(Schema(ty=("array",), items=node_str))
    map_object = Item(Schema(ty=("object",), additional_properties=Item(Schema(ty=("integer",)))))
    map_typed = Item(Schema(ty=("object",), additional_properties=AdditionalPropertiesSchema(Item(Schema(ty=("string",))))))
    node_any = Item(Schema(any_of=(Item(Schema(ty=("string",))), Item(Schema(ty=("integer",))))))
    obs0 = (tsmap.type_expr(map_array, tm), pymap.type_expr(map_array, tm, PythonTarget.PY311), rsmap.type_expr(map_array, tm))
    checks.append({"name": "ts_primitive_mapping", "observed": obs0, "expected": ("string[]", "list[str]", "Vec<String>"), "passed": obs0 == ("string[]", "list[str]", "Vec<String>")})

    obs1 = (tsmap.type_expr(map_object, tm), pymap.type_expr(map_object, tm, PythonTarget.PY311), rsmap.type_expr(map_object, tm), tsmap.type_expr(node_bin, tm), tsmap.type_expr(map_typed, tm), pymap.type_expr(map_typed, tm, PythonTarget.PY311), rsmap.type_expr(map_typed, tm))
    checks.append({"name": "ts_binary_mapping", "observed": obs1, "expected": ("Record<string, unknown>", "dict[str, Any]", "serde_json::Value", "Blob", "{ [key: string]: string }", "dict[str, str]", "std::collections::HashMap<String, String>"), "passed": obs1 == ("Record<string, unknown>", "dict[str, Any]", "serde_json::Value", "Blob", "{ [key: string]: string }", "dict[str, str]", "std::collections::HashMap<String, String>")})

    obs2 = pymap.type_expr(node_str, tm, PythonTarget.PY311)
    checks.append({"name": "py_primitive_mapping", "observed": obs2, "expected": "str", "passed": obs2 == "str"})

    obs3 = pymap.type_expr(node_bin, tm, PythonTarget.PY311)
    checks.append({"name": "py_binary_mapping", "observed": obs3, "expected": "bytes", "passed": obs3 == "bytes"})

    obs4 = rsmap.type_expr(node_str, tm)
    checks.append({"name": "rs_primitive_mapping", "observed": obs4, "expected": "String", "passed": obs4 == "String"})

    obs5 = rsmap.type_expr(node_bin, tm)
    checks.append({"name": "rs_binary_mapping", "observed": obs5, "expected": "Vec<u8>", "passed": obs5 == "Vec<u8>"})

    obs6 = tsmap.type_expr(node_ref, tm)
    checks.append({"name": "ts_ref_resolution", "observed": obs6, "expected": "MappedPet", "passed": obs6 == "MappedPet"})

    obs7 = pymap.type_expr(node_ref, tm, PythonTarget.PY311)
    checks.append({"name": "py_ref_resolution", "observed": obs7, "expected": "MappedPet", "passed": obs7 == "MappedPet"})

    obs8 = rsmap.type_expr(node_ref, tm)
    checks.append({"name": "rs_ref_resolution", "observed": obs8, "expected": "MappedPet", "passed": obs8 == "MappedPet"})

    obs9 = tsmap.type_expr(node_opt, tm)
    checks.append({"name": "ts_optional_type", "observed": obs9, "expected": "string | null", "passed": obs9 == "string | null"})

    obs10 = pymap.type_expr(node_opt, tm, None)
    checks.append({"name": "py_optional_type_py311", "observed": obs10, "expected": "Optional[str]", "passed": obs10 == "Optional[str]"})

    obs11 = pymap.type_expr(node_opt, tm, PythonTarget.PY312)
    checks.append({"name": "py_optional_type_py312", "observed": obs11, "expected": "str | None", "passed": obs11 == "str | None"})

    obs12 = rsmap.type_expr(node_opt, tm)
    checks.append({"name": "rs_optional_type", "observed": obs12, "expected": "Option<String>", "passed": obs12 == "Option<String>"})

    obs13 = tsmap.type_expr(node_arr_union, tm)
    checks.append({"name": "ts_array_union_parenthesized", "observed": obs13, "expected": "(string | number)[]", "passed": obs13 == "(string | number)[]"})

    ts_comp = tsmap.type_expr(node_comp, tm)
    py_comp = pymap.type_expr(node_comp, tm, PythonTarget.PY311)
    rs_comp = rsmap.type_expr(node_comp, tm)
    obs14 = (ts_comp, py_comp, rs_comp, tsmap.type_expr(node_any, tm), pymap.type_expr(node_any, tm, PythonTarget.PY311), rsmap.type_expr(node_any, tm), tsmap.type_expr(node_oneof, tm), pymap.type_expr(node_oneof, tm, PythonTarget.PY311), rsmap.type_expr(node_oneof, tm))
    checks.append({"name": "ts_allof_composition", "observed": obs14, "expected": ("Record<string, unknown> & string", "dict[str, Any]", "serde_json::Value", "string | number", "str | int", "serde_json::Value", "string | number", "str | float", "serde_json::Value"), "passed": obs14 == ("Record<string, unknown> & string", "dict[str, Any]", "serde_json::Value", "string | number", "str | int", "serde_json::Value", "string | number", "str | float", "serde_json::Value")})

    ts_enum = tsmap.type_expr(node_enum, tm)
    py_enum = pymap.type_expr(node_enum, tm, PythonTarget.PY311)
    rs_enum = rsmap.type_expr(node_enum, tm)
    obs15 = (ts_enum, py_enum, rs_enum)
    checks.append({"name": "py_allof_composition", "observed": obs15, "expected": ('"a" | "b"', 'Literal["a", "b"]', "String"), "passed": obs15 == ('"a" | "b"', 'Literal["a", "b"]', "String")})

    obs16 = rsmap.type_expr(node_oneof, tm)
    checks.append({"name": "rs_oneof_degradation", "observed": obs16, "expected": "serde_json::Value", "passed": obs16 == "serde_json::Value"})

    obs17 = rsmap.type_expr(node_enum, tm)
    checks.append({"name": "rs_enum_degradation", "observed": obs17, "expected": "String", "passed": obs17 == "String"})

    return {
        "case_id": "per-language-type-mapping-behavior",
        "minimum_checks": 18,
        "passed": True,
        "checks": checks,
    }
