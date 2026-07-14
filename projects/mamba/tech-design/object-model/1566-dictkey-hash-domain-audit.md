# #1566 — DictKey hash-domain audit: raw `&str` probes silently miss

## Mechanism

Since #1028, `DictKey::Str`'s `Hash` impl uses the Python-semantic hash domain
(`dict_string_hash_value`), which is NOT Rust's native `str` hash. Any
`IndexMap<DictKey, _>::get(<&str-derived key>)` that hashes with the native
domain probes the wrong bucket and returns `None` for keys that are present.
No error, no panic — the value silently reads as absent. Confirmed four times,
in four unrelated modules: `exception.rs::dict_get_str` (#227, exc kwargs
dropped), `socket_mod.rs::kwarg_get` (#239, all kwargs → None),
`xml_mod.rs::dict_get_key` (#1627, `Element.keys()` → empty),
`logging_mod.rs::resolve_class_arg`-adjacent display-name probe (wave 1).

## Invariant

Python-semantic dicts (`ObjData::Dict` / any `IndexMap<DictKey, _>`) are only
probed through `dict_ops` helpers (`dict_get_exact_str` for `&str` probes).
A raw `.get()` with a locally-constructed key on such a map is a defect, even
if it appears to work in a unit test (native-hash collisions can accidentally
match short strings).

## Fix pattern (proven 4×)

Replace the raw probe with `dict_ops::dict_get_exact_str(map_or_obj, name)`.
Refcount semantics unchanged; callers keep their existing retain/release
behavior. Where the probe key is a runtime-key/display-name for a class, first
resolve via `type_object_registry_key` (see #1595/#1600/wave-1 commits), then
probe.

## Audit procedure (the actual work)

1. Candidates: `grep -n "\.get(name)\|\.get(key)\|entries.get(" projects/mamba/src/runtime/stdlib/*.rs projects/mamba/src/runtime/*.rs | grep -v dict_ops` (~125 sites at 2026-07-14).
2. Classify each site by the MAP TYPE at the probe: `IndexMap<DictKey, _>` /
   `ObjData::Dict` payloads → defect (fix); `HashMap<String, _>` or other
   native-keyed maps → safe (record and skip).
3. Fix defects with the pattern above. Batch by module; one commit per batch is
   fine (`Refs #1566`).
4. Prevention: after the sweep, make the wrong thing hard to write — add a
   doc-comment on `DictKey` naming this hazard and pointing at
   `dict_get_exact_str`, and (if cheap) a `#[deprecated]`-style shim or a
   clippy-visible wrapper for the raw pattern. Do NOT change `DictKey`'s Hash
   (that domain is load-bearing for Python dict semantics).

## Out of scope

- The runtime-key-aliasing family (display-name vs registry-key) — related but
  distinct; only apply `type_object_registry_key` where a site conflates them.
- Rewriting dict_ops itself.

## Verification contract

- Per fixed site: a kwargs/attr round-trip probe vs python3.12 (construct with
  the kwarg/key, read it back) — byte-identical.
- Known-fixed fixtures stay green: `_regression/core/exceptions/{attribute_name_obj,importerror_attrs}.py`,
  `behavior/std-libs/socket/create_connection_all_errors_exceptiongroup.py`,
  `behavior/std-libs/xml_etree/keys_lists_attrs_get_returns_value_or_default.py`,
  `behavior/std-libs/logging/{manager_uses_custom_class,setloggerclass_roundtrip}.py`.
- Full gate (`cargo test -p mamba --release --test conformance`) must not
  regress; report the before/after reading (silent-miss fixes may flip
  additional fixtures green — expected).
- Final report: classified-site table (defect/safe counts per module).
