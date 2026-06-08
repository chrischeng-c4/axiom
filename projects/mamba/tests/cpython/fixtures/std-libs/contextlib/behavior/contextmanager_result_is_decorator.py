# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "contextmanager_result_is_decorator"
# subject = "contextlib.contextmanager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.contextmanager: a @contextmanager result is itself a ContextDecorator in 3.12, so it can decorate a function and wrap each call in enter/exit"""
import contextlib

log: list = []


@contextlib.contextmanager
def managed():
    log.append("cm_enter")
    yield
    log.append("cm_exit")


@managed()
def cm_decorated():
    log.append("cm_body")


cm_decorated()
assert log == ["cm_enter", "cm_body", "cm_exit"], log

print("contextmanager_result_is_decorator OK")
