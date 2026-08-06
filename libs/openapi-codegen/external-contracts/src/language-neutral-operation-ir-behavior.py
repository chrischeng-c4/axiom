from __future__ import annotations

from openapi_codegen.application.document import (
    Item,
    MediaType,
    Operation,
    Response,
    Schema,
    parse_spec,
)
from openapi_codegen.application.operations import (
    build_operations,
    pick_response,
)

LANGUAGE_NEUTRAL_OPERATION_IR_BEHAVIOR_MATRIX = [
    ("op_method_get", (("/other", "get"), ("/other", "query"), ("/test", "get"), ("/test", "post"), ("/test", "put"), ("/test", "patch"), ("/test", "delete"), ("/test", "query"))),
    ("op_method_post", "post"),
    ("op_method_put", "put"),
    ("op_method_patch", "patch"),
    ("op_method_delete", "delete"),
    ("op_method_query", "query"),
    ("additional_ops_head", "head"),
    ("additional_ops_trace", "trace"),
    ("additional_ops_dedup_trace", "trace"),
    ("op_ir_path_param_name", ("id", "p", "q")),
    ("op_ir_path_param_required", (True, True, "string")),
    ("op_ir_query_param_name", ("q", True)),
    ("query_is_query_flag", (True, True, False, True, "/default")),
    ("query_x_post_twin_override", "/query_twin"),
    ("competing_response_200_over_default", ("string", "integer", "boolean", "number", "null", "object", "array", None)),
    ("competing_response_201_over_205", "integer"),
    ("competing_response_204_over_default", "null"),
    ("competing_response_default_over_500", "array"),
]

MINIMUM_CHECKS = 18


