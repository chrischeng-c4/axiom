from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass, field
from typing import Generic, TypeAlias, TypeVar, Union

T = TypeVar("T")


@dataclass(frozen=True)
class RefObj:
    reference: str


@dataclass(frozen=True)
class Ref:
    ref: RefObj


@dataclass(frozen=True)
class Item(Generic[T]):
    value: T


RefOr: TypeAlias = Union[Ref, Item[T]]


def parse_ref_or(raw: object, parse_item: Callable[[object], T]) -> RefOr[T]:
    if isinstance(raw, dict) and "$ref" in raw and isinstance(raw["$ref"], str):
        return Ref(RefObj(raw["$ref"]))
    return Item(parse_item(raw))


def parse_type_field(raw: object) -> tuple[str, ...]:
    if raw is None:
        return ()
    if isinstance(raw, str):
        return (raw,)
    if isinstance(raw, list):
        return tuple(x for x in raw if isinstance(x, str))
    return ()


@dataclass(frozen=True)
class AdditionalPropertiesBool:
    value: bool


@dataclass(frozen=True)
class AdditionalPropertiesSchema:
    schema: RefOr[Schema]


AdditionalProperties = AdditionalPropertiesBool | AdditionalPropertiesSchema


def parse_additional_properties(raw: object) -> AdditionalProperties | None:
    if raw is None:
        return None
    if isinstance(raw, bool):
        return AdditionalPropertiesBool(raw)
    return AdditionalPropertiesSchema(parse_ref_or(raw, parse_schema))


@dataclass(frozen=True)
class Schema:
    ty: tuple[str, ...] = ()
    format: str | None = None
    properties: tuple[tuple[str, RefOr[Schema]], ...] = ()
    required: tuple[str, ...] = ()
    items: RefOr[Schema] | None = None
    enum_values: tuple[object, ...] = ()
    one_of: tuple[RefOr[Schema], ...] = ()
    any_of: tuple[RefOr[Schema], ...] = ()
    all_of: tuple[RefOr[Schema], ...] = ()
    nullable: bool | None = None
    additional_properties: AdditionalProperties | None = None
    description: str | None = None

    def type_names(self) -> tuple[str, ...]:
        return tuple(x for x in self.ty if x != "null")

    def is_nullable(self) -> bool:
        return self.nullable is True or "null" in self.ty


def parse_schema(raw: object) -> Schema:
    if not isinstance(raw, dict):
        return Schema()

    ty = parse_type_field(raw.get("type"))
    fmt = raw.get("format") if isinstance(raw.get("format"), str) else None

    props_raw = raw.get("properties")
    props: list[tuple[str, RefOr[Schema]]] = []
    if isinstance(props_raw, dict):
        for k, v in props_raw.items():
            if isinstance(k, str):
                props.append((k, parse_ref_or(v, parse_schema)))
        props.sort(key=lambda p: p[0])

    req_raw = raw.get("required")
    req: list[str] = []
    if isinstance(req_raw, list):
        req = [x for x in req_raw if isinstance(x, str)]

    items_raw = raw.get("items")
    items: RefOr[Schema] | None = None
    if items_raw is not None:
        items = parse_ref_or(items_raw, parse_schema)

    enum_raw = raw.get("enum")
    enum_vals: list[object] = []
    if isinstance(enum_raw, list):
        enum_vals = list(enum_raw)

    one_of_raw = raw.get("oneOf")
    one_of: list[RefOr[Schema]] = []
    if isinstance(one_of_raw, list):
        one_of = [parse_ref_or(x, parse_schema) for x in one_of_raw]

    any_of_raw = raw.get("anyOf")
    any_of: list[RefOr[Schema]] = []
    if isinstance(any_of_raw, list):
        any_of = [parse_ref_or(x, parse_schema) for x in any_of_raw]

    all_of_raw = raw.get("allOf")
    all_of: list[RefOr[Schema]] = []
    if isinstance(all_of_raw, list):
        all_of = [parse_ref_or(x, parse_schema) for x in all_of_raw]

    nullable = raw.get("nullable") if isinstance(raw.get("nullable"), bool) else None

    add_props: AdditionalProperties | None = None
    if "additionalProperties" in raw:
        add_props = parse_additional_properties(raw["additionalProperties"])

    desc = raw.get("description") if isinstance(raw.get("description"), str) else None

    return Schema(
        ty=ty,
        format=fmt,
        properties=tuple(props),
        required=tuple(req),
        items=items,
        enum_values=tuple(enum_vals),
        one_of=tuple(one_of),
        any_of=tuple(any_of),
        all_of=tuple(all_of),
        nullable=nullable,
        additional_properties=add_props,
        description=desc,
    )


