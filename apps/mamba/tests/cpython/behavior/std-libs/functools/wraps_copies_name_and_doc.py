# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "wraps_copies_name_and_doc"
# subject = "functools.wraps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.wraps: @wraps copies __name__ and __doc__ from the wrapped function to the wrapper and sets __wrapped__"""
import functools


# @wraps copies __name__/__doc__ from the wrapped function onto the
# wrapper and records the original under __wrapped__.
def _log(fn):
    @functools.wraps(fn)
    def _w(*args, **kwargs):
        return fn(*args, **kwargs)

    return _w


@_log
def _target(x: int) -> int:
    """Target docstring."""
    return x * 2


assert _target.__name__ == "_target", f"__name__ = {_target.__name__!r}"
assert _target.__doc__ == "Target docstring.", f"__doc__ = {_target.__doc__!r}"
assert _target.__wrapped__ is not None, "__wrapped__ missing"
assert _target(21) == 42, "wrapper still calls through"

print("wraps_copies_name_and_doc OK")
