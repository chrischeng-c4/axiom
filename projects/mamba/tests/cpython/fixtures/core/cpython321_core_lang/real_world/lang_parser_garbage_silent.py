# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_parser_garbage_silent"
# subject = "cpython321.lang_parser_garbage_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_parser_garbage_silent.py"
# status = "filled"
# ///
"""cpython321.lang_parser_garbage_silent: execute CPython 3.12 seed lang_parser_garbage_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython UnpicklingError / EOFError / PicklingError /
# TypeError / xml.etree.ParseError contract on the structured-data
# parser / serializer corners that mamba silently returns `None` /
# `b'I1'` / `[]` / a sentinel-dict from. Surface: CPython rejects
# (1) `pickle.loads(garbage)` because the opcode byte doesn't match
# the pickle protocol — UnpicklingError, not silent `None`; (2)
# `pickle.loads(b'')` because there's no first opcode to read —
# EOFError, not silent `None`; (3) `pickle.loads(truncated)` because
# the framed payload ends mid-instruction — UnpicklingError, not
# silent `None`; (4) `pickle.dumps(lambda)` because lambdas are
# anonymous and pickle can't recover them by qualified name —
# PicklingError, not silent fake-int payload `b'I1'`; (5)
# `pickle.dumps(generator)` because generators hold per-frame state
# that can't be reconstituted — TypeError, not silent `b'I1'`; (6)
# `csv.reader(int)` because csv.reader needs an iterable of strings
# — TypeError on iteration, not silent empty `[]`; (7)
# `ET.fromstring('not xml')` / `ET.fromstring('<root>incomplete')`
# because the byte stream doesn't parse as well-formed XML —
# ParseError, not silent placeholder Element with empty children.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • pickle.loads(b"not a pickle blob")
#                                  → mamba: None    (UnpicklingError)
#   • pickle.loads(b"")            → mamba: None    (EOFError)
#   • pickle.loads(b"\x80\x04\x95")→ mamba: None    (UnpicklingError)
#   • pickle.dumps(lambda x: x)    → mamba: b'I1'   (PicklingError)
#   • pickle.dumps(generator())    → mamba: b'I1'   (TypeError)
#   • list(csv.reader(123))        → mamba: []      (TypeError)
#   • ET.fromstring("not xml")     → mamba: <fake>  (ParseError)
#   • ET.fromstring("<r>incomp")   → mamba: <fake>  (ParseError)
#
# CPython contract:
#   pickle.loads(garbage)
#                       → pickle.UnpicklingError("invalid load
#                              key, …");
#   pickle.loads(b"")
#                       → EOFError("Ran out of input");
#   pickle.loads(truncated)
#                       → pickle.UnpicklingError("pickle data was
#                              truncated");
#   pickle.dumps(lambda)
#                       → pickle.PicklingError("Can't pickle <function
#                              <lambda> …>");
#   pickle.dumps(generator)
#                       → TypeError("cannot pickle 'generator'
#                              object");
#   list(csv.reader(int))
#                       → TypeError("'int' object is not iterable");
#   ET.fromstring(garbage)
#                       → xml.etree.ElementTree.ParseError("syntax
#                              error: line 1, column 0").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised.
from typing import Any
import pickle
import csv
import xml.etree.ElementTree as ET
_ledger: list[int] = []

_garbage_pkl: Any = b"not a pickle blob at all, definitely random bytes"
_empty_pkl: Any = b""
_truncated_pkl: Any = b"\x80\x04\x95"
_lambda: Any = lambda x: x

def _gen_factory():
    yield 1
    yield 2

_garbage_xml: Any = "not xml at all"
_truncated_xml: Any = "<root>incomplete"
_int_seq: Any = 123

# pickle.loads on garbage opcode byte
try:
    _ = pickle.loads(_garbage_pkl)
    raise AssertionError("pickle.loads(garbage) must raise UnpicklingError")
except pickle.UnpicklingError:
    _ledger.append(1)

# pickle.loads on empty input
try:
    _ = pickle.loads(_empty_pkl)
    raise AssertionError("pickle.loads(b'') must raise EOFError")
except EOFError:
    _ledger.append(1)

# pickle.loads on truncated framed payload
try:
    _ = pickle.loads(_truncated_pkl)
    raise AssertionError("pickle.loads(truncated) must raise UnpicklingError")
except pickle.UnpicklingError:
    _ledger.append(1)

# pickle.dumps a lambda — anonymous, can't be pickled
try:
    _ = pickle.dumps(_lambda)
    raise AssertionError("pickle.dumps(lambda) must raise PicklingError")
except pickle.PicklingError:
    _ledger.append(1)

# pickle.dumps a generator — frame state can't be pickled
_g: Any = _gen_factory()
try:
    _ = pickle.dumps(_g)
    raise AssertionError("pickle.dumps(generator) must raise TypeError")
except TypeError:
    _ledger.append(1)

# csv.reader(int) — int is not an iterable of strings
try:
    _ = list(csv.reader(_int_seq))
    raise AssertionError("list(csv.reader(int)) must raise TypeError")
except TypeError:
    _ledger.append(1)

# ET.fromstring on garbage
try:
    _ = ET.fromstring(_garbage_xml)
    raise AssertionError("ET.fromstring('not xml') must raise ParseError")
except ET.ParseError:
    _ledger.append(1)

# ET.fromstring on truncated input
try:
    _ = ET.fromstring(_truncated_xml)
    raise AssertionError("ET.fromstring('<root>incomp') must raise ParseError")
except ET.ParseError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_parser_garbage_silent {sum(_ledger)} asserts")
