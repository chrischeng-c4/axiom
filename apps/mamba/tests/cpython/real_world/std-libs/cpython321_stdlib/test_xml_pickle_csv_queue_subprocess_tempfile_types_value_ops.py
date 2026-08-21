# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_xml_pickle_csv_queue_subprocess_tempfile_types_value_ops"
# subject = "cpython321.test_xml_pickle_csv_queue_subprocess_tempfile_types_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_xml_pickle_csv_queue_subprocess_tempfile_types_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_xml_pickle_csv_queue_subprocess_tempfile_types_value_ops: execute CPython 3.12 seed test_xml_pickle_csv_queue_subprocess_tempfile_types_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 237 pass conformance — xml.etree.ElementTree / pickle / csv /
# urllib.parse (from-import) / queue / subprocess / tempfile / types /
# inspect partial surface + value ops that match between CPython 3.12 and
# mamba.
import xml.etree.ElementTree as ET
import pickle
import csv
import queue
import subprocess
import tempfile
import types
import inspect
import configparser
from urllib.parse import (
    urlparse,
    urljoin,
    quote,
    unquote,
    urlencode,
    parse_qs,
)


_ledger: list[int] = []

# 1) xml.etree.ElementTree surface + basic value ops
assert hasattr(ET, "Element") == True; _ledger.append(1)
assert hasattr(ET, "ElementTree") == True; _ledger.append(1)
assert hasattr(ET, "fromstring") == True; _ledger.append(1)
assert hasattr(ET, "tostring") == True; _ledger.append(1)
assert hasattr(ET, "parse") == True; _ledger.append(1)
assert hasattr(ET, "SubElement") == True; _ledger.append(1)
assert hasattr(ET, "XML") == True; _ledger.append(1)
assert hasattr(ET, "Comment") == True; _ledger.append(1)
assert hasattr(ET, "ParseError") == True; _ledger.append(1)
_root = ET.fromstring("<root><a x='1'>hello</a><b>world</b></root>")
assert _root is not None and _root.tag == "root"; _ledger.append(1)
assert ET.Element("foo").tag == "foo"; _ledger.append(1)

# 2) pickle surface + roundtrip value ops
assert hasattr(pickle, "dumps") == True; _ledger.append(1)
assert hasattr(pickle, "loads") == True; _ledger.append(1)
assert hasattr(pickle, "dump") == True; _ledger.append(1)
assert hasattr(pickle, "load") == True; _ledger.append(1)
assert hasattr(pickle, "Pickler") == True; _ledger.append(1)
assert hasattr(pickle, "Unpickler") == True; _ledger.append(1)
assert hasattr(pickle, "HIGHEST_PROTOCOL") == True; _ledger.append(1)
assert hasattr(pickle, "DEFAULT_PROTOCOL") == True; _ledger.append(1)
assert hasattr(pickle, "PickleError") == True; _ledger.append(1)
assert hasattr(pickle, "PicklingError") == True; _ledger.append(1)
assert hasattr(pickle, "UnpicklingError") == True; _ledger.append(1)
assert pickle.loads(pickle.dumps(42)) == 42; _ledger.append(1)
assert pickle.loads(pickle.dumps("hello")) == "hello"; _ledger.append(1)
assert pickle.loads(pickle.dumps([1, 2, 3])) == [1, 2, 3]; _ledger.append(1)
assert pickle.loads(pickle.dumps({"a": 1})) == {"a": 1}; _ledger.append(1)

# 3) csv surface
assert hasattr(csv, "reader") == True; _ledger.append(1)
assert hasattr(csv, "writer") == True; _ledger.append(1)
assert hasattr(csv, "DictReader") == True; _ledger.append(1)
assert hasattr(csv, "DictWriter") == True; _ledger.append(1)
assert hasattr(csv, "Dialect") == True; _ledger.append(1)
assert hasattr(csv, "excel") == True; _ledger.append(1)
assert hasattr(csv, "excel_tab") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_ALL") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_MINIMAL") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_NONNUMERIC") == True; _ledger.append(1)
assert hasattr(csv, "QUOTE_NONE") == True; _ledger.append(1)
assert hasattr(csv, "Error") == True; _ledger.append(1)