@dataclass(frozen=True)
class Parameter:
    name: str = ""
    location: str = ""
    required: bool = False
    schema: RefOr[Schema] | None = None


def parse_parameter(raw: object) -> Parameter:
    if not isinstance(raw, dict):
        return Parameter()

    name = str(raw["name"]) if "name" in raw and isinstance(raw["name"], str) else ""
    loc = str(raw["in"]) if "in" in raw and isinstance(raw["in"], str) else ""
    req = bool(raw["required"]) if "required" in raw and isinstance(raw["required"], bool) else False

    schema: RefOr[Schema] | None = None
    if "schema" in raw and raw["schema"] is not None:
        schema = parse_ref_or(raw["schema"], parse_schema)

    return Parameter(name=name, location=loc, required=req, schema=schema)


@dataclass(frozen=True)
class MediaType:
    schema: RefOr[Schema] | None = None


def parse_media_type(raw: object) -> MediaType:
    if not isinstance(raw, dict):
        return MediaType()

    schema: RefOr[Schema] | None = None
    if "schema" in raw and raw["schema"] is not None:
        schema = parse_ref_or(raw["schema"], parse_schema)

    return MediaType(schema=schema)


@dataclass(frozen=True)
class RequestBody:
    required: bool = False
    content: tuple[tuple[str, MediaType], ...] = ()


def parse_request_body(raw: object) -> RequestBody:
    if not isinstance(raw, dict):
        return RequestBody()

    req = bool(raw["required"]) if "required" in raw and isinstance(raw["required"], bool) else False

    content_raw = raw.get("content")
    content: list[tuple[str, MediaType]] = []
    if isinstance(content_raw, dict):
        for k, v in content_raw.items():
            if isinstance(k, str):
                content.append((k, parse_media_type(v)))
        content.sort(key=lambda p: p[0])

    return RequestBody(required=req, content=tuple(content))


@dataclass(frozen=True)
class Response:
    content: tuple[tuple[str, MediaType], ...] = ()


def parse_response(raw: object) -> Response:
    if not isinstance(raw, dict):
        return Response()

    content_raw = raw.get("content")
    content: list[tuple[str, MediaType]] = []
    if isinstance(content_raw, dict):
        for k, v in content_raw.items():
            if isinstance(k, str):
                content.append((k, parse_media_type(v)))
        content.sort(key=lambda p: p[0])

    return Response(content=tuple(content))


@dataclass(frozen=True)
class Operation:
    operation_id: str | None = None
    summary: str | None = None
    parameters: tuple[RefOr[Parameter], ...] = ()
    request_body: RefOr[RequestBody] | None = None
    responses: tuple[tuple[str, RefOr[Response]], ...] = ()
    tags: tuple[str, ...] = ()
    extensions: tuple[tuple[str, object], ...] = ()


def parse_operation(raw: object) -> Operation:
    if not isinstance(raw, dict):
        return Operation()

    op_id = raw.get("operationId") if isinstance(raw.get("operationId"), str) else None
    summary = raw.get("summary") if isinstance(raw.get("summary"), str) else None

    params_raw = raw.get("parameters")
    params: list[RefOr[Parameter]] = []
    if isinstance(params_raw, list):
        params = [parse_ref_or(x, parse_parameter) for x in params_raw]

    req_body_raw = raw.get("requestBody")
    req_body: RefOr[RequestBody] | None = None
    if req_body_raw is not None:
        req_body = parse_ref_or(req_body_raw, parse_request_body)

    resp_raw = raw.get("responses")
    responses: list[tuple[str, RefOr[Response]]] = []
    if isinstance(resp_raw, dict):
        for k, v in resp_raw.items():
            responses.append((str(k), parse_ref_or(v, parse_response)))
        responses.sort(key=lambda p: p[0])

    tags_raw = raw.get("tags")
    tags: list[str] = []
    if isinstance(tags_raw, list):
        tags = [x for x in tags_raw if isinstance(x, str)]

    named_keys = {"operationId", "summary", "parameters", "requestBody", "responses", "tags"}
    exts: list[tuple[str, object]] = []
    for k, v in raw.items():
        if isinstance(k, str) and k not in named_keys:
            exts.append((k, v))
    exts.sort(key=lambda p: p[0])

    return Operation(
        operation_id=op_id,
        summary=summary,
        parameters=tuple(params),
        request_body=req_body,
        responses=tuple(responses),
        tags=tuple(tags),
        extensions=tuple(exts),
    )


