# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "behavior"
# case = "array_of_tables_to_list"
# subject = "tomllib.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_data.py"
# status = "filled"
# ///
"""tomllib.loads: repeated [[products]] headers build a Python list of dicts in document order, preserving each table's keys"""
import tomllib

_d = tomllib.loads("""
[[products]]
name = "Hammer"
price = 9.99

[[products]]
name = "Wrench"
price = 14.99
""")
assert isinstance(_d["products"], list), f"array of tables type = {type(_d['products'])!r}"
assert len(_d["products"]) == 2, f"two products = {len(_d['products'])!r}"
assert _d["products"][0]["name"] == "Hammer", f"first = {_d['products'][0]['name']!r}"
assert _d["products"][1]["name"] == "Wrench", f"second = {_d['products'][1]['name']!r}"

print("array_of_tables_to_list OK")
