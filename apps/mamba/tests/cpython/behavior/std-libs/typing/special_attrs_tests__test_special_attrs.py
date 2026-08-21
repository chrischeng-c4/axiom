# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "special_attrs_tests__test_special_attrs"
# subject = "cpython.test_typing.SpecialAttrsTests.test_special_attrs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_typing.py::SpecialAttrsTests::test_special_attrs
"""Auto-ported test: SpecialAttrsTests::test_special_attrs (CPython 3.12 oracle)."""


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
TypeName = typing.NewType('SpecialAttrsTests.TypeName', Any)

def clear_caches():
    for f in typing._cleanups:
        f()
cls_to_check = {typing.AbstractSet: 'AbstractSet', typing.AsyncContextManager: 'AsyncContextManager', typing.AsyncGenerator: 'AsyncGenerator', typing.AsyncIterable: 'AsyncIterable', typing.AsyncIterator: 'AsyncIterator', typing.Awaitable: 'Awaitable', typing.ByteString: 'ByteString', typing.Callable: 'Callable', typing.ChainMap: 'ChainMap', typing.Collection: 'Collection', typing.Container: 'Container', typing.ContextManager: 'ContextManager', typing.Coroutine: 'Coroutine', typing.Counter: 'Counter', typing.DefaultDict: 'DefaultDict', typing.Deque: 'Deque', typing.Dict: 'Dict', typing.FrozenSet: 'FrozenSet', typing.Generator: 'Generator', typing.Hashable: 'Hashable', typing.ItemsView: 'ItemsView', typing.Iterable: 'Iterable', typing.Iterator: 'Iterator', typing.KeysView: 'KeysView', typing.List: 'List', typing.Mapping: 'Mapping', typing.MappingView: 'MappingView', typing.MutableMapping: 'MutableMapping', typing.MutableSequence: 'MutableSequence', typing.MutableSet: 'MutableSet', typing.OrderedDict: 'OrderedDict', typing.Reversible: 'Reversible', typing.Sequence: 'Sequence', typing.Set: 'Set', typing.Sized: 'Sized', typing.Tuple: 'Tuple', typing.Type: 'Type', typing.ValuesView: 'ValuesView', typing.AbstractSet[Any]: 'AbstractSet', typing.AsyncContextManager[Any]: 'AsyncContextManager', typing.AsyncGenerator[Any, Any]: 'AsyncGenerator', typing.AsyncIterable[Any]: 'AsyncIterable', typing.AsyncIterator[Any]: 'AsyncIterator', typing.Awaitable[Any]: 'Awaitable', typing.Callable[[], Any]: 'Callable', typing.Callable[..., Any]: 'Callable', typing.ChainMap[Any, Any]: 'ChainMap', typing.Collection[Any]: 'Collection', typing.Container[Any]: 'Container', typing.ContextManager[Any]: 'ContextManager', typing.Coroutine[Any, Any, Any]: 'Coroutine', typing.Counter[Any]: 'Counter', typing.DefaultDict[Any, Any]: 'DefaultDict', typing.Deque[Any]: 'Deque', typing.Dict[Any, Any]: 'Dict', typing.FrozenSet[Any]: 'FrozenSet', typing.Generator[Any, Any, Any]: 'Generator', typing.ItemsView[Any, Any]: 'ItemsView', typing.Iterable[Any]: 'Iterable', typing.Iterator[Any]: 'Iterator', typing.KeysView[Any]: 'KeysView', typing.List[Any]: 'List', typing.Mapping[Any, Any]: 'Mapping', typing.MappingView[Any]: 'MappingView', typing.MutableMapping[Any, Any]: 'MutableMapping', typing.MutableSequence[Any]: 'MutableSequence', typing.MutableSet[Any]: 'MutableSet', typing.OrderedDict[Any, Any]: 'OrderedDict', typing.Reversible[Any]: 'Reversible', typing.Sequence[Any]: 'Sequence', typing.Set[Any]: 'Set', typing.Tuple[Any]: 'Tuple', typing.Tuple[Any, ...]: 'Tuple', typing.Type[Any]: 'Type', typing.ValuesView[Any]: 'ValuesView', typing.Annotated: 'Annotated', typing.Any: 'Any', typing.ClassVar: 'ClassVar', typing.Concatenate: 'Concatenate', typing.Final: 'Final', typing.ForwardRef: 'ForwardRef', typing.Literal: 'Literal', typing.NewType: 'NewType', typing.NoReturn: 'NoReturn', typing.Never: 'Never', typing.Optional: 'Optional', typing.TypeAlias: 'TypeAlias', typing.TypeGuard: 'TypeGuard', typing.TypeVar: 'TypeVar', typing.Union: 'Union', typing.Self: 'Self', typing.Annotated[Any, 'Annotation']: 'Annotated', typing.Annotated[int, 'Annotation']: 'Annotated', typing.ClassVar[Any]: 'ClassVar', typing.Concatenate[Any, SpecialAttrsP]: 'Concatenate', typing.Final[Any]: 'Final', typing.Literal[Any]: 'Literal', typing.Literal[1, 2]: 'Literal', typing.Literal[True, 2]: 'Literal', typing.Optional[Any]: 'Optional', typing.TypeGuard[Any]: 'TypeGuard', typing.Union[Any]: 'Any', typing.Union[int, float]: 'Union'}
for cls, name in cls_to_check.items():

    assert cls.__name__ == name

    assert cls.__qualname__ == name

    assert cls.__module__ == 'typing'
    for proto in range(pickle.HIGHEST_PROTOCOL + 1):
        s = pickle.dumps(cls, proto)
        loaded = pickle.loads(s)

        assert cls is loaded
print("SpecialAttrsTests::test_special_attrs: ok")
