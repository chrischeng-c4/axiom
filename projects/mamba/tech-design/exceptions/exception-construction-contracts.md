# #227/#223 — exception construction: kwargs, aux fields, __init__ synthesis

Status: landed (`dde0a6e98` fix-pack). Backfill TD.

## Mechanism

1. (#227) Keyword-only exception attrs (`AttributeError(name=,obj=)`,
   `ImportError(name=,path=)`, `NameError(name=)`) silently dropped:
   `exception.rs::dict_get_str` probed the kwargs dict with a raw native-hash
   lookup — the DictKey hash-domain defect (see
   `1566-dictkey-hash-domain-audit.md`).
2. (#227) `exc.__init__()` on an instance raised `'NoneType' not callable`:
   `mb_getattr`'s bound-native-method synthesis matched only
   `__setstate__|add_note` — `__init__` missing, so the existing
   `__init__`-on-BaseException handler was unreachable from getattr+call.
3. (#223) `str(UnicodeEncodeError)` rendered empty for strict-mode encode
   failures: `raise_uee` raised via bare `mb_raise(type, msg)`, leaving
   encoding/object/start/end/reason unset, and `unicode_error_str` always
   recomputes from those fields.

## Invariant

Exception instances must carry their structured fields at construction —
message-only raises can never render for Unicode*Error; kwargs must round-trip
(construct with kwarg → read attr back).

## Fix pattern

`dict_get_exact_str` for kwargs probes; add `"__init__"` to the bound-native
synthesis arm; route strict-mode encode raises through
`raise_unicode_encode_error_instance` with explicit CPython reason strings
(`ordinal not in range(128)` / `(256)`).

## Verification contract

`_regression/core/exceptions/{attribute_name_obj,importerror_attrs}.py`,
`_regression/builtin-libs/string_methods/errors.py` + UEE field probe —
byte-identical. Open sibling shapes: #1557 (unbound `Exception.__init__(self,…)`
chain loses attrs; `__new__` args pre-store; composite NoneType crash).
