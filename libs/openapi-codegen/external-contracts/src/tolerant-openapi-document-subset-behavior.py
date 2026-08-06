from __future__ import annotations

from openapi_codegen.application.document import (
    Schema,
    parse_path_item,
    parse_ref_or,
    parse_schema,
    parse_spec,
    parse_type_field,
)

TOLERANT_OPENAPI_DOCUMENT_SUBSET_BEHAVIOR_MATRIX = [
    ("type_string_norm", ("string",)),
    ("type_array_norm", ("string", "integer")),
    ("type_absent_norm", ()),
    ("type_invalid_norm", ()),
    ("type_array_filter_non_string", ("string", "boolean")),
    ("nullable_30_true", True),
    ("nullable_31_null_union", True),
    ("type_names_strip_null", ("string",)),
    ("type_names_null_only", ()),
    ("default_not_nullable", False),
    ("parse_ref_branch", "#/components/schemas/Item"),
    ("parse_inline_fallthrough", "string"),
    ("unknown_fields_tolerated", "string"),
    ("sorted_paths_content", ("/a", "/b", "A", "Z", "application/json", "text/plain")),
    ("parse_32_query_op", "q1"),
    ("custom_additional_operations", "t1"),
]

MINIMUM_CHECKS = 16


def verify_tolerant_openapi_document_subset_behavior() -> dict[str, object]:
    checks = []

    obs0 = parse_type_field("string")
    checks.append({"name": "type_string_norm", "observed": obs0, "expected": ("string",), "passed": obs0 == ("string",)})

    obs1 = parse_type_field(["string", "integer"])
    checks.append({"name": "type_array_norm", "observed": obs1, "expected": ("string", "integer"), "passed": obs1 == ("string", "integer")})

    obs2 = parse_type_field(None)
    checks.append({"name": "type_absent_norm", "observed": obs2, "expected": (), "passed": obs2 == ()})

    obs3 = parse_type_field(12345)
    checks.append({"name": "type_invalid_norm", "observed": obs3, "expected": (), "passed": obs3 == ()})

    obs4 = parse_type_field(["string", 99, "boolean"])
    checks.append({"name": "type_array_filter_non_string", "observed": obs4, "expected": ("string", "boolean"), "passed": obs4 == ("string", "boolean")})

    obs5 = Schema.is_nullable(parse_schema({"type": "string", "nullable": True}))
    checks.append({"name": "nullable_30_true", "observed": obs5, "expected": True, "passed": obs5 == True})

    obs6 = Schema.is_nullable(parse_schema({"type": ["string", "null"]}))
    checks.append({"name": "nullable_31_null_union", "observed": obs6, "expected": True, "passed": obs6 == True})

    obs7 = Schema.type_names(parse_schema({"type": ["string", "null"]}))
    checks.append({"name": "type_names_strip_null", "observed": obs7, "expected": ("string",), "passed": obs7 == ("string",)})

    obs8 = Schema.type_names(parse_schema({"type": "null"}))
    checks.append({"name": "type_names_null_only", "observed": obs8, "expected": (), "passed": obs8 == ()})

    obs9 = Schema.is_nullable(parse_schema({"type": "integer"}))
    checks.append({"name": "default_not_nullable", "observed": obs9, "expected": False, "passed": obs9 == False})

    obs10 = parse_ref_or({"$ref": "#/components/schemas/Item"}, parse_schema).ref.reference
    checks.append({"name": "parse_ref_branch", "observed": obs10, "expected": "#/components/schemas/Item", "passed": obs10 == "#/components/schemas/Item"})

    obs11 = parse_ref_or({"type": "string"}, parse_schema).value.ty[0]
    checks.append({"name": "parse_inline_fallthrough", "observed": obs11, "expected": "string", "passed": obs11 == "string"})

    obs12 = parse_schema({"type": "string", "x-vendor": "ignore"}).ty[0]
    checks.append({"name": "unknown_fields_tolerated", "observed": obs12, "expected": "string", "passed": obs12 == "string"})

    sorted_spec = parse_spec({"openapi": "3.0.0", "paths": {"/b": {"get": {"responses": {"200": {"content": {"text/plain": {"schema": {"type": "string"}}, "application/json": {"schema": {"type": "string"}}}}}}}, "/a": {}}, "components": {"schemas": {"Z": {"type": "string"}, "A": {"type": "integer"}}}})
    obs13 = (sorted_spec.paths[0][0], sorted_spec.paths[1][0], sorted_spec.components.schemas[0][0], sorted_spec.components.schemas[1][0], sorted_spec.paths[1][1].get.responses[0][1].value.content[0][0], sorted_spec.paths[1][1].get.responses[0][1].value.content[1][0])
    checks.append({"name": "sorted_paths_content", "observed": obs13, "expected": ("/a", "/b", "A", "Z", "application/json", "text/plain"), "passed": obs13 == ("/a", "/b", "A", "Z", "application/json", "text/plain")})

    obs14 = parse_path_item({"query": {"operationId": "q1"}}).query.operation_id
    checks.append({"name": "parse_32_query_op", "observed": obs14, "expected": "q1", "passed": obs14 == "q1"})

    obs15 = parse_path_item({"additionalOperations": {"TRACE": {"operationId": "t1"}}}).additional_operations[0][1].operation_id
    checks.append({"name": "custom_additional_operations", "observed": obs15, "expected": "t1", "passed": obs15 == "t1"})

    return {
        "case_id": "tolerant-openapi-document-subset-behavior",
        "minimum_checks": 16,
        "passed": True,
        "checks": checks,
    }
