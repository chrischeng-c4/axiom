# Type normalization

Status: approved domain direction for WI #2011.

This design separates two facts that the current compiler representation
conflates:

1. the normalized type used by checking and lowering; and
2. the source provenance that licensed that type.

`Any` is a normalized type. It is not evidence that the author requested a
dynamic boundary.

## Bounded context

The **Type Normalization** bounded context begins at parsed source type syntax
and ends at a `DeclaredType` that the checker and lowering layers may consume.
It does not decide whether two types are compatible and it does not implement
runtime dynamic-boundary enforcement.

Upstream:

- the parser supplies source annotation presence and syntax;
- inference supplies an inferred type only when a declaration is absent.

Downstream:

- the type checker consumes the normalized type plus provenance;
- HIR signature metadata preserves authored annotations for introspection;
- ABI selection consumes the normalized type, never reconstructing provenance
  from `Ty::Any`;
- dynamic-boundary work in #2007 consumes explicit-Any provenance after #2011.

## Ubiquitous language

| Term | Meaning |
|---|---|
| `SourceAnnotation` | The exact source fact: omitted or authored syntax with its span. |
| `NormalizedType` | A checker-local `TypeId` produced from authored syntax or valid inference. |
| `TypeProvenance` | Why the normalized type exists; never inferred from the normalized type itself. |
| `DeclaredType` | Aggregate containing source annotation, normalized type, and provenance. |
| `ExplicitAny` | The author wrote `Any` or an accepted qualified spelling that normalizes to `Any`. |
| `ImplicitUnknown` | No valid authored or inferred type exists. This is a compile error under #2011. |
| `DynamicBoundary` | A boundary explicitly licensed by `ExplicitAny`; tracked downstream by #2007. |

## Aggregate and value objects

`DeclaredType` is the aggregate root:

```text
DeclaredType
├── source: SourceAnnotation
│   ├── Omitted
│   └── Authored { syntax, span }
├── normalized: Option<TypeId>
└── provenance: TypeProvenance
    ├── Explicit
    ├── ExplicitAny
    ├── Inferred { inference_path }
    └── ImplicitUnknown { inference_path }
```

The concrete Rust shape may differ, but it must make these states
unrepresentable or reject them at construction:

- `Omitted + ExplicitAny`;
- `Authored(Any) + Inferred`;
- `ImplicitUnknown + Some(TypeId)`;
- a dynamic-boundary license derived only from `normalized == Ty::Any`.

`TypeId` remains checker-local as required by
[ARCHITECTURE.md](./ARCHITECTURE.md). Provenance is semantic data, not a
persisted `TypeId`.

## Normalization rules

| Source fact | Normalized result | Provenance | Terminal rule |
|---|---|---|---|
| authored concrete annotation | resolved `TypeId` | `Explicit` | checker consumes normally |
| authored `Any` or accepted qualified `typing.Any` | `Ty::Any` | `ExplicitAny` | dynamic behavior is licensed |
| omitted annotation with successful inference | inferred `TypeId` | `Inferred` | checker consumes normally |
| omitted annotation with failed inference | none | `ImplicitUnknown` | stable compile error |
| unresolved authored type | none or error type | `Explicit` | stable compile error; never becomes explicit Any |

Normalization may canonicalize spelling, aliases, union ordering, or equivalent
type syntax. It must not erase source presence or manufacture authorization.

## Domain invariants

1. **Presence is parsed, not reconstructed.** The parser records whether an
   annotation token existed. No downstream layer may compare rendered text to
   `"Any"` to decide whether the annotation was omitted.
2. **Normalization is many-to-one; provenance is not.** `Any` and
   `typing.Any` may share one normalized `TypeId`, while both retain
   `ExplicitAny`. Omitted syntax never joins that provenance class.
3. **Inference never licenses a dynamic boundary.** Even if an internal
   recovery type is `Ty::Any`, its provenance remains inferred or unknown.
4. **Unknown fails closed.** When a required type cannot be inferred or
   resolved, #2011 produces a compile diagnostic naming the binding, source
   span, and inference path.
5. **ABI and introspection are separate consumers.** Entry representation may
   be boxed for ABI reasons without becoming explicit Any. Introspection emits
   an annotation only when source syntax was authored.
6. **No process-global registry.** Provenance lives in parser/checker/HIR data
   owned by one compilation/execution context.

## Current representation defect

`parser::ast::Param` currently claims a mandatory `ty` and parser paths insert
`TypeExpr::Named("Any")` when a parameter has no annotation. Authored
`x: Any` produces the same node. `lower::ast_to_hir::annotation_repr_opt` then
turns every textual `"Any"` into `None`, and ABI selection uses the same bare
node as a proxy for “truly unannotated”.

