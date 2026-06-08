# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "parameter_replace_is_non_mutating"
# subject = "inspect.Parameter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Parameter: Parameter.replace() returns a new object: no-arg replace is equal, a renamed one differs"""
import inspect

P = inspect.Parameter

q = P("foo", default=42, kind=P.KEYWORD_ONLY)
assert q is not q.replace(), "replace returns a new object"
assert q == q.replace(), "replace() with no args is equal"
assert q.replace(name="bar").name == "bar", "replace name"
assert q.replace(name="bar") != q, "renamed parameter not equal"

print("parameter_replace_is_non_mutating OK")
