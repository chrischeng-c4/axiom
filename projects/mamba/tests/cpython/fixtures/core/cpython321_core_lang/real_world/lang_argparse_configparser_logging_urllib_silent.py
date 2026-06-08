# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_argparse_configparser_logging_urllib_silent"
# subject = "cpython321.lang_argparse_configparser_logging_urllib_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_argparse_configparser_logging_urllib_silent.py"
# status = "filled"
# ///
"""cpython321.lang_argparse_configparser_logging_urllib_silent: execute CPython 3.12 seed lang_argparse_configparser_logging_urllib_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the
# `argparse` / `csv` / `configparser` / `logging` / `html` /
# `ast` / `dis` / `token` / `urllib.parse` / `xml.etree` /
# `asyncio` eleven-pack pinned to atomic 229:
# `argparse` (the documented extended `hasattr(argparse,
# "Namespace") / "Action" / "FileType" / "HelpFormatter" /
# "ArgumentDefaultsHelpFormatter" / "RawDescriptionHelpFormatter"
# / "RawTextHelpFormatter" / "SUPPRESS" / "PARSER" / "REMAINDER"
# / "OPTIONAL" / "ZERO_OR_MORE" / "ONE_OR_MORE" == True`
# extended hasattr surface), `csv` (the documented
# `hasattr(csv, "Sniffer") == True` dialect-sniffer class),
# `configparser` (the documented extended `hasattr(configparser,
# "RawConfigParser") / "BasicInterpolation" /
# "ExtendedInterpolation" / "Interpolation" /
# "MissingSectionHeaderError" / "ParsingError" /
# "DuplicateSectionError" / "DuplicateOptionError" /
# "NoSectionError" / "NoOptionError" / "Error" / "DEFAULTSECT"
# == True` extended hasattr surface), `logging` (the
# documented extended `hasattr(logging, "Logger") / "Handler" /
# "StreamHandler" / "FileHandler" / "Formatter" / "Filter" /
# "LogRecord" / "NOTSET" / "log" / "exception" /
# "captureWarnings" == True` extended hasattr surface),
# `html` (the documented `hasattr(html, "entities") == True`
# entity-table submodule), `ast` (the documented
# `ast.literal_eval("[1, 2, 3]") == [1, 2, 3]` literal-list
# value contract — mamba silently returns None for list/dict
# literals), `dis` (the documented extended
# `hasattr(dis, "Bytecode") / "hasarg" / "EXTENDED_ARG" /
# "cmp_op" / "hasconst" / "hasname" / "hasjrel" / "hasjabs" /
# "haslocal" / "hascompare" / "hasfree" == True` extended
# hasattr surface), `urllib.parse`
# (the documented `hasattr(urllib.parse, "urlparse") /
# "urlunparse" / "urlsplit" / "urlunsplit" / "quote" /
# "unquote" / "quote_plus" / "unquote_plus" / "urlencode" /
# "parse_qs" / "parse_qsl" / "urljoin" / "urldefrag" /
# "uses_relative" / "uses_netloc" == True` dotted-module
# hasattr surface — mamba's dotted-import quirk collapses
# `urllib.parse.X` to False even though the bare
# `from urllib.parse import X` call sites work),
# `xml.etree` (the documented `hasattr(ET, "iterparse") /
# "XMLParser" == True` extended class surface), and
# `asyncio` (the documented extended `hasattr(asyncio,
# "Task") / "Future" / "Event" / "Lock" / "Semaphore" /
# "Queue" / "PriorityQueue" == True` extended sync-primitive
# class surface).
#
# Behavioral edges that CONFORM on mamba (csv core, logging
# common, mimetypes core + guess_type value-op, html
# escape/unescape value-ops, ast common hasattr, dis common,
# token common, keyword full surface + value ops,
# subprocess full hasattr, asyncio top-level coroutine
# surface) are covered in the matching pass fixture
# `test_csv_logging_mimetypes_html_ast_dis_token_value_ops`.
from typing import Any
import argparse as _argparse_mod
import csv as _csv_mod
import configparser as _configparser_mod
import logging as _logging_mod
import html as _html_mod
import ast as _ast_mod
import dis as _dis_mod
import urllib.parse as _urllib_parse_mod
import xml.etree.ElementTree as _ET_mod
import asyncio as _asyncio_mod

argparse: Any = _argparse_mod
csv: Any = _csv_mod
configparser: Any = _configparser_mod
logging: Any = _logging_mod
html: Any = _html_mod
ast: Any = _ast_mod
dis: Any = _dis_mod
urllib_parse: Any = _urllib_parse_mod
ET: Any = _ET_mod
asyncio: Any = _asyncio_mod


_ledger: list[int] = []

# 1) argparse — extended module hasattr surface
#    (mamba: Namespace / Action / FileType / HelpFormatter /
#    ArgumentDefaultsHelpFormatter / RawDescriptionHelpFormatter
#    / RawTextHelpFormatter / SUPPRESS / PARSER / REMAINDER /
#    OPTIONAL / ZERO_OR_MORE / ONE_OR_MORE all False)
assert hasattr(argparse, "Namespace") == True; _ledger.append(1)
assert hasattr(argparse, "Action") == True; _ledger.append(1)
assert hasattr(argparse, "FileType") == True; _ledger.append(1)
assert hasattr(argparse, "HelpFormatter") == True; _ledger.append(1)
assert hasattr(argparse, "ArgumentDefaultsHelpFormatter") == True; _ledger.append(1)
assert hasattr(argparse, "RawDescriptionHelpFormatter") == True; _ledger.append(1)
assert hasattr(argparse, "RawTextHelpFormatter") == True; _ledger.append(1)
assert hasattr(argparse, "SUPPRESS") == True; _ledger.append(1)
assert hasattr(argparse, "PARSER") == True; _ledger.append(1)
assert hasattr(argparse, "REMAINDER") == True; _ledger.append(1)
assert hasattr(argparse, "OPTIONAL") == True; _ledger.append(1)
assert hasattr(argparse, "ZERO_OR_MORE") == True; _ledger.append(1)
assert hasattr(argparse, "ONE_OR_MORE") == True; _ledger.append(1)

# 2) csv — dialect-sniffer class hasattr
#    (mamba: Sniffer False)
assert hasattr(csv, "Sniffer") == True; _ledger.append(1)

# 3) configparser — extended module hasattr surface
#    (mamba: RawConfigParser / BasicInterpolation /
#    ExtendedInterpolation / Interpolation /
#    MissingSectionHeaderError / ParsingError /
#    DuplicateSectionError / DuplicateOptionError /
#    NoSectionError / NoOptionError / Error / DEFAULTSECT
#    all False)
assert hasattr(configparser, "RawConfigParser") == True; _ledger.append(1)
assert hasattr(configparser, "BasicInterpolation") == True; _ledger.append(1)
assert hasattr(configparser, "ExtendedInterpolation") == True; _ledger.append(1)
assert hasattr(configparser, "Interpolation") == True; _ledger.append(1)
assert hasattr(configparser, "MissingSectionHeaderError") == True; _ledger.append(1)
assert hasattr(configparser, "ParsingError") == True; _ledger.append(1)
assert hasattr(configparser, "DuplicateSectionError") == True; _ledger.append(1)
assert hasattr(configparser, "DuplicateOptionError") == True; _ledger.append(1)
assert hasattr(configparser, "NoSectionError") == True; _ledger.append(1)
assert hasattr(configparser, "NoOptionError") == True; _ledger.append(1)
assert hasattr(configparser, "Error") == True; _ledger.append(1)
assert hasattr(configparser, "DEFAULTSECT") == True; _ledger.append(1)

# 4) logging — extended module hasattr surface
#    (mamba: Logger / Handler / StreamHandler / FileHandler /
#    Formatter / Filter / LogRecord / NOTSET / log /
#    exception / captureWarnings all False)
assert hasattr(logging, "Logger") == True; _ledger.append(1)
assert hasattr(logging, "Handler") == True; _ledger.append(1)
assert hasattr(logging, "StreamHandler") == True; _ledger.append(1)
assert hasattr(logging, "FileHandler") == True; _ledger.append(1)
assert hasattr(logging, "Formatter") == True; _ledger.append(1)
assert hasattr(logging, "Filter") == True; _ledger.append(1)
assert hasattr(logging, "LogRecord") == True; _ledger.append(1)
assert hasattr(logging, "NOTSET") == True; _ledger.append(1)
assert hasattr(logging, "log") == True; _ledger.append(1)
assert hasattr(logging, "exception") == True; _ledger.append(1)
assert hasattr(logging, "captureWarnings") == True; _ledger.append(1)

# 5) html — entity-table submodule hasattr
#    (mamba: entities False)
assert hasattr(html, "entities") == True; _ledger.append(1)

# 6) ast — literal_eval value contract
#    (mamba: silently returns None for list/dict literals
#    instead of the documented evaluated value)
assert ast.literal_eval("[1, 2, 3]") == [1, 2, 3]; _ledger.append(1)

# 7) dis — extended module hasattr surface
#    (mamba: Bytecode / hasarg / EXTENDED_ARG / cmp_op /
#    hasconst / hasname / hasjrel / hasjabs / haslocal /
#    hascompare / hasfree all False)
assert hasattr(dis, "Bytecode") == True; _ledger.append(1)
assert hasattr(dis, "hasarg") == True; _ledger.append(1)
assert hasattr(dis, "EXTENDED_ARG") == True; _ledger.append(1)
assert hasattr(dis, "cmp_op") == True; _ledger.append(1)
assert hasattr(dis, "hasconst") == True; _ledger.append(1)
assert hasattr(dis, "hasname") == True; _ledger.append(1)
assert hasattr(dis, "hasjrel") == True; _ledger.append(1)
assert hasattr(dis, "hasjabs") == True; _ledger.append(1)
assert hasattr(dis, "haslocal") == True; _ledger.append(1)
assert hasattr(dis, "hascompare") == True; _ledger.append(1)
assert hasattr(dis, "hasfree") == True; _ledger.append(1)

# 8) urllib.parse — dotted-module hasattr surface
#    (mamba's dotted-import quirk collapses urllib.parse.X
#    to False even though `from urllib.parse import X` works)
assert hasattr(urllib_parse, "urlparse") == True; _ledger.append(1)
assert hasattr(urllib_parse, "urlunparse") == True; _ledger.append(1)
assert hasattr(urllib_parse, "urlsplit") == True; _ledger.append(1)
assert hasattr(urllib_parse, "urlunsplit") == True; _ledger.append(1)
assert hasattr(urllib_parse, "quote") == True; _ledger.append(1)
assert hasattr(urllib_parse, "unquote") == True; _ledger.append(1)
assert hasattr(urllib_parse, "quote_plus") == True; _ledger.append(1)
assert hasattr(urllib_parse, "unquote_plus") == True; _ledger.append(1)
assert hasattr(urllib_parse, "urlencode") == True; _ledger.append(1)
assert hasattr(urllib_parse, "parse_qs") == True; _ledger.append(1)
assert hasattr(urllib_parse, "parse_qsl") == True; _ledger.append(1)
assert hasattr(urllib_parse, "urljoin") == True; _ledger.append(1)
assert hasattr(urllib_parse, "urldefrag") == True; _ledger.append(1)

# 9) xml.etree — extended class hasattr surface
#    (mamba: iterparse / XMLParser False)
assert hasattr(ET, "iterparse") == True; _ledger.append(1)
assert hasattr(ET, "XMLParser") == True; _ledger.append(1)

# 10) asyncio — extended sync-primitive class hasattr
#     (mamba: Task / Future / Event / Lock / Semaphore /
#     Queue / PriorityQueue all False)
assert hasattr(asyncio, "Task") == True; _ledger.append(1)
assert hasattr(asyncio, "Future") == True; _ledger.append(1)
assert hasattr(asyncio, "Event") == True; _ledger.append(1)
assert hasattr(asyncio, "Lock") == True; _ledger.append(1)
assert hasattr(asyncio, "Semaphore") == True; _ledger.append(1)
assert hasattr(asyncio, "Queue") == True; _ledger.append(1)
assert hasattr(asyncio, "PriorityQueue") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_argparse_configparser_logging_urllib_silent {sum(_ledger)} asserts")