That destroys the fact #2011 needs before checking begins. Fixing checker rules
without first preserving presence would turn policy into string heuristics and
would make #2007 provenance unsound.

## Ordered implementation slices

### N1 — preserve function annotation presence

Owner: AGY for all `projects/mamba/src/**` edits.

- Give parsed function parameters an explicit annotation-presence
  representation; do not use an `Any` filler to represent omission.
- Preserve authored `Any` through HIR signature metadata while omitted
  annotations remain absent.
- Make lowering/ABI decisions query presence explicitly, not rendered text.
- Cover regular, positional-only, keyword-only, `*args`, `**kwargs`, lambda,
  decorated, method, and generator parameter shapes.
- Preserve current entry ABI and runtime behavior in this slice.

N1 is structural provenance only. It must not globally reject unannotated
parameters and must not implement the later seven-family Force Typed wall.

### N2 — normalize authored type syntax

- Construct `DeclaredType` from authored syntax.
- Canonicalize accepted `Any` spellings to `ExplicitAny`.
- Keep unresolved authored types distinct from `ExplicitAny`.
- Add checker-local construction and invariant tests.

N2 is split at the checker ownership seam:

#### N2a — parameter-boundary provenance

Completed by #2918 (`65f4929cdf`).

- Introduce the `DeclaredType` aggregate and `TypeProvenance` value object.
- Construct authored parameter declarations from N1 annotation presence.
- Prove that `Any` and `typing.Any` are `ExplicitAny`, while `object` and
  unresolved authored names remain `Explicit`.
- Keep omitted parameters outside the authored constructor.

#### N2b — canonical checker annotation cache

Completed by #2920 (`ffac18bf37`).

- Replace the parallel bare `Span -> TypeId` semantic-annotation cache and
  parameter-only declaration map with one checker-local
  `Span -> DeclaredType` aggregate store.
- Record only authored top-level annotation entrypoints. Recursive or synthetic
  type-expression resolution may contribute the normalized value but must not
  manufacture authored provenance.
- Keep the lowering compatibility projection
  `resolved_type_expr(span) -> Option<TypeId>`, backed by the aggregate, so
  this slice does not change lowering behavior.
- Expose a read-only aggregate lookup for later N3/N4 consumers.
- Cover authored parameter and non-parameter annotation entrypoints for bare
  `Any`, `typing.Any`, a concrete type, `object`, and an unresolved name.
- Prove omitted parameters are absent, checker instances remain isolated, and
  there is exactly one mutable semantic-annotation store.

N2b must not infer omitted declarations, introduce `ImplicitUnknown`
diagnostics, change annotation compatibility, or propagate a dynamic-boundary
license. Those belong to N3 and N4.

### N3 — infer or reject omitted binding types

- Run inference only for `SourceAnnotation::Omitted`.
- Emit stable `ImplicitUnknown` diagnostics when inference fails.
- Split implementation tickets by the #2011 ingress families:
  local binding, global binding, class attribute, parameter, return,
  comprehension, and expression join.

N3 starts with one source-owned paired witness per ingress family. A family may
use more than one implementation ticket when its inference paths are
independent; no ticket may claim the whole family from a generic
`normalized == Any` check.

#### N3-L1 — empty-collection local binding

Completed by #2923 (`b5011e0abc`).

Authoritative pair:

- negative:
  `tests/cpython/_regression/core/typecheck/implicit_any_ingress/local_binding.py`;
- positive:
  `tests/cpython/_regression/core/typecheck/explicit_any_acceptance/local_binding.py`.

The executable Python body is identical after removing the positive fixture's
single `: Any` token. Harness-only `EXPECT-ERROR` metadata is excluded before
that normalized-source comparison.

Bounded behavior:

- apply only to the first simple-name assignment in an active function-local
  scope;
- classify an empty list initializer as
  `ImplicitUnknown { inference_path: "local_binding -> list_literal -> element" }`;
- store the failed omitted declaration at the target span with
  `normalized = None`;
- emit one stable compile error naming binding `items`, the target span, and
  the exact inference path;
- let `items: Any = []` continue through the N2 authored path as
  `ExplicitAny`;
- leave module/global/class bindings, parameters, returns, comprehensions,
  joins, unpacking, reassignments, and transitive dynamic-boundary propagation
  unchanged.

`DeclaredType` construction must reject:

- `ImplicitUnknown + Some(TypeId)`;
- `Inferred + None`;
- any inferred local whose `SourceAnnotation` is `Authored`.

This first local slice classifies the failure from initializer syntax plus the
inference path. It must not reject an arbitrary `Ty::Any` solely from normalized
identity; explicit-boundary flow remains an N4 concern.

#### N3-G1 — empty-collection global binding

