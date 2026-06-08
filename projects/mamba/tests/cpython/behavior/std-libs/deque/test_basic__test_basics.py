# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "deque"
# dimension = "behavior"
# case = "test_basic__test_basics"
# subject = "cpython.test_deque.TestBasic.test_basics"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_deque.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_deque.py::TestBasic::test_basics
"""Auto-ported test: TestBasic::test_basics (CPython 3.12 oracle)."""


from collections import deque
import doctest
import unittest
from test import support, seq_tests
import gc
import weakref
import copy
import pickle
import random
import struct


BIG = 100000

def fail():
    raise SyntaxError
    yield 1

class BadCmp:

    def __eq__(self, other):
        raise RuntimeError

class MutateCmp:

    def __init__(self, deque, result):
        self.deque = deque
        self.result = result

    def __eq__(self, other):
        self.deque.clear()
        return self.result

class Deque(deque):
    pass

class DequeWithSlots(deque):
    __slots__ = ('x', 'y', '__dict__')

class DequeWithBadIter(deque):

    def __iter__(self):
        raise TypeError

class SubclassWithKwargs(deque):

    def __init__(self, newarg=1):
        deque.__init__(self)

libreftest = '\nExample from the Library Reference:  Doc/lib/libcollections.tex\n\n>>> from collections import deque\n>>> d = deque(\'ghi\')                 # make a new deque with three items\n>>> for elem in d:                   # iterate over the deque\'s elements\n...     print(elem.upper())\nG\nH\nI\n>>> d.append(\'j\')                    # add a new entry to the right side\n>>> d.appendleft(\'f\')                # add a new entry to the left side\n>>> d                                # show the representation of the deque\ndeque([\'f\', \'g\', \'h\', \'i\', \'j\'])\n>>> d.pop()                          # return and remove the rightmost item\n\'j\'\n>>> d.popleft()                      # return and remove the leftmost item\n\'f\'\n>>> list(d)                          # list the contents of the deque\n[\'g\', \'h\', \'i\']\n>>> d[0]                             # peek at leftmost item\n\'g\'\n>>> d[-1]                            # peek at rightmost item\n\'i\'\n>>> list(reversed(d))                # list the contents of a deque in reverse\n[\'i\', \'h\', \'g\']\n>>> \'h\' in d                         # search the deque\nTrue\n>>> d.extend(\'jkl\')                  # add multiple elements at once\n>>> d\ndeque([\'g\', \'h\', \'i\', \'j\', \'k\', \'l\'])\n>>> d.rotate(1)                      # right rotation\n>>> d\ndeque([\'l\', \'g\', \'h\', \'i\', \'j\', \'k\'])\n>>> d.rotate(-1)                     # left rotation\n>>> d\ndeque([\'g\', \'h\', \'i\', \'j\', \'k\', \'l\'])\n>>> deque(reversed(d))               # make a new deque in reverse order\ndeque([\'l\', \'k\', \'j\', \'i\', \'h\', \'g\'])\n>>> d.clear()                        # empty the deque\n>>> d.pop()                          # cannot pop from an empty deque\nTraceback (most recent call last):\n  File "<pyshell#6>", line 1, in -toplevel-\n    d.pop()\nIndexError: pop from an empty deque\n\n>>> d.extendleft(\'abc\')              # extendleft() reverses the input order\n>>> d\ndeque([\'c\', \'b\', \'a\'])\n\n\n\n>>> def delete_nth(d, n):\n...     d.rotate(-n)\n...     d.popleft()\n...     d.rotate(n)\n...\n>>> d = deque(\'abcdef\')\n>>> delete_nth(d, 2)   # remove the entry at d[2]\n>>> d\ndeque([\'a\', \'b\', \'d\', \'e\', \'f\'])\n\n\n\n>>> def roundrobin(*iterables):\n...     pending = deque(iter(i) for i in iterables)\n...     while pending:\n...         task = pending.popleft()\n...         try:\n...             yield next(task)\n...         except StopIteration:\n...             continue\n...         pending.append(task)\n...\n\n>>> for value in roundrobin(\'abc\', \'d\', \'efgh\'):\n...     print(value)\n...\na\nd\ne\nb\nf\nc\ng\nh\n\n\n>>> def maketree(iterable):\n...     d = deque(iterable)\n...     while len(d) > 1:\n...         pair = [d.popleft(), d.popleft()]\n...         d.append(pair)\n...     return list(d)\n...\n>>> print(maketree(\'abcdefgh\'))\n[[[[\'a\', \'b\'], [\'c\', \'d\']], [[\'e\', \'f\'], [\'g\', \'h\']]]]\n\n'

__test__ = {'libreftest': libreftest}

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite())
    return tests


# --- test body ---
check_sizeof = support.check_sizeof
d = deque(range(-5125, -5000))
d.__init__(range(200))
for i in range(200, 400):
    d.append(i)
for i in reversed(range(-200, 0)):
    d.appendleft(i)

assert list(d) == list(range(-200, 400))

assert len(d) == 600
left = [d.popleft() for i in range(250)]

assert left == list(range(-200, 50))

assert list(d) == list(range(50, 400))
right = [d.pop() for i in range(250)]
right.reverse()

assert right == list(range(150, 400))

assert list(d) == list(range(50, 150))
print("TestBasic::test_basics: ok")
