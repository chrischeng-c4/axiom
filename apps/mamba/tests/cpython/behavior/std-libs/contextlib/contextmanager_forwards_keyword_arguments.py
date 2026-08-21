# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "contextmanager_forwards_keyword_arguments"
# subject = "contextlib.contextmanager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.contextmanager: a @contextmanager forwards positional and keyword arguments verbatim to the wrapped generator (even argument names like self/func/args/kwds)"""
import contextlib


@contextlib.contextmanager
def forward(self, func, args, kwds):
    yield (self, func, args, kwds)


with forward(self=11, func=22, args=33, kwds=44) as target:
    assert target == (11, 22, 33, 44), f"kwarg forward = {target!r}"

print("contextmanager_forwards_keyword_arguments OK")