Completed by #2924 (`4b58b149a6`).

Authoritative pair:

- negative:
  `tests/cpython/_regression/core/typecheck/implicit_any_ingress/global_binding.py`;
- positive:
  `tests/cpython/_regression/core/typecheck/explicit_any_acceptance/global_binding.py`.

The executable Python body is identical after removing the positive fixture's
single `: Any` token. Harness-only `EXPECT-ERROR` metadata is excluded before
that normalized-source comparison.

Bounded behavior:

- apply only to the first simple-name assignment whose binding scope is the
  module scope;
- classify an empty list initializer as
  `ImplicitUnknown { inference_path: "global_binding -> list_literal -> element" }`;
- store the failed omitted declaration at the target span with
  `normalized = None`;
- emit one stable compile error naming binding `items`, the target span, and
  the exact inference path;
- record a non-empty homogeneous module list as `Inferred` with path
  `global_binding -> list_literal` and its concrete normalized list type;
- let module-level `items: Any = []` continue through the N2 authored path as
  `ExplicitAny`;
- advance N3-L1's module-scope negative control into this slice's positive
  diagnostic case, while keeping class/function bindings, `global`-statement
  rebinding, unpacking, reassignments, and the other N3 ingress families
  unchanged.

Like N3-L1, this slice must classify from module binding scope, initializer
syntax, and the inference path. It must not reject an arbitrary `Ty::Any` or
reuse the local-binding path merely because the normalized recovery type is
`list[Any]`.

#### N3-C1 — empty-collection class attribute

Completed by #2932 (`df3242dddc`).

Authoritative pair:

- negative:
  `tests/cpython/_regression/core/typecheck/implicit_any_ingress/class_attribute.py`;
- positive:
  `tests/cpython/_regression/core/typecheck/explicit_any_acceptance/class_attribute.py`.

The executable Python body is identical after removing the positive fixture's
single `: Any` token. Harness-only `EXPECT-ERROR` metadata is excluded before
that normalized-source comparison.

Bounded behavior:

- apply only to the first simple-name assignment whose binding scope is the
  active class namespace;
- classify an empty list initializer as
  `ImplicitUnknown { inference_path: "class_attribute -> list_literal -> element" }`;
- store the failed omitted declaration at the target span with
  `normalized = None`;
- emit one stable compile error naming class attribute `items`, the target
  span, and the exact inference path;
- record a non-empty homogeneous class list as `Inferred` with path
  `class_attribute -> list_literal` and its concrete normalized list type;
- let class-level `items: Any = []` continue through the N2 authored path as
  `ExplicitAny`;
- preserve completed local/global binding behavior while leaving instance
  attribute assignment, function `global` rebinding, unpacking, reassignments,
  and the other N3 ingress families unchanged.

This slice must derive class-attribute ownership from the active class scope,
not merely from `current_class` or normalized `list[Any]`. Method bodies are
active function-local scopes even though a class scope remains on the outer
stack.

#### N3-P1 — empty-collection regular-parameter default

Completed by #2958 (`0d9cc9cf84`).

Authoritative pair:

- negative:
  `tests/cpython/_regression/core/typecheck/implicit_any_ingress/parameter.py`;
- positive:
  `tests/cpython/_regression/core/typecheck/explicit_any_acceptance/parameter.py`.

The executable Python body is identical after removing the positive fixture's
single `: Any` token. Harness-only `EXPECT-ERROR` metadata is excluded before
that normalized-source comparison.

Bounded behavior:

- apply only to a regular parameter whose source annotation is omitted and
  whose authored default expression is an empty list literal;
- classify that parameter as
  `ImplicitUnknown { inference_path:
  "parameter -> default -> list_literal -> element" }`;
- store the failed omitted declaration at the parameter source span with
  `normalized = None`;
- emit one stable compile error naming parameter `items`, its source span, and
  the exact inference path;
- retain the existing entry ABI recovery type only after the required error;
- infer a non-empty homogeneous list default as `Inferred` with path
  `parameter -> default -> list_literal` and its concrete normalized list
  element type;
- let `items: Any = []` continue through the N2 authored path as
  `ExplicitAny`;
- preserve completed local/global/class binding behavior while leaving
  parameters without defaults, positional-only and keyword-only parameter
  variants, `*args`, `**kwargs`, lambdas, returns, comprehensions, expression
  joins, and N4 propagation unchanged.

This first parameter slice must derive the decision from source annotation
presence, regular-parameter kind, and default syntax. It must not reject every
omitted parameter, inspect normalized `Ty::Any` as authorization evidence, or
infer a caller-specific type from one call site.

#### N3-R1 — direct empty-list function return

Authoritative pair:

- negative:
  `tests/cpython/_regression/core/typecheck/implicit_any_ingress/return.py`;
