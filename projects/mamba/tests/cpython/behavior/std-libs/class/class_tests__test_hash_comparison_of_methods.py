# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "class"
# dimension = "behavior"
# case = "class_tests__test_hash_comparison_of_methods"
# subject = "cpython.test_class.ClassTests.testHashComparisonOfMethods"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_class.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_class.py::ClassTests::testHashComparisonOfMethods
"""Auto-ported test: ClassTests::testHashComparisonOfMethods (CPython 3.12 oracle)."""


import unittest


'Test the functionality of Python classes implementing operators.'

testmeths = ['add', 'radd', 'sub', 'rsub', 'mul', 'rmul', 'matmul', 'rmatmul', 'truediv', 'rtruediv', 'floordiv', 'rfloordiv', 'mod', 'rmod', 'divmod', 'rdivmod', 'pow', 'rpow', 'rshift', 'rrshift', 'lshift', 'rlshift', 'and', 'rand', 'or', 'ror', 'xor', 'rxor', 'contains', 'getitem', 'setitem', 'delitem', 'neg', 'pos', 'abs', 'init']

callLst = []

def trackCall(f):

    def track(*args, **kwargs):
        callLst.append((f.__name__, args))
        return f(*args, **kwargs)
    return track

statictests = '\n@trackCall\ndef __hash__(self, *args):\n    return hash(id(self))\n\n@trackCall\ndef __str__(self, *args):\n    return "AllTests"\n\n@trackCall\ndef __repr__(self, *args):\n    return "AllTests"\n\n@trackCall\ndef __int__(self, *args):\n    return 1\n\n@trackCall\ndef __index__(self, *args):\n    return 1\n\n@trackCall\ndef __float__(self, *args):\n    return 1.0\n\n@trackCall\ndef __eq__(self, *args):\n    return True\n\n@trackCall\ndef __ne__(self, *args):\n    return False\n\n@trackCall\ndef __lt__(self, *args):\n    return False\n\n@trackCall\ndef __le__(self, *args):\n    return True\n\n@trackCall\ndef __gt__(self, *args):\n    return False\n\n@trackCall\ndef __ge__(self, *args):\n    return True\n'

method_template = '@trackCall\ndef __%s__(self, *args):\n    pass\n'

d = {}

exec(statictests, globals(), d)

for method in testmeths:
    exec(method_template % method, globals(), d)

AllTests = type('AllTests', (object,), d)

del d, statictests, method, method_template


# --- test body ---
def assertCallStack(expected_calls):
    actualCallList = callLst[:]
    if expected_calls != actualCallList:

        raise AssertionError('Expected call list:\n  %s\ndoes not match actual call list\n  %s' % (expected_calls, actualCallList))

def assertNotOrderable(a, b):
    try:
        a < b
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    try:
        a > b
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    try:
        a <= b
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    try:
        a >= b
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
callLst[:] = []

class A:

    def __init__(self, x):
        self.x = x

    def f(self):
        pass

    def g(self):
        pass

    def __eq__(self, other):
        return True

    def __hash__(self):
        raise TypeError

class B(A):
    pass
a1 = A(1)
a2 = A(1)

assert a1.f == a1.f

assert not a1.f != a1.f

assert not a1.f == a2.f

assert a1.f != a2.f

assert not a1.f == a1.g

assert a1.f != a1.g
assertNotOrderable(a1.f, a1.f)

assert hash(a1.f) == hash(a1.f)

assert not A.f == a1.f

assert A.f != a1.f

assert not A.f == A.g

assert A.f != A.g

assert B.f == A.f

assert not B.f != A.f
assertNotOrderable(A.f, A.f)

assert hash(B.f) == hash(A.f)
a = A(hash(A.f) ^ -1)
hash(a.f)
print("ClassTests::testHashComparisonOfMethods: ok")
