# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_csv_dialect_registry_field_size_limit_ops"
# subject = "cpython321.test_csv_dialect_registry_field_size_limit_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_csv_dialect_registry_field_size_limit_ops.py"
# status = "filled"
# ///
"""cpython321.test_csv_dialect_registry_field_size_limit_ops: execute CPython 3.12 seed test_csv_dialect_registry_field_size_limit_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `csv` module dialect-registry
# + field-size-limit surface — the runtime configuration layer that
# every CSV-consuming library (pandas, Django's serializers, the
# stdlib `csv.DictReader/DictWriter`) reaches through to (a) register
# a custom-dialect record under a string key, (b) introspect the
# built-in dialects (`excel`, `excel-tab`, `unix`) and any user-
# registered dialects via `list_dialects` / `get_dialect`, and (c)
# raise/relax the per-field byte cap that protects against pathological
# inputs. Existing `test_csv.py` explicitly excludes
# `field_size_limit / register_dialect / unregister_dialect / Sniffer`
# from its coverage list; this seed fills that gap on the matching
# subset between mamba and CPython.
#
# Surface:
#   • csv.QUOTE_* constants — uniqueness + canonical 3.12 values
#     (QUOTE_MINIMAL=0 / QUOTE_ALL=1 / QUOTE_NONNUMERIC=2 / QUOTE_NONE=3);
#   • csv.list_dialects() → list[str]
#       — built-ins `'excel'`, `'excel-tab'`, `'unix'` always present;
#       — user-registered dialect appears after `register_dialect`,
#         disappears after `unregister_dialect`;
#   • csv.get_dialect(name) → Dialect
#       — built-in `'excel'` has `delimiter=','`, `quotechar='"'`,
#         `doublequote=True`, `skipinitialspace=False`,
#         `lineterminator='\r\n'`, `quoting=QUOTE_MINIMAL`;
#       — built-in `'excel-tab'` has `delimiter='\t'`;
#       — built-in `'unix'` has `quoting=QUOTE_ALL` and
#         `lineterminator='\n'`;
#   • csv.register_dialect(name, **kwargs) → None (side-effect:
#     registers); `csv.get_dialect(name)` returns a Dialect with the
#     overridden fields;
#   • csv.unregister_dialect(name) → None (side-effect: removes);
#   • csv.field_size_limit() → int (current per-field byte cap);
#     csv.field_size_limit(new) → int (returns old value, sets new);
#   • module-level attribute discipline — `reader`, `writer`,
#     `DictReader`, `DictWriter`, `Dialect`, `excel`, `excel_tab`,
#     `unix_dialect`, `field_size_limit`, `list_dialects`,
#     `get_dialect`, `register_dialect`, `unregister_dialect`, `Error`
#     all present; module name == 'csv'.
import csv
_ledger: list[int] = []

# QUOTE_* constants — canonical 3.12 values
assert csv.QUOTE_MINIMAL == 0; _ledger.append(1)
assert csv.QUOTE_ALL == 1; _ledger.append(1)
assert csv.QUOTE_NONNUMERIC == 2; _ledger.append(1)
assert csv.QUOTE_NONE == 3; _ledger.append(1)

# QUOTE_* constants — pairwise distinct
_quoting = {csv.QUOTE_MINIMAL, csv.QUOTE_ALL,
            csv.QUOTE_NONNUMERIC, csv.QUOTE_NONE}
assert len(_quoting) == 4; _ledger.append(1)

# list_dialects — built-ins always present
_d = csv.list_dialects()
assert isinstance(_d, list); _ledger.append(1)
assert 'excel' in _d; _ledger.append(1)
assert 'excel-tab' in _d; _ledger.append(1)
assert 'unix' in _d; _ledger.append(1)

# get_dialect('excel') — canonical attributes
_e = csv.get_dialect('excel')
assert _e.delimiter == ','; _ledger.append(1)
assert _e.quotechar == '"'; _ledger.append(1)
assert _e.doublequote == True; _ledger.append(1)
assert _e.skipinitialspace == False; _ledger.append(1)
assert _e.lineterminator == '\r\n'; _ledger.append(1)
assert _e.quoting == csv.QUOTE_MINIMAL; _ledger.append(1)

# get_dialect('excel-tab') — delimiter is TAB
_et = csv.get_dialect('excel-tab')
assert _et.delimiter == '\t'; _ledger.append(1)
assert _et.quotechar == '"'; _ledger.append(1)

# get_dialect('unix') — QUOTE_ALL + \n lineterm
_u = csv.get_dialect('unix')
assert _u.delimiter == ','; _ledger.append(1)
assert _u.quoting == csv.QUOTE_ALL; _ledger.append(1)
assert _u.lineterminator == '\n'; _ledger.append(1)

# Module-level class attributes — direct access (round-trip)
assert csv.excel.delimiter == ','; _ledger.append(1)
assert csv.excel.quotechar == '"'; _ledger.append(1)
assert csv.excel.doublequote == True; _ledger.append(1)
assert csv.excel.skipinitialspace == False; _ledger.append(1)
assert csv.excel.quoting == csv.QUOTE_MINIMAL; _ledger.append(1)

assert csv.excel_tab.delimiter == '\t'; _ledger.append(1)
assert csv.excel_tab.quotechar == '"'; _ledger.append(1)

assert csv.unix_dialect.delimiter == ','; _ledger.append(1)
assert csv.unix_dialect.quotechar == '"'; _ledger.append(1)
assert csv.unix_dialect.quoting == csv.QUOTE_ALL; _ledger.append(1)
assert csv.unix_dialect.lineterminator == '\n'; _ledger.append(1)

# register_dialect — custom dialect with delimiter ';' and quotechar "'"
csv.register_dialect('mamba_test_dialect_185',
                     delimiter=';', quotechar="'")
assert 'mamba_test_dialect_185' in csv.list_dialects(); _ledger.append(1)

_custom = csv.get_dialect('mamba_test_dialect_185')
assert _custom.delimiter == ';'; _ledger.append(1)
assert _custom.quotechar == "'"; _ledger.append(1)

# unregister_dialect — round-trip cleanup
csv.unregister_dialect('mamba_test_dialect_185')
assert 'mamba_test_dialect_185' not in csv.list_dialects(); _ledger.append(1)

# Built-in dialects survive register/unregister churn
assert 'excel' in csv.list_dialects(); _ledger.append(1)
assert 'excel-tab' in csv.list_dialects(); _ledger.append(1)
assert 'unix' in csv.list_dialects(); _ledger.append(1)

# Register a second custom dialect with different params
csv.register_dialect('mamba_pipe_185', delimiter='|',
                     quoting=csv.QUOTE_NONE)
assert 'mamba_pipe_185' in csv.list_dialects(); _ledger.append(1)
_pipe = csv.get_dialect('mamba_pipe_185')
assert _pipe.delimiter == '|'; _ledger.append(1)
assert _pipe.quoting == csv.QUOTE_NONE; _ledger.append(1)
csv.unregister_dialect('mamba_pipe_185')
assert 'mamba_pipe_185' not in csv.list_dialects(); _ledger.append(1)

# field_size_limit — getter returns int
_limit = csv.field_size_limit()
assert isinstance(_limit, int); _ledger.append(1)
assert _limit > 0; _ledger.append(1)

# field_size_limit — setter returns old value
_old = csv.field_size_limit(50000)
assert isinstance(_old, int); _ledger.append(1)
assert csv.field_size_limit() == 50000; _ledger.append(1)

# Restore previous limit
csv.field_size_limit(_old)
assert csv.field_size_limit() == _old; _ledger.append(1)

# Setter accepts a small value
csv.field_size_limit(1024)
assert csv.field_size_limit() == 1024; _ledger.append(1)
csv.field_size_limit(_old)  # restore

# Module attribute discipline — every helper present
for _name in ['reader', 'writer', 'DictReader', 'DictWriter',
              'Dialect', 'excel', 'excel_tab', 'unix_dialect',
              'field_size_limit', 'list_dialects', 'get_dialect',
              'register_dialect', 'unregister_dialect', 'Error']:
    assert hasattr(csv, _name); _ledger.append(1)

# Module name discipline
assert csv.__name__ == 'csv'; _ledger.append(1)

# Callable discipline — functions are callable
assert callable(csv.field_size_limit); _ledger.append(1)
assert callable(csv.list_dialects); _ledger.append(1)
assert callable(csv.get_dialect); _ledger.append(1)
assert callable(csv.register_dialect); _ledger.append(1)
assert callable(csv.unregister_dialect); _ledger.append(1)
assert callable(csv.reader); _ledger.append(1)
assert callable(csv.writer); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_csv_dialect_registry_field_size_limit_ops {sum(_ledger)} asserts")
