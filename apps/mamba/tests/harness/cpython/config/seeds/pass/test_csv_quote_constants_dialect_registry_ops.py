# Operational AssertionPass seed for the `csv` quoting-policy enum and
# the dialect-registry surface (plus the `selectors` event-mask enum
# that pairs with it for every poll/select wrapper). `csv` is the
# stdlib's tabular-text codec; `selectors` is the I/O readiness layer.
# No fixture coverage yet for either.
#
# The matching subset between mamba and CPython is the *constant-and-
# registry* layer: every documented `QUOTE_*` integer constant has the
# same numeric value, `field_size_limit()` returns the same default
# (131072), `list_dialects()` returns a length-3 list containing the
# three built-in dialect names (`excel`, `excel-tab`, `unix`), every
# top-level callable (`reader` / `writer` / `DictReader` / `DictWriter`
# / `register_dialect` / `unregister_dialect`) is exposed, and the
# `selectors` event bits (`EVENT_READ == 1`, `EVENT_WRITE == 2`) are
# bit-disjoint integers that OR to 3.
#
# Surface in this fixture:
#   • csv.QUOTE_MINIMAL    == 0   — quote only when needed;
#   • csv.QUOTE_ALL        == 1   — quote every field;
#   • csv.QUOTE_NONNUMERIC == 2   — quote every non-numeric field;
#   • csv.QUOTE_NONE       == 3   — never quote;
#   • each constant is `int`;
#   • csv.field_size_limit() — returns int 131072 (default cap);
#   • csv.list_dialects() — returns a list of length 3 containing
#     the names "excel", "excel-tab", "unix";
#   • hasattr(csv, X) — for reader / writer / DictReader / DictWriter /
#     register_dialect / unregister_dialect / Dialect / excel /
#     excel_tab / unix_dialect / QUOTE_ALL / QUOTE_MINIMAL / QUOTE_NONE
#     / QUOTE_NONNUMERIC / field_size_limit / get_dialect /
#     list_dialects;
#   • callable(csv.field_size_limit / list_dialects / get_dialect);
#   • selectors.EVENT_READ  == 1;
#   • selectors.EVENT_WRITE == 2;
#   • EVENT_READ & EVENT_WRITE == 0 (bit-disjoint);
#   • EVENT_READ | EVENT_WRITE == 3 (combined poll mask);
#   • hasattr(selectors, X) — for DefaultSelector / SelectSelector /
#     BaseSelector / SelectorKey / EVENT_READ / EVENT_WRITE.
#
# Behavioral edges that DIVERGE on mamba (csv.Sniffer / Dialect /
# get_dialect class identity, csv.Sniffer instance methods, csv.
# field_size_limit being a builtin_function_or_method rather than a
# python lambda, email.message.Message / EmailMessage class identity
# and Message() instance construction, selectors.DefaultSelector /
# SelectSelector / BaseSelector / SelectorKey class identity) are
# covered in `lang_csv_sniffer_email_message_silent.py`.
import csv
import selectors

_ledger: list[int] = []

# 1) csv QUOTE_* integer constants
assert csv.QUOTE_MINIMAL == 0; _ledger.append(1)
assert csv.QUOTE_ALL == 1; _ledger.append(1)
assert csv.QUOTE_NONNUMERIC == 2; _ledger.append(1)
assert csv.QUOTE_NONE == 3; _ledger.append(1)
assert isinstance(csv.QUOTE_MINIMAL, int); _ledger.append(1)
assert isinstance(csv.QUOTE_ALL, int); _ledger.append(1)
assert isinstance(csv.QUOTE_NONNUMERIC, int); _ledger.append(1)
assert isinstance(csv.QUOTE_NONE, int); _ledger.append(1)

# 2) Quote constants are pairwise distinct
_quotes = {csv.QUOTE_MINIMAL, csv.QUOTE_ALL, csv.QUOTE_NONNUMERIC, csv.QUOTE_NONE}
assert len(_quotes) == 4; _ledger.append(1)

