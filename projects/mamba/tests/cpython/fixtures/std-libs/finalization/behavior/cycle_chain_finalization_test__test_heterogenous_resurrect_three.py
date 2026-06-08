# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "finalization"
# dimension = "behavior"
# case = "cycle_chain_finalization_test__test_heterogenous_resurrect_three"
# subject = "cpython.test_finalization.CycleChainFinalizationTest.test_heterogenous_resurrect_three"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_finalization.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_finalization.py::CycleChainFinalizationTest::test_heterogenous_resurrect_three
"""Auto-ported test: CycleChainFinalizationTest::test_heterogenous_resurrect_three (CPython 3.12 oracle)."""


import contextlib
import gc
import unittest
import weakref
from test import support


'\nTests for object finalization semantics, as outlined in PEP 442.\n'

try:
    from _testcapi import with_tp_del
except ImportError:

    def with_tp_del(cls):

        class C(object):

            def __new__(cls, *args, **kwargs):
                raise TypeError('requires _testcapi.with_tp_del')
        return C

try:
    from _testcapi import without_gc
except ImportError:

    def without_gc(cls):

        class C:

            def __new__(cls, *args, **kwargs):
                raise TypeError('requires _testcapi.without_gc')
        return C

class NonGCSimpleBase:
    """
    The base class for all the objects under test, equipped with various
    testing features.
    """
    survivors = []
    del_calls = []
    tp_del_calls = []
    errors = []
    _cleaning = False
    __slots__ = ()

    @classmethod
    def _cleanup(cls):
        cls.survivors.clear()
        cls.errors.clear()
        gc.garbage.clear()
        gc.collect()
        cls.del_calls.clear()
        cls.tp_del_calls.clear()

    @classmethod
    @contextlib.contextmanager
    def test(cls):
        """
        A context manager to use around all finalization tests.
        """
        with support.disable_gc():
            cls.del_calls.clear()
            cls.tp_del_calls.clear()
            NonGCSimpleBase._cleaning = False
            try:
                yield
                if cls.errors:
                    raise cls.errors[0]
            finally:
                NonGCSimpleBase._cleaning = True
                cls._cleanup()

    def check_sanity(self):
        """
        Check the object is sane (non-broken).
        """

    def __del__(self):
        """
        PEP 442 finalizer.  Record that this was called, check the
        object is in a sane state, and invoke a side effect.
        """
        try:
            if not self._cleaning:
                self.del_calls.append(id(self))
                self.check_sanity()
                self.side_effect()
        except Exception as e:
            self.errors.append(e)

    def side_effect(self):
        """
        A side effect called on destruction.
        """

class SimpleBase(NonGCSimpleBase):

    def __init__(self):
        self.id_ = id(self)

    def check_sanity(self):
        assert self.id_ == id(self)

@without_gc
class NonGC(NonGCSimpleBase):
    __slots__ = ()

@without_gc
class NonGCResurrector(NonGCSimpleBase):
    __slots__ = ()

    def side_effect(self):
        """
        Resurrect self by storing self in a class-wide list.
        """
        self.survivors.append(self)

class Simple(SimpleBase):
    pass

class SimpleResurrector(SimpleBase):

    def side_effect(self):
        """
        Resurrect self by storing self in a class-wide list.
        """
        self.survivors.append(self)

class SelfCycleBase:

    def __init__(self):
        super().__init__()
        self.ref = self

    def check_sanity(self):
        super().check_sanity()
        assert self.ref is self

class SimpleSelfCycle(SelfCycleBase, Simple):
    pass

class SelfCycleResurrector(SelfCycleBase, SimpleResurrector):
    pass

class SuicidalSelfCycle(SelfCycleBase, Simple):

    def side_effect(self):
        """
        Explicitly break the reference cycle.
        """
        self.ref = None

class ChainedBase:

    def chain(self, left):
        self.suicided = False
        self.left = left
        left.right = self

    def check_sanity(self):
        super().check_sanity()
        if self.suicided:
            assert self.left is None
            assert self.right is None
        else:
            left = self.left
            if left.suicided:
                assert left.right is None
            else:
                assert left.right is self
            right = self.right
            if right.suicided:
                assert right.left is None
            else:
                assert right.left is self

class SimpleChained(ChainedBase, Simple):
    pass

class ChainedResurrector(ChainedBase, SimpleResurrector):
    pass

class SuicidalChained(ChainedBase, Simple):

    def side_effect(self):
        """
        Explicitly break the reference cycle.
        """
        self.suicided = True
        self.left = None
        self.right = None

class LegacyBase(SimpleBase):

    def __del__(self):
        try:
            if not self._cleaning:
                self.del_calls.append(id(self))
                self.check_sanity()
        except Exception as e:
            self.errors.append(e)

    def __tp_del__(self):
        """
        Legacy (pre-PEP 442) finalizer, mapped to a tp_del slot.
        """
        try:
            if not self._cleaning:
                self.tp_del_calls.append(id(self))
                self.check_sanity()
                self.side_effect()
        except Exception as e:
            self.errors.append(e)

@with_tp_del
class Legacy(LegacyBase):
    pass

@with_tp_del
class LegacyResurrector(LegacyBase):

    def side_effect(self):
        """
        Resurrect self by storing self in a class-wide list.
        """
        self.survivors.append(self)

@with_tp_del
class LegacySelfCycle(SelfCycleBase, LegacyBase):
    pass


# --- test body ---
def assert_del_calls(ids):

    assert sorted(SimpleBase.del_calls) == sorted(ids)

def assert_garbage(ids):

    assert sorted((id(x) for x in gc.garbage)) == sorted(ids)

def assert_survivors(ids):

    assert sorted((id(x) for x in SimpleBase.survivors)) == sorted(ids)

def assert_tp_del_calls(ids):

    assert sorted(SimpleBase.tp_del_calls) == sorted(ids)

def build_chain(classes):
    nodes = [cls() for cls in classes]
    for i in range(len(nodes)):
        nodes[i].chain(nodes[i - 1])
    return nodes

def check_non_resurrecting_chain(classes):
    N = len(classes)
    with SimpleBase.test():
        nodes = build_chain(classes)
        ids = [id(s) for s in nodes]
        wrs = [weakref.ref(s) for s in nodes]
        del nodes
        gc.collect()
        assert_del_calls(ids)
        assert_survivors([])

        assert [wr() for wr in wrs] == [None] * N
        gc.collect()
        assert_del_calls(ids)

def check_resurrecting_chain(classes):
    N = len(classes)
    with SimpleBase.test():
        nodes = build_chain(classes)
        N = len(nodes)
        ids = [id(s) for s in nodes]
        survivor_ids = [id(s) for s in nodes if isinstance(s, SimpleResurrector)]
        wrs = [weakref.ref(s) for s in nodes]
        del nodes
        gc.collect()
        assert_del_calls(ids)
        assert_survivors(survivor_ids)

        assert [wr() for wr in wrs] == [None] * N
        clear_survivors()
        gc.collect()
        assert_del_calls(ids)
        assert_survivors([])

def clear_survivors():
    SimpleBase.survivors.clear()
self_old_garbage = gc.garbage[:]
gc.garbage[:] = []
check_resurrecting_chain([ChainedResurrector] * 2 + [SimpleChained] * 2 + [SuicidalChained] * 2)
print("CycleChainFinalizationTest::test_heterogenous_resurrect_three: ok")
