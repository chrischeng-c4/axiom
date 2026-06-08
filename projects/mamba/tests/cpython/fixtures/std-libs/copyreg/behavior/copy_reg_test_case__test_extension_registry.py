# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copyreg"
# dimension = "behavior"
# case = "copy_reg_test_case__test_extension_registry"
# subject = "cpython.test_copyreg.CopyRegTestCase.test_extension_registry"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_copyreg.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_copyreg.py::CopyRegTestCase::test_extension_registry
"""Auto-ported test: CopyRegTestCase::test_extension_registry (CPython 3.12 oracle)."""


import copyreg
import unittest
from test.pickletester import ExtensionSaver


class C:
    pass

def pickle_C(c):
    return (C, ())

class WithoutSlots(object):
    pass

class WithWeakref(object):
    __slots__ = ('__weakref__',)

class WithPrivate(object):
    __slots__ = ('__spam',)

class _WithLeadingUnderscoreAndPrivate(object):
    __slots__ = ('__spam',)

class ___(object):
    __slots__ = ('__spam',)

class WithSingleString(object):
    __slots__ = 'spam'

class WithInherited(WithSingleString):
    __slots__ = ('eggs',)


# --- test body ---
mod, func, code = ('junk1 ', ' junk2', 43981)
e = ExtensionSaver(code)
try:

    try:
        copyreg.remove_extension(mod, func, code)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
    copyreg.add_extension(mod, func, code)

    assert copyreg._extension_registry[mod, func] == code

    assert copyreg._inverted_registry[code] == (mod, func)

    assert code not in copyreg._extension_cache
    copyreg.add_extension(mod, func, code)

    try:
        copyreg.add_extension(mod, func, code + 1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        copyreg.remove_extension(mod, func, code + 1)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        copyreg.add_extension(mod[1:], func, code)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        copyreg.remove_extension(mod[1:], func, code)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        copyreg.add_extension(mod, func[1:], code)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass

    try:
        copyreg.remove_extension(mod, func[1:], code)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
    if code + 1 not in copyreg._inverted_registry:

        try:
            copyreg.remove_extension(mod[1:], func[1:], code + 1)
            raise AssertionError('expected ValueError')
        except ValueError:
            pass
finally:
    e.restore()

assert (mod, func) not in copyreg._extension_registry
for code in (1, 2147483647):
    e = ExtensionSaver(code)
    try:
        copyreg.add_extension(mod, func, code)
        copyreg.remove_extension(mod, func, code)
    finally:
        e.restore()
for code in (-1, 0, 2147483648):

    try:
        copyreg.add_extension(mod, func, code)
        raise AssertionError('expected ValueError')
    except ValueError:
        pass
print("CopyRegTestCase::test_extension_registry: ok")
