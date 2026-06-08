# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "cast_is_runtime_noop"
# subject = "typing.cast"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.cast: cast is a runtime no-op that returns its argument unchanged with no coercion for any type form: cast(int,42)==42, cast(float,42) stays an int, cast(Any,'x')=='x', cast(Union[str,float],42)==42, cast(None,42)==42"""
from typing import Any, AnyStr, Union, cast

# cast is a runtime no-op: it returns its argument unchanged for any form.
assert cast(int, 42) == 42
assert cast(float, 42) == 42
assert type(cast(float, 42)) is int  # no coercion happens
assert cast(Any, "x") == "x"
assert cast(list, 42) == 42
assert cast(Union[str, float], 42) == 42
assert cast(AnyStr, 42) == 42
assert cast(None, 42) == 42

print("cast_is_runtime_noop OK")
