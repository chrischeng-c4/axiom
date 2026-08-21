# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profile"
# dimension = "behavior"
# case = "profile_test__test_calling_conventions"
# subject = "cpython.test_profile.ProfileTest.test_calling_conventions"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_profile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_profile.py::ProfileTest::test_calling_conventions
"""Auto-ported test: ProfileTest::test_calling_conventions (CPython 3.12 oracle)."""


import sys
import pstats
import unittest
import os
from difflib import unified_diff
from io import StringIO
from test.support.os_helper import TESTFN, unlink, temp_dir, change_cwd
from contextlib import contextmanager
import profile
from test.profilee import testfunc, timer
from test.support.script_helper import assert_python_failure, assert_python_ok


'Test suite for the profile module.'

def regenerate_expected_output(filename, cls):
    filename = filename.rstrip('co')
    print('Regenerating %s...' % filename)
    results = cls.do_profiling()
    newfile = []
    with open(filename, 'r') as f:
        for line in f:
            newfile.append(line)
            if line.startswith('#--cut'):
                break
    with open(filename, 'w') as f:
        f.writelines(newfile)
        f.write('_ProfileOutput = {}\n')
        for i, method in enumerate(cls.methodnames):
            f.write('_ProfileOutput[%r] = """\\\n%s"""\n' % (method, results[i + 1]))
        f.write('\nif __name__ == "__main__":\n    main()\n')

@contextmanager
def silent():
    stdout = sys.stdout
    try:
        sys.stdout = StringIO()
        yield
    finally:
        sys.stdout = stdout

def main():
    if '-r' not in sys.argv:
        unittest.main()
    else:
        regenerate_expected_output(__file__, ProfileTest)

_ProfileOutput = {}

_ProfileOutput['print_stats'] = '       28   27.972    0.999   27.972    0.999 profilee.py:110(__getattr__)\n        1  269.996  269.996  999.769  999.769 profilee.py:25(testfunc)\n     23/3  149.937    6.519  169.917   56.639 profilee.py:35(factorial)\n       20   19.980    0.999   19.980    0.999 profilee.py:48(mul)\n        2   39.986   19.993  599.830  299.915 profilee.py:55(helper)\n        4  115.984   28.996  119.964   29.991 profilee.py:73(helper1)\n        2   -0.006   -0.003  139.946   69.973 profilee.py:84(helper2_indirect)\n        8  311.976   38.997  399.912   49.989 profilee.py:88(helper2)\n        8   63.976    7.997   79.960    9.995 profilee.py:98(subhelper)'

_ProfileOutput['print_callers'] = ':0(append)                        <- profilee.py:73(helper1)(4)  119.964\n:0(exception)                     <- profilee.py:73(helper1)(4)  119.964\n:0(hasattr)                       <- profilee.py:73(helper1)(4)  119.964\n                                     profilee.py:88(helper2)(8)  399.912\nprofilee.py:110(__getattr__)      <- :0(hasattr)(12)   11.964\n                                     profilee.py:98(subhelper)(16)   79.960\nprofilee.py:25(testfunc)          <- <string>:1(<module>)(1)  999.767\nprofilee.py:35(factorial)         <- profilee.py:25(testfunc)(1)  999.769\n                                     profilee.py:35(factorial)(20)  169.917\n                                     profilee.py:84(helper2_indirect)(2)  139.946\nprofilee.py:48(mul)               <- profilee.py:35(factorial)(20)  169.917\nprofilee.py:55(helper)            <- profilee.py:25(testfunc)(2)  999.769\nprofilee.py:73(helper1)           <- profilee.py:55(helper)(4)  599.830\nprofilee.py:84(helper2_indirect)  <- profilee.py:55(helper)(2)  599.830\nprofilee.py:88(helper2)           <- profilee.py:55(helper)(6)  599.830\n                                     profilee.py:84(helper2_indirect)(2)  139.946\nprofilee.py:98(subhelper)         <- profilee.py:88(helper2)(8)  399.912'

_ProfileOutput['print_callees'] = ':0(hasattr)                       -> profilee.py:110(__getattr__)(12)   27.972\n<string>:1(<module>)              -> profilee.py:25(testfunc)(1)  999.769\nprofilee.py:110(__getattr__)      ->\nprofilee.py:25(testfunc)          -> profilee.py:35(factorial)(1)  169.917\n                                     profilee.py:55(helper)(2)  599.830\nprofilee.py:35(factorial)         -> profilee.py:35(factorial)(20)  169.917\n                                     profilee.py:48(mul)(20)   19.980\nprofilee.py:48(mul)               ->\nprofilee.py:55(helper)            -> profilee.py:73(helper1)(4)  119.964\n                                     profilee.py:84(helper2_indirect)(2)  139.946\n                                     profilee.py:88(helper2)(6)  399.912\nprofilee.py:73(helper1)           -> :0(append)(4)   -0.004\nprofilee.py:84(helper2_indirect)  -> profilee.py:35(factorial)(2)  169.917\n                                     profilee.py:88(helper2)(2)  399.912\nprofilee.py:88(helper2)           -> :0(hasattr)(8)   11.964\n                                     profilee.py:98(subhelper)(8)   79.960\nprofilee.py:98(subhelper)         -> profilee.py:110(__getattr__)(16)   27.972'


# --- test body ---
profilerclass = profile.Profile
profilermodule = profile
methodnames = ['print_stats', 'print_callers', 'print_callees']
expected_max_output = ':0(max)'
stmts = ['max([0])', 'max([0], key=int)', 'max([0], **dict(key=int))', 'max(*([0],))', 'max(*([0],), key=int)', 'max(*([0],), **dict(key=int))']
for stmt in stmts:
    s = StringIO()
    prof = profilerclass(timer, 0.001)
    prof.runctx(stmt, globals(), locals())
    stats = pstats.Stats(prof, stream=s)
    stats.print_stats()
    res = s.getvalue()

    assert expected_max_output in res
print("ProfileTest::test_calling_conventions: ok")
