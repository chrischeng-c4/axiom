# object-model — external contract (as-is, 2026-07-15)

Domain map: `tech-design/object-model/ARCHITECTURE.md` (EC surface section). Verdict law: `HARNESS.md`.
Oracle = live python3.12 byte-diff; xfail = acknowledged gap, still contract (skip, never executed).

## Positive contract — fixtures that must RUN and byte-match the python3.12 oracle

Core-owned dirs (live-counted 2026-07-15, `tests/cpython/` relative):

| Dir | .py | xfail | Covers |
|---|---|---|---|
| `_regression/core/class_system/` | 50 | 8 | registry, construction, `__new__`/`__init__`, class attrs |
| `_regression/core/mro_super/` | 5 | 0 | C3 MRO, diamond super, `__classcell__`, MRO error paths |
| `_regression/core/language/` | 145 | 2 | class stmt lifecycle; incl. `metaclasses/` (4), `descriptors/` (3), `slots/` (3), `classes/` (3) |
| `behavior/core/descr/` | 154 | 141 | CPython test_descr port — descriptor protocol deep-end (13 live) |

Adjacent dispatch/datamodel dirs also owned here:

| Dir | .py | xfail | Dir | .py | xfail |
|---|---|---|---|---|---|
| `_regression/core/descriptor_protocol/` | 4 | 0 | `_regression/core/dunder/` | 21 | 1 |
| `_regression/core/property_descriptor/` | 5 | 1 | `_regression/core/dunders/` | 4 | 0 |
| `_regression/core/getattr_getattribute/` | 4 | 0 | `_regression/core/equality_hash/` | 4 | 1 |
| `_regression/core/class_body/` | 6 | 1 | `_regression/core/datamodel_container/` | 4 | 0 |
| `_regression/core/operator_dispatch/` | 4 | 1 | `behavior/std-libs/subclassinit/` (PEP 487) | 17 | 12 |

Totals: 427 fixtures, 168 xfail (259 live). Runtime rejects (ABC/Protocol instantiation, duplicate-base /
inconsistent-MRO TypeError) are proven POSITIVELY — `errors.py` fixtures byte-matching oracle error output, not walls.
Adjacent proof: dict-key hash-domain regressions land in `behavior/std-libs/{logging,xml_etree,socket}` per
`tech-design/object-model/identity-and-keys.md` §Domain 2.

## Negative contract — what must be REJECTED

None. Object-model owns no `type/` walls; all compile-reject surfaces (incl. operator/attribute walls that touch
dunder dispatch) belong to the type-system domain. A compile reject in any dir above is a type-system false
positive by definition (README.md dimension rule).

## Known contract gaps

- `behavior/core/descr/` is 92% skipped (141/154 xfail: 91 "auto-ported" + 50 "auto-extracted" campaign markers).
  Measured campaign stale rates 18.3%/3.2% ⇒ hidden PASSes expected in this bucket; tracked: #1768.
- The 8 `class_system/` xfails are each a genuine named runtime divergence (`instance.__dict__` → None,
  `@classmethod` `cls()` → None, `cls.__bases__` → None, `__new__`-set attrs invisible, `cls.__subclasses__()`
  AttributeError, instance-only attr readable via class name, …); xfail = full skip so they rot silently
  (HARNESS.md hazard); tracked: #1768 audit family.
- Metaclass `__call__(cls, *args, **kwargs)` arg forwarding LIVE-DIVERGES (mamba passes mangled scalars, then
  TypeError from `super().__call__(*a)`); only the zero-arg `__call__` shape has a fixture
  (`_regression/core/language/metaclass.py`); tracked: #1770.
- EC-map path drift: `_regression/core/descriptors/` (named in README.md map + ARCHITECTURE EC surface) does not
  exist — real dirs are `descriptor_protocol/`, `property_descriptor/`, `language/descriptors/`; tracked: #1771.
- In-process twins `test_regression_{class_system,language}_parse` (`src/driver/tests/behavioral_lang.rs:707+`)
  walk retired pre-dimension-first paths and pass vacuously on 0 files; tracked: #1767.
- `subclassinit/test__test_init_subclass_wrong.py` verified passing but permanently skipped by a stale xfail; only
  `*_wrong.py` outside `type/` (wall-suffix name collision); tracked: #1768/#1772.
- `operator_dispatch/` xfail (unary `-` on user classes compile-rejected) is an ingress-overwalling defect wearing
  an xfail — type-system lane, `tech-design/type-system/walls-and-widening.md` §Fire/defer semantics; tracked: #1769.

## Verification

- Inner loop (seconds, runner-parity verdicts; set `MAMBA_BIN` after a release build):
  `python3 tests/harness/cpython/tools/sweep.py tests/cpython/_regression/core/class_system tests/cpython/_regression/core/mro_super tests/cpython/_regression/core/language tests/cpython/behavior/core/descr tests/cpython/behavior/std-libs/subclassinit`
  (append the adjacent `_regression/core/*` dirs from the second table for the full domain).
- Cargo gate slice (datatest filter is a path substring): `cargo test -p mamba --release --test conformance -- _regression/core/class_system` (one dir per filter run).
- Manifests: none exist for these dirs — `tests/harness/cpython/config/manifests/core/` carries only
  `args_kwargs_binding`/`cpython321_core_lang`/`generators`; object-model fixtures are hand-authored/pre-manifest.
- Full gate (the only progress signal): `cargo test -p mamba --release --test conformance` (~3 min); per-fix evidence = before/after readings.
