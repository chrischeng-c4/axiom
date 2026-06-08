# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "parameter_attributes_reflect_constructor"
# subject = "inspect.Parameter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Parameter: Parameter attributes (name, default, kind) reflect the constructor; a missing annotation is the empty sentinel"""
import inspect

P = inspect.Parameter

p = P("foo", default=10, kind=P.POSITIONAL_ONLY)
assert p.name == "foo", f"name = {p.name!r}"
assert p.default == 10, f"default = {p.default!r}"
assert p.kind == P.POSITIONAL_ONLY, f"kind = {p.kind!r}"
assert p.annotation is P.empty, "annotation is empty sentinel"

print("parameter_attributes_reflect_constructor OK")
