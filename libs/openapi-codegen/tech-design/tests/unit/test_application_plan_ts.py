from __future__ import annotations

import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from openapi_codegen.application.document import parse_spec
from openapi_codegen.application.operations import OperationIR, ParamIR, BodyIR
from openapi_codegen.application.plan_ts import (
    BodyField,
    OperationPlan,
    ParamField,
    build,
    build_one,
    fallback_name,
    op_base_name,
    param_field,
)
from openapi_codegen.application.typemap import build_type_map
from openapi_codegen.domain.names import NameRegistry


class TestApplicationPlanTs(unittest.TestCase):
    def test_fallback_name_shapes(self) -> None:
        self.assertEqual(fallback_name("get", "/pets/{petId}"), "getPetsByPetId")
        self.assertEqual(fallback_name("post", "/pets"), "postPets")
        self.assertEqual(fallback_name("GET", "/pets"), "getPets")
        self.assertEqual(fallback_name("get", "/"), "get")
        self.assertEqual(fallback_name("delete", "/a/{b}/c/{d}"), "deleteAbyBcbyD")
        self.assertEqual(fallback_name("query", "/search/items"), "querySearchItems")

    def test_operation_id_wins(self) -> None:
        spec = parse_spec({
            "paths": {
                "/pets/{petId}": {
                    "get": {
                        "operationId": "getPetById",
                        "parameters": [
                            {"name": "petId", "in": "path", "required": True, "schema": {"type": "integer"}},
                            {"name": "expand", "in": "query", "required": False, "schema": {"type": "boolean"}},
                        ],
                        "responses": {
                            "200": {"content": {"application/json": {"schema": {"type": "string"}}}}
                        },
                    }
                }
            }
        })
        tm = build_type_map(spec)
        plans = build(spec, tm)
        self.assertEqual(len(plans), 1)
        p = plans[0]
        self.assertEqual(p.fn_name, "getPetById")
        self.assertTrue(p.is_query)
        self.assertEqual(p.http_method, "GET")
        self.assertEqual(len(p.path_params), 1)
        self.assertEqual(len(p.query_params), 1)
        self.assertFalse(p.query_required())
        self.assertEqual(p.path_params[0].name, "petId")
        self.assertEqual(p.path_params[0].ts_type, "number")
        self.assertTrue(p.path_params[0].required)
        self.assertEqual(p.data_type_name, "GetPetByIdData")
        self.assertEqual(p.response_type_name, "GetPetByIdResponse")
        self.assertEqual(p.response_type, "string")
        self.assertIsNone(p.post_twin_path)

    def test_blank_whitespace_operation_id_falls_back(self) -> None:
        spec = parse_spec({
            "paths": {
                "/pets": {
                    "get": {"operationId": "   ", "responses": {}}
                }
            }
        })
        tm = build_type_map(spec)
        plans = build(spec, tm)
        self.assertEqual(plans[0].fn_name, "getPets")

    def test_no_input_operation(self) -> None:
        spec = parse_spec({
            "paths": {
                "/health": {
                    "get": {
                        "operationId": "health",
                        "responses": {
                            "200": {"content": {"application/json": {"schema": {"type": "boolean"}}}}
                        },
                    }
                }
            }
        })
        tm = build_type_map(spec)
        plans = build(spec, tm)
        p = plans[0]
        self.assertIsNone(p.data_type_name)
        self.assertFalse(p.has_inputs())
        self.assertEqual(p.response_type_name, "HealthResponse")
        self.assertEqual(p.response_type, "boolean")

    def test_no_response_type_void(self) -> None:
        spec = parse_spec({
            "paths": {
                "/ping": {
                    "get": {"operationId": "ping", "responses": {}}
                }
            }
        })
        tm = build_type_map(spec)
        plans = build(spec, tm)
        self.assertEqual(plans[0].response_type, "void")

    def test_component_name_collision_with_data_type(self) -> None:
        spec = parse_spec({
            "components": {
                "schemas": {
                    "GetThingData": {}
                }
            },
            "paths": {
                "/thing": {
                    "get": {
                        "operationId": "getThing",
                        "parameters": [{"name": "q", "in": "query", "required": True}],
                        "responses": {},
                    }
                }
            },
        })
        tm = build_type_map(spec)
        self.assertEqual(tm.get("GetThingData"), "GetThingData")
        plans = build(spec, tm)
        self.assertEqual(plans[0].data_type_name, "GetThingData_2")
        self.assertEqual(plans[0].response_type_name, "GetThingResponse")

    def test_allocation_order_no_body(self) -> None:
        spec = parse_spec({
            "components": {
                "schemas": {
                    "GetThingResponse": {}
                }
            },
            "paths": {
                "/thing": {
                    "get": {
                        "operationId": "getThing",
                        "responses": {},
                    }
                }
            },
        })
        tm = build_type_map(spec)
        plans = build(spec, tm)
        self.assertIsNone(plans[0].data_type_name)
        self.assertEqual(plans[0].response_type_name, "GetThingResponse_2")

    def test_function_name_collision(self) -> None:
        spec = parse_spec({
            "paths": {
                "/pets": {
                    "get": {"operationId": "getPets", "responses": {}},
                    "post": {"operationId": "getPets", "responses": {}},
                }
            }
        })
        tm = build_type_map(spec)
        plans = build(spec, tm)
        self.assertEqual(plans[0].fn_name, "getPets")
        self.assertEqual(plans[1].fn_name, "getPets_2")

    def test_query_required_and_headers_required(self) -> None:
        spec = parse_spec({
            "paths": {
                "/test": {
                    "get": {
                        "operationId": "testOp",
                        "parameters": [
                            {"name": "X-Auth", "in": "header", "required": True},
                            {"name": "q", "in": "query", "required": True},
                        ],
                        "responses": {},
                    }
                }
            }
        })
        tm = build_type_map(spec)
        plans = build(spec, tm)
        p = plans[0]
        self.assertTrue(p.query_required())
        self.assertTrue(p.headers_required())

    def test_param_no_schema_fallback_string(self) -> None:
        spec = parse_spec({
            "paths": {
                "/test": {
                    "get": {
                        "operationId": "testOp",
                        "parameters": [{"name": "q", "in": "query"}],
                        "responses": {},
                    }
                }
            }
        })
        tm = build_type_map(spec)
        plans = build(spec, tm)
        self.assertEqual(plans[0].query_params[0].ts_type, "string")

    def test_query_post_twin_path(self) -> None:
        spec = parse_spec({
            "paths": {
                "/search": {
                    "query": {
                        "operationId": "search",
                        "responses": {},
                    }
                }
            }
        })
        tm = build_type_map(spec)
        plans = build(spec, tm)
        p = plans[0]
        self.assertEqual(p.http_method, "QUERY")
        self.assertEqual(p.post_twin_path, "/search")
        self.assertEqual(p.response_type, "void")

    def test_plan_order_matches_build_operations(self) -> None:
        spec = parse_spec({
            "paths": {
                "/b": {"get": {"operationId": "opB"}},
                "/a": {"get": {"operationId": "opA"}},
            }
        })
        tm = build_type_map(spec)
        plans = build(spec, tm)
        fn_names = tuple(p.fn_name for p in plans)
        self.assertEqual(fn_names, ("opA", "opB"))

    def test_body_field_dataclass(self) -> None:
        bf = BodyField(ts_type="Pet", required=True)
        self.assertEqual(bf.ts_type, "Pet")
        self.assertTrue(bf.required)

    def test_param_field_dataclass(self) -> None:
        pf = ParamField(name="id", ts_type="number", required=True)
        self.assertEqual(pf.name, "id")
        self.assertEqual(pf.ts_type, "number")
        self.assertTrue(pf.required)

    def test_operation_plan_dataclass_fields(self) -> None:
        op = OperationPlan(
            fn_name="getPets",
            http_method="GET",
            is_query=True,
            path_raw="/pets",
            path_params=(),
            query_params=(),
            header_params=(),
            body=None,
            response_type="string",
            data_type_name=None,
            response_type_name="GetPetsResponse",
            post_twin_path=None,
        )
        self.assertEqual(op.fn_name, "getPets")
        self.assertEqual(op.http_method, "GET")
        self.assertTrue(op.is_query)

    def test_fallback_name_multiple_braced_segments(self) -> None:
        self.assertEqual(
            fallback_name("get", "/users/{userId}/posts/{postId}"),
            "getUsersByUserIdPostsByPostId",
        )

    def test_fallback_name_hyphenated_path(self) -> None:
        self.assertEqual(
            fallback_name("get", "/user-profiles/{id}"),
            "getUserProfilesById",
        )

    def test_operation_plan_headers_required_false(self) -> None:
        op = OperationPlan(
            fn_name="f",
            http_method="GET",
            is_query=True,
            path_raw="/",
            path_params=(),
            query_params=(),
            header_params=(ParamField("h", "string", False),),
            body=None,
            response_type="void",
            data_type_name=None,
            response_type_name="FResponse",
            post_twin_path=None,
        )
        self.assertFalse(op.headers_required())

    def test_operation_plan_query_required_false(self) -> None:
        op = OperationPlan(
            fn_name="f",
            http_method="GET",
            is_query=True,
            path_raw="/",
            path_params=(),
            query_params=(ParamField("q", "string", False),),
            header_params=(),
            body=None,
            response_type="void",
            data_type_name=None,
            response_type_name="FResponse",
            post_twin_path=None,
        )
        self.assertFalse(op.query_required())

    def test_op_base_name_fallback(self) -> None:
        ir = OperationIR(
            operation_id=None,
            method="get",
            http_method="GET",
            is_query=True,
            path="/items",
            path_params=(),
            query_params=(),
            header_params=(),
            body=None,
            response=None,
            post_twin_path=None,
        )
        self.assertEqual(op_base_name(ir), "getItems")

    def test_build_one_header_params(self) -> None:
        spec = parse_spec({
            "paths": {
                "/test": {
                    "get": {
                        "operationId": "testHeaders",
                        "parameters": [{"name": "X-Key", "in": "header", "required": True}],
                        "responses": {},
                    }
                }
            }
        })
        tm = build_type_map(spec)
        plans = build(spec, tm)
        self.assertEqual(len(plans[0].header_params), 1)
        self.assertEqual(plans[0].header_params[0].name, "X-Key")

    def test_build_one_body_required(self) -> None:
        spec = parse_spec({
            "paths": {
                "/test": {
                    "post": {
                        "operationId": "createItem",
                        "requestBody": {
                            "required": True,
                            "content": {"application/json": {"schema": {"type": "string"}}},
                        },
                        "responses": {},
                    }
                }
            }
        })
        tm = build_type_map(spec)
        plans = build(spec, tm)
        self.assertIsNotNone(plans[0].body)
        assert plans[0].body is not None
        self.assertTrue(plans[0].body.required)

    def test_build_one_response_type_ref(self) -> None:
        spec = parse_spec({
            "components": {"schemas": {"Pet": {}}},
            "paths": {
                "/pet": {
                    "get": {
                        "operationId": "getPet",
                        "responses": {
                            "200": {"content": {"application/json": {"schema": {"$ref": "#/components/schemas/Pet"}}}}
                        },
                    }
                }
            },
        })
        tm = build_type_map(spec)
        plans = build(spec, tm)
        self.assertEqual(plans[0].response_type, "Pet")


if __name__ == "__main__":
    unittest.main()