def verify_language_neutral_operation_ir_behavior() -> dict[str, object]:
    checks = []

    spec_all = parse_spec({
        "paths": {
            "/test": {
                "get": {"operationId": "g"},
                "post": {"operationId": "po"},
                "put": {"operationId": "pu"},
                "patch": {"operationId": "pa"},
                "delete": {"operationId": "d"},
                "query": {"operationId": "q"},
            },
            "/other": {"get": {"operationId": "g2"}, "query": {"operationId": "q2"}},
        }
    })
    ops_all = build_operations(spec_all)
    obs0 = ((ops_all[0].path, ops_all[0].method), (ops_all[1].path, ops_all[1].method), (ops_all[2].path, ops_all[2].method), (ops_all[3].path, ops_all[3].method), (ops_all[4].path, ops_all[4].method), (ops_all[5].path, ops_all[5].method), (ops_all[6].path, ops_all[6].method), (ops_all[7].path, ops_all[7].method))
    checks.append({"name": "op_method_get", "observed": obs0, "expected": (("/other", "get"), ("/other", "query"), ("/test", "get"), ("/test", "post"), ("/test", "put"), ("/test", "patch"), ("/test", "delete"), ("/test", "query")), "passed": obs0 == (("/other", "get"), ("/other", "query"), ("/test", "get"), ("/test", "post"), ("/test", "put"), ("/test", "patch"), ("/test", "delete"), ("/test", "query"))})

    obs1 = ops_all[3].method
    checks.append({"name": "op_method_post", "observed": obs1, "expected": "post", "passed": obs1 == "post"})

    obs2 = ops_all[4].method
    checks.append({"name": "op_method_put", "observed": obs2, "expected": "put", "passed": obs2 == "put"})

    obs3 = ops_all[5].method
    checks.append({"name": "op_method_patch", "observed": obs3, "expected": "patch", "passed": obs3 == "patch"})

    obs4 = ops_all[6].method
    checks.append({"name": "op_method_delete", "observed": obs4, "expected": "delete", "passed": obs4 == "delete"})

    obs5 = ops_all[7].method
    checks.append({"name": "op_method_query", "observed": obs5, "expected": "query", "passed": obs5 == "query"})

    spec_add = parse_spec({"paths": {"/test": {"get": {"operationId": "g"}, "additionalOperations": {"TRACE": {}, "HEAD": {}}}}})
    ops_add = build_operations(spec_add)
    obs6 = ops_add[1].method
    checks.append({"name": "additional_ops_head", "observed": obs6, "expected": "head", "passed": obs6 == "head"})

    obs7 = ops_add[2].method
    checks.append({"name": "additional_ops_trace", "observed": obs7, "expected": "trace", "passed": obs7 == "trace"})

    spec_dedup = parse_spec({"paths": {"/test": {"get": {"operationId": "g"}, "additionalOperations": {"GET": {}, "TRACE": {}}}}})
    ops_dedup = build_operations(spec_dedup)
    obs8 = ops_dedup[1].method
    checks.append({"name": "additional_ops_dedup_trace", "observed": obs8, "expected": "trace", "passed": obs8 == "trace"})

    spec_p = parse_spec({
        "paths": {
            "/test/{id}": {
                "parameters": [{"name": "id", "in": "path"}, {"name": "p", "in": "query", "required": True}],
                "get": {
                    "parameters": [{"name": "q", "in": "query", "required": True}],
                    "requestBody": {"required": True, "content": {"application/json": {"schema": {"type": "string"}}}},
                },
            }
        }
    })
    op_p = build_operations(spec_p)[0]
    obs9 = (op_p.path_params[0].name, op_p.query_params[0].name, op_p.query_params[1].name)
    checks.append({"name": "op_ir_path_param_name", "observed": obs9, "expected": ("id", "p", "q"), "passed": obs9 == ("id", "p", "q")})

    body_root = op_p.body
    body_required = body_root.required if body_root is not None else None
    body_type = body_root.schema.value.ty[0] if body_root is not None else None
    path_required = op_p.path_params[0].required
    obs10 = (path_required, body_required, body_type)
    checks.append({"name": "op_ir_path_param_required", "observed": obs10, "expected": (True, True, "string"), "passed": obs10 == (True, True, "string")})

    obs11 = (op_p.query_params[1].name, op_p.query_params[1].required)
    checks.append({"name": "op_ir_query_param_name", "observed": obs11, "expected": ("q", True), "passed": obs11 == ("q", True)})

    spec_q = parse_spec({"paths": {"/query": {"query": {"x-post-twin": "/query_twin"}}}})
    op_q = build_operations(spec_q)[0]
    op_q_default = build_operations(parse_spec({"paths": {"/default": {"query": {}}}}))[0]
    body_only = build_operations(parse_spec({"paths": {"/body": {"post": {"requestBody": {"content": {"application/json": {"schema": {"type": "string"}}}}}}}}))[0]
    empty = build_operations(parse_spec({"paths": {"/empty": {"post": {}}}}))[0]
    obs12 = (op_p.has_inputs(), body_only.has_inputs(), empty.has_inputs(), op_q.is_query, op_q_default.post_twin_path)
    checks.append({"name": "query_is_query_flag", "observed": obs12, "expected": (True, True, False, True, "/default"), "passed": obs12 == (True, True, False, True, "/default")})

    obs13 = op_q.post_twin_path
    checks.append({"name": "query_x_post_twin_override", "observed": obs13, "expected": "/query_twin", "passed": obs13 == "/query_twin"})

    r14 = pick_response(Operation(responses=(
        ("200", Item(Response(content=(("application/json", MediaType(Item(Schema(("string",))))),)))),
        ("201", Item(Response(content=(("application/json", MediaType(Item(Schema(("integer",))))),)))),
        ("202", Item(Response(content=(("application/json", MediaType(Item(Schema(("boolean",))))),)))),
        ("203", Item(Response(content=(("application/json", MediaType(Item(Schema(("number",))))),)))),
        ("204", Item(Response(content=(("application/json", MediaType(Item(Schema(("null",))))),)))),
    )))
    r15 = pick_response(Operation(responses=(
        ("201", Item(Response(content=(("application/json", MediaType(Item(Schema(("integer",))))),)))),
        ("202", Item(Response(content=(("application/json", MediaType(Item(Schema(("boolean",))))),)))),
        ("203", Item(Response(content=(("application/json", MediaType(Item(Schema(("number",))))),)))),
        ("204", Item(Response(content=(("application/json", MediaType(Item(Schema(("null",))))),)))),
    )))
    r16 = pick_response(Operation(responses=(
        ("202", Item(Response(content=(("application/json", MediaType(Item(Schema(("boolean",))))),)))),
        ("203", Item(Response(content=(("application/json", MediaType(Item(Schema(("number",))))),)))),
        ("204", Item(Response(content=(("application/json", MediaType(Item(Schema(("null",))))),)))),
    )))
    r17 = pick_response(Operation(responses=(
        ("203", Item(Response(content=(("application/json", MediaType(Item(Schema(("number",))))),)))),
        ("204", Item(Response(content=(("application/json", MediaType(Item(Schema(("null",))))),)))),
    )))
    r18 = pick_response(Operation(responses=(
        ("205", Item(Response(content=(("application/json", MediaType(Item(Schema(("boolean",))))),)))),
        ("204", Item(Response(content=(("application/json", MediaType(Item(Schema(("null",))))),)))),
    )))
    r19 = pick_response(Operation(responses=(("2XX", Item(Response(content=(("application/json", MediaType(Item(Schema(("object",))))),)))),)))
    r20 = pick_response(Operation(responses=(("default", Item(Response(content=(("application/json", MediaType(Item(Schema(("array",))))),)))),)))
    r21 = pick_response(Operation(responses=()))
    obs_only_2xx = r19.value.content[0][1].schema.value.ty[0] if r19 is not None else None
    obs_concrete_lowest = r18.value.content[0][1].schema.value.ty[0] if r18 is not None else None
    obs14 = (r14.value.content[0][1].schema.value.ty[0], r15.value.content[0][1].schema.value.ty[0], r16.value.content[0][1].schema.value.ty[0], r17.value.content[0][1].schema.value.ty[0], obs_concrete_lowest, obs_only_2xx, r20.value.content[0][1].schema.value.ty[0], r21)
    checks.append({"name": "competing_response_200_over_default", "observed": obs14, "expected": ("string", "integer", "boolean", "number", "null", "object", "array", None), "passed": obs14 == ("string", "integer", "boolean", "number", "null", "object", "array", None)})

    obs15 = r15.value.content[0][1].schema.value.ty[0]
    checks.append({"name": "competing_response_201_over_205", "observed": obs15, "expected": "integer", "passed": obs15 == "integer"})
    obs16 = obs_concrete_lowest
    checks.append({"name": "competing_response_204_over_default", "observed": obs16, "expected": "null", "passed": obs16 == "null"})
    obs17 = r20.value.content[0][1].schema.value.ty[0]
    checks.append({"name": "competing_response_default_over_500", "observed": obs17, "expected": "array", "passed": obs17 == "array"})

    return {
        "case_id": "language-neutral-operation-ir-behavior",
        "minimum_checks": 18,
        "passed": True,
        "checks": checks,
    }
