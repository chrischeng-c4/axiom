# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "bind_positional_only_and_same_named_keyword"
# subject = "inspect.Signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Signature: a positional-only parameter and a same-named keyword do not collide: the positional binds the param, the keyword lands in **kwargs"""
import inspect

def posonly(bar, /, **kwargs):
    pass

sig3 = inspect.signature(posonly)
res = sig3.bind("pos-only", bar="keyword")
assert ("bar", "pos-only") in res.arguments.items(), "posonly captured positionally"
assert res.kwargs == {"bar": "keyword"}, f"posonly kwargs = {res.kwargs!r}"

print("bind_positional_only_and_same_named_keyword OK")
