# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sched"
# dimension = "behavior"
# case = "test_case__test_args_kwargs"
# subject = "cpython.test_sched.TestCase.test_args_kwargs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sched.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sched.py::TestCase::test_args_kwargs
"""Auto-ported test: TestCase::test_args_kwargs (CPython 3.12 oracle)."""


import sched
import time


seq = []


def fun(*args, **kwargs):
    seq.append((args, kwargs))


now = time.time()
scheduler = sched.scheduler(time.time, time.sleep)
scheduler.enterabs(now, 1, fun)
scheduler.enterabs(now, 1, fun, argument=(1, 2))
scheduler.enterabs(now, 1, fun, argument=("a", "b"))
scheduler.enterabs(now, 1, fun, argument=(1, 2), kwargs={"foo": 3})
scheduler.run()

expected = [
    ((), {}),
    ((1, 2), {}),
    (("a", "b"), {}),
    ((1, 2), {"foo": 3}),
]
assert sorted(seq, key=repr) == sorted(expected, key=repr)

print("TestCase::test_args_kwargs: ok")