# 3) csv.field_size_limit() — default cap
_lim = csv.field_size_limit()
assert isinstance(_lim, int); _ledger.append(1)
assert _lim == 131072; _ledger.append(1)

# 4) csv.list_dialects() — three documented built-in dialects
_dialects = csv.list_dialects()
assert isinstance(_dialects, list); _ledger.append(1)
assert len(_dialects) == 3; _ledger.append(1)
assert "excel" in _dialects; _ledger.append(1)
assert "excel-tab" in _dialects; _ledger.append(1)
assert "unix" in _dialects; _ledger.append(1)

# 5) csv module-level callables
assert callable(csv.field_size_limit); _ledger.append(1)
assert callable(csv.list_dialects); _ledger.append(1)
assert callable(csv.get_dialect); _ledger.append(1)
assert callable(csv.register_dialect); _ledger.append(1)
assert callable(csv.unregister_dialect); _ledger.append(1)

# 6) csv module attribute surface — every documented public entry
assert hasattr(csv, "reader"); _ledger.append(1)
assert hasattr(csv, "writer"); _ledger.append(1)
assert hasattr(csv, "DictReader"); _ledger.append(1)
assert hasattr(csv, "DictWriter"); _ledger.append(1)
assert hasattr(csv, "register_dialect"); _ledger.append(1)
assert hasattr(csv, "unregister_dialect"); _ledger.append(1)
assert hasattr(csv, "Dialect"); _ledger.append(1)
assert hasattr(csv, "excel"); _ledger.append(1)
assert hasattr(csv, "excel_tab"); _ledger.append(1)
assert hasattr(csv, "unix_dialect"); _ledger.append(1)
assert hasattr(csv, "QUOTE_ALL"); _ledger.append(1)
assert hasattr(csv, "QUOTE_MINIMAL"); _ledger.append(1)
assert hasattr(csv, "QUOTE_NONE"); _ledger.append(1)
assert hasattr(csv, "QUOTE_NONNUMERIC"); _ledger.append(1)
assert hasattr(csv, "field_size_limit"); _ledger.append(1)
assert hasattr(csv, "get_dialect"); _ledger.append(1)
assert hasattr(csv, "list_dialects"); _ledger.append(1)

# 7) selectors event-mask integer constants
assert selectors.EVENT_READ == 1; _ledger.append(1)
assert selectors.EVENT_WRITE == 2; _ledger.append(1)
assert isinstance(selectors.EVENT_READ, int); _ledger.append(1)
assert isinstance(selectors.EVENT_WRITE, int); _ledger.append(1)

# 8) Event bits are disjoint and OR to the combined poll mask
assert (selectors.EVENT_READ & selectors.EVENT_WRITE) == 0; _ledger.append(1)
assert (selectors.EVENT_READ | selectors.EVENT_WRITE) == 3; _ledger.append(1)

# 9) selectors module attribute surface
assert hasattr(selectors, "DefaultSelector"); _ledger.append(1)
assert hasattr(selectors, "SelectSelector"); _ledger.append(1)
assert hasattr(selectors, "BaseSelector"); _ledger.append(1)
assert hasattr(selectors, "SelectorKey"); _ledger.append(1)
assert hasattr(selectors, "EVENT_READ"); _ledger.append(1)
assert hasattr(selectors, "EVENT_WRITE"); _ledger.append(1)

# NB: csv.Sniffer / Dialect / get_dialect class identity, csv.Sniffer()
# instance methods, csv.field_size_limit being a
# builtin_function_or_method (not a python function), email.message.
# Message / EmailMessage class identity and instance construction, and
# selectors.DefaultSelector / SelectSelector / BaseSelector /
# SelectorKey class identity all DIVERGE on mamba — moved to the
# divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_csv_quote_constants_dialect_registry_ops {sum(_ledger)} asserts")
