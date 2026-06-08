# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "quote_safe_param_keeps_chars"
# subject = "urllib.parse.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.quote: safe= (str or bytes) names otherwise-reserved chars to leave unescaped; quote.__defaults__[0] is the single '/' default"""
from urllib.parse import quote

assert quote.__defaults__[0] == "/", f"default safe = {quote.__defaults__[0]!r}"
assert quote("a/b") == "a/b", "slash safe by default"
assert quote("a/b", safe="") == "a%2Fb", "empty safe escapes slash"
assert quote("<>", safe="<>") == "<>", "safe str keeps chars"
assert quote("<>", safe=b"<>") == "<>", "safe bytes keeps chars"

print("quote_safe_param_keeps_chars OK")
