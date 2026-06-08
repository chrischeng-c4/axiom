# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cprofile"
# dimension = "behavior"
# case = "c_profile_test__test_evil_external_timer"
# subject = "cpython.test_cprofile.CProfileTest.test_evil_external_timer"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cprofile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cprofile.py::CProfileTest::test_evil_external_timer
"""Auto-ported test: CProfileTest::test_evil_external_timer (CPython 3.12 oracle)."""


import sys
import unittest
import cProfile
from test.test_profile import ProfileTest, regenerate_expected_output
from test.support.script_helper import assert_python_failure
from test import support


'Test suite for the cProfile module.'

def main():
    if '-r' not in sys.argv:
        unittest.main()
    else:
        regenerate_expected_output(__file__, CProfileTest)

_ProfileOutput = {}

_ProfileOutput['print_stats'] = '       28    0.028    0.001    0.028    0.001 profilee.py:110(__getattr__)\n        1    0.270    0.270    1.000    1.000 profilee.py:25(testfunc)\n     23/3    0.150    0.007    0.170    0.057 profilee.py:35(factorial)\n       20    0.020    0.001    0.020    0.001 profilee.py:48(mul)\n        2    0.040    0.020    0.600    0.300 profilee.py:55(helper)\n        4    0.116    0.029    0.120    0.030 profilee.py:73(helper1)\n        2    0.000    0.000    0.140    0.070 profilee.py:84(helper2_indirect)\n        8    0.312    0.039    0.400    0.050 profilee.py:88(helper2)\n        8    0.064    0.008    0.080    0.010 profilee.py:98(subhelper)'

_ProfileOutput['print_callers'] = "profilee.py:110(__getattr__)                      <-      16    0.016    0.016  profilee.py:98(subhelper)\nprofilee.py:25(testfunc)                          <-       1    0.270    1.000  <string>:1(<module>)\nprofilee.py:35(factorial)                         <-       1    0.014    0.130  profilee.py:25(testfunc)\n                                                        20/3    0.130    0.147  profilee.py:35(factorial)\n                                                           2    0.006    0.040  profilee.py:84(helper2_indirect)\nprofilee.py:48(mul)                               <-      20    0.020    0.020  profilee.py:35(factorial)\nprofilee.py:55(helper)                            <-       2    0.040    0.600  profilee.py:25(testfunc)\nprofilee.py:73(helper1)                           <-       4    0.116    0.120  profilee.py:55(helper)\nprofilee.py:84(helper2_indirect)                  <-       2    0.000    0.140  profilee.py:55(helper)\nprofilee.py:88(helper2)                           <-       6    0.234    0.300  profilee.py:55(helper)\n                                                           2    0.078    0.100  profilee.py:84(helper2_indirect)\nprofilee.py:98(subhelper)                         <-       8    0.064    0.080  profilee.py:88(helper2)\n{built-in method builtins.hasattr}                <-       4    0.000    0.004  profilee.py:73(helper1)\n                                                           8    0.000    0.008  profilee.py:88(helper2)\n{built-in method sys.exception}                   <-       4    0.000    0.000  profilee.py:73(helper1)\n{method 'append' of 'list' objects}               <-       4    0.000    0.000  profilee.py:73(helper1)"

_ProfileOutput['print_callees'] = '<string>:1(<module>)                              ->       1    0.270    1.000  profilee.py:25(testfunc)\nprofilee.py:110(__getattr__)                      ->\nprofilee.py:25(testfunc)                          ->       1    0.014    0.130  profilee.py:35(factorial)\n                                                           2    0.040    0.600  profilee.py:55(helper)\nprofilee.py:35(factorial)                         ->    20/3    0.130    0.147  profilee.py:35(factorial)\n                                                          20    0.020    0.020  profilee.py:48(mul)\nprofilee.py:48(mul)                               ->\nprofilee.py:55(helper)                            ->       4    0.116    0.120  profilee.py:73(helper1)\n                                                           2    0.000    0.140  profilee.py:84(helper2_indirect)\n                                                           6    0.234    0.300  profilee.py:88(helper2)\nprofilee.py:73(helper1)                           ->       4    0.000    0.004  {built-in method builtins.hasattr}\nprofilee.py:84(helper2_indirect)                  ->       2    0.006    0.040  profilee.py:35(factorial)\n                                                           2    0.078    0.100  profilee.py:88(helper2)\nprofilee.py:88(helper2)                           ->       8    0.064    0.080  profilee.py:98(subhelper)\nprofilee.py:98(subhelper)                         ->      16    0.016    0.016  profilee.py:110(__getattr__)\n{built-in method builtins.hasattr}                ->      12    0.012    0.012  profilee.py:110(__getattr__)'


# --- test body ---
profilerclass = cProfile.Profile
profilermodule = cProfile
expected_max_output = '{built-in method builtins.max}'

def get_expected_output():
    return _ProfileOutput
import _lsprof

class EvilTimer:

    def __init__(self, disable_count):
        self.count = 0
        self.disable_count = disable_count

    def __call__(self):
        self.count += 1
        if self.count == self.disable_count:
            profiler_with_evil_timer.disable()
        return self.count
with support.catch_unraisable_exception() as cm:
    profiler_with_evil_timer = _lsprof.Profiler(EvilTimer(1))
    profiler_with_evil_timer.enable()
    (lambda: None)()
    profiler_with_evil_timer.disable()
    profiler_with_evil_timer.clear()

    assert cm.unraisable.exc_type == RuntimeError
with support.catch_unraisable_exception() as cm:
    profiler_with_evil_timer = _lsprof.Profiler(EvilTimer(2))
    profiler_with_evil_timer.enable()
    (lambda: None)()
    profiler_with_evil_timer.disable()
    profiler_with_evil_timer.clear()

    assert cm.unraisable.exc_type == RuntimeError
print("CProfileTest::test_evil_external_timer: ok")
