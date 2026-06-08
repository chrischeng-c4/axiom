# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "class"
# dimension = "behavior"
# case = "class_tests__test_predefined_attrs"
# subject = "cpython.test_class.ClassTests.testPredefinedAttrs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_class.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_class.py::ClassTests::testPredefinedAttrs
"""Auto-ported test: ClassTests::testPredefinedAttrs (CPython 3.12 oracle)."""


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
o = object()

class Custom:
    pass
c = Custom()
methods = ('__class__', '__delattr__', '__dir__', '__eq__', '__format__', '__ge__', '__getattribute__', '__getstate__', '__gt__', '__hash__', '__init__', '__init_subclass__', '__le__', '__lt__', '__ne__', '__new__', '__reduce__', '__reduce_ex__', '__repr__', '__setattr__', '__sizeof__', '__str__', '__subclasshook__')
for name in methods:

    assert callable(getattr(object, name, None))

    assert callable(getattr(o, name, None))

    assert callable(getattr(Custom, name, None))

    assert callable(getattr(c, name, None))
not_defined = ['__abs__', '__aenter__', '__aexit__', '__aiter__', '__anext__', '__await__', '__bool__', '__bytes__', '__ceil__', '__complex__', '__contains__', '__del__', '__delete__', '__delitem__', '__divmod__', '__enter__', '__exit__', '__float__', '__floor__', '__get__', '__getattr__', '__getitem__', '__index__', '__int__', '__invert__', '__iter__', '__len__', '__length_hint__', '__missing__', '__neg__', '__next__', '__objclass__', '__pos__', '__rdivmod__', '__reversed__', '__round__', '__set__', '__setitem__', '__trunc__']
augment = ('add', 'and', 'floordiv', 'lshift', 'matmul', 'mod', 'mul', 'pow', 'rshift', 'sub', 'truediv', 'xor')
not_defined.extend(map('__{}__'.format, augment))
not_defined.extend(map('__r{}__'.format, augment))
not_defined.extend(map('__i{}__'.format, augment))
for name in not_defined:

    assert not hasattr(object, name)

    assert not hasattr(o, name)

    assert not hasattr(Custom, name)

    assert not hasattr(c, name)

assert not hasattr(o, '__call__')

assert not hasattr(c, '__call__')
print("ClassTests::testPredefinedAttrs: ok")
