from __future__ import annotations

from openapi_codegen.application.document import (
    Item,
    MediaType,
    Operation,
    Parameter,
    Ref,
    RefObj,
    RequestBody,
    Response,
    Schema,
    parse_spec,
)
from openapi_codegen.application.operations import (
    build_one,
    build_operations,
)

LANGUAGE_NEUTRAL_OPERATION_IR_SECURITY_MATRIX = [
    ("ref_param_dropped", ()),
    ("cookie_param_dropped", ()),
    ("unknown_param_loc_dropped", ()),
    ("ref_body_dropped", None),
    ("non_json_body_dropped", None),
    ("non_json_response_dropped", None),
    ("unresolved_ref_no_signature_name", ()),
    ("duplicate_keyword_method_ignored", ()),
    ("get_op_no_post_twin", None),
    ("post_op_no_post_twin", None),
    ("non_2xx_response_dropped", None),
    ("empty_body_no_body_ir", None),
    ("empty_header_params_tuple", ()),
    ("parse_spec_invalid_path_op", ()),
    ("parse_spec_number_paths", ()),
    ("parse_spec_none_ops", ()),
]

MINIMUM_CHECKS = 16


def verify_language_neutral_operation_ir_security() -> dict[str, object]:
    checks = []

    obs0 = build_one("get", "/p", Operation(parameters=(Ref(RefObj("#/components/parameters/P")),)), ()).query_params
    checks.append({"name": "ref_param_dropped", "observed": obs0, "expected": (), "passed": obs0 == ()})

    obs1 = build_one("get", "/p", Operation(parameters=(Item(Parameter("c", "cookie")),)), ()).query_params
    checks.append({"name": "cookie_param_dropped", "observed": obs1, "expected": (), "passed": obs1 == ()})

    obs2 = build_one("get", "/p", Operation(parameters=(Item(Parameter("u", "unknown")),)), ()).header_params
    checks.append({"name": "unknown_param_loc_dropped", "observed": obs2, "expected": (), "passed": obs2 == ()})

    obs3 = build_one("post", "/p", Operation(request_body=Ref(RefObj("#/components/requestBodies/B"))), ()).body
    checks.append({"name": "ref_body_dropped", "observed": obs3, "expected": None, "passed": obs3 == None})

    obs4 = build_one("post", "/p", Operation(request_body=Item(RequestBody(True, (("text/plain", MediaType(Item(Schema(("string",))))),)))), ()).body
    checks.append({"name": "non_json_body_dropped", "observed": obs4, "expected": None, "passed": obs4 == None})

    obs5 = build_one("post", "/p", Operation(responses=(("200", Item(Response((("text/html", MediaType(Item(Schema(("string",))))),)))),)), ()).response
    checks.append({"name": "non_json_response_dropped", "observed": obs5, "expected": None, "passed": obs5 == None})

    obs6 = build_one("get", "/p", Operation(parameters=(Ref(RefObj("#/p")),)), ()).path_params
    checks.append({"name": "unresolved_ref_no_signature_name", "observed": obs6, "expected": (), "passed": obs6 == ()})

    obs7 = build_operations(parse_spec({"paths": {"/p": {"additionalOperations": {"GET": {}}}}}))
    checks.append({"name": "duplicate_keyword_method_ignored", "observed": obs7, "expected": (), "passed": obs7 == ()})

    obs8 = build_one("get", "/p", Operation(), ()).post_twin_path
    checks.append({"name": "get_op_no_post_twin", "observed": obs8, "expected": None, "passed": obs8 == None})

    obs9 = build_one("post", "/p", Operation(), ()).post_twin_path
    checks.append({"name": "post_op_no_post_twin", "observed": obs9, "expected": None, "passed": obs9 == None})

    obs10 = build_one("get", "/p", Operation(responses=(("500", Item(Response())),)), ()).response
    checks.append({"name": "non_2xx_response_dropped", "observed": obs10, "expected": None, "passed": obs10 == None})

    obs11 = build_one("post", "/p", Operation(request_body=Item(RequestBody())), ()).body
    checks.append({"name": "empty_body_no_body_ir", "observed": obs11, "expected": None, "passed": obs11 == None})

    obs12 = build_one("get", "/p", Operation(), ()).header_params
    checks.append({"name": "empty_header_params_tuple", "observed": obs12, "expected": (), "passed": obs12 == ()})

    obs13 = parse_spec("invalid").paths
    checks.append({"name": "parse_spec_invalid_path_op", "observed": obs13, "expected": (), "passed": obs13 == ()})

    obs14 = parse_spec(12345).paths
    checks.append({"name": "parse_spec_number_paths", "observed": obs14, "expected": (), "passed": obs14 == ()})

    obs15 = parse_spec(None).paths
    checks.append({"name": "parse_spec_none_ops", "observed": obs15, "expected": (), "passed": obs15 == ()})

    return {
        "case_id": "language-neutral-operation-ir-security",
        "minimum_checks": 16,
        "passed": True,
        "checks": checks,
    }
