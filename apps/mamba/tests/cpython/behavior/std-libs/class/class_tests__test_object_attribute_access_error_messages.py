# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "class"
# dimension = "behavior"
# case = "class_tests__test_object_attribute_access_error_messages"
# subject = "cpython.test_class.ClassTests.testObjectAttributeAccessErrorMessages"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_class.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_class.py::ClassTests::testObjectAttributeAccessErrorMessages
"""Auto-ported test: ClassTests::testObjectAttributeAccessErrorMessages (CPython 3.12 oracle)."""


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
    pass

class B:
    y = 0
    __slots__ = ('z',)
error_msg = "'A' object has no attribute 'x'"
try:
    A().x
    raise AssertionError('expected AttributeError')
except AttributeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(error_msg, str(_aR_e))
try:
    del A().x
    raise AssertionError('expected AttributeError')
except AttributeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(error_msg, str(_aR_e))
error_msg = "'B' object has no attribute 'x'"
try:
    B().x
    raise AssertionError('expected AttributeError')
except AttributeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(error_msg, str(_aR_e))
try:
    del B().x
    raise AssertionError('expected AttributeError')
except AttributeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(error_msg, str(_aR_e))
try:
    B().x = 0
    raise AssertionError('expected AttributeError')
except AttributeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(error_msg, str(_aR_e))
error_msg = "'B' object attribute 'y' is read-only"
try:
    del B().y
    raise AssertionError('expected AttributeError')
except AttributeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(error_msg, str(_aR_e))
try:
    B().y = 0
    raise AssertionError('expected AttributeError')
except AttributeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(error_msg, str(_aR_e))
error_msg = 'z'
try:
    B().z
    raise AssertionError('expected AttributeError')
except AttributeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(error_msg, str(_aR_e))
try:
    del B().z
    raise AssertionError('expected AttributeError')
except AttributeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search(error_msg, str(_aR_e))
print("ClassTests::testObjectAttributeAccessErrorMessages: ok")
