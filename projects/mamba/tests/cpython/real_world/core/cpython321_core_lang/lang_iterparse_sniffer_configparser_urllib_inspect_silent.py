# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_iterparse_sniffer_configparser_urllib_inspect_silent"
# subject = "cpython321.lang_iterparse_sniffer_configparser_urllib_inspect_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_iterparse_sniffer_configparser_urllib_inspect_silent.py"
# status = "filled"
# ///
"""cpython321.lang_iterparse_sniffer_configparser_urllib_inspect_silent: execute CPython 3.12 seed lang_iterparse_sniffer_configparser_urllib_inspect_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the
# `xml.etree.ElementTree.iterparse` / `csv.Sniffer` /
# `configparser` surface / `urllib.parse` top-level surface /
# `inspect` deep surface five-pack pinned to atomic 237:
# `xml.etree.ElementTree.iterparse` (the documented top-level
# event-pull parser surface — mamba's `xml.etree.ElementTree`
# module dict does not expose it so `hasattr(...)` collapses
# to False), `csv.Sniffer` (the documented top-level
# dialect-sniffer class — mamba's `csv` module dict does not
# expose it), `configparser.RawConfigParser /
# BasicInterpolation / ExtendedInterpolation /
# DuplicateSectionError / NoSectionError` (the documented
# top-level surface — mamba's `configparser` module dict only
# exposes `ConfigParser` and silently drops every other
# documented name), `urllib.parse.urlparse / urlunparse /
# urljoin / urlencode / parse_qs / parse_qsl / quote /
# unquote / quote_plus / unquote_plus` (the documented
# top-level submodule surface via the dotted-attribute access
# pattern `urllib.parse.X` — mamba's `import urllib.parse`
# binds `urllib` to the parent module whose `.parse` dotted
# resolution silently drops every name so `hasattr(urllib.
# parse, ...)` collapses to False even though
# `from urllib.parse import X` works), and `inspect.Parameter
# / Signature / ismodule / isbuiltin / getsource /
# getsourcelines / getfile / getdoc / stack / currentframe`
# (the documented top-level surface — mamba's `inspect`
# module dict only exposes `getmembers / signature / isclass
# / isfunction / ismethod` and silently drops the rest).
#
# Behavioral edges that CONFORM on mamba (xml.etree.ElementTree
# Element/ElementTree/fromstring/tostring/parse/SubElement/XML/
# Comment/ParseError + fromstring tag + Element ctor; pickle
# dumps/loads/dump/load/Pickler/Unpickler/HIGHEST_PROTOCOL/
# DEFAULT_PROTOCOL/PickleError/PicklingError/UnpicklingError +
# int/str/list/dict roundtrip; csv reader/writer/DictReader/
# DictWriter/Dialect/excel/excel_tab/QUOTE_*/Error; urllib.parse
# from-import urlparse/urljoin/quote/unquote/urlencode/parse_qs
# value ops; queue Queue/LifoQueue/PriorityQueue/SimpleQueue/
# Empty/Full; subprocess run/Popen/PIPE/DEVNULL/STDOUT/
# CalledProcessError/TimeoutExpired/check_output/check_call/
# call/CompletedProcess; tempfile TemporaryFile/
# NamedTemporaryFile/TemporaryDirectory/SpooledTemporaryFile/
# mkstemp/mkdtemp/gettempdir/gettempprefix; types FunctionType/
# MethodType/ModuleType/SimpleNamespace/MappingProxyType/
# GeneratorType/CoroutineType/BuiltinFunctionType/LambdaType/
# CodeType/CellType/TracebackType/FrameType; inspect
# getmembers/signature/isclass/isfunction/ismethod;
# configparser ConfigParser) are covered in the matching
# pass fixture
# `test_xml_pickle_csv_queue_subprocess_tempfile_types_value_ops`.
from typing import Any
import xml.etree.ElementTree as _ET_mod
import csv as _csv_mod
import configparser as _configparser_mod
import urllib.parse as _urllib_parse_mod  # noqa: F401
import urllib as _urllib_mod
import inspect as _inspect_mod

ET_mod: Any = _ET_mod
csv_mod: Any = _csv_mod
configparser_mod: Any = _configparser_mod
urllib_mod: Any = _urllib_mod
inspect_mod: Any = _inspect_mod


_ledger: list[int] = []

# 1) xml.etree.ElementTree.iterparse — top-level event-pull surface
#    (mamba: missing)
assert hasattr(ET_mod, "iterparse") == True; _ledger.append(1)

# 2) csv.Sniffer — top-level dialect-sniffer class
#    (mamba: missing)
assert hasattr(csv_mod, "Sniffer") == True; _ledger.append(1)

# 3) configparser surface
#    (mamba: only ConfigParser is exposed)
assert hasattr(configparser_mod, "RawConfigParser") == True; _ledger.append(1)
assert hasattr(configparser_mod, "BasicInterpolation") == True; _ledger.append(1)
assert hasattr(configparser_mod, "ExtendedInterpolation") == True; _ledger.append(1)
assert hasattr(configparser_mod, "DuplicateSectionError") == True; _ledger.append(1)
assert hasattr(configparser_mod, "NoSectionError") == True; _ledger.append(1)

# 4) urllib.parse dotted-attribute access surface
#    (mamba: `import urllib.parse` then `urllib.parse.X` collapses every name
#    even though `from urllib.parse import X` works)
assert hasattr(urllib_mod.parse, "urlparse") == True; _ledger.append(1)
assert hasattr(urllib_mod.parse, "urlunparse") == True; _ledger.append(1)
assert hasattr(urllib_mod.parse, "urljoin") == True; _ledger.append(1)
assert hasattr(urllib_mod.parse, "urlencode") == True; _ledger.append(1)
assert hasattr(urllib_mod.parse, "parse_qs") == True; _ledger.append(1)
assert hasattr(urllib_mod.parse, "parse_qsl") == True; _ledger.append(1)
assert hasattr(urllib_mod.parse, "quote") == True; _ledger.append(1)
assert hasattr(urllib_mod.parse, "unquote") == True; _ledger.append(1)
assert hasattr(urllib_mod.parse, "quote_plus") == True; _ledger.append(1)
assert hasattr(urllib_mod.parse, "unquote_plus") == True; _ledger.append(1)

# 5) inspect deep surface
#    (mamba: only getmembers/signature/isclass/isfunction/ismethod exposed)
assert hasattr(inspect_mod, "Parameter") == True; _ledger.append(1)
assert hasattr(inspect_mod, "Signature") == True; _ledger.append(1)
assert hasattr(inspect_mod, "ismodule") == True; _ledger.append(1)
assert hasattr(inspect_mod, "isbuiltin") == True; _ledger.append(1)
assert hasattr(inspect_mod, "getsource") == True; _ledger.append(1)
assert hasattr(inspect_mod, "getsourcelines") == True; _ledger.append(1)
assert hasattr(inspect_mod, "getfile") == True; _ledger.append(1)
assert hasattr(inspect_mod, "getdoc") == True; _ledger.append(1)
assert hasattr(inspect_mod, "stack") == True; _ledger.append(1)
assert hasattr(inspect_mod, "currentframe") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_iterparse_sniffer_configparser_urllib_inspect_silent {sum(_ledger)} asserts")
