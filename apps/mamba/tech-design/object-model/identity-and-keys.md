# Identity and keys — the three name/key domains that must never mix

mamba has three distinct identity domains around classes and dicts. Every
confirmed silent-miss bug in this area came from crossing two of them.

## Domain 1: registry keys vs display names

- A user class's CLASS_REGISTRY key is namespaced:
  `__mamba_user_class__:<file>:<symId>:<Name>[@serial]`. Its display
  `__name__` is just `Name`.
- An `Instance`'s raw `class_name` field holds the REGISTRY key. A `type`
  object's `__name__` holds the DISPLAY name.
- Accessors: `type_object_registry_key` (type object → registry key);
  raw `class_name` field (instance → registry key); `class_display_name`
  (registry key → human name, for ERROR MESSAGES ONLY).
- Anti-pattern (5 confirmed sites fixed: total_ordering, singledispatch
  register/lookup, sdm type-receiver, logging resolve_class_arg, setLoggerClass):
  feeding `__name__` into a registry lookup. Symptoms: dispatch falls to base
  impl, `cls` becomes the literal string "type", roundtrips fail.
- Classmethod-stacking corollary: a receiver may be a `type` OBJECT — branch
  on that before treating `class_name` as an instance's class.

## Domain 2: Python-semantic dict hashing vs Rust native hashing

- `DictKey::Str` hashes in the Python domain (`dict_string_hash_value`) —
  NOT Rust's `str` hash. Any `IndexMap<DictKey,_>::get(<&str>)` probes the
  wrong bucket and silently returns None for present keys.
- Invariant: Python-semantic dicts are probed only through `dict_ops`
  helpers (`dict_get_exact_str` for &str probes). A raw `.get()` is a defect
  even if a unit test passes (short-string hash collisions can mask it).
- 5 confirmed sites fixed (exception kwargs, socket kwarg_get, xml
  dict_get_key, logging probe, test.*-module `__name__` carve-out — the last
  was DEAD CODE for months because of this). ~120 unaudited candidate sites
  remain; audit procedure and prevention design tracked: #1566. Do NOT change
  DictKey's Hash — the Python domain is load-bearing.

## Domain 3: value identity vs boxing

- NaN-boxed values can carry the same logical value under different box bits
  (known: match-subject bools vs literal bools — `mb_is_identity` fails where
  `mb_eq` tolerates). Identity-sensitive features (PEP 634 True/False/None
  patterns) are blocked on canonical subject boxing (see
  memory/ARCHITECTURE.md boxing notes).

## Working rules

1. Never construct a registry lookup key from `__name__`.
2. Never probe an ObjData::Dict payload without dict_ops.
3. Error messages: always `class_display_name`; never leak
   `__mamba_user_class__:` strings to users (regression class: #226's
   message fix).
4. New kwargs-dict consumers: round-trip probe (construct with kwarg → read
   back) is the mandatory verification shape.
