# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "test_custom_exception_attribute_ops"
# subject = "cpython321.test_custom_exception_attribute_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_custom_exception_attribute_ops.py"
# status = "filled"
# ///
"""cpython321.test_custom_exception_attribute_ops: execute CPython 3.12 seed test_custom_exception_attribute_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for user-defined exception classes
# with extra attributes and for multi-branch try/except dispatch.
# Surface not covered by `test_exception_ops` or
# `test_exception_hierarchy_ops`:
#   * Custom `Exception` subclass with an `__init__` that calls
#     `super().__init__(message)` and stores additional attributes
#     (status code, payload).
#   * Multi-branch `try / except A / except B / except C` dispatches
#     by raised type, not by listing order.
#   * Hierarchical except catches by the parent class (catch C via A
#     when C subclasses B subclasses A).
#   * `isinstance(e, Exception)` chains correctly through user
#     exception classes.
#   * Two-arg user exception preserves both the message (via str(e))
#     and the extra attribute (via e.attr).
_ledger: list[int] = []


class _HttpError(Exception):
    def __init__(self, status, message):
        super().__init__(message)
        self.status = status


# Raise and catch with custom attribute access
caught_status = None
caught_msg = None
caught_isinst_http = None
caught_isinst_exc = None
try:
    raise _HttpError(404, "not found")
except _HttpError as e:
    caught_status = e.status
    caught_msg = str(e)
    caught_isinst_http = isinstance(e, _HttpError)
    caught_isinst_exc = isinstance(e, Exception)

assert caught_status == 404; _ledger.append(1)
assert caught_msg == "not found"; _ledger.append(1)
assert caught_isinst_http; _ledger.append(1)
assert caught_isinst_exc; _ledger.append(1)


# Second HttpError instance — independent state
def raise_and_catch(status, message):
    try:
        raise _HttpError(status, message)
    except _HttpError as e:
        return e.status, str(e)

s1, m1 = raise_and_catch(500, "server error")
s2, m2 = raise_and_catch(403, "forbidden")
assert s1 == 500; _ledger.append(1)
assert m1 == "server error"; _ledger.append(1)
assert s2 == 403; _ledger.append(1)
assert m2 == "forbidden"; _ledger.append(1)


# Multi-branch try/except dispatches by type
def handle(value):
    try:
        if value == 1:
            raise ValueError("v")
        elif value == 2:
            raise TypeError("t")
        elif value == 3:
            raise KeyError("k")
        elif value == 4:
            raise RuntimeError("r")
    except ValueError:
        return "val"
    except TypeError:
        return "type"
    except KeyError:
        return "key"
    except RuntimeError:
        return "runtime"
    return "none"


assert handle(1) == "val"; _ledger.append(1)
assert handle(2) == "type"; _ledger.append(1)
assert handle(3) == "key"; _ledger.append(1)
assert handle(4) == "runtime"; _ledger.append(1)
assert handle(99) == "none"; _ledger.append(1)


# Exception inheritance chain: catch a subclass instance via the
# parent class
class _A(Exception):
    pass


class _B(_A):
    pass


class _C(_B):
    pass


# A C instance is caught by `except _A`
caught_via_a_isinst_c = None
caught_via_a_isinst_b = None
caught_via_a_isinst_a = None
caught_via_a_isinst_exc = None
try:
    raise _C("c-instance")
except _A as e:
    caught_via_a_isinst_c = isinstance(e, _C)
    caught_via_a_isinst_b = isinstance(e, _B)
    caught_via_a_isinst_a = isinstance(e, _A)
    caught_via_a_isinst_exc = isinstance(e, Exception)


assert caught_via_a_isinst_c; _ledger.append(1)
assert caught_via_a_isinst_b; _ledger.append(1)
assert caught_via_a_isinst_a; _ledger.append(1)
assert caught_via_a_isinst_exc; _ledger.append(1)


# Custom exception with two attributes (status + payload)
class _ApiError(Exception):
    def __init__(self, status, payload):
        super().__init__(f"api status {status}")
        self.status = status
        self.payload = payload


try:
    raise _ApiError(500, {"error": "internal"})
except _ApiError as e:
    assert e.status == 500; _ledger.append(1)
    assert e.payload == {"error": "internal"}; _ledger.append(1)
    assert str(e) == "api status 500"; _ledger.append(1)
    assert isinstance(e, _ApiError); _ledger.append(1)
    assert isinstance(e, Exception); _ledger.append(1)


# A try with no matching except re-raises (here, ValueError raised
# but only TypeError caught — should propagate)
outer_caught = False
try:
    try:
        raise ValueError("v")
    except TypeError:
        # This branch should NOT match
        pass
except ValueError as e:
    outer_caught = True
    assert str(e) == "v"; _ledger.append(1)

assert outer_caught; _ledger.append(1)

# A bare except: catches everything
def bare_catch():
    try:
        raise RuntimeError("any")
    except:  # noqa
        return "caught"
    return "missed"


assert bare_catch() == "caught"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_custom_exception_attribute_ops {sum(_ledger)} asserts")
