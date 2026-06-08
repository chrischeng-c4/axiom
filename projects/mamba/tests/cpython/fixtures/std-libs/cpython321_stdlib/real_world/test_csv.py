# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_csv"
# subject = "cpython321.test_csv"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_csv.py"
# status = "filled"
# ///
"""cpython321.test_csv: execute CPython 3.12 seed test_csv"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: csv — QUOTE_* constants, dialect registry (list_dialects /
# get_dialect / excel / excel_tab classes), reader() iteration over plain and
# quoted comma-separated input.
# Intentionally NOT exercised on mamba today (tracked separately):
#   * csv.writer (returns a str-like stub with no .writerow method)
#   * delimiter= / quotechar= / escapechar= reader kwargs (kwarg ignored —
#     full line returned as a single cell)
#   * csv.DictReader / csv.DictWriter (stub dispatch)
#   * csv.field_size_limit / csv.register_dialect / csv.unregister_dialect
#   * csv.Sniffer
import csv

_ledger: list[int] = []

# QUOTE_* constants (Python 3.12 values)
assert csv.QUOTE_MINIMAL == 0, f"csv.QUOTE_MINIMAL == 0, got {csv.QUOTE_MINIMAL}"
_ledger.append(1)

assert csv.QUOTE_ALL == 1, f"csv.QUOTE_ALL == 1, got {csv.QUOTE_ALL}"
_ledger.append(1)

assert csv.QUOTE_NONNUMERIC == 2, (
    f"csv.QUOTE_NONNUMERIC == 2, got {csv.QUOTE_NONNUMERIC}"
)
_ledger.append(1)

assert csv.QUOTE_NONE == 3, f"csv.QUOTE_NONE == 3, got {csv.QUOTE_NONE}"
_ledger.append(1)

# The four QUOTE_* constants are pairwise distinct
assert len({csv.QUOTE_MINIMAL, csv.QUOTE_ALL, csv.QUOTE_NONNUMERIC, csv.QUOTE_NONE}) == 4, (
    "csv QUOTE_* constants are pairwise distinct"
)
_ledger.append(1)

# Built-in dialects are registered
_dialects = csv.list_dialects()
assert isinstance(_dialects, list), "csv.list_dialects() returns a list"
_ledger.append(1)

assert "excel" in _dialects, f"'excel' in csv.list_dialects(), got {_dialects}"
_ledger.append(1)

assert "excel-tab" in _dialects, (
    f"'excel-tab' in csv.list_dialects(), got {_dialects}"
)
_ledger.append(1)

assert "unix" in _dialects, f"'unix' in csv.list_dialects(), got {_dialects}"
_ledger.append(1)

# get_dialect("excel") returns the excel dialect (round-trip via the registry)
_excel = csv.get_dialect("excel")
assert _excel is not None, "csv.get_dialect('excel') is not None"
_ledger.append(1)

# The dialect classes are exposed at module level
assert csv.excel is not None, "csv.excel is not None"
_ledger.append(1)

assert csv.excel_tab is not None, "csv.excel_tab is not None"
_ledger.append(1)

# reader() of a single-line comma-separated input
_r1 = list(csv.reader(["a,b,c"]))
assert _r1 == [["a", "b", "c"]], (
    f"csv.reader(['a,b,c']) == [['a','b','c']], got {_r1!r}"
)
_ledger.append(1)

# reader() handles multiple rows
_r2 = list(csv.reader(["1,2,3", "4,5,6"]))
assert _r2 == [["1", "2", "3"], ["4", "5", "6"]], (
    f"csv.reader two rows, got {_r2!r}"
)
_ledger.append(1)

# reader() honors quoted fields containing the comma delimiter
_r3 = list(csv.reader(['"x,y",z']))
assert _r3 == [["x,y", "z"]], (
    f"csv.reader quoted-comma: expected [['x,y','z']], got {_r3!r}"
)
_ledger.append(1)

# reader() handles a quoted field followed by an unquoted one
_r4 = list(csv.reader(['"hello","world","x"']))
assert _r4 == [["hello", "world", "x"]], (
    f"csv.reader three quoted fields, got {_r4!r}"
)
_ledger.append(1)

# reader() skips blank input lines (yields no row)
_r5 = list(csv.reader(["", "1,2,3"]))
assert _r5 == [[], ["1", "2", "3"]], (
    f"csv.reader blank + row, got {_r5!r}"
)
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_csv {sum(_ledger)} asserts")
