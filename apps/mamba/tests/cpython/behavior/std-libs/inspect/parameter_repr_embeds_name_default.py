# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "parameter_repr_embeds_name_default"
# subject = "inspect.Parameter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Parameter: repr(Parameter) starts with '<Parameter' and embeds the rendered name=default form"""
import inspect

P = inspect.Parameter

r = P("a", default=42, kind=P.POSITIONAL_OR_KEYWORD)
assert repr(r).startswith("<Parameter"), f"repr = {repr(r)!r}"
assert "a=42" in repr(r), f"repr lacks a=42: {repr(r)!r}"

print("parameter_repr_embeds_name_default OK")