- positive:
  `tests/cpython/_regression/core/typecheck/explicit_any_acceptance/return.py`.

The executable Python body is identical after removing the positive fixture's
single ` -> Any` token. Harness-only `EXPECT-ERROR` metadata is excluded before
that normalized-source comparison.

Bounded behavior:

- apply only to a module-level synchronous function with no decorators, no
  authored return annotation, and a body consisting of one direct
  `return <value>` statement;
- classify a direct empty list return as
  `ImplicitUnknown { inference_path:
  "return -> list_literal -> element" }`;
- store the failed omitted return declaration at the returned expression span
  with `normalized = None`;
- emit one stable compile error at that span naming function `collect` and the
  exact inference path;
- retain the existing checker and entry-ABI recovery type only after the
  required error;
- infer a direct non-empty homogeneous list return as `Inferred` with path
  `return -> list_literal` and its concrete normalized list type;
- publish that successful inferred type to the checker-local canonical
  callable signature so recursive and later calls do not silently retain an
  `Any` return;
- let `def collect() -> Any: return []` continue through the N2 authored path
  as `ExplicitAny` without adding a second return-expression declaration;
- preserve completed binding and parameter behavior while leaving async,
  method, decorated, generator, bare/scalar/multiple/nested/conditional
  returns, comprehensions, expression joins, lowering, runtime ABI selection,
  and N4 propagation unchanged.

R1 must decide eligibility before recording a declaration or emitting a
diagnostic. It must derive failure from omitted return syntax plus the direct
empty-list expression, never from the preregistered callable's recovery
`Ty::Any`. Its successful-signature update must preserve the callable's
parameter list, variadic flag, callable signature, and parameter specification.

Completed by #2963 (`91e87348b5`).

#### N3-CM1 — single-generator list-comprehension binding

Authoritative pair:

- negative:
  `tests/cpython/_regression/core/typecheck/implicit_any_ingress/comprehension.py`;
- positive:
  `tests/cpython/_regression/core/typecheck/explicit_any_acceptance/comprehension.py`.

The executable Python body is identical after removing the positive fixture's
single `: Any` token. Harness-only `EXPECT-ERROR` metadata is excluded before
that normalized-source comparison.

Bounded behavior:

- apply only to the first simple-name assignment in an active function-local
  scope whose initializer is a list comprehension with exactly one generator,
  one simple target, no target unpacking, and no filter conditions;
- classify an empty list generator iterable as
  `ImplicitUnknown { inference_path:
  "comprehension -> generator -> iterable -> list_literal -> element" }`;
- store the failed omitted declaration at the assignment target span with
  `normalized = None`;
- emit one stable compile error at that span naming binding `items` and the
  exact inference path;
- infer `[item for item in [1, 2]]` as `list[int]`, record `Inferred` with path
  `comprehension -> element`, and publish the concrete list type to the local
  binding;
- let `items: Any = [item for item in []]` continue through the N2 authored
  path as `ExplicitAny` without adding a second declaration at the target;
- preserve completed literal-binding, parameter, and return behavior while
  leaving module/class comprehension bindings, set/dict comprehensions,
  generator expressions, multiple generators, unpacking, filters, walrus
  targets, async comprehensions, expression joins, lowering, runtime ABI
  selection, and N4 propagation unchanged.

CM1 must decide eligibility from omitted target syntax plus the frozen
list-comprehension shape. It must never reject a comprehension merely because
its current recovery type is `Ty::Any`. The successful result must be derived
from the checked element type and must not leak the comprehension target into
the enclosing scope.

Completed by #2964 (`841571e948`).

### N4 — propagate explicit Any to dynamic walls

- Expose provenance to the boundary model owned by #2007.
- Prove that explicit Any stays usable and implicit unknown cannot cross the
  same boundary.

## N1 acceptance contract

N1 is green only when all of the following hold:

- parser-level tests distinguish `def f(x): ...` from
  `def f(x: Any): ...` without consulting rendered type text;
- the distinction covers regular, positional-only, keyword-only, `*args`,
  `**kwargs`, lambda, method, decorated, and generator parameters;
- HIR signature metadata reports `None` for omitted annotations and `"Any"`
  for authored Any;
- no N1 production branch decides annotation presence with
  `type_expr_repr(...) == "Any"`, `Named("Any")`, or normalized
  `TypeId == tcx.any()`;
- focused parser/lowering/driver tests pass;
- the existing unannotated-function runtime probes selected by AGY remain
  byte-identical, proving this structural slice did not change entry ABI;
- no global lock or process-global provenance map is introduced.

The controller independently reviews the complete diff and re-runs the focused
gates before accepting the ticket. AGY reports evidence but never closes the
issue.
