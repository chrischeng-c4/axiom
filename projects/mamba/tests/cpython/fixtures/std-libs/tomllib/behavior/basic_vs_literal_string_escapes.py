# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "behavior"
# case = "basic_vs_literal_string_escapes"
# subject = "tomllib.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_data.py"
# status = "filled"
# ///
"""tomllib.loads: basic strings process backslash escapes (\\n -> newline); literal single-quoted strings keep the backslash literal"""
import tomllib

_toml = (
    'basic = "hello\\nworld"\n'
    "literal = 'no\\escape'\n"  # literal: backslash stays literal
)
_d = tomllib.loads(_toml)
assert _d["basic"] == "hello\nworld", f"basic string escape = {_d['basic']!r}"
assert _d["literal"] == "no\\escape", f"literal no escape = {_d['literal']!r}"

print("basic_vs_literal_string_escapes OK")
