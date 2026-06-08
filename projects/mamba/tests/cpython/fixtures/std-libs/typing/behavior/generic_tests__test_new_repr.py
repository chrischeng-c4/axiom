# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "generic_tests__test_new_repr"
# subject = "cpython.test_typing.GenericTests.test_new_repr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_typing.py::GenericTests::test_new_repr
"""Auto-ported test: GenericTests::test_new_repr (CPython 3.12 oracle)."""


import contextlib
import collections
import collections.abc
from collections import defaultdict
from functools import lru_cache, wraps, reduce
import gc
import inspect
import itertools
import operator
import pickle
import re
import sys
import warnings
from unittest import TestCase, main, skip
from unittest.mock import patch
from copy import copy, deepcopy
from typing import Any, NoReturn, Never, assert_never
from typing import overload, get_overloads, clear_overloads
from typing import TypeVar, TypeVarTuple, Unpack, AnyStr
from typing import T, KT, VT
from typing import Union, Optional, Literal
from typing import Tuple, List, Dict, MutableMapping
from typing import Callable
from typing import Generic, ClassVar, Final, final, Protocol
from typing import assert_type, cast, runtime_checkable
from typing import get_type_hints
from typing import get_origin, get_args
from typing import override
from typing import is_typeddict
from typing import reveal_type
from typing import dataclass_transform
from typing import no_type_check, no_type_check_decorator
from typing import Type
from typing import NamedTuple, NotRequired, Required, TypedDict
from typing import IO, TextIO, BinaryIO
from typing import Pattern, Match
from typing import Annotated, ForwardRef
from typing import Self, LiteralString
from typing import TypeAlias
from typing import ParamSpec, Concatenate, ParamSpecArgs, ParamSpecKwargs
from typing import TypeGuard
import abc
import textwrap
import typing
import weakref
import types
from test.support import captured_stderr, cpython_only
from test.support.testcase import ExtraAssertions
from test.typinganndata import ann_module695, mod_generics_cache, _typed_dict_helper
from test.typinganndata import ann_module8
from test.typinganndata import ann_module, ann_module2, ann_module3, ann_module5, ann_module6


CANNOT_SUBCLASS_TYPE = 'Cannot subclass special typing classes'

NOT_A_BASE_TYPE = "type 'typing.%s' is not an acceptable base type"

CANNOT_SUBCLASS_INSTANCE = 'Cannot subclass an instance of %s'

def all_pickle_protocols(test_func):
    """Runs `test_func` with various values for `proto` argument."""

    @wraps(test_func)
    def wrapper(self):
        for proto in range(pickle.HIGHEST_PROTOCOL + 1):
            with self.subTest(pickle_proto=proto):
                test_func(self, proto=proto)
    return wrapper

class Employee:
    pass

class Manager(Employee):
    pass

class Founder(Employee):
    pass

class ManagingFounder(Manager, Founder):
    pass

def template_replace(templates: list[str], replacements: dict[str, list[str]]) -> list[tuple[str]]:
    """Renders templates with possible combinations of replacements.

    Example 1: Suppose that:
      templates = ["dog_breed are awesome", "dog_breed are cool"]
      replacements = {"dog_breed": ["Huskies", "Beagles"]}
    Then we would return:
      [
          ("Huskies are awesome", "Huskies are cool"),
          ("Beagles are awesome", "Beagles are cool")
      ]

    Example 2: Suppose that:
      templates = ["Huskies are word1 but also word2"]
      replacements = {"word1": ["playful", "cute"],
                      "word2": ["feisty", "tiring"]}
    Then we would return:
      [
          ("Huskies are playful but also feisty"),
          ("Huskies are playful but also tiring"),
          ("Huskies are cute but also feisty"),
          ("Huskies are cute but also tiring")
      ]

    Note that if any of the replacements do not occur in any template:
      templates = ["Huskies are word1", "Beagles!"]
      replacements = {"word1": ["playful", "cute"],
                      "word2": ["feisty", "tiring"]}
    Then we do not generate duplicates, returning:
      [
          ("Huskies are playful", "Beagles!"),
          ("Huskies are cute", "Beagles!")
      ]
    """
    replacement_combos = []
    for original, possible_replacements in replacements.items():
        original_replacement_tuples = []
        for replacement in possible_replacements:
            original_replacement_tuples.append((original, replacement))
        replacement_combos.append(original_replacement_tuples)
    rendered_templates = []
    for replacement_combo in itertools.product(*replacement_combos):
        templates_with_replacements = []
        for template in templates:
            for original, replacement in replacement_combo:
                template = template.replace(original, replacement)
            templates_with_replacements.append(template)
        rendered_templates.append(tuple(templates_with_replacements))
    rendered_templates_no_duplicates = []
    for x in rendered_templates:
        if x not in rendered_templates_no_duplicates:
            rendered_templates_no_duplicates.append(x)
    return rendered_templates_no_duplicates

XK = TypeVar('XK', str, bytes)

XV = TypeVar('XV')

class SimpleMapping(Generic[XK, XV]):

    def __getitem__(self, key: XK) -> XV:
        ...

    def __setitem__(self, key: XK, value: XV):
        ...

    def get(self, key: XK, default: XV=None) -> XV:
        ...

class MySimpleMapping(SimpleMapping[XK, XV]):

    def __init__(self):
        self.store = {}

    def __getitem__(self, key: str):
        return self.store[key]

    def __setitem__(self, key: str, value):
        self.store[key] = value

    def get(self, key: str, default=None):
        try:
            return self.store[key]
        except KeyError:
            return default

class Coordinate(Protocol):
    x: int
    y: int

@runtime_checkable
class Point(Coordinate, Protocol):
    label: str

