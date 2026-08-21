# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "signature_keyword_only_after_star"
# subject = "inspect.signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.signature: parameters declared after a bare * are KEYWORD_ONLY and keep their defaults"""
import inspect

def _kwonly(a, *, b, c=10):
    pass

_kp = inspect.signature(_kwonly).parameters
assert _kp["b"].kind == inspect.Parameter.KEYWORD_ONLY, "b is KEYWORD_ONLY"
assert _kp["c"].kind == inspect.Parameter.KEYWORD_ONLY, "c is KEYWORD_ONLY"
assert _kp["c"].default == 10, "c default = 10"

print("signature_keyword_only_after_star OK")
