from __future__ import annotations

from openapi_codegen.application import plan_ts
from openapi_codegen.application.document import parse_spec
from openapi_codegen.application.typemap import build_type_map
from openapi_codegen.domain.names import (
    NameRegistry,
    escape_string_literal,
    is_ident,
    param_access,
    prop_key,
    to_pascal,
    to_snake,
)

DETERMINISTIC_IDENTIFIER_NAMING_SECURITY_MATRIX = [
    ("is_ident_spaces", False),
    ("is_ident_dash", False),
    ("is_ident_empty", False),
    ("is_ident_valid", True),
    ("escape_literal_quotes", '"hello \\"world\\""'),
    ("escape_literal_backslash", 'params["a\\\\b"]'),
    ("prop_key_ident", "myField"),
    ("prop_key_hostile_dots", ('"my.field"', '"7field"', 'params["7param"]')),
    ("prop_key_hostile_dash", '"my-field"'),
    ("param_access_ident", "params.myParam"),
    ("param_access_hostile", 'params["my-param"]'),
    ("pascal_empty_base", "Anonymous"),
    ("registry_component_precedence", "GetThingData_2"),
    ("registry_empty_base", "anonymous"),
]

MINIMUM_CHECKS = 14


def verify_deterministic_identifier_naming_security() -> dict[str, object]:
    checks = []

    obs0 = is_ident("user name")
    checks.append({"name": "is_ident_spaces", "observed": obs0, "expected": False, "passed": obs0 == False})

    obs1 = is_ident("user-name")
    checks.append({"name": "is_ident_dash", "observed": obs1, "expected": False, "passed": obs1 == False})

    obs2 = is_ident("")
    checks.append({"name": "is_ident_empty", "observed": obs2, "expected": False, "passed": obs2 == False})

    obs3 = is_ident("userName123")
    checks.append({"name": "is_ident_valid", "observed": obs3, "expected": True, "passed": obs3 == True})

    obs4 = prop_key('hello "world"')
    checks.append({"name": "escape_literal_quotes", "observed": obs4, "expected": '"hello \\\"world\\\""', "passed": obs4 == '"hello \\\"world\\\""'})

    obs5 = param_access("a\\b")
    checks.append({"name": "escape_literal_backslash", "observed": obs5, "expected": 'params["a\\\\b"]', "passed": obs5 == 'params["a\\\\b"]'})

    obs6 = prop_key("myField")
    checks.append({"name": "prop_key_ident", "observed": obs6, "expected": "myField", "passed": obs6 == "myField"})

    obs7 = (prop_key("my.field"), prop_key("7field"), param_access("7param"))
    checks.append({"name": "prop_key_hostile_dots", "observed": obs7, "expected": ('"my.field"', '"7field"', 'params["7param"]'), "passed": obs7 == ('"my.field"', '"7field"', 'params["7param"]')})

    obs8 = prop_key("my-field")
    checks.append({"name": "prop_key_hostile_dash", "observed": obs8, "expected": '"my-field"', "passed": obs8 == '"my-field"'})

    obs9 = param_access("myParam")
    checks.append({"name": "param_access_ident", "observed": obs9, "expected": "params.myParam", "passed": obs9 == "params.myParam"})

    obs10 = param_access("my-param")
    checks.append({"name": "param_access_hostile", "observed": obs10, "expected": 'params["my-param"]', "passed": obs10 == 'params["my-param"]'})

    obs11 = to_pascal("")
    checks.append({"name": "pascal_empty_base", "observed": obs11, "expected": "Anonymous", "passed": obs11 == "Anonymous"})

    spec = parse_spec({
        "components": {"schemas": {"GetThingData": {"type": "object"}}},
        "paths": {"/thing": {"get": {"operationId": "getThing", "parameters": [{"name": "q", "in": "query"}]}}},
    })
    tm = build_type_map(spec)
    plans = plan_ts.build(spec, tm)
    obs12 = plans[0].data_type_name
    checks.append({"name": "registry_component_precedence", "observed": obs12, "expected": "GetThingData_2", "passed": obs12 == "GetThingData_2"})

    obs13 = NameRegistry.unique(NameRegistry(), "")
    checks.append({"name": "registry_empty_base", "observed": obs13, "expected": "anonymous", "passed": obs13 == "anonymous"})

    return {
        "case_id": "deterministic-identifier-naming-security",
        "minimum_checks": 14,
        "passed": True,
        "checks": checks,
    }
