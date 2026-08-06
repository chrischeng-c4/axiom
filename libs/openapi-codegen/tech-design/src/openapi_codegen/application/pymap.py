from __future__ import annotations

from openapi_codegen.application.document import (
    AdditionalPropertiesSchema,
    Item,
    Ref,
    RefOr,
    Schema,
)
from openapi_codegen.application.typemap import TypeMap
from openapi_codegen.domain.target import PythonTarget


def optional(ty: str, target: PythonTarget | None) -> str:
    if (
        ty == "Any"
        or ty == "None"
        or ty.endswith(" | None")
        or (target is None and ty.startswith("Optional["))
    ):
        return ty
    if target is not None:
        return ty + " | None"
    return "Optional[" + ty + "]"


def union_expr(members: list[str], target: PythonTarget | None) -> str:
    if not members:
        return "Any"
    if len(members) == 1:
        return members[0]
    if target is not None:
        return " | ".join(members)
    return "Union[" + ", ".join(members) + "]"


def union(
    items: tuple[RefOr[Schema], ...], tm: TypeMap, target: PythonTarget | None
) -> str:
    return union_expr([type_expr(i, tm, target) for i in items], target)


def scalar(ty: str, schema: Schema) -> str:
    if ty == "string":
        return string_expr(schema)
    if ty == "integer":
        return "int"
    if ty == "number":
        return "float"
    if ty == "boolean":
        return "bool"
    if ty == "null":
        return "None"
    return "Any"


def string_expr(schema: Schema) -> str:
    return "bytes" if schema.format == "binary" else "str"


def array_expr(
    schema: Schema, tm: TypeMap, target: PythonTarget | None
) -> str:
    if schema.items is None:
        return "list[Any]"
    return "list[" + type_expr(schema.items, tm, target) + "]"


def object_expr(
    schema: Schema, tm: TypeMap, target: PythonTarget | None
) -> str:
    if isinstance(schema.additional_properties, AdditionalPropertiesSchema):
        return (
            "dict[str, "
            + type_expr(schema.additional_properties.schema, tm, target)
            + "]"
        )
    return "dict[str, Any]"


def enum_literal(schema: Schema) -> str:
    members: list[str] = []
    for v in schema.enum_values:
        if isinstance(v, bool):
            members.append("True" if v else "False")
        elif isinstance(v, str):
            members.append('"' + v.replace('"', '\\"') + '"')
        elif isinstance(v, (int, float)):
            members.append(str(v))
        elif v is None:
            members.append("None")
        else:
            members.append("Any")
    if not members:
        return "Any"
    return "Literal[" + ", ".join(members) + "]"


def type_expr(
    node: RefOr[Schema], tm: TypeMap, target: PythonTarget | None
) -> str:
    if isinstance(node, Ref):
        resolved = tm.resolve_ref(node.ref.reference)
        return resolved if resolved is not None else "Any"
    if isinstance(node, Item):
        schema = node.value
        base = base_expr(schema, tm, target)
        if schema.is_nullable():
            return optional(base, target)
        return base
    raise ValueError(f"Invalid RefOr node: {node}")


def base_expr(
    schema: Schema, tm: TypeMap, target: PythonTarget | None
) -> str:
    if schema.all_of:
        return type_expr(schema.all_of[0], tm, target)
    if schema.one_of:
        return union(schema.one_of, tm, target)
    if schema.any_of:
        return union(schema.any_of, tm, target)
    if schema.enum_values:
        return enum_literal(schema)

    tnames = schema.type_names()
    if len(tnames) > 1:
        return union_expr([scalar(t, schema) for t in tnames], target)

    if tnames:
        t0 = tnames[0]
        if t0 == "object":
            return object_expr(schema, tm, target)
        if t0 == "array":
            return array_expr(schema, tm, target)
        if t0 == "string":
            return string_expr(schema)
        if t0 == "integer":
            return "int"
        if t0 == "number":
            return "float"
        if t0 == "boolean":
            return "bool"
        if t0 == "null":
            return "None"
        return "Any"

    if schema.properties or schema.additional_properties is not None:
        return object_expr(schema, tm, target)
    return "Any"
