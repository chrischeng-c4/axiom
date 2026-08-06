from __future__ import annotations

from openapi_codegen.application.document import (
    AdditionalPropertiesBool,
    AdditionalPropertiesSchema,
    Item,
    Ref,
    RefOr,
    Schema,
)
from openapi_codegen.application.typemap import TypeMap
from openapi_codegen.domain.names import prop_key


def type_expr(node: RefOr[Schema], tm: TypeMap) -> str:
    if isinstance(node, Ref):
        resolved = tm.resolve_ref(node.ref.reference)
        if resolved is not None:
            return resolved
        return f"unknown /* unsupported $ref {node.ref.reference} */"
    if isinstance(node, Item):
        schema = node.value
        base = base_expr(schema, tm)
        if schema.is_nullable() and base != "null" and not base.startswith("unknown"):
            return base + " | null"
        return base
    raise ValueError(f"Invalid RefOr node: {node}")


def scalar_expr(ty: str, schema: Schema) -> str:
    if ty == "string":
        return string_expr(schema)
    if ty in ("integer", "number"):
        return "number"
    if ty == "boolean":
        return "boolean"
    if ty == "null":
        return "null"
    return "unknown"


def string_expr(schema: Schema) -> str:
    return "Blob" if schema.format == "binary" else "string"


def array_expr(schema: Schema, tm: TypeMap) -> str:
    if schema.items is None:
        return "unknown[]"
    inner = type_expr(schema.items, tm)
    if " | " in inner or " & " in inner:
        return f"({inner})[]"
    return f"{inner}[]"


def join(items: tuple[RefOr[Schema], ...], sep: str, tm: TypeMap) -> str:
    return sep.join(type_expr(i, tm) for i in items)


def object_expr(schema: Schema, tm: TypeMap) -> str:
    parts: list[str] = []
    for key, prop in schema.properties:
        optional = key not in schema.required
        opt_str = "?" if optional else ""
        parts.append(f"{prop_key(key)}{opt_str}: {type_expr(prop, tm)}")

    add_props = schema.additional_properties
    if isinstance(add_props, AdditionalPropertiesBool):
        if add_props.value:
            parts.append("[key: string]: unknown")
    elif isinstance(add_props, AdditionalPropertiesSchema):
        parts.append(f"[key: string]: {type_expr(add_props.schema, tm)}")

    if not parts:
        return "Record<string, unknown>"
    return "{ " + "; ".join(parts) + " }"


def enum_union(schema: Schema) -> str:
    members: list[str] = []
    for v in schema.enum_values:
        if isinstance(v, bool):
            members.append("true" if v else "false")
        elif isinstance(v, str):
            members.append('"' + v.replace('"', '\\"') + '"')
        elif isinstance(v, (int, float)):
            members.append(str(v))
        elif v is None:
            members.append("null")
        else:
            members.append("unknown")
    if not members:
        members = ["never"]
    return " | ".join(members)


def base_expr(schema: Schema, tm: TypeMap) -> str:
    if schema.all_of:
        return join(schema.all_of, " & ", tm)
    if schema.one_of:
        return join(schema.one_of, " | ", tm)
    if schema.any_of:
        return join(schema.any_of, " | ", tm)
    if schema.enum_values:
        return enum_union(schema)

    tnames = schema.type_names()
    if len(tnames) > 1:
        return " | ".join(scalar_expr(t, schema) for t in tnames)

    if tnames:
        t0 = tnames[0]
        if t0 == "object":
            return object_expr(schema, tm)
        if t0 == "array":
            return array_expr(schema, tm)
        if t0 == "string":
            return string_expr(schema)
        if t0 in ("integer", "number"):
            return "number"
        if t0 == "boolean":
            return "boolean"
        if t0 == "null":
            return "null"
        return "unknown"

    if schema.properties or schema.additional_properties is not None:
        return object_expr(schema, tm)
    return "unknown"
