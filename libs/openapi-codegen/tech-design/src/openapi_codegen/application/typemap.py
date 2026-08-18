from __future__ import annotations

from dataclasses import dataclass

from openapi_codegen.application.document import Spec
from openapi_codegen.domain.names import NameRegistry, to_pascal


@dataclass(frozen=True)
class TypeMap:
    names: tuple[tuple[str, str], ...] = ()

    def get(self, key: str) -> str | None:
        for k, v in self.names:
            if k == key:
                return v
        return None

    def resolve_ref(self, reference: str) -> str | None:
        prefix = "#/components/schemas/"
        if not reference.startswith(prefix):
            return None
        key = reference[len(prefix) :]
        mapped = self.get(key)
        if mapped is not None:
            return mapped
        return to_pascal(key)


def build_type_map(spec: Spec) -> TypeMap:
    reg = NameRegistry()
    pairs: list[tuple[str, str]] = []
    for key, _ in spec.components.schemas:
        pascal_name = to_pascal(key)
        unique_name = reg.unique(pascal_name)
        pairs.append((key, unique_name))
    pairs.sort(key=lambda p: p[0])
    return TypeMap(tuple(pairs))
