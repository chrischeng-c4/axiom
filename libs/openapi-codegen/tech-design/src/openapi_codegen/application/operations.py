from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass

from openapi_codegen.application.document import (
    Item,
    Operation,
    Parameter,
    PathItem,
    RefOr,
    RequestBody,
    Response,
    Schema,
    Spec,
)


@dataclass(frozen=True)
class ParamIR:
    name: str
    schema: RefOr[Schema] | None
    required: bool


@dataclass(frozen=True)
class BodyIR:
    schema: RefOr[Schema]
    required: bool


@dataclass(frozen=True)
class OperationIR:
    operation_id: str | None
    method: str
    http_method: str
    is_query: bool
    path: str
    path_params: tuple[ParamIR, ...]
    query_params: tuple[ParamIR, ...]
    header_params: tuple[ParamIR, ...]
    body: BodyIR | None
    response: RefOr[Schema] | None
    post_twin_path: str | None

    def has_inputs(self) -> bool:
        return (
            bool(self.path_params)
            or bool(self.query_params)
            or bool(self.header_params)
            or (self.body is not None)
        )


METHOD_FIELDS: tuple[tuple[str, Callable[[PathItem], Operation | None]], ...] = (
    ("get", lambda item: item.get),
    ("post", lambda item: item.post),
    ("put", lambda item: item.put),
    ("patch", lambda item: item.patch),
    ("delete", lambda item: item.delete),
    ("query", lambda item: item.query),
)

METHODS: tuple[str, ...] = tuple(name for name, _ in METHOD_FIELDS)


def pick_response(op: Operation) -> RefOr[Response] | None:
    res_dict = dict(op.responses)
    for code in ("200", "201", "202", "203"):
        if code in res_dict:
            return res_dict[code]
    remaining = sorted(k for k in res_dict if k.startswith("2") and k != "2XX")
    if remaining:
        return res_dict[remaining[0]]
    if "2XX" in res_dict:
        return res_dict["2XX"]
    if "default" in res_dict:
        return res_dict["default"]
    return None


def inline_params(params: tuple[RefOr[Parameter], ...]) -> list[Parameter]:
    result: list[Parameter] = []
    for p in params:
        if isinstance(p, Item) and isinstance(p.value, Parameter):
            result.append(p.value)
    return result


def build_one(
    method: str,
    path: str,
    op: Operation,
    path_level_params: tuple[RefOr[Parameter], ...],
) -> OperationIR:
    path_params: list[ParamIR] = []
    query_params: list[ParamIR] = []
    header_params: list[ParamIR] = []

    all_params = inline_params(path_level_params) + inline_params(op.parameters)
    for p in all_params:
        if p.location == "path":
            path_params.append(ParamIR(p.name, p.schema, True))
        elif p.location == "query":
            query_params.append(ParamIR(p.name, p.schema, p.required))
        elif p.location == "header":
            header_params.append(ParamIR(p.name, p.schema, p.required))

    body: BodyIR | None = None
    if isinstance(op.request_body, Item) and isinstance(
        op.request_body.value, RequestBody
    ):
        req_body = op.request_body.value
        content_dict = dict(req_body.content)
        if "application/json" in content_dict:
            mt = content_dict["application/json"]
            if mt.schema is not None:
                body = BodyIR(mt.schema, req_body.required)

    response: RefOr[Schema] | None = None
    r = pick_response(op)
    if isinstance(r, Item) and isinstance(r.value, Response):
        resp_obj = r.value
        content_dict = dict(resp_obj.content)
        if "application/json" in content_dict:
            mt = content_dict["application/json"]
            if mt.schema is not None:
                response = mt.schema

    post_twin_path: str | None = None
    if method == "query":
        ext_dict = dict(op.extensions)
        if "x-post-twin" in ext_dict and isinstance(
            ext_dict["x-post-twin"], str
        ):
            post_twin_path = ext_dict["x-post-twin"]
        else:
            post_twin_path = path

    is_query = method == "get" or method == "query"
    http_method = method.upper()

    return OperationIR(
        operation_id=op.operation_id,
        method=method,
        http_method=http_method,
        is_query=is_query,
        path=path,
        path_params=tuple(path_params),
        query_params=tuple(query_params),
        header_params=tuple(header_params),
        body=body,
        response=response,
        post_twin_path=post_twin_path,
    )


def build_operations(spec: Spec) -> tuple[OperationIR, ...]:
    ops: list[OperationIR] = []
    for path, item in spec.paths:
        for method, accessor in METHOD_FIELDS:
            op = accessor(item)
            if op is not None:
                ops.append(build_one(method, path, op, item.parameters))
        for method_upper, op in item.additional_operations:
            method_lower = method_upper.lower()
            if method_lower in METHODS:
                continue
            ops.append(build_one(method_lower, path, op, item.parameters))
    return tuple(ops)
