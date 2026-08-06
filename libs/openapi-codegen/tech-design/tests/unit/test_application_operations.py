from __future__ import annotations

import sys
import unittest

sys.path.insert(0, __file__.rsplit("/", 3)[0] + "/src")

from openapi_codegen.application.document import (
    Item,
    Operation,
    Response,
    Schema,
    parse_operation,
    parse_spec,
)
from openapi_codegen.application.operations import (
    METHODS,
    BodyIR,
    OperationIR,
    ParamIR,
    build_operations,
    pick_response,
)


class TestApplicationOperations(unittest.TestCase):
    def test_methods_constant_order(self) -> None:
        self.assertEqual(
            METHODS, ("get", "post", "put", "patch", "delete", "query")
        )

    def test_path_item_methods_order(self) -> None:
        spec = parse_spec({
            "paths": {
                "/all": {
                    "patch": {},
                    "post": {},
                    "get": {},
                    "query": {},
                    "delete": {},
                    "put": {},
                }
            }
        })
        ops = build_operations(spec)
        methods = tuple(op.method for op in ops)
        self.assertEqual(
            methods, ("get", "post", "put", "patch", "delete", "query")
        )

    def test_additional_operations_dedup(self) -> None:
        spec = parse_spec({
            "paths": {
                "/test": {
                    "get": {"operationId": "standard_get"},
                    "additionalOperations": {
                        "GET": {"operationId": "extra_get"}
                    },
                }
            }
        })
        ops = build_operations(spec)
        self.assertEqual(len(ops), 1)
        self.assertEqual(ops[0].operation_id, "standard_get")

    def test_additional_operations_custom(self) -> None:
        spec = parse_spec({
            "paths": {
                "/test": {
                    "additionalOperations": {
                        "PURGE": {"operationId": "purge_op"}
                    }
                }
            }
        })
        ops = build_operations(spec)
        self.assertEqual(len(ops), 1)
        self.assertEqual(ops[0].method, "purge")
        self.assertEqual(ops[0].http_method, "PURGE")

    def test_path_param_required_forced(self) -> None:
        spec = parse_spec({
            "paths": {
                "/test/{id}": {
                    "get": {
                        "parameters": [
                            {"name": "id", "in": "path", "required": False}
                        ]
                    }
                }
            }
        })
        ops = build_operations(spec)
        self.assertEqual(len(ops[0].path_params), 1)
        self.assertTrue(ops[0].path_params[0].required)

    def test_query_param_required_preserved(self) -> None:
        spec = parse_spec({
            "paths": {
                "/test": {
                    "get": {
                        "parameters": [
                            {"name": "q", "in": "query", "required": False}
                        ]
                    }
                }
            }
        })
        ops = build_operations(spec)
        self.assertEqual(len(ops[0].query_params), 1)
        self.assertFalse(ops[0].query_params[0].required)

    def test_param_duplication_path_and_operation(self) -> None:
        spec = parse_spec({
            "paths": {
                "/test/{petId}": {
                    "parameters": [
                        {"name": "petId", "in": "path", "required": True}
                    ],
                    "get": {
                        "parameters": [
                            {"name": "petId", "in": "path", "required": True}
                        ]
                    },
                }
            }
        })
        ops = build_operations(spec)
        self.assertEqual(len(ops[0].path_params), 2)
        self.assertEqual(ops[0].path_params[0].name, "petId")
        self.assertEqual(ops[0].path_params[1].name, "petId")

    def test_ref_parameter_dropped(self) -> None:
        spec = parse_spec({
            "paths": {
                "/test": {
                    "get": {
                        "parameters": [
                            {"$ref": "#/components/parameters/PetId"}
                        ]
                    }
                }
            }
        })
        ops = build_operations(spec)
        self.assertEqual(ops[0].path_params, ())
        self.assertEqual(ops[0].query_params, ())

    def test_cookie_and_unknown_param_location_dropped(self) -> None:
        spec = parse_spec({
            "paths": {
                "/test": {
                    "get": {
                        "parameters": [
                            {"name": "session", "in": "cookie"},
                            {"name": "custom", "in": "custom_loc"},
                        ]
                    }
                }
            }
        })
        ops = build_operations(spec)
        self.assertEqual(ops[0].path_params, ())
        self.assertEqual(ops[0].query_params, ())
        self.assertEqual(ops[0].header_params, ())

    def test_body_non_json_content_dropped(self) -> None:
        spec = parse_spec({
            "paths": {
                "/test": {
                    "post": {
                        "requestBody": {
                            "content": {
                                "application/x-www-form-urlencoded": {
                                    "schema": {"type": "object"}
                                }
                            }
                        }
                    }
                }
            }
        })
        ops = build_operations(spec)
        self.assertIsNone(ops[0].body)

    def test_body_json_exact_key(self) -> None:
        spec = parse_spec({
            "paths": {
                "/test": {
                    "post": {
                        "requestBody": {
                            "content": {
                                "application/json; charset=utf-8": {
                                    "schema": {"type": "object"}
                                }
                            }
                        }
                    }
                }
            }
        })
        ops = build_operations(spec)
        self.assertIsNone(ops[0].body)

    def test_body_ref_dropped(self) -> None:
        spec = parse_spec({
            "paths": {
                "/test": {
                    "post": {
                        "requestBody": {
                            "$ref": "#/components/requestBodies/PetBody"
                        }
                    }
                }
            }
        })
        ops = build_operations(spec)
        self.assertIsNone(ops[0].body)

    def test_post_twin_path_non_query(self) -> None:
        spec = parse_spec({
            "paths": {
                "/test": {
                    "post": {"x-post-twin": "/other_path"}
                }
            }
        })
        ops = build_operations(spec)
        self.assertIsNone(ops[0].post_twin_path)

    def test_post_twin_path_query_extension(self) -> None:
        spec = parse_spec({
            "paths": {
                "/test": {
                    "query": {"x-post-twin": "/twin_path"}
                }
            }
        })
        ops = build_operations(spec)
        self.assertEqual(ops[0].post_twin_path, "/twin_path")

    def test_post_twin_path_query_default(self) -> None:
        spec = parse_spec({
            "paths": {
                "/test": {
                    "query": {}
                }
            }
        })
        ops = build_operations(spec)
        self.assertEqual(ops[0].post_twin_path, "/test")

    def test_is_query(self) -> None:
        spec = parse_spec({
            "paths": {
                "/test": {
                    "get": {},
                    "post": {},
                    "query": {},
                }
            }
        })
        ops = build_operations(spec)
        op_dict = {op.method: op.is_query for op in ops}
        self.assertTrue(op_dict["get"])
        self.assertTrue(op_dict["query"])
        self.assertFalse(op_dict["post"])

    def test_pick_response_preferred_order(self) -> None:
        op = parse_operation({
            "responses": {
                "203": {"content": {"application/json": {}}},
                "201": {"content": {"application/json": {}}},
            }
        })
        res = pick_response(op)
        self.assertIsInstance(res, Item)
        assert isinstance(res, Item)
        self.assertEqual(op.responses[0][0], "201")  # 201 is first in sorted order ("201" < "203")

    def test_pick_response_sorted_range(self) -> None:
        op = parse_operation({
            "responses": {
                "299": {"content": {"application/json": {}}},
                "204": {"content": {"application/json": {}}},
            }
        })
        res = pick_response(op)
        self.assertEqual(res, op.responses[0][1])
        self.assertEqual(op.responses[0][0], "204")

    def test_pick_response_direct_operation_is_order_independent(self) -> None:
        response_205 = Item(Response())
        response_204 = Item(Response())
        op = Operation(responses=(("205", response_205), ("204", response_204)))
        self.assertIs(pick_response(op), response_204)

    def test_pick_response_concrete_beats_generic_2xx(self) -> None:
        response_2xx = Item(Response())
        response_204 = Item(Response())
        op = Operation(responses=(("2XX", response_2xx), ("204", response_204)))
        self.assertIs(pick_response(op), response_204)

    def test_pick_response_unreachable_2xx_scan(self) -> None:
        # "2XX" starts with "2", so step 2 (range scan) returns it before step 3
        op_only_2xx = parse_operation({
            "responses": {"2XX": {"content": {"application/json": {}}}}
        })
        self.assertIsNotNone(pick_response(op_only_2xx))

        # "204" sorts before "2XX" because digits sort before uppercase letters
        op_204_and_2xx = parse_operation({
            "responses": {
                "2XX": {"content": {"application/json": {}}},
                "204": {"content": {"application/json": {}}},
            }
        })
        self.assertEqual(pick_response(op_204_and_2xx), op_204_and_2xx.responses[0][1])
        self.assertEqual(op_204_and_2xx.responses[0][0], "204")

    def test_pick_response_default_fallback(self) -> None:
        op = parse_operation({
            "responses": {
                "404": {"content": {"application/json": {}}},
                "default": {"content": {"application/json": {}}},
            }
        })
        res = pick_response(op)
        self.assertEqual(res, op.responses[1][1])
        self.assertEqual(op.responses[1][0], "default")

    def test_pick_response_default_ignored_if_2xx(self) -> None:
        op = parse_operation({
            "responses": {
                "201": {"content": {"application/json": {}}},
                "default": {"content": {"application/json": {}}},
            }
        })
        res = pick_response(op)
        self.assertEqual(res, op.responses[0][1])
        self.assertEqual(op.responses[0][0], "201")

    def test_pick_response_empty(self) -> None:
        op = parse_operation({"responses": {}})
        self.assertIsNone(pick_response(op))

    def test_has_inputs_true_params(self) -> None:
        op_ir = OperationIR(
            operation_id="op",
            method="get",
            http_method="GET",
            is_query=True,
            path="/test",
            path_params=(),
            query_params=(ParamIR("q", None, False),),
            header_params=(),
            body=None,
            response=None,
            post_twin_path=None,
        )
        self.assertTrue(op_ir.has_inputs())

    def test_has_inputs_true_body(self) -> None:
        op_ir = OperationIR(
            operation_id="op",
            method="post",
            http_method="POST",
            is_query=False,
            path="/test",
            path_params=(),
            query_params=(),
            header_params=(),
            body=BodyIR(Item(Schema()), True),
            response=None,
            post_twin_path=None,
        )
        self.assertTrue(op_ir.has_inputs())

    def test_has_inputs_false(self) -> None:
        op_ir = OperationIR(
            operation_id="op",
            method="get",
            http_method="GET",
            is_query=True,
            path="/test",
            path_params=(),
            query_params=(),
            header_params=(),
            body=None,
            response=None,
            post_twin_path=None,
        )
        self.assertFalse(op_ir.has_inputs())


if __name__ == "__main__":
    unittest.main()
