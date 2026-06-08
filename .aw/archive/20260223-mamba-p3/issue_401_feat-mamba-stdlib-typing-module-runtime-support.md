---
number: 401
title: "feat(mamba): stdlib typing module runtime support"
state: open
labels: [enhancement, crate:mamba, P3]
---

# #401 — feat(mamba): stdlib typing module runtime support

## Summary
Make `typing` module importable with runtime representations of type constructs.

## Required
- `typing.Optional[T]`, `typing.Union[A, B]`
- `typing.List[T]`, `typing.Dict[K, V]`, `typing.Tuple[T, ...]`, `typing.Set[T]`
- `typing.Any`, `typing.NoReturn`
- `typing.Callable[[Args], Return]`
- `typing.TypeVar('T', bound=...)`
- `typing.Protocol` — for structural subtyping
- `typing.ClassVar`, `typing.Final`
- `typing.get_type_hints(obj)`
- `typing.TYPE_CHECKING` constant (False at runtime)

## Implementation Notes
- Most typing constructs are no-ops at runtime; they just need to exist
- The type checker already handles these internally
- This issue is about making `from typing import X` work
