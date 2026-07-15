# Stdlib module hazards — the traps every `*_mod.rs` shares

The `src/runtime/stdlib/*_mod.rs` surface repeats a handful of cross-cutting
traps. This doc is the checklist for touching any stdlib module; per-domain
mechanisms live in their own domain docs and are referenced, not restated.

## The shared traps (checklist)

| Trap | Symptom | Rule |
|---|---|---|
| DictKey hash-domain probe | module reads a Python dict with raw `&str` → silent None | `dict_get_exact_str` (object-model/identity-and-keys.md) |
| Runtime-key aliasing | `__name__` fed into CLASS_REGISTRY → miss | `type_object_registry_key` (object-model/identity-and-keys.md) |
| Active-module in a dispatcher | native dispatcher runs with ITS `__module__` active; user-scope name lookups resolve in the wrong module | read closure/instance state directly, never active-module map (closures/ARCHITECTURE.md) |
| Shared dispatch-shell address | many stub fns share one `dispatch_class_shell` addr → isinstance can't tell them apart | register a real class-name string + `mb_class_register` (isinstance-of-special-class recipe) |
| TESTFN CWD droppings | `@mamba_test_<pid>` litters CWD from direct runs | scratch-CWD (harness/testfn-scratch-cwd.md) |

## Vendored vs native vs sentinel

- Vendored: real CPython source resolved via `VENDORED_MODULES`/`vendor_lib.rs`
  (de-register the native shell so the source wins).
- Native kit: `mambalibs` linked in (C3).
- **Sentinel shim** (pydantic_core_mod etc.): a 58-77 line fake surface that
  answers attribute-identity reads only — NO real implementation. These exist
  to unblock import-shape tests but MUST NOT read as readiness: their perf
  pins measure a lie. Marker + readiness exclusion tracked: #1514/#1119. When
  auditing a module's completeness, check whether it is a sentinel first.

## Per-module open mechanisms (thin, isolated)

- xml_etree child-list/SubElement: Element must behave list-like over children
  (len/index/iter/mutate the same backing sequence); distinct from the attrib
  dict-probe fix (tracked: #1629).
- tempfile mkstemp fd validity (Errno 9) + os path-APIs must accept
  `os.PathLike`, not just `str` (the latter is a type-system widening, see
  type-system/walls-and-widening.md) (tracked: #1630).
- codecs error-handler registry (`register_error`/`lookup_error`) + built-in
  xmlcharrefreplace/backslashreplace/namereplace handlers are literal stubs
  (tracked: #238's epic).

## EC surface

`behavior|errors|real_world/std-libs/<mod>` per module; readiness gates
(#711/#714) consume — but must exclude sentinel — pin signals.
