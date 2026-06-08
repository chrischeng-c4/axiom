# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "errors"
# case = "contextmanager_reraises_chained_runtimeerror"
# subject = "contextlib.contextmanager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.contextmanager: a @contextmanager that converts a body exception via `raise RuntimeError(...) from exc` propagates the new RuntimeError out, with __cause__ set to the original"""
import contextlib


@contextlib.contextmanager
def wrap():
    try:
        yield
    except Exception as exc:
        raise RuntimeError(f"caught {type(exc).__name__}") from exc


_raised = False
try:
    with wrap():
        1 / 0  # ZeroDivisionError, converted by the manager
except RuntimeError as e:
    _raised = True
    assert str(e) == "caught ZeroDivisionError", str(e)
    assert isinstance(e.__cause__, ZeroDivisionError), repr(e.__cause__)
assert _raised, "expected the converted RuntimeError to propagate"

print("contextmanager_reraises_chained_runtimeerror OK")
