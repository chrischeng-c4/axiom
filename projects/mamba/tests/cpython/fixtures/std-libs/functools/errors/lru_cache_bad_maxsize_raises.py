# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "errors"
# case = "lru_cache_bad_maxsize_raises"
# subject = "functools.lru_cache"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.lru_cache: lru_cache_bad_maxsize_raises (errors)."""
import functools

_raised = False
try:
    functools.lru_cache(maxsize="all")(lambda x: x)
except TypeError:
    _raised = True
assert _raised, "lru_cache_bad_maxsize_raises: expected TypeError"
print("lru_cache_bad_maxsize_raises OK")