# 4) urllib.parse from-import value ops + class binding (top-level import
#    surface is divergent — see spec fixture)
assert urlparse("https://example.com/a?b=1").scheme == "https"; _ledger.append(1)
assert urljoin("https://example.com/a/", "b") == "https://example.com/a/b"; _ledger.append(1)
assert quote("hello world") == "hello%20world"; _ledger.append(1)
assert unquote("hello%20world") == "hello world"; _ledger.append(1)
assert urlencode({"a": "1", "b": "2"}) == "a=1&b=2"; _ledger.append(1)
assert parse_qs("a=1&b=2") == {"a": ["1"], "b": ["2"]}; _ledger.append(1)

# 5) queue surface
assert hasattr(queue, "Queue") == True; _ledger.append(1)
assert hasattr(queue, "LifoQueue") == True; _ledger.append(1)
assert hasattr(queue, "PriorityQueue") == True; _ledger.append(1)
assert hasattr(queue, "SimpleQueue") == True; _ledger.append(1)
assert hasattr(queue, "Empty") == True; _ledger.append(1)
assert hasattr(queue, "Full") == True; _ledger.append(1)

# 6) subprocess full surface
assert hasattr(subprocess, "run") == True; _ledger.append(1)
assert hasattr(subprocess, "Popen") == True; _ledger.append(1)
assert hasattr(subprocess, "PIPE") == True; _ledger.append(1)
assert hasattr(subprocess, "DEVNULL") == True; _ledger.append(1)
assert hasattr(subprocess, "STDOUT") == True; _ledger.append(1)
assert hasattr(subprocess, "CalledProcessError") == True; _ledger.append(1)
assert hasattr(subprocess, "TimeoutExpired") == True; _ledger.append(1)
assert hasattr(subprocess, "check_output") == True; _ledger.append(1)
assert hasattr(subprocess, "check_call") == True; _ledger.append(1)
assert hasattr(subprocess, "call") == True; _ledger.append(1)
assert hasattr(subprocess, "CompletedProcess") == True; _ledger.append(1)

# 7) tempfile full surface
assert hasattr(tempfile, "TemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "NamedTemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "TemporaryDirectory") == True; _ledger.append(1)
assert hasattr(tempfile, "SpooledTemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "mkstemp") == True; _ledger.append(1)
assert hasattr(tempfile, "mkdtemp") == True; _ledger.append(1)
assert hasattr(tempfile, "gettempdir") == True; _ledger.append(1)
assert hasattr(tempfile, "gettempprefix") == True; _ledger.append(1)

# 8) types full surface
assert hasattr(types, "FunctionType") == True; _ledger.append(1)
assert hasattr(types, "MethodType") == True; _ledger.append(1)
assert hasattr(types, "ModuleType") == True; _ledger.append(1)
assert hasattr(types, "SimpleNamespace") == True; _ledger.append(1)
assert hasattr(types, "MappingProxyType") == True; _ledger.append(1)
assert hasattr(types, "GeneratorType") == True; _ledger.append(1)
assert hasattr(types, "CoroutineType") == True; _ledger.append(1)
assert hasattr(types, "BuiltinFunctionType") == True; _ledger.append(1)
assert hasattr(types, "LambdaType") == True; _ledger.append(1)
assert hasattr(types, "CodeType") == True; _ledger.append(1)
assert hasattr(types, "CellType") == True; _ledger.append(1)
assert hasattr(types, "TracebackType") == True; _ledger.append(1)
assert hasattr(types, "FrameType") == True; _ledger.append(1)

# 9) inspect partial surface
assert hasattr(inspect, "getmembers") == True; _ledger.append(1)
assert hasattr(inspect, "signature") == True; _ledger.append(1)
assert hasattr(inspect, "isclass") == True; _ledger.append(1)
assert hasattr(inspect, "isfunction") == True; _ledger.append(1)
assert hasattr(inspect, "ismethod") == True; _ledger.append(1)

# 10) configparser partial — ConfigParser binding only
assert hasattr(configparser, "ConfigParser") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_xml_pickle_csv_queue_subprocess_tempfile_types_value_ops {sum(_ledger)} asserts")
