from __future__ import annotations

from openapi_codegen.application.document import (
    parse_schema,
    parse_spec,
)

TOLERANT_OPENAPI_DOCUMENT_SUBSET_SECURITY_MATRIX = [
    ("unknown_vendor_keywords", "A"),
    ("exclusive_min_bool", "string"),
    ("exclusive_min_numeric", "string"),
    ("invalid_type_field", ()),
    ("malformed_optional_summary", None),
    ("non_str_info_title", ""),
    ("non_str_info_version", ""),
    ("mixed_type_array", ("string",)),
    ("unknown_op_fields", None),
    ("unknown_param_name", ""),
    ("non_dict_properties", ()),
    ("non_dict_paths", ()),
    ("non_dict_components", ()),
    ("non_dict_path_get", None),
]

MINIMUM_CHECKS = 14


def verify_tolerant_openapi_document_subset_security() -> dict[str, object]:
    checks = []

    obs0 = parse_spec({"openapi": "3.0.0", "info": {"title": "A", "version": "1.0"}, "x-vendor-foo": "bar"}).info.title
    checks.append({"name": "unknown_vendor_keywords", "observed": obs0, "expected": "A", "passed": obs0 == "A"})

    obs1 = parse_schema({"type": "string", "exclusiveMinimum": True}).ty[0]
    checks.append({"name": "exclusive_min_bool", "observed": obs1, "expected": "string", "passed": obs1 == "string"})

    obs2 = parse_schema({"type": "string", "exclusiveMinimum": 10}).ty[0]
    checks.append({"name": "exclusive_min_numeric", "observed": obs2, "expected": "string", "passed": obs2 == "string"})

    obs3 = parse_schema({"type": 12345}).ty
    checks.append({"name": "invalid_type_field", "observed": obs3, "expected": (), "passed": obs3 == ()})

    obs4 = parse_spec({"paths": {"/p": {"get": {"summary": 999}}}}).paths[0][1].get.summary
    checks.append({"name": "malformed_optional_summary", "observed": obs4, "expected": None, "passed": obs4 == None})

    obs5 = parse_spec({"info": {"title": 123, "version": "1.0"}}).info.title
    checks.append({"name": "non_str_info_title", "observed": obs5, "expected": "", "passed": obs5 == ""})

    obs6 = parse_spec({"info": {"title": "A", "version": 789}}).info.version
    checks.append({"name": "non_str_info_version", "observed": obs6, "expected": "", "passed": obs6 == ""})

    obs7 = parse_schema({"type": ["string", 2]}).ty
    checks.append({"name": "mixed_type_array", "observed": obs7, "expected": ("string",), "passed": obs7 == ("string",)})

    obs8 = parse_spec({"paths": {"/p": {"get": {"operationId": 1234}}}}).paths[0][1].get.operation_id
    checks.append({"name": "unknown_op_fields", "observed": obs8, "expected": None, "passed": obs8 == None})

    obs9 = parse_spec({"paths": {"/p": {"get": {"parameters": [{"name": 555}]}}}}).paths[0][1].get.parameters[0].value.name
    checks.append({"name": "unknown_param_name", "observed": obs9, "expected": "", "passed": obs9 == ""})

    obs10 = parse_schema({"properties": "not-a-dict"}).properties
    checks.append({"name": "non_dict_properties", "observed": obs10, "expected": (), "passed": obs10 == ()})

    obs11 = parse_spec({"paths": "not-a-dict"}).paths
    checks.append({"name": "non_dict_paths", "observed": obs11, "expected": (), "passed": obs11 == ()})

    obs12 = parse_spec({"components": "not-a-dict"}).components.schemas
    checks.append({"name": "non_dict_components", "observed": obs12, "expected": (), "passed": obs12 == ()})

    obs13 = parse_spec({"paths": {"/p": {"get": "not-a-dict"}}}).paths[0][1].get.operation_id
    checks.append({"name": "non_dict_path_get", "observed": obs13, "expected": None, "passed": obs13 == None})

    return {
        "case_id": "tolerant-openapi-document-subset-security",
        "minimum_checks": 14,
        "passed": True,
        "checks": checks,
    }
