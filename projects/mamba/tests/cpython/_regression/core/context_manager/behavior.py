# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/context_manager: with-protocol behavior asserts (CPython 3.12 oracle).

Runtime-observable rules of the `with` statement that complement the
ordering/suppression cases in with_enter_exit.py.
"""


# On normal (no-exception) exit, __exit__ receives the all-None triple.
seen = {}


class Normal:
    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, tb):
        seen["triple"] = (exc_type, exc, tb)
        return False


with Normal():
    pass
assert seen["triple"] == (None, None, None), seen["triple"]


# When the block raises, __exit__ receives the exception class, the actual
# exception instance, and a real traceback object.
caught = {}


class Capture:
    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, tb):
        caught["type_is_keyerror"] = exc_type is KeyError
        caught["exc_is_instance"] = isinstance(exc, BaseException)
        caught["tb_present"] = tb is not None
        return True  # suppress so we can keep asserting below


with Capture():
    raise KeyError("k")
assert caught["type_is_keyerror"] is True
assert caught["exc_is_instance"] is True
assert caught["tb_present"] is True


# An exception inside __enter__ means the body and __exit__ never run.
trace = []


class EnterRaises:
    def __enter__(self):
        trace.append("enter")
        raise RuntimeError("enter-fail")

    def __exit__(self, *a):
        trace.append("exit")
        return False


try:
    with EnterRaises():
        trace.append("body")
except RuntimeError as e:
    trace.append(f"caught:{e}")
assert trace == ["enter", "caught:enter-fail"], trace


# __exit__ fires once per iteration even when the body breaks/continues.
order = []


class Loop:
    def __init__(self, n):
        self.n = n

    def __enter__(self):
        order.append(f"enter{self.n}")
        return self

    def __exit__(self, *a):
        order.append(f"exit{self.n}")
        return False


for i in range(3):
    with Loop(i):
        if i == 1:
            continue
        if i == 2:
            break
        order.append(f"body{i}")
assert order == ["enter0", "body0", "exit0", "enter1", "exit1", "enter2", "exit2"], order


print("behavior OK")
