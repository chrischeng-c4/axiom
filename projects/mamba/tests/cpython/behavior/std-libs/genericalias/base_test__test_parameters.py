# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericalias"
# dimension = "behavior"
# case = "base_test__test_parameters"
# subject = "cpython.test_genericalias.BaseTest.test_parameters"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_genericalias.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_genericalias.py::BaseTest::test_parameters
"""Auto-ported test: BaseTest::test_parameters (CPython 3.12 oracle)."""


import unittest
import pickle
from array import array
import copy
from collections import defaultdict, deque, OrderedDict, Counter, UserDict, UserList
from collections.abc import *
from concurrent.futures import Future
from concurrent.futures.thread import _WorkItem
from contextlib import AbstractContextManager, AbstractAsyncContextManager
from contextvars import ContextVar, Token
from csv import DictReader, DictWriter
from dataclasses import Field
from functools import partial, partialmethod, cached_property
from graphlib import TopologicalSorter
from logging import LoggerAdapter, StreamHandler
from mailbox import Mailbox, _PartialFile
from difflib import SequenceMatcher
from filecmp import dircmp
from fileinput import FileInput
from itertools import chain
from http.cookies import Morsel
from os import DirEntry
from re import Pattern, Match
from types import GenericAlias, MappingProxyType, AsyncGeneratorType
from tempfile import TemporaryDirectory, SpooledTemporaryFile
from urllib.parse import SplitResult, ParseResult
from unittest.case import _AssertRaisesContext
from queue import Queue, SimpleQueue
from weakref import WeakSet, ReferenceType, ref
import typing
from typing import Unpack
from typing import TypeVar


'Tests for C-implemented GenericAlias.'

try:
    import ctypes
except ImportError:
    ctypes = None

try:
    from multiprocessing.managers import ValueProxy
    from multiprocessing.pool import ApplyResult
    from multiprocessing.queues import SimpleQueue as MPSimpleQueue
    from multiprocessing.queues import Queue as MPQueue
    from multiprocessing.queues import JoinableQueue as MPJoinableQueue
except ImportError:
    ValueProxy = None
    ApplyResult = None
    MPSimpleQueue = None
    MPQueue = None
    MPJoinableQueue = None

try:
    from multiprocessing.shared_memory import ShareableList
except ImportError:
    ShareableList = None

T = TypeVar('T')

K = TypeVar('K')

V = TypeVar('V')

_UNPACKED_TUPLES = [(*tuple[int],)[0], (*tuple[T],)[0], (*tuple[int, str],)[0], (*tuple[int, ...],)[0], (*tuple[T, ...],)[0], tuple[*tuple[int, ...],], tuple[*tuple[T, ...],], tuple[str, *tuple[int, ...]], tuple[*tuple[int, ...], str], tuple[float, *tuple[int, ...], str], tuple[*tuple[*tuple[int, ...],],], Unpack[tuple[int]], Unpack[tuple[T]], Unpack[tuple[int, str]], Unpack[tuple[int, ...]], Unpack[tuple[T, ...]], tuple[Unpack[tuple[int, ...]]], tuple[Unpack[tuple[T, ...]]], tuple[str, Unpack[tuple[int, ...]]], tuple[Unpack[tuple[int, ...]], str], tuple[float, Unpack[tuple[int, ...]], str], tuple[Unpack[tuple[Unpack[tuple[int, ...]]]]], tuple[Unpack[tuple[*tuple[int, ...],]]], tuple[*tuple[Unpack[tuple[int, ...]]],]]


# --- test body ---
generic_types = [type, tuple, list, dict, set, frozenset, enumerate, defaultdict, deque, SequenceMatcher, dircmp, FileInput, OrderedDict, Counter, UserDict, UserList, Pattern, Match, partial, partialmethod, cached_property, TopologicalSorter, AbstractContextManager, AbstractAsyncContextManager, Awaitable, Coroutine, AsyncIterable, AsyncIterator, AsyncGenerator, Generator, Iterable, Iterator, Reversible, Container, Collection, Mailbox, _PartialFile, ContextVar, Token, Field, Set, MutableSet, Mapping, MutableMapping, MappingView, KeysView, ItemsView, ValuesView, Sequence, MutableSequence, MappingProxyType, AsyncGeneratorType, DirEntry, chain, LoggerAdapter, StreamHandler, TemporaryDirectory, SpooledTemporaryFile, Queue, SimpleQueue, _AssertRaisesContext, SplitResult, ParseResult, WeakSet, ReferenceType, ref, ShareableList, Future, _WorkItem, Morsel, DictReader, DictWriter, array]
from typing import List, Dict, Callable
D0 = dict[str, int]

assert D0.__args__ == (str, int)

assert D0.__parameters__ == ()
D1a = dict[str, V]

assert D1a.__args__ == (str, V)

assert D1a.__parameters__ == (V,)
D1b = dict[K, int]

assert D1b.__args__ == (K, int)

assert D1b.__parameters__ == (K,)
D2a = dict[K, V]

assert D2a.__args__ == (K, V)

assert D2a.__parameters__ == (K, V)
D2b = dict[T, T]

assert D2b.__args__ == (T, T)

assert D2b.__parameters__ == (T,)
L0 = list[str]

assert L0.__args__ == (str,)

assert L0.__parameters__ == ()
L1 = list[T]

assert L1.__args__ == (T,)

assert L1.__parameters__ == (T,)
L2 = list[list[T]]

assert L2.__args__ == (list[T],)

assert L2.__parameters__ == (T,)
L3 = list[List[T]]

assert L3.__args__ == (List[T],)

assert L3.__parameters__ == (T,)
L4a = list[Dict[K, V]]

assert L4a.__args__ == (Dict[K, V],)

assert L4a.__parameters__ == (K, V)
L4b = list[Dict[T, int]]

assert L4b.__args__ == (Dict[T, int],)

assert L4b.__parameters__ == (T,)
L5 = list[Callable[[K, V], K]]

assert L5.__args__ == (Callable[[K, V], K],)

assert L5.__parameters__ == (K, V)
T1 = tuple[*tuple[int],]

assert T1.__args__ == (*tuple[int],)

assert T1.__parameters__ == ()
T2 = tuple[*tuple[T],]

assert T2.__args__ == (*tuple[T],)

assert T2.__parameters__ == (T,)
T4 = tuple[*tuple[int, str],]

assert T4.__args__ == (*tuple[int, str],)

assert T4.__parameters__ == ()
print("BaseTest::test_parameters: ok")
