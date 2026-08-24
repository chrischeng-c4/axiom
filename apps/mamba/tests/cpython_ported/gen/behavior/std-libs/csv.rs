use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/csv/builtin_dialects_registered.py`.
#[test]
fn test_gen_behavior_std_libs_csv_builtin_dialects_registered() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "builtin_dialects_registered"
# subject = "csv.list_dialects"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.list_dialects: the built-in excel, excel-tab, and unix dialects are always present in list_dialects"""
import csv

builtins = csv.list_dialects()
assert "excel" in builtins, f"excel missing from {builtins!r}"
assert "excel-tab" in builtins, f"excel-tab missing from {builtins!r}"
assert "unix" in builtins, f"unix missing from {builtins!r}"

print("builtin_dialects_registered OK")
"###);
    assert_output(&out, r###"builtin_dialects_registered OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/custom_dialect_subclass_instance_accepted.py`.
#[test]
fn test_gen_behavior_std_libs_csv_custom_dialect_subclass_instance_accepted() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "custom_dialect_subclass_instance_accepted"
# subject = "csv.Dialect"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.Dialect: a custom Dialect subclass instance drives parsing: adjacent delimiters yield empty fields"""
import csv


class SpaceDialect(csv.excel):
    delimiter = " "
    quoting = csv.QUOTE_NONE
    escapechar = "\\"


space = list(csv.reader(["abc   def", "one two"], dialect=SpaceDialect()))
# Adjacent delimiters produce empty fields (skipinitialspace is False).
assert space[0] == ["abc", "", "", "def"], f"space row 0 = {space[0]!r}"
assert space[1] == ["one", "two"], f"space row 1 = {space[1]!r}"

