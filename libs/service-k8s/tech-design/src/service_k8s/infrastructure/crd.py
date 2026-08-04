"""CRD normalization and CEL validation rules for service-k8s.

CustomResourceDefinitions describe an operator's public schema. This module
enforces Kubernetes admission invariants, normalizes unsigned integer formats
dropped by Kubernetes structural OpenAPI, attaches CEL rules for admission-time
validation, and quotes legacy YAML 1.1 boolean-like default strings.
"""

from __future__ import annotations

from typing import Final

BOOLEAN_LIKE: Final[tuple[str, ...]] = ("y", "yes", "n", "no", "on", "off")


class CelRuleError(ValueError):
    """A CEL rule that the API server would reject at install time."""


def normalize_unsigned_integer_formats(value: object) -> None:
    if isinstance(value, dict):
        if value.get("format") in ("uint32", "uint64"):
            del value["format"]
            if "minimum" not in value:
                value["minimum"] = 0
        for child in value.values():
            normalize_unsigned_integer_formats(child)
    elif isinstance(value, list):
        for child in value:
            normalize_unsigned_integer_formats(child)


def add_spec_validation_rule(
    crd: dict[str, object], rule: str, message: str
) -> int:
    clean_rule = rule.replace(" ", "")
    if "!=null" in clean_rule or "==null" in clean_rule:
        raise CelRuleError(
            "CEL rules must not compare against null directly; use has(self.field)"
        )

    spec = crd.get("spec")
    if not isinstance(spec, dict):
        return 0

    versions = spec.get("versions")
    if not isinstance(versions, list):
        return 0

    attached = 0
    for version in versions:
        if not isinstance(version, dict):
            continue
        schema = version.get("schema")
        if not isinstance(schema, dict):
            continue
        open_api = schema.get("openAPIV3Schema")
        if not isinstance(open_api, dict):
            continue
        props = open_api.get("properties")
        if not isinstance(props, dict):
            continue
        spec_schema = props.get("spec")
        if not isinstance(spec_schema, dict):
            continue

        rules = spec_schema.setdefault("x-kubernetes-validations", [])
        if not isinstance(rules, list):
            continue

        rules.append({"rule": rule, "message": message})
        attached += 1

    return attached


def quote_yaml_1_1_boolean_like_strings(yaml_text: str) -> str:
    trailing_newline = yaml_text.endswith("\n")
    out: list[str] = []
    for line in yaml_text.splitlines():
        trimmed = line.lstrip()
        scalar: str | None = None
        if ": " in trimmed:
            scalar = trimmed.split(": ", 1)[1]
        elif trimmed.startswith("- "):
            scalar = trimmed[2:]

        if scalar is not None and scalar.lower() in BOOLEAN_LIKE:
            prefix_len = len(line) - len(scalar)
            out.append(line[:prefix_len] + '"' + scalar + '"')
        else:
            out.append(line)

    result = "\n".join(out)
    if trailing_newline:
        result += "\n"
    return result
