# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "old_test_int_flag__test_programatic_function_string"
# subject = "cpython.test_enum.OldTestIntFlag.test_programatic_function_string"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_enum.py::OldTestIntFlag::test_programatic_function_string
"""Auto-ported test: OldTestIntFlag::test_programatic_function_string (CPython 3.12 oracle)."""


import copy
import enum
import doctest
import inspect
import os
import pydoc
import sys
import unittest
import threading
import typing
import builtins as bltns
from collections import OrderedDict
from datetime import date
from enum import Enum, EnumMeta, IntEnum, StrEnum, EnumType, Flag, IntFlag, unique, auto
from enum import STRICT, CONFORM, EJECT, KEEP, _simple_enum, _test_simple_enum
from enum import verify, UNIQUE, CONTINUOUS, NAMED_FLAGS, ReprEnum
from enum import member, nonmember, _iter_bits_lsb
from io import StringIO
from pickle import dumps, loads, PicklingError, HIGHEST_PROTOCOL
from test import support
from test.support import ALWAYS_EQ, REPO_ROOT
from test.support import threading_helper
from datetime import timedelta


python_version = sys.version_info[:2]

def load_tests(loader, tests, ignore):
    tests.addTests(doctest.DocTestSuite(enum))
    lib_tests = os.path.join(REPO_ROOT, 'Doc/library/enum.rst')
    if os.path.exists(lib_tests):
        tests.addTests(doctest.DocFileSuite(lib_tests, module_relative=False, optionflags=doctest.ELLIPSIS | doctest.NORMALIZE_WHITESPACE))
    howto_tests = os.path.join(REPO_ROOT, 'Doc/howto/enum.rst')
    if os.path.exists(howto_tests):
        tests.addTests(doctest.DocFileSuite(howto_tests, module_relative=False, optionflags=doctest.ELLIPSIS | doctest.NORMALIZE_WHITESPACE))
    return tests

MODULE = __name__

SHORT_MODULE = MODULE.split('.')[-1]

try:

    class Stooges(Enum):
        LARRY = 1
        CURLY = 2
        MOE = 3
except Exception as exc:
    Stooges = exc

try:

    class IntStooges(int, Enum):
        LARRY = 1
        CURLY = 2
        MOE = 3
except Exception as exc:
    IntStooges = exc

try:

    class FloatStooges(float, Enum):
        LARRY = 1.39
        CURLY = 2.72
        MOE = 3.142596
except Exception as exc:
    FloatStooges = exc

try:

    class FlagStooges(Flag):
        LARRY = 1
        CURLY = 2
        MOE = 4
        BIG = 389
except Exception as exc:
    FlagStooges = exc

class FlagStoogesWithZero(Flag):
    NOFLAG = 0
    LARRY = 1
    CURLY = 2
    MOE = 4
    BIG = 389

class IntFlagStooges(IntFlag):
    LARRY = 1
    CURLY = 2
    MOE = 4
    BIG = 389

class IntFlagStoogesWithZero(IntFlag):
    NOFLAG = 0
    LARRY = 1
    CURLY = 2
    MOE = 4
    BIG = 389

class Name(StrEnum):
    BDFL = 'Guido van Rossum'
    FLUFL = 'Barry Warsaw'

try:
    Question = Enum('Question', 'who what when where why', module=__name__)
except Exception as exc:
    Question = exc

try:
    Answer = Enum('Answer', 'him this then there because')
except Exception as exc:
    Answer = exc

try:
    Theory = Enum('Theory', 'rule law supposition', qualname='spanish_inquisition')
except Exception as exc:
    Theory = exc

try:

    class Fruit(Enum):
        TOMATO = 1
        BANANA = 2
        CHERRY = 3
except Exception:
    pass

def test_pickle_dump_load(assertion, source, target=None):
    if target is None:
        target = source
    for protocol in range(HIGHEST_PROTOCOL + 1):
        assertion(loads(dumps(source, protocol=protocol)), target)

def test_pickle_exception(assertion, exception, obj):
    for protocol in range(HIGHEST_PROTOCOL + 1):
        with assertion(exception):
            dumps(obj, protocol=protocol)

class classproperty:

    def __init__(self, fget=None, fset=None, fdel=None, doc=None):
        self.fget = fget
        self.fset = fset
        self.fdel = fdel
        if doc is None and fget is not None:
            doc = fget.__doc__
        self.__doc__ = doc

    def __get__(self, instance, ownerclass):
        return self.fget(ownerclass)

@enum.global_enum
class HeadlightsK(IntFlag, boundary=enum.KEEP):
    OFF_K = 0
    LOW_BEAM_K = auto()
    HIGH_BEAM_K = auto()
    FOG_K = auto()

@enum.global_enum
class HeadlightsC(IntFlag, boundary=enum.CONFORM):
    OFF_C = 0
    LOW_BEAM_C = auto()
    HIGH_BEAM_C = auto()
    FOG_C = auto()

@enum.global_enum
class NoName(Flag):
    ONE = 1
    TWO = 2

