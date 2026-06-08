# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "listcomps"
# dimension = "behavior"
# case = "list_comprehension_test__test_free_var_in_comp_child"
# subject = "cpython.test_listcomps.ListComprehensionTest.test_free_var_in_comp_child"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_listcomps.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_listcomps.py::ListComprehensionTest::test_free_var_in_comp_child
"""Auto-ported test: ListComprehensionTest::test_free_var_in_comp_child (CPython 3.12 oracle)."""


import doctest
import textwrap
import traceback
import types
import unittest
from test.support import BrokenIter


doctests = "\n########### Tests borrowed from or inspired by test_genexps.py ############\n\nTest simple loop with conditional\n\n    >>> sum([i*i for i in range(100) if i&1 == 1])\n    166650\n\nTest simple nesting\n\n    >>> [(i,j) for i in range(3) for j in range(4)]\n    [(0, 0), (0, 1), (0, 2), (0, 3), (1, 0), (1, 1), (1, 2), (1, 3), (2, 0), (2, 1), (2, 2), (2, 3)]\n\nTest nesting with the inner expression dependent on the outer\n\n    >>> [(i,j) for i in range(4) for j in range(i)]\n    [(1, 0), (2, 0), (2, 1), (3, 0), (3, 1), (3, 2)]\n\nTest the idiom for temporary variable assignment in comprehensions.\n\n    >>> [j*j for i in range(4) for j in [i+1]]\n    [1, 4, 9, 16]\n    >>> [j*k for i in range(4) for j in [i+1] for k in [j+1]]\n    [2, 6, 12, 20]\n    >>> [j*k for i in range(4) for j, k in [(i+1, i+2)]]\n    [2, 6, 12, 20]\n\nNot assignment\n\n    >>> [i*i for i in [*range(4)]]\n    [0, 1, 4, 9]\n    >>> [i*i for i in (*range(4),)]\n    [0, 1, 4, 9]\n\nMake sure the induction variable is not exposed\n\n    >>> i = 20\n    >>> sum([i*i for i in range(100)])\n    328350\n\n    >>> i\n    20\n\nVerify that syntax error's are raised for listcomps used as lvalues\n\n    >>> [y for y in (1,2)] = 10          # doctest: +IGNORE_EXCEPTION_DETAIL\n    Traceback (most recent call last):\n       ...\n    SyntaxError: ...\n\n    >>> [y for y in (1,2)] += 10         # doctest: +IGNORE_EXCEPTION_DETAIL\n    Traceback (most recent call last):\n       ...\n    SyntaxError: ...\n\n\n########### Tests borrowed from or inspired by test_generators.py ############\n\nMake a nested list comprehension that acts like range()\n\n    >>> def frange(n):\n    ...     return [i for i in range(n)]\n    >>> frange(10)\n    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]\n\nSame again, only as a lambda expression instead of a function definition\n\n    >>> lrange = lambda n:  [i for i in range(n)]\n    >>> lrange(10)\n    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]\n\nGenerators can call other generators:\n\n    >>> def grange(n):\n    ...     for x in [i for i in range(n)]:\n    ...         yield x\n    >>> list(grange(5))\n    [0, 1, 2, 3, 4]\n\n\nMake sure that None is a valid return value\n\n    >>> [None for i in range(10)]\n    [None, None, None, None, None, None, None, None, None, None]\n\n"

__test__ = {'doctests': doctests}

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite())
    return tests


# --- test body ---
def _check_in_scopes(code, outputs=None, ns=None, scopes=None, raises=(), exec_func=exec):
    code = textwrap.dedent(code)
    scopes = scopes or ['module', 'class', 'function']
    for scope in scopes:
        if scope == 'class':
            newcode = textwrap.dedent('\n                        class _C:\n                            {code}\n                    ').format(code=textwrap.indent(code, '    '))

            def get_output(moddict, name):
                return getattr(moddict['_C'], name)
        elif scope == 'function':
            newcode = textwrap.dedent('\n                        def _f():\n                            {code}\n                            return locals()\n                        _out = _f()\n                    ').format(code=textwrap.indent(code, '    '))

            def get_output(moddict, name):
                return moddict['_out'][name]
        else:
            newcode = code

            def get_output(moddict, name):
                return moddict[name]
        newns = ns.copy() if ns else {}
        try:
            exec_func(newcode, newns)
        except raises as e:

            assert type(e) is raises
        else:
            for k, v in (outputs or {}).items():

                assert get_output(newns, k) == v

def _recursive_replace(maybe_code):
    if not isinstance(maybe_code, types.CodeType):
        return maybe_code
    return maybe_code.replace(co_consts=tuple((_recursive_replace(c) for c in maybe_code.co_consts)))

def _replacing_exec(code_string, ns):
    co = compile(code_string, '<string>', 'exec')
    co = _recursive_replace(co)
    exec(co, ns)
code = '\n            lst = range(3)\n            funcs = [lambda: x for x in lst]\n            inc = [x + 1 for x in lst]\n            [x for x in inc]\n            x = funcs[0]()\n        '
outputs = {'x': 2}
_check_in_scopes(code, outputs)
print("ListComprehensionTest::test_free_var_in_comp_child: ok")
