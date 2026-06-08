# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "cast_is_runtime_noop"
# subject = "typing.cast"
# kind = "semantic"
# xfail = "mamba diverges on the typing cast runtime machinery (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.cast: cast is a runtime no-op returning its argument unchanged with no coercion: cast(int,'hello')=='hello' and cast(int,'still a str')=='still a str'"""
import typing
from typing import cast

# cast is a no-op at runtime: the value passes through unchanged, uncoerced.
assert cast(int, "hello") == "hello", f"cast no-op = {cast(int, 'hello')!r}"
assert cast(int, "still a str") == "still a str", "cast no-op str"

print("cast_is_runtime_noop OK")