@dataclass(frozen=True)
class PathItem:
    get: Operation | None = None
    put: Operation | None = None
    post: Operation | None = None
    delete: Operation | None = None
    patch: Operation | None = None
    query: Operation | None = None
    additional_operations: tuple[tuple[str, Operation], ...] = ()
    parameters: tuple[RefOr[Parameter], ...] = ()


def parse_path_item(raw: object) -> PathItem:
    if not isinstance(raw, dict):
        return PathItem()

    get_op = parse_operation(raw["get"]) if "get" in raw and raw["get"] is not None else None
    put_op = parse_operation(raw["put"]) if "put" in raw and raw["put"] is not None else None
    post_op = parse_operation(raw["post"]) if "post" in raw and raw["post"] is not None else None
    delete_op = parse_operation(raw["delete"]) if "delete" in raw and raw["delete"] is not None else None
    patch_op = parse_operation(raw["patch"]) if "patch" in raw and raw["patch"] is not None else None
    query_op = parse_operation(raw["query"]) if "query" in raw and raw["query"] is not None else None

    add_ops_raw = raw.get("additionalOperations")
    add_ops: list[tuple[str, Operation]] = []
    if isinstance(add_ops_raw, dict):
        for k, v in add_ops_raw.items():
            if isinstance(k, str):
                add_ops.append((k, parse_operation(v)))
        add_ops.sort(key=lambda p: p[0])

    params_raw = raw.get("parameters")
    params: list[RefOr[Parameter]] = []
    if isinstance(params_raw, list):
        params = [parse_ref_or(x, parse_parameter) for x in params_raw]

    return PathItem(
        get=get_op,
        put=put_op,
        post=post_op,
        delete=delete_op,
        patch=patch_op,
        query=query_op,
        additional_operations=tuple(add_ops),
        parameters=tuple(params),
    )


@dataclass(frozen=True)
class Components:
    schemas: tuple[tuple[str, RefOr[Schema]], ...] = ()


def parse_components(raw: object) -> Components:
    if not isinstance(raw, dict):
        return Components()

    schemas_raw = raw.get("schemas")
    schemas: list[tuple[str, RefOr[Schema]]] = []
    if isinstance(schemas_raw, dict):
        for k, v in schemas_raw.items():
            if isinstance(k, str):
                schemas.append((k, parse_ref_or(v, parse_schema)))
        schemas.sort(key=lambda p: p[0])

    return Components(schemas=tuple(schemas))


@dataclass(frozen=True)
class Info:
    title: str = ""
    version: str = ""


def parse_info(raw: object) -> Info:
    if not isinstance(raw, dict):
        return Info()

    title = str(raw["title"]) if "title" in raw and isinstance(raw["title"], str) else ""
    version = str(raw["version"]) if "version" in raw and isinstance(raw["version"], str) else ""

    return Info(title=title, version=version)


@dataclass(frozen=True)
class Spec:
    openapi: str = ""
    info: Info = field(default_factory=Info)
    paths: tuple[tuple[str, PathItem], ...] = ()
    components: Components = field(default_factory=Components)


def parse_spec(raw: object) -> Spec:
    if not isinstance(raw, dict):
        return Spec()

    openapi = str(raw["openapi"]) if "openapi" in raw and isinstance(raw["openapi"], str) else ""
    info = parse_info(raw.get("info"))

    paths_raw = raw.get("paths")
    paths: list[tuple[str, PathItem]] = []
    if isinstance(paths_raw, dict):
        for k, v in paths_raw.items():
            if isinstance(k, str):
                paths.append((k, parse_path_item(v)))
        paths.sort(key=lambda p: p[0])

    comp: Components = Components()
    if "components" in raw:
        comp = parse_components(raw["components"])

    return Spec(
        openapi=openapi,
        info=info,
        paths=tuple(paths),
        components=comp,
    )
