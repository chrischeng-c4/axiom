# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "behavior"
# case = "deep_nested_tables"
# subject = "tomllib.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_data.py"
# status = "filled"
# ///
"""tomllib.loads: a dotted [a.b.c] header builds nested dicts so data['a']['b']['c']['value'] resolves"""
import tomllib

_d = tomllib.loads("""
[a.b.c]
value = "deep"
""")
assert _d["a"]["b"]["c"]["value"] == "deep", f"deep nesting = {_d!r}"

print("deep_nested_tables OK")
