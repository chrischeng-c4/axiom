# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_p_e_p380_operation__test_delegating_generators_claim_to_be_running_ucf056b3"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_delegating_generators_claim_to_be_running"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import inspect

def one():
    yield 0
    yield from two()
    yield 3

def two():
    yield 1
    try:
        yield from g1
    except ValueError:
        pass
    yield 2
g1 = one()
assert list(g1) == [0, 1, 2, 3]
g1 = one()
res = [next(g1)]
try:
    while True:
        res.append(g1.send(42))
except StopIteration:
    pass
assert res == [0, 1, 2, 3]

class MyErr(Exception):
    pass

def one():
    try:
        yield 0
    except MyErr:
        pass
    yield from two()
    try:
        yield 3
    except MyErr:
        pass

def two():
    try:
        yield 1
    except MyErr:
        pass
    try:
        yield from g1
    except ValueError:
        pass
    try:
        yield 2
    except MyErr:
        pass
g1 = one()
res = [next(g1)]
try:
    while True:
        res.append(g1.throw(MyErr))
except StopIteration:
    pass
except:
    assert res == [0, 1, 2, 3]
    raise

class MyIt(object):

    def __iter__(self):
        return self

    def __next__(self):
        return 42

    def close(self_):
        assert g1.gi_running
        try:
            next(g1)
            raise AssertionError('assertRaises: no raise')
        except ValueError:
            pass

def one():
    yield from MyIt()
g1 = one()
next(g1)
g1.close()

print("TestPEP380Operation::test_delegating_generators_claim_to_be_running: ok")
