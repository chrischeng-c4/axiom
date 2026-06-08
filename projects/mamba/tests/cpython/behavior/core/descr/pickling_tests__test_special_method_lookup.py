# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "descr"
# dimension = "behavior"
# case = "pickling_tests__test_special_method_lookup"
# subject = "cpython.test_descr.PicklingTests.test_special_method_lookup"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_descr.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import builtins
import copyreg
import gc
import itertools
import math
import pickle
import random
import string
import sys
import types
import warnings
import weakref
from copy import deepcopy
from contextlib import redirect_stdout

def _check_reduce(proto, obj, args=(), kwargs={}, state=None, listitems=None, dictitems=None):
    if proto >= 2:
        reduce_value = obj.__reduce_ex__(proto)
        if kwargs:
            assert reduce_value[0] == copyreg.__newobj_ex__
            assert reduce_value[1] == (type(obj), args, kwargs)
        else:
            assert reduce_value[0] == copyreg.__newobj__
            assert reduce_value[1] == (type(obj),) + args
        assert reduce_value[2] == state
        if listitems is not None:
            assert list(reduce_value[3]) == listitems
        else:
            assert reduce_value[3] is None
        if dictitems is not None:
            assert dict(reduce_value[4]) == dictitems
        else:
            assert reduce_value[4] is None
    else:
        base_type = type(obj).__base__
        reduce_value = (copyreg._reconstructor, (type(obj), base_type, None if base_type is object else base_type(obj)))
        if state is not None:
            reduce_value += (state,)
        assert obj.__reduce_ex__(proto) == reduce_value
        assert obj.__reduce__() == reduce_value

def _assert_is_copy(obj, objcopy, msg=None):
    """Utility method to verify if two objects are copies of each others.
        """
    if msg is None:
        msg = '{!r} is not a copy of {!r}'.format(obj, objcopy)
    if type(obj).__repr__ is object.__repr__:
        raise ValueError('object passed to _assert_is_copy must ' + 'override the __repr__ method.')
    assert obj is not objcopy
    assert type(obj) is type(objcopy)
    if hasattr(obj, '__dict__'):
        assert obj.__dict__ == objcopy.__dict__
        assert obj.__dict__ is not objcopy.__dict__
    if hasattr(obj, '__slots__'):
        assert obj.__slots__ == objcopy.__slots__
        for slot in obj.__slots__:
            assert hasattr(obj, slot) == hasattr(objcopy, slot)
            assert getattr(obj, slot, None) == getattr(objcopy, slot, None)
    assert repr(obj) == repr(objcopy)
protocols = range(pickle.HIGHEST_PROTOCOL + 1)

class Picky:

    def __getstate__(self):
        return {}

    def __getattr__(self, attr):
        if attr in ('__getnewargs__', '__getnewargs_ex__'):
            raise AssertionError(attr)
        return None
for protocol in protocols:
    state = {} if protocol >= 2 else None
    _check_reduce(protocol, Picky(), state=state)

print("PicklingTests::test_special_method_lookup: ok")
