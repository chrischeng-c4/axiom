use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/core/param_types/chr_rejects_str_argument.py`.
#[test]
fn test_gen_type_core_param_types_chr_rejects_str_argument() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "chr_rejects_str_argument"
# subject = "chr"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""chr: chr_rejects_str_argument (errors)."""
try:
    result = chr("65")
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/param_types/configparser_read_string_rejects_int_argument.py`.
#[test]
fn test_gen_type_core_param_types_configparser_read_string_rejects_int_argument() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "configparser_read_string_rejects_int_argument"
# subject = "configparser.ConfigParser.read_string"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""configparser.ConfigParser.read_string: configparser_read_string_rejects_int_argument (errors)."""
import configparser

try:
    result = configparser.ConfigParser().read_string(1)
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/param_types/csv_reader_rejects_non_iterable_argument.py`.
#[test]
fn test_gen_type_core_param_types_csv_reader_rejects_non_iterable_argument() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "csv_reader_rejects_non_iterable_argument"
# subject = "csv.reader"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""csv.reader: csv_reader_rejects_non_iterable_argument (errors)."""
import csv

try:
    result = csv.reader(1)
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/param_types/datetime_date_rejects_str_year_argument.py`.
#[test]
fn test_gen_type_core_param_types_datetime_date_rejects_str_year_argument() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "datetime_date_rejects_str_year_argument"
# subject = "datetime.date"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""datetime.date: datetime_date_rejects_str_year_argument (errors)."""
import datetime

try:
    result = datetime.date("2024", 1, 1)
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/param_types/format_rejects_int_spec_argument.py`.
#[test]
fn test_gen_type_core_param_types_format_rejects_int_spec_argument() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "format_rejects_int_spec_argument"
# subject = "format"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""format: format_rejects_int_spec_argument (errors)."""
try:
    result = format(1, 2)
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/param_types/getattr_rejects_int_name_argument.py`.
#[test]
fn test_gen_type_core_param_types_getattr_rejects_int_name_argument() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "getattr_rejects_int_name_argument"
# subject = "getattr"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""getattr: getattr_rejects_int_name_argument (errors)."""
try:
    result = getattr(1, 2)
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/param_types/glob_escape_rejects_int_argument.py`.
#[test]
fn test_gen_type_core_param_types_glob_escape_rejects_int_argument() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "glob_escape_rejects_int_argument"
# subject = "glob.escape"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""glob.escape: glob_escape_rejects_int_argument (errors)."""
import glob

try:
    result = glob.escape(1)
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/param_types/hasattr_rejects_int_name_argument.py`.
#[test]
fn test_gen_type_core_param_types_hasattr_rejects_int_name_argument() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "hasattr_rejects_int_name_argument"
# subject = "hasattr"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""hasattr: hasattr_rejects_int_name_argument (errors)."""
try:
    result = hasattr(1, 2)
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/param_types/heapq_heappush_rejects_int_heap_argument.py`.
#[test]
fn test_gen_type_core_param_types_heapq_heappush_rejects_int_heap_argument() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "heapq_heappush_rejects_int_heap_argument"
# subject = "heapq.heappush"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""heapq.heappush: heapq_heappush_rejects_int_heap_argument (errors)."""
import heapq

try:
    result = heapq.heappush(1, 2)
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/param_types/hmac_new_rejects_str_key_argument.py`.
#[test]
fn test_gen_type_core_param_types_hmac_new_rejects_str_key_argument() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "hmac_new_rejects_str_key_argument"
# subject = "hmac.new"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""hmac.new: hmac_new_rejects_str_key_argument (errors)."""
import hmac

try:
    result = hmac.new("key", b"msg")
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/param_types/int_rejects_str_base_argument.py`.
#[test]
fn test_gen_type_core_param_types_int_rejects_str_base_argument() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "int_rejects_str_base_argument"
# subject = "int"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""int: int_rejects_str_base_argument (errors)."""
try:
    result = int("10", "2")
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/param_types/isinstance_rejects_str_classinfo_argument.py`.
#[test]
fn test_gen_type_core_param_types_isinstance_rejects_str_classinfo_argument() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "isinstance_rejects_str_classinfo_argument"
# subject = "isinstance"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""isinstance: isinstance_rejects_str_classinfo_argument (errors)."""
try:
    result = isinstance(1, "int")
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/param_types/issubclass_rejects_str_classinfo_argument.py`.
#[test]
fn test_gen_type_core_param_types_issubclass_rejects_str_classinfo_argument() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "issubclass_rejects_str_classinfo_argument"
# subject = "issubclass"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""issubclass: issubclass_rejects_str_classinfo_argument (errors)."""
try:
    result = issubclass(int, "object")
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/param_types/ord_rejects_int_argument.py`.
#[test]
fn test_gen_type_core_param_types_ord_rejects_int_argument() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "ord_rejects_int_argument"
# subject = "ord"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""ord: ord_rejects_int_argument (errors)."""
try:
    result = ord(123)
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/param_types/pathlib_path_rejects_int_argument.py`.
#[test]
fn test_gen_type_core_param_types_pathlib_path_rejects_int_argument() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "pathlib_path_rejects_int_argument"
# subject = "pathlib.Path"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""pathlib.Path: pathlib_path_rejects_int_argument (errors)."""
import pathlib

try:
    result = pathlib.Path(1)
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/param_types/setattr_rejects_int_name_argument.py`.
#[test]
fn test_gen_type_core_param_types_setattr_rejects_int_name_argument() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "setattr_rejects_int_name_argument"
# subject = "setattr"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""setattr: setattr_rejects_int_name_argument (errors)."""
try:
    result = setattr(1, 2, 3)
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/param_types/socket_htons_rejects_str_argument.py`.
#[test]
fn test_gen_type_core_param_types_socket_htons_rejects_str_argument() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "socket_htons_rejects_str_argument"
# subject = "socket.htons"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""socket.htons: socket_htons_rejects_str_argument (errors)."""
import socket

try:
    result = socket.htons("80")
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/param_types/sorted_rejects_non_iterable_argument.py`.
#[test]
fn test_gen_type_core_param_types_sorted_rejects_non_iterable_argument() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "sorted_rejects_non_iterable_argument"
# subject = "sorted"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""sorted: sorted_rejects_non_iterable_argument (errors)."""
try:
    result = sorted(1)
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/param_types/statistics_mean_rejects_int_argument.py`.
#[test]
fn test_gen_type_core_param_types_statistics_mean_rejects_int_argument() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "statistics_mean_rejects_int_argument"
# subject = "statistics.mean"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""statistics.mean: statistics_mean_rejects_int_argument (errors)."""
import statistics

try:
    result = statistics.mean(1)
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/param_types/urllib_quote_from_bytes_rejects_str_argument.py`.
#[test]
fn test_gen_type_core_param_types_urllib_quote_from_bytes_rejects_str_argument() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "urllib_quote_from_bytes_rejects_str_argument"
# subject = "urllib.parse.quote_from_bytes"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""urllib.parse.quote_from_bytes: urllib_quote_from_bytes_rejects_str_argument (errors)."""
import urllib.parse

try:
    result = urllib.parse.quote_from_bytes("abc")
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
