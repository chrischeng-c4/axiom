# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericalias"
# dimension = "behavior"
# case = "base_test__test_isinstance"
# subject = "cpython.test_genericalias.BaseTest.test_isinstance"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_genericalias.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_genericalias.py::BaseTest::test_isinstance
"""Auto-ported test: BaseTest::test_isinstance (CPython 3.12 oracle)."""


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

assert isinstance([], list)
try:
    isinstance([], list[str])
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("BaseTest::test_isinstance: ok")
