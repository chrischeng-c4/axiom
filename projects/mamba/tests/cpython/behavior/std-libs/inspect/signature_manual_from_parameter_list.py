# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "signature_manual_from_parameter_list"
# subject = "inspect.Signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Signature: a Signature assembled from a Parameter list renders the same as a parsed function signature"""
import inspect

S = inspect.Signature
P = inspect.Parameter

assert str(S(parameters=[P("foo", P.POSITIONAL_ONLY)])) == "(foo, /)", "manual posonly"
assert (
    str(S(parameters=[P("foo", P.POSITIONAL_ONLY), P("bar", P.VAR_KEYWORD)]))
    == "(foo, /, **bar)"
), "manual posonly + **kw"

print("signature_manual_from_parameter_list OK")
