use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/iterlen/test_dict_items__test_invariant.py`.
#[test]
fn test_gen_behavior_std_libs_iterlen_test_dict_items__test_invariant() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "iterlen"
# dimension = "behavior"
# case = "test_dict_items__test_invariant"
# subject = "cpython.test_iterlen.TestDictItems.test_invariant"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_iterlen.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_iterlen.py::TestDictItems::test_invariant
"""Auto-ported test: TestDictItems::test_invariant (CPython 3.12 oracle)."""


import unittest
from itertools import repeat
from collections import deque
from operator import length_hint


" Test Iterator Length Transparency\n\nSome functions or methods which accept general iterable arguments have\noptional, more efficient code paths if they know how many items to expect.\nFor instance, map(func, iterable), will pre-allocate the exact amount of\nspace required whenever the iterable can report its length.\n\nThe desired invariant is:  len(it)==len(list(it)).\n\nA complication is that an iterable and iterator can be the same object. To\nmaintain the invariant, an iterator needs to dynamically update its length.\nFor instance, an iterable such as range(10) always reports its length as ten,\nbut it=iter(range(10)) starts at ten, and then goes to nine after next(it).\nHaving this capability means that map() can ignore the distinction between\nmap(func, iterable) and map(func, iter(iterable)).\n\nWhen the iterable is immutable, the implementation can straight-forwardly\nreport the original length minus the cumulative number of calls to next().\nThis is the case for tuples, range objects, and itertools.repeat().\n\nSome containers become temporarily immutable during iteration.  This includes\ndicts, sets, and collections.deque.  Their implementation is equally simple\nthough they need to permanently set their length to zero whenever there is\nan attempt to iterate after a length mutation.\n\nThe situation slightly more involved whenever an object allows length mutation\nduring iteration.  Lists and sequence iterators are dynamically updatable.\nSo, if a list is extended during iteration, the iterator will continue through\nthe new items.  If it shrinks to a point before the most recent iteration,\nthen no further items are available and the length is reported at zero.\n\nReversed objects can also be wrapped around mutable objects; however, any\nappends after the current position are ignored.  Any other approach leads\nto confusion and possibly returning the same item more than once.\n\nThe iterators not listed above, such as enumerate and the other itertools,\nare not length transparent because they have no way to distinguish between\niterables that report static length and iterators whose length changes with\neach call (i.e. the difference between enumerate('abc') and\nenumerate(iter('abc')).\n\n"

n = 10

class BadLen(object):

    def __iter__(self):
        return iter(range(10))

    def __len__(self):
        raise RuntimeError('hello')

class BadLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        raise RuntimeError('hello')

class NoneLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        return NotImplemented


# --- test body ---
d = dict.fromkeys(range(n))
self_it = iter(d.items())
self_mutate = d.popitem
it = self_it
for i in reversed(range(1, n + 1)):

    assert length_hint(it) == i
    next(it)

assert length_hint(it) == 0

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass

assert length_hint(it) == 0
print("TestDictItems::test_invariant: ok")
"###);
    assert_output(&out, r###"TestDictItems::test_invariant: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/iterlen/test_dict_keys__test_invariant.py`.
#[test]
fn test_gen_behavior_std_libs_iterlen_test_dict_keys__test_invariant() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "iterlen"
# dimension = "behavior"
# case = "test_dict_keys__test_invariant"
# subject = "cpython.test_iterlen.TestDictKeys.test_invariant"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_iterlen.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_iterlen.py::TestDictKeys::test_invariant
"""Auto-ported test: TestDictKeys::test_invariant (CPython 3.12 oracle)."""


import unittest
from itertools import repeat
from collections import deque
from operator import length_hint