print("custom_dialect_subclass_instance_accepted OK")
"###);
    assert_output(&out, r###"custom_dialect_subclass_instance_accepted OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/custom_quotechar_used_for_quoting.py`.
#[test]
fn test_gen_behavior_std_libs_csv_custom_quotechar_used_for_quoting() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "custom_quotechar_used_for_quoting"
# subject = "csv.writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.writer: a custom quotechar replaces the default double-quote when QUOTE_ALL wraps fields"""
import csv
import io

buf = io.StringIO()
csv.writer(buf, quotechar="'", quoting=csv.QUOTE_ALL).writerow(["hello", "world"])
out = buf.getvalue().strip()
assert out == "'hello','world'", f"custom quotechar = {out!r}"

print("custom_quotechar_used_for_quoting OK")
"###);
    assert_output(&out, r###"custom_quotechar_used_for_quoting OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/default_lineterminator_is_crlf.py`.
#[test]
fn test_gen_behavior_std_libs_csv_default_lineterminator_is_crlf() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "default_lineterminator_is_crlf"
# subject = "csv.writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.writer: the default line terminator written after a row is carriage-return + newline"""
import csv
import io

buf = io.StringIO()
csv.writer(buf).writerow(["x"])
raw = buf.getvalue()
assert raw == "x\r\n", f"default line terminator = {raw!r}"

print("default_lineterminator_is_crlf OK")
"###);
    assert_output(&out, r###"default_lineterminator_is_crlf OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/delimiter_changes_separator_both_ways.py`.
#[test]
fn test_gen_behavior_std_libs_csv_delimiter_changes_separator_both_ways() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "delimiter_changes_separator_both_ways"
# subject = "csv.writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.writer: a custom delimiter is used by writer output and honored by reader parsing"""
import csv
import io

buf = io.StringIO()
csv.writer(buf, delimiter="|").writerow(["x", "y", "z"])
out = buf.getvalue().strip()
assert out == "x|y|z", f"pipe delimiter = {out!r}"

rows = list(csv.reader(io.StringIO("x|y|z"), delimiter="|"))
assert rows == [["x", "y", "z"]], f"pipe reader = {rows!r}"

print("delimiter_changes_separator_both_ways OK")
"###);
    assert_output(&out, r###"delimiter_changes_separator_both_ways OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/dict_field_order_preserved_round_trip.py`.
#[test]
fn test_gen_behavior_std_libs_csv_dict_field_order_preserved_round_trip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "dict_field_order_preserved_round_trip"
# subject = "csv.DictWriter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.DictWriter: fieldname order survives a DictWriter header -> DictReader round trip across several orderings"""
import csv
import io

for keys in (["a", "b", "c"], ["c", "a", "b"], ["b", "c", "a"]):
    buf = io.StringIO()
    csv.DictWriter(buf, keys).writeheader()
    buf.seek(0)
    assert csv.DictReader(buf).fieldnames == keys, f"order {keys!r}"

print("dict_field_order_preserved_round_trip OK")
"###);
    assert_output(&out, r###"dict_field_order_preserved_round_trip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/dict_fieldnames_materialized_from_iterable.py`.
#[test]
fn test_gen_behavior_std_libs_csv_dict_fieldnames_materialized_from_iterable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "dict_fieldnames_materialized_from_iterable"
# subject = "csv.DictReader"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.DictReader: DictReader and DictWriter accept any iterable for fieldnames and store it as a list"""
import csv
import io

dr = csv.DictReader(io.StringIO("1,2\n"), fieldnames=iter(["a", "b"]))
assert dr.fieldnames == ["a", "b"], f"dr iter = {dr.fieldnames!r}"

dw = csv.DictWriter(io.StringIO(), iter(["a", "b", "c"]))
assert dw.fieldnames == ["a", "b", "c"], f"dw iter = {dw.fieldnames!r}"
assert isinstance(dw.fieldnames, list), "fieldnames stored as list"

print("dict_fieldnames_materialized_from_iterable OK")
"###);
    assert_output(&out, r###"dict_fieldnames_materialized_from_iterable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/dictreader_infers_fieldnames_from_first_row.py`.
#[test]
fn test_gen_behavior_std_libs_csv_dictreader_infers_fieldnames_from_first_row() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "dictreader_infers_fieldnames_from_first_row"
# subject = "csv.DictReader"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.DictReader: DictReader uses the first row as fieldnames and maps each later row to those keys"""
import csv
import io

data = "id,name,score\n1,Alice,95\n2,Bob,87\n"
dr = csv.DictReader(io.StringIO(data))
assert dr.fieldnames == ["id", "name", "score"], f"fieldnames = {dr.fieldnames!r}"
rows = list(dr)
assert rows[0]["name"] == "Alice", f"name = {rows[0]['name']!r}"
assert rows[1]["score"] == "87", f"score = {rows[1]['score']!r}"

print("dictreader_infers_fieldnames_from_first_row OK")
"###);
    assert_output(&out, r###"dictreader_infers_fieldnames_from_first_row OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/dictreader_restkey_collects_extra_values.py`.
#[test]
fn test_gen_behavior_std_libs_csv_dictreader_restkey_collects_extra_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "dictreader_restkey_collects_extra_values"
# subject = "csv.DictReader"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.DictReader: trailing values beyond fieldnames are collected into a list under restkey"""
import csv

reader = csv.DictReader(
    ["1,2,abc,4,5,6\r\n"], fieldnames=["f1", "f2"], restkey="_rest"
)
assert next(reader) == {"f1": "1", "f2": "2", "_rest": ["abc", "4", "5", "6"]}, "restkey"

print("dictreader_restkey_collects_extra_values OK")
"###);
    assert_output(&out, r###"dictreader_restkey_collects_extra_values OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/dictreader_restval_fills_missing_values.py`.
#[test]
fn test_gen_behavior_std_libs_csv_dictreader_restval_fills_missing_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "dictreader_restval_fills_missing_values"
# subject = "csv.DictReader"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.DictReader: missing trailing values are filled with restval; the default restval is None"""
import csv

r2 = csv.DictReader(["a,b\r\n"], fieldnames=["x", "y", "z"], restval="DEFAULT")
assert next(r2) == {"x": "a", "y": "b", "z": "DEFAULT"}, "restval"

r3 = csv.DictReader(["a\r\n"], fieldnames=["x", "y"])
assert next(r3) == {"x": "a", "y": None}, "default restval is None"

print("dictreader_restval_fills_missing_values OK")
"###);
    assert_output(&out, r###"dictreader_restval_fills_missing_values OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/dictwriter_extrasaction_ignore_drops_unknown_keys.py`.
#[test]
fn test_gen_behavior_std_libs_csv_dictwriter_extrasaction_ignore_drops_unknown_keys() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "dictwriter_extrasaction_ignore_drops_unknown_keys"
# subject = "csv.DictWriter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.DictWriter: extrasaction='ignore' silently drops keys not in fieldnames and writes only the known fields"""
import csv
import io

buf = io.StringIO()
dw = csv.DictWriter(buf, ["f1", "f2"], extrasaction="ignore")
dw.writerow({"f0": 0, "f1": 1, "f2": 2, "f3": 3})
assert buf.getvalue() == "1,2\r\n", f"ignore = {buf.getvalue()!r}"

print("dictwriter_extrasaction_ignore_drops_unknown_keys OK")
"###);
    assert_output(&out, r###"dictwriter_extrasaction_ignore_drops_unknown_keys OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/dictwriter_writeheader_returns_char_count.py`.
#[test]
fn test_gen_behavior_std_libs_csv_dictwriter_writeheader_returns_char_count() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "dictwriter_writeheader_returns_char_count"
# subject = "csv.DictWriter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.DictWriter: writeheader returns the number of characters written and emits the CRLF-terminated header"""
import csv
import io

buf = io.StringIO()
dw = csv.DictWriter(buf, fieldnames=["f1", "f2", "f3"])
assert dw.writeheader() == 10, "writeheader returns chars written"
assert buf.getvalue() == "f1,f2,f3\r\n", f"header = {buf.getvalue()!r}"

print("dictwriter_writeheader_returns_char_count OK")
"###);
    assert_output(&out, r###"dictwriter_writeheader_returns_char_count OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/dictwriter_writeheader_writes_fieldnames_row.py`.
#[test]
fn test_gen_behavior_std_libs_csv_dictwriter_writeheader_writes_fieldnames_row() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "dictwriter_writeheader_writes_fieldnames_row"
# subject = "csv.DictWriter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.DictWriter: DictWriter.writeheader emits the fieldnames as the first row, followed by mapped data rows"""
import csv
import io

buf = io.StringIO()
dw = csv.DictWriter(buf, fieldnames=["col1", "col2"])
dw.writeheader()
dw.writerow({"col1": "v1", "col2": "v2"})
buf.seek(0)
lines = buf.readlines()
assert lines[0].strip() == "col1,col2", f"header line = {lines[0]!r}"
assert lines[1].strip() == "v1,v2", f"data line = {lines[1]!r}"

print("dictwriter_writeheader_writes_fieldnames_row OK")
"###);
    assert_output(&out, r###"dictwriter_writeheader_writes_fieldnames_row OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/empty_field_round_trips.py`.
#[test]
fn test_gen_behavior_std_libs_csv_empty_field_round_trips() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "empty_field_round_trips"
# subject = "csv.writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.writer: an empty middle field survives a writer -> reader round trip as an empty string"""
import csv
import io

buf = io.StringIO()
csv.writer(buf).writerow(["a", "", "c"])
buf.seek(0)
rows = list(csv.reader(buf))
assert rows == [["a", "", "c"]], f"empty field = {rows!r}"

print("empty_field_round_trips OK")
"###);
    assert_output(&out, r###"empty_field_round_trips OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/escapechar_round_trips_under_quote_none.py`.
#[test]
fn test_gen_behavior_std_libs_csv_escapechar_round_trips_under_quote_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "escapechar_round_trips_under_quote_none"
# subject = "csv.writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.writer: under QUOTE_NONE the escapechar escapes an embedded delimiter and the value round-trips through reader"""
import csv
import io

buf = io.StringIO()
csv.writer(buf, escapechar="\\", quoting=csv.QUOTE_NONE).writerow(["a,b", "c"])
assert buf.getvalue() == "a\\,b,c\r\n", f"escape write = {buf.getvalue()!r}"

buf.seek(0)
rows = list(csv.reader(buf, escapechar="\\", quoting=csv.QUOTE_NONE))
assert rows == [["a,b", "c"]], f"escape read = {rows!r}"

print("escapechar_round_trips_under_quote_none OK")
"###);
    assert_output(&out, r###"escapechar_round_trips_under_quote_none OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/key_ordering_test__test_ordering_for_the_dict_reader_and_writer.py`.
#[test]
fn test_gen_behavior_std_libs_csv_key_ordering_test__test_ordering_for_the_dict_reader_and_writer() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "key_ordering_test__test_ordering_for_the_dict_reader_and_writer"
# subject = "cpython.test_csv.KeyOrderingTest.test_ordering_for_the_dict_reader_and_writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::KeyOrderingTest::test_ordering_for_the_dict_reader_and_writer
"""Auto-ported test: KeyOrderingTest::test_ordering_for_the_dict_reader_and_writer (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
resultset = set()
for keys in permutations('abcde'):
    with TemporaryFile('w+', newline='', encoding='utf-8') as fileobject:
        dw = csv.DictWriter(fileobject, keys)
        dw.writeheader()
        fileobject.seek(0)
        dr = csv.DictReader(fileobject)
        kt = tuple(dr.fieldnames)

        assert keys == kt
        resultset.add(kt)

assert len(resultset) == 120
print("KeyOrderingTest::test_ordering_for_the_dict_reader_and_writer: ok")
"###);
    assert_output(&out, r###"KeyOrderingTest::test_ordering_for_the_dict_reader_and_writer: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/misc_test_case__test_subclassable.py`.
#[test]
fn test_gen_behavior_std_libs_csv_misc_test_case__test_subclassable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "misc_test_case__test_subclassable"
# subject = "cpython.test_csv.MiscTestCase.test_subclassable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::MiscTestCase::test_subclassable
"""Auto-ported test: MiscTestCase::test_subclassable (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
class Foo(csv.Error):
    ...
print("MiscTestCase::test_subclassable: ok")
"###);
    assert_output(&out, r###"MiscTestCase::test_subclassable: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/quote_all_wraps_every_field.py`.
#[test]
fn test_gen_behavior_std_libs_csv_quote_all_wraps_every_field() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "quote_all_wraps_every_field"
# subject = "csv.writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.writer: quoting=QUOTE_ALL wraps each written field in the quotechar"""
import csv
import io

buf = io.StringIO()
csv.writer(buf, quoting=csv.QUOTE_ALL).writerow(["a", "b", "c"])
out = buf.getvalue().strip()
assert out == '"a","b","c"', f"QUOTE_ALL = {out!r}"

print("quote_all_wraps_every_field OK")
"###);
    assert_output(&out, r###"quote_all_wraps_every_field OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/reader_line_num_tracks_consumed_lines.py`.
#[test]
fn test_gen_behavior_std_libs_csv_reader_line_num_tracks_consumed_lines() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "reader_line_num_tracks_consumed_lines"
# subject = "csv.reader"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.reader: reader.line_num starts at 0, increments per consumed source line, and stays put at EOF"""
import csv

reader = csv.reader(["line,1", "line,2", "line,3"])
assert reader.line_num == 0, f"start line_num = {reader.line_num}"
for expected in (1, 2, 3):
    next(reader)
    assert reader.line_num == expected, f"line_num = {reader.line_num}"

_stopped = False
try:
    next(reader)
except StopIteration:
    _stopped = True
assert _stopped, "expected StopIteration"
assert reader.line_num == 3, f"line_num after EOF = {reader.line_num}"

print("reader_line_num_tracks_consumed_lines OK")
"###);
    assert_output(&out, r###"reader_line_num_tracks_consumed_lines OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/reader_quoted_field_keeps_embedded_comma.py`.
#[test]
fn test_gen_behavior_std_libs_csv_reader_quoted_field_keeps_embedded_comma() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "reader_quoted_field_keeps_embedded_comma"
# subject = "csv.reader"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.reader: a double-quoted field preserves an embedded comma; remaining fields split normally and all come back as str"""
import csv
import io

rows = list(csv.reader(io.StringIO('"hello, world",42,True')))
assert rows == [["hello, world", "42", "True"]], f"quoted comma = {rows!r}"

print("reader_quoted_field_keeps_embedded_comma OK")
"###);
    assert_output(&out, r###"reader_quoted_field_keeps_embedded_comma OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/register_get_unregister_dialect_round_trip.py`.
#[test]
fn test_gen_behavior_std_libs_csv_register_get_unregister_dialect_round_trip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "register_get_unregister_dialect_round_trip"
# subject = "csv.register_dialect"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.register_dialect: a registered named dialect is usable by reader/writer and disappears after unregister_dialect"""
import csv
import io

csv.register_dialect("pipes", delimiter="|", quoting=csv.QUOTE_MINIMAL)
assert "pipes" in csv.list_dialects(), "pipes not registered"
assert csv.get_dialect("pipes").delimiter == "|", "pipes delimiter"

buf = io.StringIO()
csv.writer(buf, dialect="pipes").writerow(["x", "y", "z"])
assert buf.getvalue() == "x|y|z\r\n", f"pipes write = {buf.getvalue()!r}"

rows = list(csv.reader(io.StringIO("x|y|z"), dialect="pipes"))
assert rows == [["x", "y", "z"]], f"pipes read = {rows!r}"

csv.unregister_dialect("pipes")
assert "pipes" not in csv.list_dialects(), "pipes still registered"

print("register_get_unregister_dialect_round_trip OK")
"###);
    assert_output(&out, r###"register_get_unregister_dialect_round_trip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_csv__test_read_linenum.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_csv__test_read_linenum() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_csv__test_read_linenum"
# subject = "cpython.test_csv.Test_Csv.test_read_linenum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::Test_Csv::test_read_linenum
"""Auto-ported test: Test_Csv::test_read_linenum (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
r = csv.reader(['line,1', 'line,2', 'line,3'])

assert r.line_num == 0
next(r)

assert r.line_num == 1
next(r)

assert r.line_num == 2
next(r)

assert r.line_num == 3

try:
    next(r)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass

assert r.line_num == 3
print("Test_Csv::test_read_linenum: ok")
"###);
    assert_output(&out, r###"Test_Csv::test_read_linenum: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_csv__test_writerows.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_csv__test_writerows() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_csv__test_writerows"
# subject = "cpython.test_csv.Test_Csv.test_writerows"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::Test_Csv::test_writerows
"""Auto-ported test: Test_Csv::test_writerows (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
class BrokenFile:

    def write(self, buf):
        raise OSError
writer = csv.writer(BrokenFile())

try:
    writer.writerows([['a']])
    raise AssertionError('expected OSError')
except OSError:
    pass
with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
    writer = csv.writer(fileobj)

    try:
        writer.writerows(None)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    writer.writerows([['a', 'b'], ['c', 'd']])
    fileobj.seek(0)

    assert fileobj.read() == 'a,b\r\nc,d\r\n'
print("Test_Csv::test_writerows: ok")
"###);
    assert_output(&out, r###"Test_Csv::test_writerows: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_csv__test_writerows_with_none.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_csv__test_writerows_with_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_csv__test_writerows_with_none"
# subject = "cpython.test_csv.Test_Csv.test_writerows_with_none"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::Test_Csv::test_writerows_with_none
"""Auto-ported test: Test_Csv::test_writerows_with_none (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
    writer = csv.writer(fileobj)
    writer.writerows([['a', None], [None, 'd']])
    fileobj.seek(0)

    assert fileobj.read() == 'a,\r\n,d\r\n'
with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
    writer = csv.writer(fileobj)
    writer.writerows([[None], ['a']])
    fileobj.seek(0)

    assert fileobj.read() == '""\r\na\r\n'
with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
    writer = csv.writer(fileobj)
    writer.writerows([['a'], [None]])
    fileobj.seek(0)

    assert fileobj.read() == 'a\r\n""\r\n'
print("Test_Csv::test_writerows_with_none: ok")
"###);
    assert_output(&out, r###"Test_Csv::test_writerows_with_none: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_blankline.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_blankline() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_blankline"
# subject = "cpython.test_csv.TestDialectExcel.test_blankline"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_blankline
"""Auto-ported test: TestDialectExcel::test_blankline (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
readerAssertEqual('', [])
print("TestDialectExcel::test_blankline: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_blankline: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_dubious_quote.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_dubious_quote() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_dubious_quote"
# subject = "cpython.test_csv.TestDialectExcel.test_dubious_quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_dubious_quote
"""Auto-ported test: TestDialectExcel::test_dubious_quote (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
readerAssertEqual('12,12,1",', [['12', '12', '1"', '']])
print("TestDialectExcel::test_dubious_quote: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_dubious_quote: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_empty_fields.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_empty_fields() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_empty_fields"
# subject = "cpython.test_csv.TestDialectExcel.test_empty_fields"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_empty_fields
"""Auto-ported test: TestDialectExcel::test_empty_fields (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
readerAssertEqual(',', [['', '']])
print("TestDialectExcel::test_empty_fields: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_empty_fields: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_inline_quote.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_inline_quote() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_inline_quote"
# subject = "cpython.test_csv.TestDialectExcel.test_inline_quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_inline_quote
"""Auto-ported test: TestDialectExcel::test_inline_quote (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
readerAssertEqual('a""b', [['a""b']])
print("TestDialectExcel::test_inline_quote: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_inline_quote: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_inline_quotes.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_inline_quotes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_inline_quotes"
# subject = "cpython.test_csv.TestDialectExcel.test_inline_quotes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_inline_quotes
"""Auto-ported test: TestDialectExcel::test_inline_quotes (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
readerAssertEqual('a"b"c', [['a"b"c']])
print("TestDialectExcel::test_inline_quotes: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_inline_quotes: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_lone_quote.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_lone_quote() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_lone_quote"
# subject = "cpython.test_csv.TestDialectExcel.test_lone_quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_lone_quote
"""Auto-ported test: TestDialectExcel::test_lone_quote (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
readerAssertEqual('a"b', [['a"b']])
print("TestDialectExcel::test_lone_quote: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_lone_quote: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_newlines.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_newlines() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_newlines"
# subject = "cpython.test_csv.TestDialectExcel.test_newlines"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_newlines
"""Auto-ported test: TestDialectExcel::test_newlines (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
writerAssertEqual([[1, 2, 'a\nbc', 3, 4]], '1,2,"a\nbc",3,4\r\n')
print("TestDialectExcel::test_newlines: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_newlines: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_null.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_null() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_null"
# subject = "cpython.test_csv.TestDialectExcel.test_null"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_null
"""Auto-ported test: TestDialectExcel::test_null (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
writerAssertEqual([], '')
print("TestDialectExcel::test_null: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_null: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_quote_and_quote.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_quote_and_quote() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_quote_and_quote"
# subject = "cpython.test_csv.TestDialectExcel.test_quote_and_quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_quote_and_quote
"""Auto-ported test: TestDialectExcel::test_quote_and_quote (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
readerAssertEqual('"a" "b"', [['a "b"']])
print("TestDialectExcel::test_quote_and_quote: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_quote_and_quote: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_quote_fieldsep.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_quote_fieldsep() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_quote_fieldsep"
# subject = "cpython.test_csv.TestDialectExcel.test_quote_fieldsep"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_quote_fieldsep
"""Auto-ported test: TestDialectExcel::test_quote_fieldsep (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
writerAssertEqual([['abc,def']], '"abc,def"\r\n')
print("TestDialectExcel::test_quote_fieldsep: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_quote_fieldsep: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_quoted.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_quoted() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_quoted"
# subject = "cpython.test_csv.TestDialectExcel.test_quoted"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_quoted
"""Auto-ported test: TestDialectExcel::test_quoted (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
readerAssertEqual('1,2,3,"I think, therefore I am",5,6', [['1', '2', '3', 'I think, therefore I am', '5', '6']])
print("TestDialectExcel::test_quoted: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_quoted: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_quoted_quote.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_quoted_quote() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_quoted_quote"
# subject = "cpython.test_csv.TestDialectExcel.test_quoted_quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_quoted_quote
"""Auto-ported test: TestDialectExcel::test_quoted_quote (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
readerAssertEqual('1,2,3,"""I see,"" said the blind man","as he picked up his hammer and saw"', [['1', '2', '3', '"I see," said the blind man', 'as he picked up his hammer and saw']])
print("TestDialectExcel::test_quoted_quote: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_quoted_quote: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_quoted_quotes.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_quoted_quotes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_quoted_quotes"
# subject = "cpython.test_csv.TestDialectExcel.test_quoted_quotes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_quoted_quotes
"""Auto-ported test: TestDialectExcel::test_quoted_quotes (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
readerAssertEqual('""""""', [['""']])
print("TestDialectExcel::test_quoted_quotes: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_quoted_quotes: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_quotes.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_quotes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_quotes"
# subject = "cpython.test_csv.TestDialectExcel.test_quotes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_quotes
"""Auto-ported test: TestDialectExcel::test_quotes (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
writerAssertEqual([[1, 2, 'a"bc"', 3, 4]], '1,2,"a""bc""",3,4\r\n')
print("TestDialectExcel::test_quotes: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_quotes: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_quotes_and_more.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_quotes_and_more() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_quotes_and_more"
# subject = "cpython.test_csv.TestDialectExcel.test_quotes_and_more"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_quotes_and_more
"""Auto-ported test: TestDialectExcel::test_quotes_and_more (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
readerAssertEqual('"a"b', [['ab']])
print("TestDialectExcel::test_quotes_and_more: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_quotes_and_more: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_simple.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_simple() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_simple"
# subject = "cpython.test_csv.TestDialectExcel.test_simple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_simple
"""Auto-ported test: TestDialectExcel::test_simple (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
readerAssertEqual('1,2,3,4,5', [['1', '2', '3', '4', '5']])
print("TestDialectExcel::test_simple: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_simple: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_simple_writer.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_simple_writer() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_simple_writer"
# subject = "cpython.test_csv.TestDialectExcel.test_simple_writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_simple_writer
"""Auto-ported test: TestDialectExcel::test_simple_writer (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
writerAssertEqual([[1, 2, 'abc', 3, 4]], '1,2,abc,3,4\r\n')
print("TestDialectExcel::test_simple_writer: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_simple_writer: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_single.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_single() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_single"
# subject = "cpython.test_csv.TestDialectExcel.test_single"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_single
"""Auto-ported test: TestDialectExcel::test_single (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
readerAssertEqual('abc', [['abc']])
print("TestDialectExcel::test_single: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_single: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_single_quoted_quote.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_single_quoted_quote() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_single_quoted_quote"
# subject = "cpython.test_csv.TestDialectExcel.test_single_quoted_quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_single_quoted_quote
"""Auto-ported test: TestDialectExcel::test_single_quoted_quote (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
readerAssertEqual('""""', [['"']])
print("TestDialectExcel::test_single_quoted_quote: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_single_quoted_quote: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_single_writer.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_single_writer() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_single_writer"
# subject = "cpython.test_csv.TestDialectExcel.test_single_writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_single_writer
"""Auto-ported test: TestDialectExcel::test_single_writer (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
writerAssertEqual([['abc']], 'abc\r\n')
print("TestDialectExcel::test_single_writer: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_single_writer: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_singlequoted.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_singlequoted() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_singlequoted"
# subject = "cpython.test_csv.TestDialectExcel.test_singlequoted"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_singlequoted
"""Auto-ported test: TestDialectExcel::test_singlequoted (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
readerAssertEqual('""', [['']])
print("TestDialectExcel::test_singlequoted: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_singlequoted: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_singlequoted_left_empty.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_singlequoted_left_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_singlequoted_left_empty"
# subject = "cpython.test_csv.TestDialectExcel.test_singlequoted_left_empty"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_singlequoted_left_empty
"""Auto-ported test: TestDialectExcel::test_singlequoted_left_empty (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
readerAssertEqual('"",', [['', '']])
print("TestDialectExcel::test_singlequoted_left_empty: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_singlequoted_left_empty: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_singlequoted_right_empty.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_singlequoted_right_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_singlequoted_right_empty"
# subject = "cpython.test_csv.TestDialectExcel.test_singlequoted_right_empty"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_singlequoted_right_empty
"""Auto-ported test: TestDialectExcel::test_singlequoted_right_empty (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
readerAssertEqual(',""', [['', '']])
print("TestDialectExcel::test_singlequoted_right_empty: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_singlequoted_right_empty: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_excel__test_space_and_quote.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_excel__test_space_and_quote() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_excel__test_space_and_quote"
# subject = "cpython.test_csv.TestDialectExcel.test_space_and_quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectExcel::test_space_and_quote
"""Auto-ported test: TestDialectExcel::test_space_and_quote (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'excel'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
readerAssertEqual(' "a"', [[' "a"']])
print("TestDialectExcel::test_space_and_quote: ok")
"###);
    assert_output(&out, r###"TestDialectExcel::test_space_and_quote: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_registry__test_incomplete_dialect.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_registry__test_incomplete_dialect() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_registry__test_incomplete_dialect"
# subject = "cpython.test_csv.TestDialectRegistry.test_incomplete_dialect"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectRegistry::test_incomplete_dialect
"""Auto-ported test: TestDialectRegistry::test_incomplete_dialect (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
class myexceltsv(csv.Dialect):
    delimiter = '\t'

try:
    myexceltsv()
    raise AssertionError('expected csv.Error')
except csv.Error:
    pass
print("TestDialectRegistry::test_incomplete_dialect: ok")
"###);
    assert_output(&out, r###"TestDialectRegistry::test_incomplete_dialect: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_registry__test_register_kwargs.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_registry__test_register_kwargs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_registry__test_register_kwargs"
# subject = "cpython.test_csv.TestDialectRegistry.test_register_kwargs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectRegistry::test_register_kwargs
"""Auto-ported test: TestDialectRegistry::test_register_kwargs (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
def compare_dialect_123(expected, *writeargs, **kwwriteargs):
    with TemporaryFile('w+', newline='', encoding='utf-8') as fileobj:
        writer = csv.writer(fileobj, *writeargs, **kwwriteargs)
        writer.writerow([1, 2, 3])
        fileobj.seek(0)

        assert fileobj.read() == expected
name = 'fedcba'
csv.register_dialect(name, delimiter=';')
pass

assert csv.get_dialect(name).delimiter == ';'

assert [['X', 'Y', 'Z']] == list(csv.reader(['X;Y;Z'], name))
print("TestDialectRegistry::test_register_kwargs: ok")
"###);
    assert_output(&out, r###"TestDialectRegistry::test_register_kwargs: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_registry__test_register_kwargs_override.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_registry__test_register_kwargs_override() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_registry__test_register_kwargs_override"
# subject = "cpython.test_csv.TestDialectRegistry.test_register_kwargs_override"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectRegistry::test_register_kwargs_override
"""Auto-ported test: TestDialectRegistry::test_register_kwargs_override (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
def compare_dialect_123(expected, *writeargs, **kwwriteargs):
    with TemporaryFile('w+', newline='', encoding='utf-8') as fileobj:
        writer = csv.writer(fileobj, *writeargs, **kwwriteargs)
        writer.writerow([1, 2, 3])
        fileobj.seek(0)

        assert fileobj.read() == expected

class mydialect(csv.Dialect):
    delimiter = '\t'
    quotechar = '"'
    doublequote = True
    skipinitialspace = False
    lineterminator = '\r\n'
    quoting = csv.QUOTE_MINIMAL
name = 'test_dialect'
csv.register_dialect(name, mydialect, delimiter=';', quotechar="'", doublequote=False, skipinitialspace=True, lineterminator='\n', quoting=csv.QUOTE_ALL)
pass
dialect = csv.get_dialect(name)

assert dialect.delimiter == ';'

assert dialect.quotechar == "'"

assert dialect.doublequote == False

assert dialect.skipinitialspace == True

assert dialect.lineterminator == '\n'

assert dialect.quoting == csv.QUOTE_ALL
print("TestDialectRegistry::test_register_kwargs_override: ok")
"###);
    assert_output(&out, r###"TestDialectRegistry::test_register_kwargs_override: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_registry__test_registry.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_registry__test_registry() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_registry__test_registry"
# subject = "cpython.test_csv.TestDialectRegistry.test_registry"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectRegistry::test_registry
"""Auto-ported test: TestDialectRegistry::test_registry (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
def compare_dialect_123(expected, *writeargs, **kwwriteargs):
    with TemporaryFile('w+', newline='', encoding='utf-8') as fileobj:
        writer = csv.writer(fileobj, *writeargs, **kwwriteargs)
        writer.writerow([1, 2, 3])
        fileobj.seek(0)

        assert fileobj.read() == expected

class myexceltsv(csv.excel):
    delimiter = '\t'
name = 'myexceltsv'
expected_dialects = csv.list_dialects() + [name]
expected_dialects.sort()
csv.register_dialect(name, myexceltsv)
pass

assert csv.get_dialect(name).delimiter == '\t'
got_dialects = sorted(csv.list_dialects())

assert expected_dialects == got_dialects
print("TestDialectRegistry::test_registry: ok")
"###);
    assert_output(&out, r###"TestDialectRegistry::test_registry: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_registry__test_space_dialect.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_registry__test_space_dialect() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_registry__test_space_dialect"
# subject = "cpython.test_csv.TestDialectRegistry.test_space_dialect"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectRegistry::test_space_dialect
"""Auto-ported test: TestDialectRegistry::test_space_dialect (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
class space(csv.excel):
    delimiter = ' '
    quoting = csv.QUOTE_NONE
    escapechar = '\\'
with TemporaryFile('w+', encoding='utf-8') as fileobj:
    fileobj.write('abc   def\nc1ccccc1 benzene\n')
    fileobj.seek(0)
    reader = csv.reader(fileobj, dialect=space())

    assert next(reader) == ['abc', '', '', 'def']

    assert next(reader) == ['c1ccccc1', 'benzene']
print("TestDialectRegistry::test_space_dialect: ok")
"###);
    assert_output(&out, r###"TestDialectRegistry::test_space_dialect: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_unix__test_simple_reader.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_unix__test_simple_reader() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_unix__test_simple_reader"
# subject = "cpython.test_csv.TestDialectUnix.test_simple_reader"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectUnix::test_simple_reader
"""Auto-ported test: TestDialectUnix::test_simple_reader (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'unix'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
readerAssertEqual('"1","abc def","abc"\n', [['1', 'abc def', 'abc']])
print("TestDialectUnix::test_simple_reader: ok")
"###);
    assert_output(&out, r###"TestDialectUnix::test_simple_reader: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_unix__test_simple_writer.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_unix__test_simple_writer() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_unix__test_simple_writer"
# subject = "cpython.test_csv.TestDialectUnix.test_simple_writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectUnix::test_simple_writer
"""Auto-ported test: TestDialectUnix::test_simple_writer (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = 'unix'

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
writerAssertEqual([[1, 'abc def', 'abc']], '"1","abc def","abc"\n')
print("TestDialectUnix::test_simple_writer: ok")
"###);
    assert_output(&out, r###"TestDialectUnix::test_simple_writer: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_validity__test_delimiter.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_validity__test_delimiter() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_validity__test_delimiter"
# subject = "cpython.test_csv.TestDialectValidity.test_delimiter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectValidity::test_delimiter
"""Auto-ported test: TestDialectValidity::test_delimiter (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
class mydialect(csv.Dialect):
    delimiter = ';'
    escapechar = '\\'
    doublequote = False
    skipinitialspace = True
    lineterminator = '\r\n'
    quoting = csv.QUOTE_NONE
d = mydialect()

assert d.delimiter == ';'
mydialect.delimiter = ':::'
try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == '"delimiter" must be a 1-character string'
mydialect.delimiter = ''
try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == '"delimiter" must be a 1-character string'
mydialect.delimiter = b','
try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == '"delimiter" must be string, not bytes'
mydialect.delimiter = 4
try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == '"delimiter" must be string, not int'
mydialect.delimiter = None
try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == '"delimiter" must be string, not NoneType'
print("TestDialectValidity::test_delimiter: ok")
"###);
    assert_output(&out, r###"TestDialectValidity::test_delimiter: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_validity__test_escapechar.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_validity__test_escapechar() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_validity__test_escapechar"
# subject = "cpython.test_csv.TestDialectValidity.test_escapechar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectValidity::test_escapechar
"""Auto-ported test: TestDialectValidity::test_escapechar (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
class mydialect(csv.Dialect):
    delimiter = ';'
    escapechar = '\\'
    doublequote = False
    skipinitialspace = True
    lineterminator = '\r\n'
    quoting = csv.QUOTE_NONE
d = mydialect()

assert d.escapechar == '\\'
mydialect.escapechar = ''
try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error as _aR_e:
    import re as _re_aR
    assert _re_aR.search('"escapechar" must be a 1-character string', str(_aR_e))
mydialect.escapechar = '**'
try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error as _aR_e:
    import re as _re_aR
    assert _re_aR.search('"escapechar" must be a 1-character string', str(_aR_e))
mydialect.escapechar = b'*'
try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error as _aR_e:
    import re as _re_aR
    assert _re_aR.search('"escapechar" must be string or None, not bytes', str(_aR_e))
mydialect.escapechar = 4
try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error as _aR_e:
    import re as _re_aR
    assert _re_aR.search('"escapechar" must be string or None, not int', str(_aR_e))
print("TestDialectValidity::test_escapechar: ok")
"###);
    assert_output(&out, r###"TestDialectValidity::test_escapechar: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dialect_validity__test_lineterminator.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dialect_validity__test_lineterminator() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dialect_validity__test_lineterminator"
# subject = "cpython.test_csv.TestDialectValidity.test_lineterminator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDialectValidity::test_lineterminator
"""Auto-ported test: TestDialectValidity::test_lineterminator (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
class mydialect(csv.Dialect):
    delimiter = ';'
    escapechar = '\\'
    doublequote = False
    skipinitialspace = True
    lineterminator = '\r\n'
    quoting = csv.QUOTE_NONE
d = mydialect()

assert d.lineterminator == '\r\n'
mydialect.lineterminator = ':::'
d = mydialect()

assert d.lineterminator == ':::'
mydialect.lineterminator = 4
try:
    mydialect()
    raise AssertionError('expected csv.Error')
except csv.Error as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)

assert str(cm.exception) == '"lineterminator" must be a string'
print("TestDialectValidity::test_lineterminator: ok")
"###);
    assert_output(&out, r###"TestDialectValidity::test_lineterminator: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dict_fields__test_dict_reader_fieldnames_accepts_iter.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dict_fields__test_dict_reader_fieldnames_accepts_iter() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_dict_reader_fieldnames_accepts_iter"
# subject = "cpython.test_csv.TestDictFields.test_dict_reader_fieldnames_accepts_iter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_dict_reader_fieldnames_accepts_iter
"""Auto-ported test: TestDictFields::test_dict_reader_fieldnames_accepts_iter (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
fieldnames = ['a', 'b', 'c']
f = StringIO()
reader = csv.DictReader(f, iter(fieldnames))

assert reader.fieldnames == fieldnames
print("TestDictFields::test_dict_reader_fieldnames_accepts_iter: ok")
"###);
    assert_output(&out, r###"TestDictFields::test_dict_reader_fieldnames_accepts_iter: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dict_fields__test_dict_reader_fieldnames_accepts_list.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dict_fields__test_dict_reader_fieldnames_accepts_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_dict_reader_fieldnames_accepts_list"
# subject = "cpython.test_csv.TestDictFields.test_dict_reader_fieldnames_accepts_list"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_dict_reader_fieldnames_accepts_list
"""Auto-ported test: TestDictFields::test_dict_reader_fieldnames_accepts_list (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
fieldnames = ['a', 'b', 'c']
f = StringIO()
reader = csv.DictReader(f, fieldnames)

assert reader.fieldnames == fieldnames
print("TestDictFields::test_dict_reader_fieldnames_accepts_list: ok")
"###);
    assert_output(&out, r###"TestDictFields::test_dict_reader_fieldnames_accepts_list: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dict_fields__test_dict_reader_fieldnames_is_optional.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dict_fields__test_dict_reader_fieldnames_is_optional() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_dict_reader_fieldnames_is_optional"
# subject = "cpython.test_csv.TestDictFields.test_dict_reader_fieldnames_is_optional"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_dict_reader_fieldnames_is_optional
"""Auto-ported test: TestDictFields::test_dict_reader_fieldnames_is_optional (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
f = StringIO()
reader = csv.DictReader(f, fieldnames=None)
print("TestDictFields::test_dict_reader_fieldnames_is_optional: ok")
"###);
    assert_output(&out, r###"TestDictFields::test_dict_reader_fieldnames_is_optional: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dict_fields__test_dict_writer_fieldnames_accepts_list.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dict_fields__test_dict_writer_fieldnames_accepts_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_dict_writer_fieldnames_accepts_list"
# subject = "cpython.test_csv.TestDictFields.test_dict_writer_fieldnames_accepts_list"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_dict_writer_fieldnames_accepts_list
"""Auto-ported test: TestDictFields::test_dict_writer_fieldnames_accepts_list (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
fieldnames = ['a', 'b', 'c']
f = StringIO()
writer = csv.DictWriter(f, fieldnames)

assert writer.fieldnames == fieldnames
print("TestDictFields::test_dict_writer_fieldnames_accepts_list: ok")
"###);
    assert_output(&out, r###"TestDictFields::test_dict_writer_fieldnames_accepts_list: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dict_fields__test_dict_writer_fieldnames_rejects_iter.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dict_fields__test_dict_writer_fieldnames_rejects_iter() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_dict_writer_fieldnames_rejects_iter"
# subject = "cpython.test_csv.TestDictFields.test_dict_writer_fieldnames_rejects_iter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_dict_writer_fieldnames_rejects_iter
"""Auto-ported test: TestDictFields::test_dict_writer_fieldnames_rejects_iter (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
fieldnames = ['a', 'b', 'c']
f = StringIO()
writer = csv.DictWriter(f, iter(fieldnames))

assert writer.fieldnames == fieldnames
print("TestDictFields::test_dict_writer_fieldnames_rejects_iter: ok")
"###);
    assert_output(&out, r###"TestDictFields::test_dict_writer_fieldnames_rejects_iter: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dict_fields__test_read_dict_fieldnames_chain.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dict_fields__test_read_dict_fieldnames_chain() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_read_dict_fieldnames_chain"
# subject = "cpython.test_csv.TestDictFields.test_read_dict_fieldnames_chain"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_read_dict_fieldnames_chain
"""Auto-ported test: TestDictFields::test_read_dict_fieldnames_chain (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
import itertools
with TemporaryFile('w+', encoding='utf-8') as fileobj:
    fileobj.write('f1,f2,f3\r\n1,2,abc\r\n')
    fileobj.seek(0)
    reader = csv.DictReader(fileobj)
    first = next(reader)
    for row in itertools.chain([first], reader):

        assert reader.fieldnames == ['f1', 'f2', 'f3']

        assert row == {'f1': '1', 'f2': '2', 'f3': 'abc'}
print("TestDictFields::test_read_dict_fieldnames_chain: ok")
"###);
    assert_output(&out, r###"TestDictFields::test_read_dict_fieldnames_chain: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dict_fields__test_read_dict_fields.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dict_fields__test_read_dict_fields() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_read_dict_fields"
# subject = "cpython.test_csv.TestDictFields.test_read_dict_fields"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_read_dict_fields
"""Auto-ported test: TestDictFields::test_read_dict_fields (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
with TemporaryFile('w+', encoding='utf-8') as fileobj:
    fileobj.write('1,2,abc\r\n')
    fileobj.seek(0)
    reader = csv.DictReader(fileobj, fieldnames=['f1', 'f2', 'f3'])

    assert next(reader) == {'f1': '1', 'f2': '2', 'f3': 'abc'}
print("TestDictFields::test_read_dict_fields: ok")
"###);
    assert_output(&out, r###"TestDictFields::test_read_dict_fields: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dict_fields__test_read_dict_no_fieldnames.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dict_fields__test_read_dict_no_fieldnames() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_read_dict_no_fieldnames"
# subject = "cpython.test_csv.TestDictFields.test_read_dict_no_fieldnames"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_read_dict_no_fieldnames
"""Auto-ported test: TestDictFields::test_read_dict_no_fieldnames (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
with TemporaryFile('w+', encoding='utf-8') as fileobj:
    fileobj.write('f1,f2,f3\r\n1,2,abc\r\n')
    fileobj.seek(0)
    reader = csv.DictReader(fileobj)

    assert next(reader) == {'f1': '1', 'f2': '2', 'f3': 'abc'}

    assert reader.fieldnames == ['f1', 'f2', 'f3']
print("TestDictFields::test_read_dict_no_fieldnames: ok")
"###);
    assert_output(&out, r###"TestDictFields::test_read_dict_no_fieldnames: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dict_fields__test_read_long.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dict_fields__test_read_long() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_read_long"
# subject = "cpython.test_csv.TestDictFields.test_read_long"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_read_long
"""Auto-ported test: TestDictFields::test_read_long (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
with TemporaryFile('w+', encoding='utf-8') as fileobj:
    fileobj.write('1,2,abc,4,5,6\r\n')
    fileobj.seek(0)
    reader = csv.DictReader(fileobj, fieldnames=['f1', 'f2'])

    assert next(reader) == {'f1': '1', 'f2': '2', None: ['abc', '4', '5', '6']}
print("TestDictFields::test_read_long: ok")
"###);
    assert_output(&out, r###"TestDictFields::test_read_long: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dict_fields__test_read_long_with_rest.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dict_fields__test_read_long_with_rest() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_read_long_with_rest"
# subject = "cpython.test_csv.TestDictFields.test_read_long_with_rest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_read_long_with_rest
"""Auto-ported test: TestDictFields::test_read_long_with_rest (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
with TemporaryFile('w+', encoding='utf-8') as fileobj:
    fileobj.write('1,2,abc,4,5,6\r\n')
    fileobj.seek(0)
    reader = csv.DictReader(fileobj, fieldnames=['f1', 'f2'], restkey='_rest')

    assert next(reader) == {'f1': '1', 'f2': '2', '_rest': ['abc', '4', '5', '6']}
print("TestDictFields::test_read_long_with_rest: ok")
"###);
    assert_output(&out, r###"TestDictFields::test_read_long_with_rest: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dict_fields__test_read_long_with_rest_no_fieldnames.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dict_fields__test_read_long_with_rest_no_fieldnames() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_read_long_with_rest_no_fieldnames"
# subject = "cpython.test_csv.TestDictFields.test_read_long_with_rest_no_fieldnames"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_read_long_with_rest_no_fieldnames
"""Auto-ported test: TestDictFields::test_read_long_with_rest_no_fieldnames (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
with TemporaryFile('w+', encoding='utf-8') as fileobj:
    fileobj.write('f1,f2\r\n1,2,abc,4,5,6\r\n')
    fileobj.seek(0)
    reader = csv.DictReader(fileobj, restkey='_rest')

    assert reader.fieldnames == ['f1', 'f2']

    assert next(reader) == {'f1': '1', 'f2': '2', '_rest': ['abc', '4', '5', '6']}
print("TestDictFields::test_read_long_with_rest_no_fieldnames: ok")
"###);
    assert_output(&out, r###"TestDictFields::test_read_long_with_rest_no_fieldnames: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dict_fields__test_read_multi.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dict_fields__test_read_multi() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_read_multi"
# subject = "cpython.test_csv.TestDictFields.test_read_multi"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_read_multi
"""Auto-ported test: TestDictFields::test_read_multi (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
sample = ['2147483648,43.0e12,17,abc,def\r\n', '147483648,43.0e2,17,abc,def\r\n', '47483648,43.0,170,abc,def\r\n']
reader = csv.DictReader(sample, fieldnames='i1 float i2 s1 s2'.split())

assert next(reader) == {'i1': '2147483648', 'float': '43.0e12', 'i2': '17', 's1': 'abc', 's2': 'def'}
print("TestDictFields::test_read_multi: ok")
"###);
    assert_output(&out, r###"TestDictFields::test_read_multi: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dict_fields__test_read_semi_sep.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dict_fields__test_read_semi_sep() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_read_semi_sep"
# subject = "cpython.test_csv.TestDictFields.test_read_semi_sep"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_read_semi_sep
"""Auto-ported test: TestDictFields::test_read_semi_sep (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
reader = csv.DictReader(['1;2;abc;4;5;6\r\n'], fieldnames='1 2 3 4 5 6'.split(), delimiter=';')

assert next(reader) == {'1': '1', '2': '2', '3': 'abc', '4': '4', '5': '5', '6': '6'}
print("TestDictFields::test_read_semi_sep: ok")
"###);
    assert_output(&out, r###"TestDictFields::test_read_semi_sep: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dict_fields__test_read_short.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dict_fields__test_read_short() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_read_short"
# subject = "cpython.test_csv.TestDictFields.test_read_short"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_read_short
"""Auto-ported test: TestDictFields::test_read_short (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
with TemporaryFile('w+', encoding='utf-8') as fileobj:
    fileobj.write('1,2,abc,4,5,6\r\n1,2,abc\r\n')
    fileobj.seek(0)
    reader = csv.DictReader(fileobj, fieldnames='1 2 3 4 5 6'.split(), restval='DEFAULT')

    assert next(reader) == {'1': '1', '2': '2', '3': 'abc', '4': '4', '5': '5', '6': '6'}

    assert next(reader) == {'1': '1', '2': '2', '3': 'abc', '4': 'DEFAULT', '5': 'DEFAULT', '6': 'DEFAULT'}
print("TestDictFields::test_read_short: ok")
"###);
    assert_output(&out, r###"TestDictFields::test_read_short: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dict_fields__test_read_with_blanks.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dict_fields__test_read_with_blanks() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_read_with_blanks"
# subject = "cpython.test_csv.TestDictFields.test_read_with_blanks"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_read_with_blanks
"""Auto-ported test: TestDictFields::test_read_with_blanks (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
reader = csv.DictReader(['1,2,abc,4,5,6\r\n', '\r\n', '1,2,abc,4,5,6\r\n'], fieldnames='1 2 3 4 5 6'.split())

assert next(reader) == {'1': '1', '2': '2', '3': 'abc', '4': '4', '5': '5', '6': '6'}

assert next(reader) == {'1': '1', '2': '2', '3': 'abc', '4': '4', '5': '5', '6': '6'}
print("TestDictFields::test_read_with_blanks: ok")
"###);
    assert_output(&out, r###"TestDictFields::test_read_with_blanks: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dict_fields__test_write_fields_not_in_fieldnames.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dict_fields__test_write_fields_not_in_fieldnames() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_write_fields_not_in_fieldnames"
# subject = "cpython.test_csv.TestDictFields.test_write_fields_not_in_fieldnames"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_write_fields_not_in_fieldnames
"""Auto-ported test: TestDictFields::test_write_fields_not_in_fieldnames (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
    writer = csv.DictWriter(fileobj, fieldnames=['f1', 'f2', 'f3'])
    try:
        writer.writerow({'f4': 10, 'f2': 'spam', 1: 'abc'})
        raise AssertionError('expected ValueError')
    except ValueError as _aR_e:
        import types as _types_aR
        cx = _types_aR.SimpleNamespace(exception=_aR_e)
    exception = str(cx.exception)

    assert 'fieldnames' in exception

    assert "'f4'" in exception

    assert "'f2'" not in exception

    assert '1' in exception
print("TestDictFields::test_write_fields_not_in_fieldnames: ok")
"###);
    assert_output(&out, r###"TestDictFields::test_write_fields_not_in_fieldnames: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dict_fields__test_write_multiple_dict_rows.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dict_fields__test_write_multiple_dict_rows() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_write_multiple_dict_rows"
# subject = "cpython.test_csv.TestDictFields.test_write_multiple_dict_rows"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_write_multiple_dict_rows
"""Auto-ported test: TestDictFields::test_write_multiple_dict_rows (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
fileobj = StringIO()
writer = csv.DictWriter(fileobj, fieldnames=['f1', 'f2', 'f3'])
writer.writeheader()

assert fileobj.getvalue() == 'f1,f2,f3\r\n'
writer.writerows([{'f1': 1, 'f2': 'abc', 'f3': 'f'}, {'f1': 2, 'f2': 5, 'f3': 'xyz'}])

assert fileobj.getvalue() == 'f1,f2,f3\r\n1,abc,f\r\n2,5,xyz\r\n'
print("TestDictFields::test_write_multiple_dict_rows: ok")
"###);
    assert_output(&out, r###"TestDictFields::test_write_multiple_dict_rows: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dict_fields__test_write_simple_dict.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dict_fields__test_write_simple_dict() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_write_simple_dict"
# subject = "cpython.test_csv.TestDictFields.test_write_simple_dict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_write_simple_dict
"""Auto-ported test: TestDictFields::test_write_simple_dict (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
    writer = csv.DictWriter(fileobj, fieldnames=['f1', 'f2', 'f3'])
    writer.writeheader()
    fileobj.seek(0)

    assert fileobj.readline() == 'f1,f2,f3\r\n'
    writer.writerow({'f1': 10, 'f3': 'abc'})
    fileobj.seek(0)
    fileobj.readline()

    assert fileobj.read() == '10,,abc\r\n'
print("TestDictFields::test_write_simple_dict: ok")
"###);
    assert_output(&out, r###"TestDictFields::test_write_simple_dict: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_dict_fields__test_writeheader_return_value.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_dict_fields__test_writeheader_return_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_dict_fields__test_writeheader_return_value"
# subject = "cpython.test_csv.TestDictFields.test_writeheader_return_value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestDictFields::test_writeheader_return_value
"""Auto-ported test: TestDictFields::test_writeheader_return_value (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
    writer = csv.DictWriter(fileobj, fieldnames=['f1', 'f2', 'f3'])
    writeheader_return_value = writer.writeheader()

    assert writeheader_return_value == 10
print("TestDictFields::test_writeheader_return_value: ok")
"###);
    assert_output(&out, r###"TestDictFields::test_writeheader_return_value: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_escaped_excel__test_read_escape_fieldsep.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_escaped_excel__test_read_escape_fieldsep() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_escaped_excel__test_read_escape_fieldsep"
# subject = "cpython.test_csv.TestEscapedExcel.test_read_escape_fieldsep"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestEscapedExcel::test_read_escape_fieldsep
"""Auto-ported test: TestEscapedExcel::test_read_escape_fieldsep (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = EscapedExcel()

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
readerAssertEqual('abc\\,def\r\n', [['abc,def']])
print("TestEscapedExcel::test_read_escape_fieldsep: ok")
"###);
    assert_output(&out, r###"TestEscapedExcel::test_read_escape_fieldsep: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_quoted_escaped_excel__test_read_escape_fieldsep.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_quoted_escaped_excel__test_read_escape_fieldsep() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_quoted_escaped_excel__test_read_escape_fieldsep"
# subject = "cpython.test_csv.TestQuotedEscapedExcel.test_read_escape_fieldsep"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestQuotedEscapedExcel::test_read_escape_fieldsep
"""Auto-ported test: TestQuotedEscapedExcel::test_read_escape_fieldsep (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = QuotedEscapedExcel()

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
readerAssertEqual('"abc\\,def"\r\n', [['abc,def']])
print("TestQuotedEscapedExcel::test_read_escape_fieldsep: ok")
"###);
    assert_output(&out, r###"TestQuotedEscapedExcel::test_read_escape_fieldsep: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_quoted_escaped_excel__test_write_escape_fieldsep.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_quoted_escaped_excel__test_write_escape_fieldsep() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_quoted_escaped_excel__test_write_escape_fieldsep"
# subject = "cpython.test_csv.TestQuotedEscapedExcel.test_write_escape_fieldsep"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestQuotedEscapedExcel::test_write_escape_fieldsep
"""Auto-ported test: TestQuotedEscapedExcel::test_write_escape_fieldsep (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
dialect = QuotedEscapedExcel()

def readerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        fileobj.write(input)
        fileobj.seek(0)
        reader = csv.reader(fileobj, dialect=dialect)
        fields = list(reader)

        assert fields == expected_result

def writerAssertEqual(input, expected_result):
    with TemporaryFile('w+', encoding='utf-8', newline='') as fileobj:
        writer = csv.writer(fileobj, dialect=dialect)
        writer.writerows(input)
        fileobj.seek(0)

        assert fileobj.read() == expected_result
writerAssertEqual([['abc,def']], '"abc,def"\r\n')
print("TestQuotedEscapedExcel::test_write_escape_fieldsep: ok")
"###);
    assert_output(&out, r###"TestQuotedEscapedExcel::test_write_escape_fieldsep: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_sniffer__test_delimiters.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_sniffer__test_delimiters() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_sniffer__test_delimiters"
# subject = "cpython.test_csv.TestSniffer.test_delimiters"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestSniffer::test_delimiters
"""Auto-ported test: TestSniffer::test_delimiters (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
sample1 = "Harry's, Arlington Heights, IL, 2/1/03, Kimi Hayes\nShark City, Glendale Heights, IL, 12/28/02, Prezence\nTommy's Place, Blue Island, IL, 12/28/02, Blue Sunday/White Crow\nStonecutters Seafood and Chop House, Lemont, IL, 12/19/02, Week Back\n"
sample2 = "'Harry''s':'Arlington Heights':'IL':'2/1/03':'Kimi Hayes'\n'Shark City':'Glendale Heights':'IL':'12/28/02':'Prezence'\n'Tommy''s Place':'Blue Island':'IL':'12/28/02':'Blue Sunday/White Crow'\n'Stonecutters ''Seafood'' and Chop House':'Lemont':'IL':'12/19/02':'Week Back'\n"
header1 = '"venue","city","state","date","performers"\n'
sample3 = '05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n'
sample4 = '2147483648;43.0e12;17;abc;def\n147483648;43.0e2;17;abc;def\n47483648;43.0;170;abc;def\n'
sample5 = 'aaa\tbbb\r\nAAA\t\r\nBBB\t\r\n'
sample6 = 'a|b|c\r\nd|e|f\r\n'
sample7 = "'a'|'b'|'c'\r\n'd'|e|f\r\n"
header2 = '"venue"+"city"+"state"+"date"+"performers"\n'
sample8 = "Harry's+ Arlington Heights+ IL+ 2/1/03+ Kimi Hayes\nShark City+ Glendale Heights+ IL+ 12/28/02+ Prezence\nTommy's Place+ Blue Island+ IL+ 12/28/02+ Blue Sunday/White Crow\nStonecutters Seafood and Chop House+ Lemont+ IL+ 12/19/02+ Week Back\n"
sample9 = "'Harry''s'+ Arlington Heights'+ 'IL'+ '2/1/03'+ 'Kimi Hayes'\n'Shark City'+ Glendale Heights'+' IL'+ '12/28/02'+ 'Prezence'\n'Tommy''s Place'+ Blue Island'+ 'IL'+ '12/28/02'+ 'Blue Sunday/White Crow'\n'Stonecutters ''Seafood'' and Chop House'+ 'Lemont'+ 'IL'+ '12/19/02'+ 'Week Back'\n"
sample10 = dedent('\n                        abc,def\n                        ghijkl,mno\n                        ghi,jkl\n                        ')
sample11 = dedent('\n                        abc,def\n                        ghijkl,mnop\n                        ghi,jkl\n                         ')
sample12 = dedent('"time","forces"\n                        1,1.5\n                        0.5,5+0j\n                        0,0\n                        1+1j,6\n                        ')
sample13 = dedent('"time","forces"\n                        0,0\n                        1,2\n                        a,b\n                        ')
sample14 = 'abc\x00def\nghijkl\x00mno\nghi\x00jkl\n'
sniffer = csv.Sniffer()
dialect = sniffer.sniff(sample3)

assert dialect.delimiter in sample3
dialect = sniffer.sniff(sample3, delimiters='?,')

assert dialect.delimiter == '?'
dialect = sniffer.sniff(sample3, delimiters='/,')

assert dialect.delimiter == '/'
dialect = sniffer.sniff(sample4)

assert dialect.delimiter == ';'
dialect = sniffer.sniff(sample5)

assert dialect.delimiter == '\t'
dialect = sniffer.sniff(sample6)

assert dialect.delimiter == '|'
dialect = sniffer.sniff(sample7)

assert dialect.delimiter == '|'

assert dialect.quotechar == "'"
dialect = sniffer.sniff(sample8)

assert dialect.delimiter == '+'
dialect = sniffer.sniff(sample9)

assert dialect.delimiter == '+'

assert dialect.quotechar == "'"
dialect = sniffer.sniff(sample14)

assert dialect.delimiter == '\x00'
print("TestSniffer::test_delimiters: ok")
"###);
    assert_output(&out, r###"TestSniffer::test_delimiters: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_sniffer__test_doublequote.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_sniffer__test_doublequote() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_sniffer__test_doublequote"
# subject = "cpython.test_csv.TestSniffer.test_doublequote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestSniffer::test_doublequote
"""Auto-ported test: TestSniffer::test_doublequote (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
sample1 = "Harry's, Arlington Heights, IL, 2/1/03, Kimi Hayes\nShark City, Glendale Heights, IL, 12/28/02, Prezence\nTommy's Place, Blue Island, IL, 12/28/02, Blue Sunday/White Crow\nStonecutters Seafood and Chop House, Lemont, IL, 12/19/02, Week Back\n"
sample2 = "'Harry''s':'Arlington Heights':'IL':'2/1/03':'Kimi Hayes'\n'Shark City':'Glendale Heights':'IL':'12/28/02':'Prezence'\n'Tommy''s Place':'Blue Island':'IL':'12/28/02':'Blue Sunday/White Crow'\n'Stonecutters ''Seafood'' and Chop House':'Lemont':'IL':'12/19/02':'Week Back'\n"
header1 = '"venue","city","state","date","performers"\n'
sample3 = '05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n'
sample4 = '2147483648;43.0e12;17;abc;def\n147483648;43.0e2;17;abc;def\n47483648;43.0;170;abc;def\n'
sample5 = 'aaa\tbbb\r\nAAA\t\r\nBBB\t\r\n'
sample6 = 'a|b|c\r\nd|e|f\r\n'
sample7 = "'a'|'b'|'c'\r\n'd'|e|f\r\n"
header2 = '"venue"+"city"+"state"+"date"+"performers"\n'
sample8 = "Harry's+ Arlington Heights+ IL+ 2/1/03+ Kimi Hayes\nShark City+ Glendale Heights+ IL+ 12/28/02+ Prezence\nTommy's Place+ Blue Island+ IL+ 12/28/02+ Blue Sunday/White Crow\nStonecutters Seafood and Chop House+ Lemont+ IL+ 12/19/02+ Week Back\n"
sample9 = "'Harry''s'+ Arlington Heights'+ 'IL'+ '2/1/03'+ 'Kimi Hayes'\n'Shark City'+ Glendale Heights'+' IL'+ '12/28/02'+ 'Prezence'\n'Tommy''s Place'+ Blue Island'+ 'IL'+ '12/28/02'+ 'Blue Sunday/White Crow'\n'Stonecutters ''Seafood'' and Chop House'+ 'Lemont'+ 'IL'+ '12/19/02'+ 'Week Back'\n"
sample10 = dedent('\n                        abc,def\n                        ghijkl,mno\n                        ghi,jkl\n                        ')
sample11 = dedent('\n                        abc,def\n                        ghijkl,mnop\n                        ghi,jkl\n                         ')
sample12 = dedent('"time","forces"\n                        1,1.5\n                        0.5,5+0j\n                        0,0\n                        1+1j,6\n                        ')
sample13 = dedent('"time","forces"\n                        0,0\n                        1,2\n                        a,b\n                        ')
sample14 = 'abc\x00def\nghijkl\x00mno\nghi\x00jkl\n'
sniffer = csv.Sniffer()
dialect = sniffer.sniff(header1)

assert not dialect.doublequote
dialect = sniffer.sniff(header2)

assert not dialect.doublequote
dialect = sniffer.sniff(sample2)

assert dialect.doublequote
dialect = sniffer.sniff(sample8)

assert not dialect.doublequote
dialect = sniffer.sniff(sample9)

assert dialect.doublequote
print("TestSniffer::test_doublequote: ok")
"###);
    assert_output(&out, r###"TestSniffer::test_doublequote: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_sniffer__test_guess_quote_and_delimiter.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_sniffer__test_guess_quote_and_delimiter() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_sniffer__test_guess_quote_and_delimiter"
# subject = "cpython.test_csv.TestSniffer.test_guess_quote_and_delimiter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestSniffer::test_guess_quote_and_delimiter
"""Auto-ported test: TestSniffer::test_guess_quote_and_delimiter (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
sample1 = "Harry's, Arlington Heights, IL, 2/1/03, Kimi Hayes\nShark City, Glendale Heights, IL, 12/28/02, Prezence\nTommy's Place, Blue Island, IL, 12/28/02, Blue Sunday/White Crow\nStonecutters Seafood and Chop House, Lemont, IL, 12/19/02, Week Back\n"
sample2 = "'Harry''s':'Arlington Heights':'IL':'2/1/03':'Kimi Hayes'\n'Shark City':'Glendale Heights':'IL':'12/28/02':'Prezence'\n'Tommy''s Place':'Blue Island':'IL':'12/28/02':'Blue Sunday/White Crow'\n'Stonecutters ''Seafood'' and Chop House':'Lemont':'IL':'12/19/02':'Week Back'\n"
header1 = '"venue","city","state","date","performers"\n'
sample3 = '05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n'
sample4 = '2147483648;43.0e12;17;abc;def\n147483648;43.0e2;17;abc;def\n47483648;43.0;170;abc;def\n'
sample5 = 'aaa\tbbb\r\nAAA\t\r\nBBB\t\r\n'
sample6 = 'a|b|c\r\nd|e|f\r\n'
sample7 = "'a'|'b'|'c'\r\n'd'|e|f\r\n"
header2 = '"venue"+"city"+"state"+"date"+"performers"\n'
sample8 = "Harry's+ Arlington Heights+ IL+ 2/1/03+ Kimi Hayes\nShark City+ Glendale Heights+ IL+ 12/28/02+ Prezence\nTommy's Place+ Blue Island+ IL+ 12/28/02+ Blue Sunday/White Crow\nStonecutters Seafood and Chop House+ Lemont+ IL+ 12/19/02+ Week Back\n"
sample9 = "'Harry''s'+ Arlington Heights'+ 'IL'+ '2/1/03'+ 'Kimi Hayes'\n'Shark City'+ Glendale Heights'+' IL'+ '12/28/02'+ 'Prezence'\n'Tommy''s Place'+ Blue Island'+ 'IL'+ '12/28/02'+ 'Blue Sunday/White Crow'\n'Stonecutters ''Seafood'' and Chop House'+ 'Lemont'+ 'IL'+ '12/19/02'+ 'Week Back'\n"
sample10 = dedent('\n                        abc,def\n                        ghijkl,mno\n                        ghi,jkl\n                        ')
sample11 = dedent('\n                        abc,def\n                        ghijkl,mnop\n                        ghi,jkl\n                         ')
sample12 = dedent('"time","forces"\n                        1,1.5\n                        0.5,5+0j\n                        0,0\n                        1+1j,6\n                        ')
sample13 = dedent('"time","forces"\n                        0,0\n                        1,2\n                        a,b\n                        ')
sample14 = 'abc\x00def\nghijkl\x00mno\nghi\x00jkl\n'
sniffer = csv.Sniffer()
for header in (";'123;4';", "'123;4';", ";'123;4'", "'123;4'"):
    dialect = sniffer.sniff(header, ',;')

    assert dialect.delimiter == ';'

    assert dialect.quotechar == "'"

    assert dialect.doublequote is False

    assert dialect.skipinitialspace is False
print("TestSniffer::test_guess_quote_and_delimiter: ok")
"###);
    assert_output(&out, r###"TestSniffer::test_guess_quote_and_delimiter: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_sniffer__test_has_header.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_sniffer__test_has_header() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_sniffer__test_has_header"
# subject = "cpython.test_csv.TestSniffer.test_has_header"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestSniffer::test_has_header
"""Auto-ported test: TestSniffer::test_has_header (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
sample1 = "Harry's, Arlington Heights, IL, 2/1/03, Kimi Hayes\nShark City, Glendale Heights, IL, 12/28/02, Prezence\nTommy's Place, Blue Island, IL, 12/28/02, Blue Sunday/White Crow\nStonecutters Seafood and Chop House, Lemont, IL, 12/19/02, Week Back\n"
sample2 = "'Harry''s':'Arlington Heights':'IL':'2/1/03':'Kimi Hayes'\n'Shark City':'Glendale Heights':'IL':'12/28/02':'Prezence'\n'Tommy''s Place':'Blue Island':'IL':'12/28/02':'Blue Sunday/White Crow'\n'Stonecutters ''Seafood'' and Chop House':'Lemont':'IL':'12/19/02':'Week Back'\n"
header1 = '"venue","city","state","date","performers"\n'
sample3 = '05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n'
sample4 = '2147483648;43.0e12;17;abc;def\n147483648;43.0e2;17;abc;def\n47483648;43.0;170;abc;def\n'
sample5 = 'aaa\tbbb\r\nAAA\t\r\nBBB\t\r\n'
sample6 = 'a|b|c\r\nd|e|f\r\n'
sample7 = "'a'|'b'|'c'\r\n'd'|e|f\r\n"
header2 = '"venue"+"city"+"state"+"date"+"performers"\n'
sample8 = "Harry's+ Arlington Heights+ IL+ 2/1/03+ Kimi Hayes\nShark City+ Glendale Heights+ IL+ 12/28/02+ Prezence\nTommy's Place+ Blue Island+ IL+ 12/28/02+ Blue Sunday/White Crow\nStonecutters Seafood and Chop House+ Lemont+ IL+ 12/19/02+ Week Back\n"
sample9 = "'Harry''s'+ Arlington Heights'+ 'IL'+ '2/1/03'+ 'Kimi Hayes'\n'Shark City'+ Glendale Heights'+' IL'+ '12/28/02'+ 'Prezence'\n'Tommy''s Place'+ Blue Island'+ 'IL'+ '12/28/02'+ 'Blue Sunday/White Crow'\n'Stonecutters ''Seafood'' and Chop House'+ 'Lemont'+ 'IL'+ '12/19/02'+ 'Week Back'\n"
sample10 = dedent('\n                        abc,def\n                        ghijkl,mno\n                        ghi,jkl\n                        ')
sample11 = dedent('\n                        abc,def\n                        ghijkl,mnop\n                        ghi,jkl\n                         ')
sample12 = dedent('"time","forces"\n                        1,1.5\n                        0.5,5+0j\n                        0,0\n                        1+1j,6\n                        ')
sample13 = dedent('"time","forces"\n                        0,0\n                        1,2\n                        a,b\n                        ')
sample14 = 'abc\x00def\nghijkl\x00mno\nghi\x00jkl\n'
sniffer = csv.Sniffer()

assert sniffer.has_header(sample1) is False

assert sniffer.has_header(header1 + sample1) is True
print("TestSniffer::test_has_header: ok")
"###);
    assert_output(&out, r###"TestSniffer::test_has_header: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_sniffer__test_has_header_regex_special_delimiter.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_sniffer__test_has_header_regex_special_delimiter() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_sniffer__test_has_header_regex_special_delimiter"
# subject = "cpython.test_csv.TestSniffer.test_has_header_regex_special_delimiter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestSniffer::test_has_header_regex_special_delimiter
"""Auto-ported test: TestSniffer::test_has_header_regex_special_delimiter (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
sample1 = "Harry's, Arlington Heights, IL, 2/1/03, Kimi Hayes\nShark City, Glendale Heights, IL, 12/28/02, Prezence\nTommy's Place, Blue Island, IL, 12/28/02, Blue Sunday/White Crow\nStonecutters Seafood and Chop House, Lemont, IL, 12/19/02, Week Back\n"
sample2 = "'Harry''s':'Arlington Heights':'IL':'2/1/03':'Kimi Hayes'\n'Shark City':'Glendale Heights':'IL':'12/28/02':'Prezence'\n'Tommy''s Place':'Blue Island':'IL':'12/28/02':'Blue Sunday/White Crow'\n'Stonecutters ''Seafood'' and Chop House':'Lemont':'IL':'12/19/02':'Week Back'\n"
header1 = '"venue","city","state","date","performers"\n'
sample3 = '05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n'
sample4 = '2147483648;43.0e12;17;abc;def\n147483648;43.0e2;17;abc;def\n47483648;43.0;170;abc;def\n'
sample5 = 'aaa\tbbb\r\nAAA\t\r\nBBB\t\r\n'
sample6 = 'a|b|c\r\nd|e|f\r\n'
sample7 = "'a'|'b'|'c'\r\n'd'|e|f\r\n"
header2 = '"venue"+"city"+"state"+"date"+"performers"\n'
sample8 = "Harry's+ Arlington Heights+ IL+ 2/1/03+ Kimi Hayes\nShark City+ Glendale Heights+ IL+ 12/28/02+ Prezence\nTommy's Place+ Blue Island+ IL+ 12/28/02+ Blue Sunday/White Crow\nStonecutters Seafood and Chop House+ Lemont+ IL+ 12/19/02+ Week Back\n"
sample9 = "'Harry''s'+ Arlington Heights'+ 'IL'+ '2/1/03'+ 'Kimi Hayes'\n'Shark City'+ Glendale Heights'+' IL'+ '12/28/02'+ 'Prezence'\n'Tommy''s Place'+ Blue Island'+ 'IL'+ '12/28/02'+ 'Blue Sunday/White Crow'\n'Stonecutters ''Seafood'' and Chop House'+ 'Lemont'+ 'IL'+ '12/19/02'+ 'Week Back'\n"
sample10 = dedent('\n                        abc,def\n                        ghijkl,mno\n                        ghi,jkl\n                        ')
sample11 = dedent('\n                        abc,def\n                        ghijkl,mnop\n                        ghi,jkl\n                         ')
sample12 = dedent('"time","forces"\n                        1,1.5\n                        0.5,5+0j\n                        0,0\n                        1+1j,6\n                        ')
sample13 = dedent('"time","forces"\n                        0,0\n                        1,2\n                        a,b\n                        ')
sample14 = 'abc\x00def\nghijkl\x00mno\nghi\x00jkl\n'
sniffer = csv.Sniffer()

assert sniffer.has_header(sample8) is False

assert sniffer.has_header(header2 + sample8) is True
print("TestSniffer::test_has_header_regex_special_delimiter: ok")
"###);
    assert_output(&out, r###"TestSniffer::test_has_header_regex_special_delimiter: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_sniffer__test_has_header_strings.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_sniffer__test_has_header_strings() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_sniffer__test_has_header_strings"
# subject = "cpython.test_csv.TestSniffer.test_has_header_strings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestSniffer::test_has_header_strings
"""Auto-ported test: TestSniffer::test_has_header_strings (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
sample1 = "Harry's, Arlington Heights, IL, 2/1/03, Kimi Hayes\nShark City, Glendale Heights, IL, 12/28/02, Prezence\nTommy's Place, Blue Island, IL, 12/28/02, Blue Sunday/White Crow\nStonecutters Seafood and Chop House, Lemont, IL, 12/19/02, Week Back\n"
sample2 = "'Harry''s':'Arlington Heights':'IL':'2/1/03':'Kimi Hayes'\n'Shark City':'Glendale Heights':'IL':'12/28/02':'Prezence'\n'Tommy''s Place':'Blue Island':'IL':'12/28/02':'Blue Sunday/White Crow'\n'Stonecutters ''Seafood'' and Chop House':'Lemont':'IL':'12/19/02':'Week Back'\n"
header1 = '"venue","city","state","date","performers"\n'
sample3 = '05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n'
sample4 = '2147483648;43.0e12;17;abc;def\n147483648;43.0e2;17;abc;def\n47483648;43.0;170;abc;def\n'
sample5 = 'aaa\tbbb\r\nAAA\t\r\nBBB\t\r\n'
sample6 = 'a|b|c\r\nd|e|f\r\n'
sample7 = "'a'|'b'|'c'\r\n'd'|e|f\r\n"
header2 = '"venue"+"city"+"state"+"date"+"performers"\n'
sample8 = "Harry's+ Arlington Heights+ IL+ 2/1/03+ Kimi Hayes\nShark City+ Glendale Heights+ IL+ 12/28/02+ Prezence\nTommy's Place+ Blue Island+ IL+ 12/28/02+ Blue Sunday/White Crow\nStonecutters Seafood and Chop House+ Lemont+ IL+ 12/19/02+ Week Back\n"
sample9 = "'Harry''s'+ Arlington Heights'+ 'IL'+ '2/1/03'+ 'Kimi Hayes'\n'Shark City'+ Glendale Heights'+' IL'+ '12/28/02'+ 'Prezence'\n'Tommy''s Place'+ Blue Island'+ 'IL'+ '12/28/02'+ 'Blue Sunday/White Crow'\n'Stonecutters ''Seafood'' and Chop House'+ 'Lemont'+ 'IL'+ '12/19/02'+ 'Week Back'\n"
sample10 = dedent('\n                        abc,def\n                        ghijkl,mno\n                        ghi,jkl\n                        ')
sample11 = dedent('\n                        abc,def\n                        ghijkl,mnop\n                        ghi,jkl\n                         ')
sample12 = dedent('"time","forces"\n                        1,1.5\n                        0.5,5+0j\n                        0,0\n                        1+1j,6\n                        ')
sample13 = dedent('"time","forces"\n                        0,0\n                        1,2\n                        a,b\n                        ')
sample14 = 'abc\x00def\nghijkl\x00mno\nghi\x00jkl\n'
'More to document existing (unexpected?) behavior than anything else.'
sniffer = csv.Sniffer()

assert not sniffer.has_header(sample10)

assert not sniffer.has_header(sample11)
print("TestSniffer::test_has_header_strings: ok")
"###);
    assert_output(&out, r###"TestSniffer::test_has_header_strings: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_sniffer__test_issue43625.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_sniffer__test_issue43625() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_sniffer__test_issue43625"
# subject = "cpython.test_csv.TestSniffer.test_issue43625"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestSniffer::test_issue43625
"""Auto-ported test: TestSniffer::test_issue43625 (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
sample1 = "Harry's, Arlington Heights, IL, 2/1/03, Kimi Hayes\nShark City, Glendale Heights, IL, 12/28/02, Prezence\nTommy's Place, Blue Island, IL, 12/28/02, Blue Sunday/White Crow\nStonecutters Seafood and Chop House, Lemont, IL, 12/19/02, Week Back\n"
sample2 = "'Harry''s':'Arlington Heights':'IL':'2/1/03':'Kimi Hayes'\n'Shark City':'Glendale Heights':'IL':'12/28/02':'Prezence'\n'Tommy''s Place':'Blue Island':'IL':'12/28/02':'Blue Sunday/White Crow'\n'Stonecutters ''Seafood'' and Chop House':'Lemont':'IL':'12/19/02':'Week Back'\n"
header1 = '"venue","city","state","date","performers"\n'
sample3 = '05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n'
sample4 = '2147483648;43.0e12;17;abc;def\n147483648;43.0e2;17;abc;def\n47483648;43.0;170;abc;def\n'
sample5 = 'aaa\tbbb\r\nAAA\t\r\nBBB\t\r\n'
sample6 = 'a|b|c\r\nd|e|f\r\n'
sample7 = "'a'|'b'|'c'\r\n'd'|e|f\r\n"
header2 = '"venue"+"city"+"state"+"date"+"performers"\n'
sample8 = "Harry's+ Arlington Heights+ IL+ 2/1/03+ Kimi Hayes\nShark City+ Glendale Heights+ IL+ 12/28/02+ Prezence\nTommy's Place+ Blue Island+ IL+ 12/28/02+ Blue Sunday/White Crow\nStonecutters Seafood and Chop House+ Lemont+ IL+ 12/19/02+ Week Back\n"
sample9 = "'Harry''s'+ Arlington Heights'+ 'IL'+ '2/1/03'+ 'Kimi Hayes'\n'Shark City'+ Glendale Heights'+' IL'+ '12/28/02'+ 'Prezence'\n'Tommy''s Place'+ Blue Island'+ 'IL'+ '12/28/02'+ 'Blue Sunday/White Crow'\n'Stonecutters ''Seafood'' and Chop House'+ 'Lemont'+ 'IL'+ '12/19/02'+ 'Week Back'\n"
sample10 = dedent('\n                        abc,def\n                        ghijkl,mno\n                        ghi,jkl\n                        ')
sample11 = dedent('\n                        abc,def\n                        ghijkl,mnop\n                        ghi,jkl\n                         ')
sample12 = dedent('"time","forces"\n                        1,1.5\n                        0.5,5+0j\n                        0,0\n                        1+1j,6\n                        ')
sample13 = dedent('"time","forces"\n                        0,0\n                        1,2\n                        a,b\n                        ')
sample14 = 'abc\x00def\nghijkl\x00mno\nghi\x00jkl\n'
sniffer = csv.Sniffer()

assert sniffer.has_header(sample12)

assert not sniffer.has_header(sample13)
print("TestSniffer::test_issue43625: ok")
"###);
    assert_output(&out, r###"TestSniffer::test_issue43625: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_sniffer__test_sniff.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_sniffer__test_sniff() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_sniffer__test_sniff"
# subject = "cpython.test_csv.TestSniffer.test_sniff"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestSniffer::test_sniff
"""Auto-ported test: TestSniffer::test_sniff (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
sample1 = "Harry's, Arlington Heights, IL, 2/1/03, Kimi Hayes\nShark City, Glendale Heights, IL, 12/28/02, Prezence\nTommy's Place, Blue Island, IL, 12/28/02, Blue Sunday/White Crow\nStonecutters Seafood and Chop House, Lemont, IL, 12/19/02, Week Back\n"
sample2 = "'Harry''s':'Arlington Heights':'IL':'2/1/03':'Kimi Hayes'\n'Shark City':'Glendale Heights':'IL':'12/28/02':'Prezence'\n'Tommy''s Place':'Blue Island':'IL':'12/28/02':'Blue Sunday/White Crow'\n'Stonecutters ''Seafood'' and Chop House':'Lemont':'IL':'12/19/02':'Week Back'\n"
header1 = '"venue","city","state","date","performers"\n'
sample3 = '05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n05/05/03?05/05/03?05/05/03?05/05/03?05/05/03?05/05/03\n'
sample4 = '2147483648;43.0e12;17;abc;def\n147483648;43.0e2;17;abc;def\n47483648;43.0;170;abc;def\n'
sample5 = 'aaa\tbbb\r\nAAA\t\r\nBBB\t\r\n'
sample6 = 'a|b|c\r\nd|e|f\r\n'
sample7 = "'a'|'b'|'c'\r\n'd'|e|f\r\n"
header2 = '"venue"+"city"+"state"+"date"+"performers"\n'
sample8 = "Harry's+ Arlington Heights+ IL+ 2/1/03+ Kimi Hayes\nShark City+ Glendale Heights+ IL+ 12/28/02+ Prezence\nTommy's Place+ Blue Island+ IL+ 12/28/02+ Blue Sunday/White Crow\nStonecutters Seafood and Chop House+ Lemont+ IL+ 12/19/02+ Week Back\n"
sample9 = "'Harry''s'+ Arlington Heights'+ 'IL'+ '2/1/03'+ 'Kimi Hayes'\n'Shark City'+ Glendale Heights'+' IL'+ '12/28/02'+ 'Prezence'\n'Tommy''s Place'+ Blue Island'+ 'IL'+ '12/28/02'+ 'Blue Sunday/White Crow'\n'Stonecutters ''Seafood'' and Chop House'+ 'Lemont'+ 'IL'+ '12/19/02'+ 'Week Back'\n"
sample10 = dedent('\n                        abc,def\n                        ghijkl,mno\n                        ghi,jkl\n                        ')
sample11 = dedent('\n                        abc,def\n                        ghijkl,mnop\n                        ghi,jkl\n                         ')
sample12 = dedent('"time","forces"\n                        1,1.5\n                        0.5,5+0j\n                        0,0\n                        1+1j,6\n                        ')
sample13 = dedent('"time","forces"\n                        0,0\n                        1,2\n                        a,b\n                        ')
sample14 = 'abc\x00def\nghijkl\x00mno\nghi\x00jkl\n'
sniffer = csv.Sniffer()
dialect = sniffer.sniff(sample1)

assert dialect.delimiter == ','

assert dialect.quotechar == '"'

assert dialect.skipinitialspace is True
dialect = sniffer.sniff(sample2)

assert dialect.delimiter == ':'

assert dialect.quotechar == "'"

assert dialect.skipinitialspace is False
print("TestSniffer::test_sniff: ok")
"###);
    assert_output(&out, r###"TestSniffer::test_sniff: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_unicode__test_unicode_read.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_unicode__test_unicode_read() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_unicode__test_unicode_read"
# subject = "cpython.test_csv.TestUnicode.test_unicode_read"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestUnicode::test_unicode_read
"""Auto-ported test: TestUnicode::test_unicode_read (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
names = ['Martin von Löwis', 'Marc André Lemburg', 'Guido van Rossum', 'François Pinard']
with TemporaryFile('w+', newline='', encoding='utf-8') as fileobj:
    fileobj.write(','.join(names) + '\r\n')
    fileobj.seek(0)
    reader = csv.reader(fileobj)

    assert list(reader) == [names]
print("TestUnicode::test_unicode_read: ok")
"###);
    assert_output(&out, r###"TestUnicode::test_unicode_read: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/test_unicode__test_unicode_write.py`.
#[test]
fn test_gen_behavior_std_libs_csv_test_unicode__test_unicode_write() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "test_unicode__test_unicode_write"
# subject = "cpython.test_csv.TestUnicode.test_unicode_write"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_csv.py::TestUnicode::test_unicode_write
"""Auto-ported test: TestUnicode::test_unicode_write (CPython 3.12 oracle)."""


import copy
import sys
import unittest
from io import StringIO
from tempfile import TemporaryFile
import csv
import gc
import pickle
from test import support
from test.support import warnings_helper, import_helper, check_disallow_instantiation
from itertools import permutations
from textwrap import dedent
from collections import OrderedDict


class BadIterable:

    def __iter__(self):
        raise OSError

class EscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONE
    escapechar = '\\'

class QuotedEscapedExcel(csv.excel):
    quoting = csv.QUOTE_NONNUMERIC
    escapechar = '\\'

class NUL:

    def write(s, *args):
        pass
    writelines = write


# --- test body ---
names = ['Martin von Löwis', 'Marc André Lemburg', 'Guido van Rossum', 'François Pinard']
with TemporaryFile('w+', newline='', encoding='utf-8') as fileobj:
    writer = csv.writer(fileobj)
    writer.writerow(names)
    expected = ','.join(names) + '\r\n'
    fileobj.seek(0)

    assert fileobj.read() == expected
print("TestUnicode::test_unicode_write: ok")
"###);
    assert_output(&out, r###"TestUnicode::test_unicode_write: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/writer_accepts_any_value_iterable.py`.
#[test]
fn test_gen_behavior_std_libs_csv_writer_accepts_any_value_iterable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "writer_accepts_any_value_iterable"
# subject = "csv.writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.writer: writerow consumes an arbitrary iterable (e.g. a generator) and stringifies each value"""
import csv
import io

buf = io.StringIO()
csv.writer(buf).writerow((i * i for i in range(4)))
assert buf.getvalue() == "0,1,4,9\r\n", f"generator row = {buf.getvalue()!r}"

print("writer_accepts_any_value_iterable OK")
"###);
    assert_output(&out, r###"writer_accepts_any_value_iterable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/writer_none_fields_render_empty.py`.
#[test]
fn test_gen_behavior_std_libs_csv_writer_none_fields_render_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "writer_none_fields_render_empty"
# subject = "csv.writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.writer: None fields render as empty; a lone None field becomes a quoted empty string"""
import csv
import io

buf = io.StringIO()
csv.writer(buf).writerows([["a", None], [None, "d"]])
assert buf.getvalue() == "a,\r\n,d\r\n", f"none mix = {buf.getvalue()!r}"

buf2 = io.StringIO()
csv.writer(buf2).writerows([[None], ["a"]])
assert buf2.getvalue() == '""\r\na\r\n', f"lone none = {buf2.getvalue()!r}"

print("writer_none_fields_render_empty OK")
"###);
    assert_output(&out, r###"writer_none_fields_render_empty OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/writer_stringifies_each_value.py`.
#[test]
fn test_gen_behavior_std_libs_csv_writer_stringifies_each_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "writer_stringifies_each_value"
# subject = "csv.writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.writer: writerow renders int/float/bool/None via str(): None becomes empty, others their str() form"""
import csv
import io

buf = io.StringIO()
csv.writer(buf).writerow([1, 2.5, True, None])
out = buf.getvalue().strip()
assert out == "1,2.5,True,", f"number writing = {out!r}"

print("writer_stringifies_each_value OK")
"###);
    assert_output(&out, r###"writer_stringifies_each_value OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/csv/writerows_emits_each_row.py`.
#[test]
fn test_gen_behavior_std_libs_csv_writerows_emits_each_row() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "csv"
# dimension = "behavior"
# case = "writerows_emits_each_row"
# subject = "csv.writer"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_csv.py"
# status = "filled"
# ///
"""csv.writer: writerows emits one parsed row per input sequence in order"""
import csv
import io

buf = io.StringIO()
csv.writer(buf).writerows([["a", "b"], ["c", "d"], ["e", "f"]])
buf.seek(0)
rows = list(csv.reader(buf))
assert len(rows) == 3, f"writerows count = {len(rows)!r}"
assert rows[2] == ["e", "f"], f"row 2 = {rows[2]!r}"

print("writerows_emits_each_row OK")
"###);
    assert_output(&out, r###"writerows_emits_each_row OK
"###);
}
