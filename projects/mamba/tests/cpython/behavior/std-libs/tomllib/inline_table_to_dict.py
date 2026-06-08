# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "behavior"
# case = "inline_table_to_dict"
# subject = "tomllib.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_data.py"
# status = "filled"
# ///
"""tomllib.loads: an inline table point = {x = 1, y = 2} parses to the Python dict {'x': 1, 'y': 2}"""
import tomllib

_d = tomllib.loads('point = {x = 1, y = 2}')
assert _d["point"] == {"x": 1, "y": 2}, f"inline table = {_d['point']!r}"

print("inline_table_to_dict OK")