" Test Iterator Length Transparency\n\nSome functions or methods which accept general iterable arguments have\noptional, more efficient code paths if they know how many items to expect.\nFor instance, map(func, iterable), will pre-allocate the exact amount of\nspace required whenever the iterable can report its length.\n\nThe desired invariant is:  len(it)==len(list(it)).\n\nA complication is that an iterable and iterator can be the same object. To\nmaintain the invariant, an iterator needs to dynamically update its length.\nFor instance, an iterable such as range(10) always reports its length as ten,\nbut it=iter(range(10)) starts at ten, and then goes to nine after next(it).\nHaving this capability means that map() can ignore the distinction between\nmap(func, iterable) and map(func, iter(iterable)).\n\nWhen the iterable is immutable, the implementation can straight-forwardly\nreport the original length minus the cumulative number of calls to next().\nThis is the case for tuples, range objects, and itertools.repeat().\n\nSome containers become temporarily immutable during iteration.  This includes\ndicts, sets, and collections.deque.  Their implementation is equally simple\nthough they need to permanently set their length to zero whenever there is\nan attempt to iterate after a length mutation.\n\nThe situation slightly more involved whenever an object allows length mutation\nduring iteration.  Lists and sequence iterators are dynamically updatable.\nSo, if a list is extended during iteration, the iterator will continue through\nthe new items.  If it shrinks to a point before the most recent iteration,\nthen no further items are available and the length is reported at zero.\n\nReversed objects can also be wrapped around mutable objects; however, any\nappends after the current position are ignored.  Any other approach leads\nto confusion and possibly returning the same item more than once.\n\nThe iterators not listed above, such as enumerate and the other itertools,\nare not length transparent because they have no way to distinguish between\niterables that report static length and iterators whose length changes with\neach call (i.e. the difference between enumerate('abc') and\nenumerate(iter('abc')).\n\n"

n = 10

class BadLen(object):

    def __iter__(self):
        return iter(range(10))

    def __len__(self):
        raise RuntimeError('hello')

class BadLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        raise RuntimeError('hello')

class NoneLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        return NotImplemented


# --- test body ---
d = dict.fromkeys(range(n))
self_it = iter(d)
self_mutate = d.popitem
it = self_it
for i in reversed(range(1, n + 1)):

    assert length_hint(it) == i
    next(it)

assert length_hint(it) == 0

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass

assert length_hint(it) == 0
print("TestDictKeys::test_invariant: ok")
"###);
    assert_output(&out, r###"TestDictKeys::test_invariant: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/iterlen/test_dict_values__test_invariant.py`.
#[test]
fn test_gen_behavior_std_libs_iterlen_test_dict_values__test_invariant() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "iterlen"
# dimension = "behavior"
# case = "test_dict_values__test_invariant"
# subject = "cpython.test_iterlen.TestDictValues.test_invariant"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_iterlen.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_iterlen.py::TestDictValues::test_invariant
"""Auto-ported test: TestDictValues::test_invariant (CPython 3.12 oracle)."""


import unittest
from itertools import repeat
from collections import deque
from operator import length_hint


" Test Iterator Length Transparency\n\nSome functions or methods which accept general iterable arguments have\noptional, more efficient code paths if they know how many items to expect.\nFor instance, map(func, iterable), will pre-allocate the exact amount of\nspace required whenever the iterable can report its length.\n\nThe desired invariant is:  len(it)==len(list(it)).\n\nA complication is that an iterable and iterator can be the same object. To\nmaintain the invariant, an iterator needs to dynamically update its length.\nFor instance, an iterable such as range(10) always reports its length as ten,\nbut it=iter(range(10)) starts at ten, and then goes to nine after next(it).\nHaving this capability means that map() can ignore the distinction between\nmap(func, iterable) and map(func, iter(iterable)).\n\nWhen the iterable is immutable, the implementation can straight-forwardly\nreport the original length minus the cumulative number of calls to next().\nThis is the case for tuples, range objects, and itertools.repeat().\n\nSome containers become temporarily immutable during iteration.  This includes\ndicts, sets, and collections.deque.  Their implementation is equally simple\nthough they need to permanently set their length to zero whenever there is\nan attempt to iterate after a length mutation.\n\nThe situation slightly more involved whenever an object allows length mutation\nduring iteration.  Lists and sequence iterators are dynamically updatable.\nSo, if a list is extended during iteration, the iterator will continue through\nthe new items.  If it shrinks to a point before the most recent iteration,\nthen no further items are available and the length is reported at zero.\n\nReversed objects can also be wrapped around mutable objects; however, any\nappends after the current position are ignored.  Any other approach leads\nto confusion and possibly returning the same item more than once.\n\nThe iterators not listed above, such as enumerate and the other itertools,\nare not length transparent because they have no way to distinguish between\niterables that report static length and iterators whose length changes with\neach call (i.e. the difference between enumerate('abc') and\nenumerate(iter('abc')).\n\n"

n = 10

class BadLen(object):

    def __iter__(self):
        return iter(range(10))

    def __len__(self):
        raise RuntimeError('hello')

class BadLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        raise RuntimeError('hello')

class NoneLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        return NotImplemented


# --- test body ---
d = dict.fromkeys(range(n))
self_it = iter(d.values())
self_mutate = d.popitem
it = self_it
for i in reversed(range(1, n + 1)):

    assert length_hint(it) == i
    next(it)

assert length_hint(it) == 0

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass

assert length_hint(it) == 0
print("TestDictValues::test_invariant: ok")
"###);
    assert_output(&out, r###"TestDictValues::test_invariant: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/iterlen/test_length_hint_exceptions__test_invalid_hint.py`.
#[test]
fn test_gen_behavior_std_libs_iterlen_test_length_hint_exceptions__test_invalid_hint() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "iterlen"
# dimension = "behavior"
# case = "test_length_hint_exceptions__test_invalid_hint"
# subject = "cpython.test_iterlen.TestLengthHintExceptions.test_invalid_hint"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_iterlen.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_iterlen.py::TestLengthHintExceptions::test_invalid_hint
"""Auto-ported test: TestLengthHintExceptions::test_invalid_hint (CPython 3.12 oracle)."""


import unittest
from itertools import repeat
from collections import deque
from operator import length_hint


" Test Iterator Length Transparency\n\nSome functions or methods which accept general iterable arguments have\noptional, more efficient code paths if they know how many items to expect.\nFor instance, map(func, iterable), will pre-allocate the exact amount of\nspace required whenever the iterable can report its length.\n\nThe desired invariant is:  len(it)==len(list(it)).\n\nA complication is that an iterable and iterator can be the same object. To\nmaintain the invariant, an iterator needs to dynamically update its length.\nFor instance, an iterable such as range(10) always reports its length as ten,\nbut it=iter(range(10)) starts at ten, and then goes to nine after next(it).\nHaving this capability means that map() can ignore the distinction between\nmap(func, iterable) and map(func, iter(iterable)).\n\nWhen the iterable is immutable, the implementation can straight-forwardly\nreport the original length minus the cumulative number of calls to next().\nThis is the case for tuples, range objects, and itertools.repeat().\n\nSome containers become temporarily immutable during iteration.  This includes\ndicts, sets, and collections.deque.  Their implementation is equally simple\nthough they need to permanently set their length to zero whenever there is\nan attempt to iterate after a length mutation.\n\nThe situation slightly more involved whenever an object allows length mutation\nduring iteration.  Lists and sequence iterators are dynamically updatable.\nSo, if a list is extended during iteration, the iterator will continue through\nthe new items.  If it shrinks to a point before the most recent iteration,\nthen no further items are available and the length is reported at zero.\n\nReversed objects can also be wrapped around mutable objects; however, any\nappends after the current position are ignored.  Any other approach leads\nto confusion and possibly returning the same item more than once.\n\nThe iterators not listed above, such as enumerate and the other itertools,\nare not length transparent because they have no way to distinguish between\niterables that report static length and iterators whose length changes with\neach call (i.e. the difference between enumerate('abc') and\nenumerate(iter('abc')).\n\n"

n = 10

class BadLen(object):

    def __iter__(self):
        return iter(range(10))

    def __len__(self):
        raise RuntimeError('hello')

class BadLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        raise RuntimeError('hello')

class NoneLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        return NotImplemented


# --- test body ---

assert list(NoneLengthHint()) == list(range(10))
print("TestLengthHintExceptions::test_invalid_hint: ok")
"###);
    assert_output(&out, r###"TestLengthHintExceptions::test_invalid_hint: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/iterlen/test_list__test_invariant.py`.
#[test]
fn test_gen_behavior_std_libs_iterlen_test_list__test_invariant() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "iterlen"
# dimension = "behavior"
# case = "test_list__test_invariant"
# subject = "cpython.test_iterlen.TestList.test_invariant"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_iterlen.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_iterlen.py::TestList::test_invariant
"""Auto-ported test: TestList::test_invariant (CPython 3.12 oracle)."""


import unittest
from itertools import repeat
from collections import deque
from operator import length_hint


" Test Iterator Length Transparency\n\nSome functions or methods which accept general iterable arguments have\noptional, more efficient code paths if they know how many items to expect.\nFor instance, map(func, iterable), will pre-allocate the exact amount of\nspace required whenever the iterable can report its length.\n\nThe desired invariant is:  len(it)==len(list(it)).\n\nA complication is that an iterable and iterator can be the same object. To\nmaintain the invariant, an iterator needs to dynamically update its length.\nFor instance, an iterable such as range(10) always reports its length as ten,\nbut it=iter(range(10)) starts at ten, and then goes to nine after next(it).\nHaving this capability means that map() can ignore the distinction between\nmap(func, iterable) and map(func, iter(iterable)).\n\nWhen the iterable is immutable, the implementation can straight-forwardly\nreport the original length minus the cumulative number of calls to next().\nThis is the case for tuples, range objects, and itertools.repeat().\n\nSome containers become temporarily immutable during iteration.  This includes\ndicts, sets, and collections.deque.  Their implementation is equally simple\nthough they need to permanently set their length to zero whenever there is\nan attempt to iterate after a length mutation.\n\nThe situation slightly more involved whenever an object allows length mutation\nduring iteration.  Lists and sequence iterators are dynamically updatable.\nSo, if a list is extended during iteration, the iterator will continue through\nthe new items.  If it shrinks to a point before the most recent iteration,\nthen no further items are available and the length is reported at zero.\n\nReversed objects can also be wrapped around mutable objects; however, any\nappends after the current position are ignored.  Any other approach leads\nto confusion and possibly returning the same item more than once.\n\nThe iterators not listed above, such as enumerate and the other itertools,\nare not length transparent because they have no way to distinguish between\niterables that report static length and iterators whose length changes with\neach call (i.e. the difference between enumerate('abc') and\nenumerate(iter('abc')).\n\n"

n = 10

class BadLen(object):

    def __iter__(self):
        return iter(range(10))

    def __len__(self):
        raise RuntimeError('hello')

class BadLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        raise RuntimeError('hello')

class NoneLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        return NotImplemented


# --- test body ---
self_it = iter(range(n))
it = self_it
for i in reversed(range(1, n + 1)):

    assert length_hint(it) == i
    next(it)

assert length_hint(it) == 0

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass

assert length_hint(it) == 0
print("TestList::test_invariant: ok")
"###);
    assert_output(&out, r###"TestList::test_invariant: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/iterlen/test_list_reversed__test_invariant.py`.
#[test]
fn test_gen_behavior_std_libs_iterlen_test_list_reversed__test_invariant() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "iterlen"
# dimension = "behavior"
# case = "test_list_reversed__test_invariant"
# subject = "cpython.test_iterlen.TestListReversed.test_invariant"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_iterlen.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_iterlen.py::TestListReversed::test_invariant
"""Auto-ported test: TestListReversed::test_invariant (CPython 3.12 oracle)."""


import unittest
from itertools import repeat
from collections import deque
from operator import length_hint


" Test Iterator Length Transparency\n\nSome functions or methods which accept general iterable arguments have\noptional, more efficient code paths if they know how many items to expect.\nFor instance, map(func, iterable), will pre-allocate the exact amount of\nspace required whenever the iterable can report its length.\n\nThe desired invariant is:  len(it)==len(list(it)).\n\nA complication is that an iterable and iterator can be the same object. To\nmaintain the invariant, an iterator needs to dynamically update its length.\nFor instance, an iterable such as range(10) always reports its length as ten,\nbut it=iter(range(10)) starts at ten, and then goes to nine after next(it).\nHaving this capability means that map() can ignore the distinction between\nmap(func, iterable) and map(func, iter(iterable)).\n\nWhen the iterable is immutable, the implementation can straight-forwardly\nreport the original length minus the cumulative number of calls to next().\nThis is the case for tuples, range objects, and itertools.repeat().\n\nSome containers become temporarily immutable during iteration.  This includes\ndicts, sets, and collections.deque.  Their implementation is equally simple\nthough they need to permanently set their length to zero whenever there is\nan attempt to iterate after a length mutation.\n\nThe situation slightly more involved whenever an object allows length mutation\nduring iteration.  Lists and sequence iterators are dynamically updatable.\nSo, if a list is extended during iteration, the iterator will continue through\nthe new items.  If it shrinks to a point before the most recent iteration,\nthen no further items are available and the length is reported at zero.\n\nReversed objects can also be wrapped around mutable objects; however, any\nappends after the current position are ignored.  Any other approach leads\nto confusion and possibly returning the same item more than once.\n\nThe iterators not listed above, such as enumerate and the other itertools,\nare not length transparent because they have no way to distinguish between\niterables that report static length and iterators whose length changes with\neach call (i.e. the difference between enumerate('abc') and\nenumerate(iter('abc')).\n\n"

n = 10

class BadLen(object):

    def __iter__(self):
        return iter(range(10))

    def __len__(self):
        raise RuntimeError('hello')

class BadLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        raise RuntimeError('hello')

class NoneLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        return NotImplemented


# --- test body ---
self_it = reversed(range(n))
it = self_it
for i in reversed(range(1, n + 1)):

    assert length_hint(it) == i
    next(it)

assert length_hint(it) == 0

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass

assert length_hint(it) == 0
print("TestListReversed::test_invariant: ok")
"###);
    assert_output(&out, r###"TestListReversed::test_invariant: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/iterlen/test_repeat__test_invariant.py`.
#[test]
fn test_gen_behavior_std_libs_iterlen_test_repeat__test_invariant() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "iterlen"
# dimension = "behavior"
# case = "test_repeat__test_invariant"
# subject = "cpython.test_iterlen.TestRepeat.test_invariant"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_iterlen.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_iterlen.py::TestRepeat::test_invariant
"""Auto-ported test: TestRepeat::test_invariant (CPython 3.12 oracle)."""


import unittest
from itertools import repeat
from collections import deque
from operator import length_hint


" Test Iterator Length Transparency\n\nSome functions or methods which accept general iterable arguments have\noptional, more efficient code paths if they know how many items to expect.\nFor instance, map(func, iterable), will pre-allocate the exact amount of\nspace required whenever the iterable can report its length.\n\nThe desired invariant is:  len(it)==len(list(it)).\n\nA complication is that an iterable and iterator can be the same object. To\nmaintain the invariant, an iterator needs to dynamically update its length.\nFor instance, an iterable such as range(10) always reports its length as ten,\nbut it=iter(range(10)) starts at ten, and then goes to nine after next(it).\nHaving this capability means that map() can ignore the distinction between\nmap(func, iterable) and map(func, iter(iterable)).\n\nWhen the iterable is immutable, the implementation can straight-forwardly\nreport the original length minus the cumulative number of calls to next().\nThis is the case for tuples, range objects, and itertools.repeat().\n\nSome containers become temporarily immutable during iteration.  This includes\ndicts, sets, and collections.deque.  Their implementation is equally simple\nthough they need to permanently set their length to zero whenever there is\nan attempt to iterate after a length mutation.\n\nThe situation slightly more involved whenever an object allows length mutation\nduring iteration.  Lists and sequence iterators are dynamically updatable.\nSo, if a list is extended during iteration, the iterator will continue through\nthe new items.  If it shrinks to a point before the most recent iteration,\nthen no further items are available and the length is reported at zero.\n\nReversed objects can also be wrapped around mutable objects; however, any\nappends after the current position are ignored.  Any other approach leads\nto confusion and possibly returning the same item more than once.\n\nThe iterators not listed above, such as enumerate and the other itertools,\nare not length transparent because they have no way to distinguish between\niterables that report static length and iterators whose length changes with\neach call (i.e. the difference between enumerate('abc') and\nenumerate(iter('abc')).\n\n"

n = 10

class BadLen(object):

    def __iter__(self):
        return iter(range(10))

    def __len__(self):
        raise RuntimeError('hello')

class BadLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        raise RuntimeError('hello')

class NoneLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        return NotImplemented


# --- test body ---
self_it = repeat(None, n)
it = self_it
for i in reversed(range(1, n + 1)):

    assert length_hint(it) == i
    next(it)

assert length_hint(it) == 0

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass

assert length_hint(it) == 0
print("TestRepeat::test_invariant: ok")
"###);
    assert_output(&out, r###"TestRepeat::test_invariant: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/iterlen/test_set__test_invariant.py`.
#[test]
fn test_gen_behavior_std_libs_iterlen_test_set__test_invariant() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "iterlen"
# dimension = "behavior"
# case = "test_set__test_invariant"
# subject = "cpython.test_iterlen.TestSet.test_invariant"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_iterlen.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_iterlen.py::TestSet::test_invariant
"""Auto-ported test: TestSet::test_invariant (CPython 3.12 oracle)."""


import unittest
from itertools import repeat
from collections import deque
from operator import length_hint


" Test Iterator Length Transparency\n\nSome functions or methods which accept general iterable arguments have\noptional, more efficient code paths if they know how many items to expect.\nFor instance, map(func, iterable), will pre-allocate the exact amount of\nspace required whenever the iterable can report its length.\n\nThe desired invariant is:  len(it)==len(list(it)).\n\nA complication is that an iterable and iterator can be the same object. To\nmaintain the invariant, an iterator needs to dynamically update its length.\nFor instance, an iterable such as range(10) always reports its length as ten,\nbut it=iter(range(10)) starts at ten, and then goes to nine after next(it).\nHaving this capability means that map() can ignore the distinction between\nmap(func, iterable) and map(func, iter(iterable)).\n\nWhen the iterable is immutable, the implementation can straight-forwardly\nreport the original length minus the cumulative number of calls to next().\nThis is the case for tuples, range objects, and itertools.repeat().\n\nSome containers become temporarily immutable during iteration.  This includes\ndicts, sets, and collections.deque.  Their implementation is equally simple\nthough they need to permanently set their length to zero whenever there is\nan attempt to iterate after a length mutation.\n\nThe situation slightly more involved whenever an object allows length mutation\nduring iteration.  Lists and sequence iterators are dynamically updatable.\nSo, if a list is extended during iteration, the iterator will continue through\nthe new items.  If it shrinks to a point before the most recent iteration,\nthen no further items are available and the length is reported at zero.\n\nReversed objects can also be wrapped around mutable objects; however, any\nappends after the current position are ignored.  Any other approach leads\nto confusion and possibly returning the same item more than once.\n\nThe iterators not listed above, such as enumerate and the other itertools,\nare not length transparent because they have no way to distinguish between\niterables that report static length and iterators whose length changes with\neach call (i.e. the difference between enumerate('abc') and\nenumerate(iter('abc')).\n\n"

n = 10

class BadLen(object):

    def __iter__(self):
        return iter(range(10))

    def __len__(self):
        raise RuntimeError('hello')

class BadLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        raise RuntimeError('hello')

class NoneLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        return NotImplemented


# --- test body ---
d = set(range(n))
self_it = iter(d)
self_mutate = d.pop
it = self_it
for i in reversed(range(1, n + 1)):

    assert length_hint(it) == i
    next(it)

assert length_hint(it) == 0

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass

assert length_hint(it) == 0
print("TestSet::test_invariant: ok")
"###);
    assert_output(&out, r###"TestSet::test_invariant: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/iterlen/test_tuple__test_invariant.py`.
#[test]
fn test_gen_behavior_std_libs_iterlen_test_tuple__test_invariant() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "iterlen"
# dimension = "behavior"
# case = "test_tuple__test_invariant"
# subject = "cpython.test_iterlen.TestTuple.test_invariant"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_iterlen.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_iterlen.py::TestTuple::test_invariant
"""Auto-ported test: TestTuple::test_invariant (CPython 3.12 oracle)."""


import unittest
from itertools import repeat
from collections import deque
from operator import length_hint


" Test Iterator Length Transparency\n\nSome functions or methods which accept general iterable arguments have\noptional, more efficient code paths if they know how many items to expect.\nFor instance, map(func, iterable), will pre-allocate the exact amount of\nspace required whenever the iterable can report its length.\n\nThe desired invariant is:  len(it)==len(list(it)).\n\nA complication is that an iterable and iterator can be the same object. To\nmaintain the invariant, an iterator needs to dynamically update its length.\nFor instance, an iterable such as range(10) always reports its length as ten,\nbut it=iter(range(10)) starts at ten, and then goes to nine after next(it).\nHaving this capability means that map() can ignore the distinction between\nmap(func, iterable) and map(func, iter(iterable)).\n\nWhen the iterable is immutable, the implementation can straight-forwardly\nreport the original length minus the cumulative number of calls to next().\nThis is the case for tuples, range objects, and itertools.repeat().\n\nSome containers become temporarily immutable during iteration.  This includes\ndicts, sets, and collections.deque.  Their implementation is equally simple\nthough they need to permanently set their length to zero whenever there is\nan attempt to iterate after a length mutation.\n\nThe situation slightly more involved whenever an object allows length mutation\nduring iteration.  Lists and sequence iterators are dynamically updatable.\nSo, if a list is extended during iteration, the iterator will continue through\nthe new items.  If it shrinks to a point before the most recent iteration,\nthen no further items are available and the length is reported at zero.\n\nReversed objects can also be wrapped around mutable objects; however, any\nappends after the current position are ignored.  Any other approach leads\nto confusion and possibly returning the same item more than once.\n\nThe iterators not listed above, such as enumerate and the other itertools,\nare not length transparent because they have no way to distinguish between\niterables that report static length and iterators whose length changes with\neach call (i.e. the difference between enumerate('abc') and\nenumerate(iter('abc')).\n\n"

n = 10

class BadLen(object):

    def __iter__(self):
        return iter(range(10))

    def __len__(self):
        raise RuntimeError('hello')

class BadLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        raise RuntimeError('hello')

class NoneLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        return NotImplemented


# --- test body ---
self_it = iter(tuple(range(n)))
it = self_it
for i in reversed(range(1, n + 1)):

    assert length_hint(it) == i
    next(it)

assert length_hint(it) == 0

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass

assert length_hint(it) == 0
print("TestTuple::test_invariant: ok")
"###);
    assert_output(&out, r###"TestTuple::test_invariant: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/iterlen/test_xrange__test_invariant.py`.
#[test]
fn test_gen_behavior_std_libs_iterlen_test_xrange__test_invariant() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "iterlen"
# dimension = "behavior"
# case = "test_xrange__test_invariant"
# subject = "cpython.test_iterlen.TestXrange.test_invariant"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_iterlen.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_iterlen.py::TestXrange::test_invariant
"""Auto-ported test: TestXrange::test_invariant (CPython 3.12 oracle)."""


import unittest
from itertools import repeat
from collections import deque
from operator import length_hint


" Test Iterator Length Transparency\n\nSome functions or methods which accept general iterable arguments have\noptional, more efficient code paths if they know how many items to expect.\nFor instance, map(func, iterable), will pre-allocate the exact amount of\nspace required whenever the iterable can report its length.\n\nThe desired invariant is:  len(it)==len(list(it)).\n\nA complication is that an iterable and iterator can be the same object. To\nmaintain the invariant, an iterator needs to dynamically update its length.\nFor instance, an iterable such as range(10) always reports its length as ten,\nbut it=iter(range(10)) starts at ten, and then goes to nine after next(it).\nHaving this capability means that map() can ignore the distinction between\nmap(func, iterable) and map(func, iter(iterable)).\n\nWhen the iterable is immutable, the implementation can straight-forwardly\nreport the original length minus the cumulative number of calls to next().\nThis is the case for tuples, range objects, and itertools.repeat().\n\nSome containers become temporarily immutable during iteration.  This includes\ndicts, sets, and collections.deque.  Their implementation is equally simple\nthough they need to permanently set their length to zero whenever there is\nan attempt to iterate after a length mutation.\n\nThe situation slightly more involved whenever an object allows length mutation\nduring iteration.  Lists and sequence iterators are dynamically updatable.\nSo, if a list is extended during iteration, the iterator will continue through\nthe new items.  If it shrinks to a point before the most recent iteration,\nthen no further items are available and the length is reported at zero.\n\nReversed objects can also be wrapped around mutable objects; however, any\nappends after the current position are ignored.  Any other approach leads\nto confusion and possibly returning the same item more than once.\n\nThe iterators not listed above, such as enumerate and the other itertools,\nare not length transparent because they have no way to distinguish between\niterables that report static length and iterators whose length changes with\neach call (i.e. the difference between enumerate('abc') and\nenumerate(iter('abc')).\n\n"

n = 10

class BadLen(object):

    def __iter__(self):
        return iter(range(10))

    def __len__(self):
        raise RuntimeError('hello')

class BadLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        raise RuntimeError('hello')

class NoneLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        return NotImplemented


# --- test body ---
self_it = iter(range(n))
it = self_it
for i in reversed(range(1, n + 1)):

    assert length_hint(it) == i
    next(it)

assert length_hint(it) == 0

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass

assert length_hint(it) == 0
print("TestXrange::test_invariant: ok")
"###);
    assert_output(&out, r###"TestXrange::test_invariant: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/iterlen/test_xrange_custom_reversed__test_invariant.py`.
#[test]
fn test_gen_behavior_std_libs_iterlen_test_xrange_custom_reversed__test_invariant() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "iterlen"
# dimension = "behavior"
# case = "test_xrange_custom_reversed__test_invariant"
# subject = "cpython.test_iterlen.TestXrangeCustomReversed.test_invariant"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_iterlen.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_iterlen.py::TestXrangeCustomReversed::test_invariant
"""Auto-ported test: TestXrangeCustomReversed::test_invariant (CPython 3.12 oracle)."""


import unittest
from itertools import repeat
from collections import deque
from operator import length_hint


" Test Iterator Length Transparency\n\nSome functions or methods which accept general iterable arguments have\noptional, more efficient code paths if they know how many items to expect.\nFor instance, map(func, iterable), will pre-allocate the exact amount of\nspace required whenever the iterable can report its length.\n\nThe desired invariant is:  len(it)==len(list(it)).\n\nA complication is that an iterable and iterator can be the same object. To\nmaintain the invariant, an iterator needs to dynamically update its length.\nFor instance, an iterable such as range(10) always reports its length as ten,\nbut it=iter(range(10)) starts at ten, and then goes to nine after next(it).\nHaving this capability means that map() can ignore the distinction between\nmap(func, iterable) and map(func, iter(iterable)).\n\nWhen the iterable is immutable, the implementation can straight-forwardly\nreport the original length minus the cumulative number of calls to next().\nThis is the case for tuples, range objects, and itertools.repeat().\n\nSome containers become temporarily immutable during iteration.  This includes\ndicts, sets, and collections.deque.  Their implementation is equally simple\nthough they need to permanently set their length to zero whenever there is\nan attempt to iterate after a length mutation.\n\nThe situation slightly more involved whenever an object allows length mutation\nduring iteration.  Lists and sequence iterators are dynamically updatable.\nSo, if a list is extended during iteration, the iterator will continue through\nthe new items.  If it shrinks to a point before the most recent iteration,\nthen no further items are available and the length is reported at zero.\n\nReversed objects can also be wrapped around mutable objects; however, any\nappends after the current position are ignored.  Any other approach leads\nto confusion and possibly returning the same item more than once.\n\nThe iterators not listed above, such as enumerate and the other itertools,\nare not length transparent because they have no way to distinguish between\niterables that report static length and iterators whose length changes with\neach call (i.e. the difference between enumerate('abc') and\nenumerate(iter('abc')).\n\n"

n = 10

class BadLen(object):

    def __iter__(self):
        return iter(range(10))

    def __len__(self):
        raise RuntimeError('hello')

class BadLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        raise RuntimeError('hello')

class NoneLengthHint(object):

    def __iter__(self):
        return iter(range(10))

    def __length_hint__(self):
        return NotImplemented


# --- test body ---
self_it = reversed(range(n))
it = self_it
for i in reversed(range(1, n + 1)):

    assert length_hint(it) == i
    next(it)

assert length_hint(it) == 0

try:
    next(it)
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass

assert length_hint(it) == 0
print("TestXrangeCustomReversed::test_invariant: ok")
"###);
    assert_output(&out, r###"TestXrangeCustomReversed::test_invariant: ok
"###);
}
