from __future__ import annotations

from dataclasses import dataclass

from openapi_codegen.application import tsmap
from openapi_codegen.application.document import Spec
from openapi_codegen.application.operations import (
    OperationIR,
    ParamIR,
    build_operations,
)
from openapi_codegen.application.typemap import TypeMap
from openapi_codegen.domain.names import NameRegistry, to_camel, to_pascal


@dataclass(frozen=True)
class ParamField:
    name: str
    ts_type: str
    required: bool


@dataclass(frozen=True)
class BodyField:
    ts_type: str
    required: bool


@dataclass(frozen=True)
class OperationPlan:
    fn_name: str
    http_method: str
    is_query: bool
    path_raw: str
    path_params: tuple[ParamField, ...]
    query_params: tuple[ParamField, ...]
    header_params: tuple[ParamField, ...]
    body: BodyField | None
    response_type: str
    data_type_name: str | None
    response_type_name: str
    post_twin_path: str | None

    def has_inputs(self) -> bool:
        return self.data_type_name is not None

    def query_required(self) -> bool:
        return any(p.required for p in self.query_params)

    def headers_required(self) -> bool:
        return any(p.required for p in self.header_params)


def fallback_name(method: str, path: str) -> str:
    s = method.lower()
    for seg in path.split("/"):
        if not seg:
            continue
        if seg.startswith("{"):
            inner = seg.lstrip("{").rstrip("}")
            s = s + "By" + to_pascal(inner)
        else:
            s = s + to_pascal(seg)
    return to_camel(s)


def op_base_name(ir: OperationIR) -> str:
    if ir.operation_id is not None and ir.operation_id.strip() != "":
        return to_camel(ir.operation_id)
    return fallback_name(ir.method, ir.path)


def param_field(p: ParamIR, tm: TypeMap) -> ParamField:
    ts_type = tsmap.type_expr(p.schema, tm) if p.schema is not None else "string"
    return ParamField(p.name, ts_type, p.required)


def build_one(
    ir: OperationIR, tm: TypeMap, fn_reg: NameRegistry, type_reg: NameRegistry
) -> OperationPlan:
    fn_name = fn_reg.unique(op_base_name(ir))

    path_params = tuple(param_field(p, tm) for p in ir.path_params)
    query_params = tuple(param_field(p, tm) for p in ir.query_params)
    header_params = tuple(param_field(p, tm) for p in ir.header_params)

    body: BodyField | None = None
    if ir.body is not None:
        body = BodyField(
            ts_type=tsmap.type_expr(ir.body.schema, tm),
            required=ir.body.required,
        )

    response_type = (
        tsmap.type_expr(ir.response, tm) if ir.response is not None else "void"
    )

    pascal = to_pascal(fn_name)

    # ORDER IS PART OF THE CONTRACT: Data is claimed before Response
    data_type_name = (
        type_reg.unique(pascal + "Data") if ir.has_inputs() else None
    )
    response_type_name = type_reg.unique(pascal + "Response")

    return OperationPlan(
        fn_name=fn_name,
        http_method=ir.http_method,
        is_query=ir.is_query,
        path_raw=ir.path,
        path_params=path_params,
        query_params=query_params,
        header_params=header_params,
        body=body,
        response_type=response_type,
        data_type_name=data_type_name,
        response_type_name=response_type_name,
        post_twin_path=ir.post_twin_path,
    )


def build(spec: Spec, tm: TypeMap) -> tuple[OperationPlan, ...]:
    fn_reg = NameRegistry()
    type_reg = NameRegistry()

    # Reserve every component type name FIRST
    for _, type_name in tm.names:
        type_reg.unique(type_name)

    return tuple(
        build_one(ir, tm, fn_reg, type_reg) for ir in build_operations(spec)
    )
