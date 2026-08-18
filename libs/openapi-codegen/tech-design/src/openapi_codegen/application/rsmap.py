from __future__ import annotations

from openapi_codegen.application.document import (
    AdditionalPropertiesSchema,
    Item,
    Ref,
    RefOr,
    Schema,
)
from openapi_codegen.application.typemap import TypeMap

ANY = "serde_json::Value"


def optional(ty: str) -> str:
    if ty == ANY or ty.startswith("Option<"):
        return ty
    return "Option<" + ty + ">"


def string_expr(schema: Schema) -> str:
    return "Vec<u8>" if schema.format == "binary" else "String"


def array_expr(schema: Schema, tm: TypeMap) -> str:
    if schema.items is None:
        return "Vec<" + ANY + ">"
    return "Vec<" + type_expr(schema.items, tm) + ">"


def object_expr(schema: Schema, tm: TypeMap) -> str:
    if isinstance(schema.additional_properties, AdditionalPropertiesSchema):
        return (
            "std::collections::HashMap<String, "
            + type_expr(schema.additional_properties.schema, tm)
            + ">"
        )
    return ANY


def type_expr(node: RefOr[Schema], tm: TypeMap) -> str:
    if isinstance(node, Ref):
        resolved = tm.resolve_ref(node.ref.reference)
        return resolved if resolved is not None else ANY
    if isinstance(node, Item):
        schema = node.value
        base = base_expr(schema, tm)
        if schema.is_nullable() and base != ANY:
            return "Option<" + base + ">"
        return base
    raise ValueError(f"Invalid RefOr node: {node}")


def base_expr(schema: Schema, tm: TypeMap) -> str:
    if schema.all_of:
        return type_expr(schema.all_of[0], tm)
    if schema.one_of or schema.any_of:
        return ANY
    if schema.enum_values:
        return "String"

    tnames = schema.type_names()
    if len(tnames) > 1:
        return ANY

    if tnames:
        t0 = tnames[0]
        if t0 == "object":
            return object_expr(schema, tm)
        if t0 == "array":
            return array_expr(schema, tm)
        if t0 == "string":
            return string_expr(schema)
        if t0 == "integer":
            return "i64"
        if t0 == "number":
            return "f64"
        if t0 == "boolean":
            return "bool"
        if t0 == "null":
            return ANY
        return ANY

    if schema.properties or schema.additional_properties is not None:
        return object_expr(schema, tm)
    return ANY
