# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "old_test_flag__test_init_subclass"
# subject = "cpython.test_enum.OldTestFlag.test_init_subclass"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import copy
import enum
import doctest
import inspect
import os
import pydoc
import sys
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
from datetime import timedelta

class MyEnum(Flag):

    def __init_subclass__(cls, **kwds):
        super().__init_subclass__(**kwds)
        assert not cls.__dict__.get('_test', False)
        cls._test1 = 'MyEnum'

class TheirEnum(MyEnum):

    def __init_subclass__(cls, **kwds):
        super(TheirEnum, cls).__init_subclass__(**kwds)
        cls._test2 = 'TheirEnum'

class WhoseEnum(TheirEnum):

    def __init_subclass__(cls, **kwds):
        pass

class NoEnum(WhoseEnum):
    ONE = 1
assert TheirEnum.__dict__['_test1'] == 'MyEnum'
assert WhoseEnum.__dict__['_test1'] == 'MyEnum'
assert WhoseEnum.__dict__['_test2'] == 'TheirEnum'
assert not NoEnum.__dict__.get('_test1', False)
assert not NoEnum.__dict__.get('_test2', False)

class OurEnum(MyEnum):

    def __init_subclass__(cls, **kwds):
        cls._test2 = 'OurEnum'

class WhereEnum(OurEnum):

    def __init_subclass__(cls, **kwds):
        pass

class NeverEnum(WhereEnum):
    ONE = 1
assert OurEnum.__dict__['_test1'] == 'MyEnum'
assert not WhereEnum.__dict__.get('_test1', False)
assert WhereEnum.__dict__['_test2'] == 'OurEnum'
assert not NeverEnum.__dict__.get('_test1', False)
assert not NeverEnum.__dict__.get('_test2', False)

print("OldTestFlag::test_init_subclass: ok")