expected_help_output_with_docs = "Help on class Color in module %s:\n\nclass Color(enum.Enum)\n |  Color(*values)\n |\n |  Method resolution order:\n |      Color\n |      enum.Enum\n |      builtins.object\n |\n |  Data and other attributes defined here:\n |\n |  CYAN = <Color.CYAN: 1>\n |\n |  MAGENTA = <Color.MAGENTA: 2>\n |\n |  YELLOW = <Color.YELLOW: 3>\n |\n |  ----------------------------------------------------------------------\n |  Data descriptors inherited from enum.Enum:\n |\n |  name\n |      The name of the Enum member.\n |\n |  value\n |      The value of the Enum member.\n |\n |  ----------------------------------------------------------------------\n |  Static methods inherited from enum.EnumType:\n |\n |  __contains__(value)\n |      Return True if `value` is in `cls`.\n |\n |      `value` is in `cls` if:\n |      1) `value` is a member of `cls`, or\n |      2) `value` is the value of one of the `cls`'s members.\n |      3) `value` is a pseudo-member (flags)\n |\n |  __getitem__(name)\n |      Return the member matching `name`.\n |\n |  __iter__()\n |      Return members in definition order.\n |\n |  __len__()\n |      Return the number of members (no aliases)\n |\n |  ----------------------------------------------------------------------\n |  Readonly properties inherited from enum.EnumType:\n |\n |  __members__\n |      Returns a mapping of member name->value.\n |\n |      This mapping lists all enum members, including aliases. Note that this\n |      is a read-only view of the internal mapping."

expected_help_output_without_docs = 'Help on class Color in module %s:\n\nclass Color(enum.Enum)\n |  Color(*values)\n |\n |  Method resolution order:\n |      Color\n |      enum.Enum\n |      builtins.object\n |\n |  Data and other attributes defined here:\n |\n |  CYAN = <Color.CYAN: 1>\n |\n |  MAGENTA = <Color.MAGENTA: 2>\n |\n |  YELLOW = <Color.YELLOW: 3>\n |\n |  ----------------------------------------------------------------------\n |  Data descriptors inherited from enum.Enum:\n |\n |  name\n |\n |  value\n |\n |  ----------------------------------------------------------------------\n |  Static methods inherited from enum.EnumType:\n |\n |  __contains__(value)\n |\n |  __getitem__(name)\n |\n |  __iter__()\n |\n |  __len__()\n |\n |  ----------------------------------------------------------------------\n |  Readonly properties inherited from enum.EnumType:\n |\n |  __members__'

CONVERT_TEST_NAME_D = 5

CONVERT_TEST_NAME_C = 5

CONVERT_TEST_NAME_B = 5

CONVERT_TEST_NAME_A = 5

CONVERT_TEST_NAME_E = 5

CONVERT_TEST_NAME_F = 5

CONVERT_STRING_TEST_NAME_D = 5

CONVERT_STRING_TEST_NAME_C = 5

CONVERT_STRING_TEST_NAME_B = 5

CONVERT_STRING_TEST_NAME_A = 5

CONVERT_STRING_TEST_NAME_E = 5

CONVERT_STRING_TEST_NAME_F = 5

CONVERT_STR_TEST_2 = 'goodbye'

CONVERT_STR_TEST_1 = 'hello'

UNCOMPARABLE_A = 5

UNCOMPARABLE_C = (9, 1)

UNCOMPARABLE_B = 'value'

COMPLEX_C = 1j

COMPLEX_A = 2j

COMPLEX_B = 3j

def enum_dir(cls):
    interesting = set(['__class__', '__contains__', '__doc__', '__getitem__', '__iter__', '__len__', '__members__', '__module__', '__name__', '__qualname__'] + cls._member_names_)
    if cls._new_member_ is not object.__new__:
        interesting.add('__new__')
    if cls.__init_subclass__ is not object.__init_subclass__:
        interesting.add('__init_subclass__')
    if cls._member_type_ is object:
        return sorted(interesting)
    else:
        return sorted(set(dir(cls._member_type_)) | interesting)

def member_dir(member):
    if member.__class__._member_type_ is object:
        allowed = set(['__class__', '__doc__', '__eq__', '__hash__', '__module__', 'name', 'value'])
    else:
        allowed = set(dir(member))
    for cls in member.__class__.mro():
        for name, obj in cls.__dict__.items():
            if name[0] == '_':
                continue
            if isinstance(obj, enum.property):
                if obj.fget is not None or name not in member._member_map_:
                    allowed.add(name)
                else:
                    allowed.discard(name)
            elif name not in member._member_map_:
                allowed.add(name)
    return sorted(allowed)


# --- test body ---
Perm = IntFlag('Perm', 'R W X')
lst = list(Perm)

assert len(lst) == len(Perm)

assert len(Perm) == 3

assert lst == [Perm.R, Perm.W, Perm.X]
for i, n in enumerate('R W X'.split()):
    v = 1 << i
    e = Perm(v)

    assert e.value == v

    assert type(e.value) == int

    assert e == v

    assert e.name == n

    assert e in Perm

    assert type(e) is Perm
print("OldTestIntFlag::test_programatic_function_string: ok")