class MyPoint:
    x: int
    y: int
    label: str

class XAxis(Protocol):
    x: int

class YAxis(Protocol):
    y: int

@runtime_checkable
class Position(XAxis, YAxis, Protocol):
    pass

@runtime_checkable
class Proto(Protocol):
    attr: int

    def meth(self, arg: str) -> int:
        ...

class Concrete(Proto):
    pass

class Other:
    attr: int = 1

    def meth(self, arg: str) -> int:
        if arg == 'this':
            return 1
        return 0

class NT(NamedTuple):
    x: int
    y: int

@runtime_checkable
class HasCallProtocol(Protocol):
    __call__: typing.Callable

@no_type_check
class NoTypeCheck_Outer:
    Inner = ann_module8.NoTypeCheck_Outer.Inner

@no_type_check
class NoTypeCheck_WithFunction:
    NoTypeCheck_function = ann_module8.NoTypeCheck_function

@lru_cache()
def cached_func(x, y):
    return 3 * x + y

class MethodHolder:

    @classmethod
    def clsmethod(cls):
        ...

    @staticmethod
    def stmethod():
        ...

    def method(self):
        ...

T_a = TypeVar('T_a')

class AwaitableWrapper(typing.Awaitable[T_a]):

    def __init__(self, value):
        self.value = value

    def __await__(self) -> typing.Iterator[T_a]:
        yield
        return self.value

class AsyncIteratorWrapper(typing.AsyncIterator[T_a]):

    def __init__(self, value: typing.Iterable[T_a]):
        self.value = value

    def __aiter__(self) -> typing.AsyncIterator[T_a]:
        return self

    async def __anext__(self) -> T_a:
        data = await self.value
        if data:
            return data
        else:
            raise StopAsyncIteration

class ACM:

    async def __aenter__(self) -> int:
        return 42

    async def __aexit__(self, etype, eval, tb):
        return None

class A:
    y: float

class B(A):
    x: ClassVar[Optional['B']] = None
    y: int
    b: int

class CSub(B):
    z: ClassVar['CSub'] = B()

class G(Generic[T]):
    lst: ClassVar[List[T]] = []

class Loop:
    attr: Final['Loop']

class NoneAndForward:
    parent: 'NoneAndForward'
    meaning: None

class CoolEmployee(NamedTuple):
    name: str
    cool: int

class CoolEmployeeWithDefault(NamedTuple):
    name: str
    cool: int = 0

class XMeth(NamedTuple):
    x: int

    def double(self):
        return 2 * self.x

class XRepr(NamedTuple):
    x: int
    y: int = 1

    def __str__(self):
        return f'{self.x} -> {self.y}'

    def __add__(self, other):
        return 0

Label = TypedDict('Label', [('label', str)])

class Point2D(TypedDict):
    x: int
    y: int

class Point2DGeneric(Generic[T], TypedDict):
    a: T
    b: T

class Bar(_typed_dict_helper.Foo, total=False):
    b: int

class BarGeneric(_typed_dict_helper.FooGeneric[T], total=False):
    b: int

class LabelPoint2D(Point2D, Label):
    ...

class Options(TypedDict, total=False):
    log_level: int
    log_path: str

class TotalMovie(TypedDict):
    title: str
    year: NotRequired[int]

class NontotalMovie(TypedDict, total=False):
    title: Required[str]
    year: int

class ParentNontotalMovie(TypedDict, total=False):
    title: Required[str]

class ChildTotalMovie(ParentNontotalMovie):
    year: NotRequired[int]

class ParentDeeplyAnnotatedMovie(TypedDict):
    title: Annotated[Annotated[Required[str], 'foobar'], 'another level']

class ChildDeeplyAnnotatedMovie(ParentDeeplyAnnotatedMovie):
    year: NotRequired[Annotated[int, 2000]]

class AnnotatedMovie(TypedDict):
    title: Annotated[Required[str], 'foobar']
    year: NotRequired[Annotated[int, 2000]]

class DeeplyAnnotatedMovie(TypedDict):
    title: Annotated[Annotated[Required[str], 'foobar'], 'another level']
    year: NotRequired[Annotated[int, 2000]]

class WeirdlyQuotedMovie(TypedDict):
    title: Annotated['Annotated[Required[str], "foobar"]', 'another level']
    year: NotRequired['Annotated[int, 2000]']

class HasForeignBaseClass(mod_generics_cache.A):
    some_xrepr: 'XRepr'
    other_a: 'mod_generics_cache.A'

async def g_with(am: typing.AsyncContextManager[int]):
    x: int
    async with am as x:
        return x

try:
    g_with(ACM()).send(None)
except StopIteration as e:
    assert e.args[0] == 42

gth = get_type_hints

class ForRefExample:

    @ann_module.dec
    def func(self: 'ForRefExample'):
        pass

    @ann_module.dec
    @ann_module.dec
    def nested(self: 'ForRefExample'):
        pass

SpecialAttrsP = typing.ParamSpec('SpecialAttrsP')

SpecialAttrsT = typing.TypeVar('SpecialAttrsT', int, float, complex)

def load_tests(loader, tests, pattern):
    import doctest
    tests.addTests(doctest.DocTestSuite(typing))
    return tests


# --- test body ---
def clear_caches():
    for f in typing._cleanups:
        f()
T = TypeVar('T')
U = TypeVar('U', covariant=True)
S = TypeVar('S')

assert repr(List) == 'typing.List'

assert repr(List[T]) == 'typing.List[~T]'

assert repr(List[U]) == 'typing.List[+U]'

assert repr(List[S][T][int]) == 'typing.List[int]'

assert repr(List[int]) == 'typing.List[int]'
print("GenericTests::test_new_repr: ok")
