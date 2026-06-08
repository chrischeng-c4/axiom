# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "behavior"
# case = "mixed_and_nested_arrays"
# subject = "tomllib.loads"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tomllib/test_data.py"
# status = "filled"
# ///
"""tomllib.loads: TOML 1.0 arrays hold mixed types ([1, 2.0, 'three']) and nest ([[1,2],[3,4]]) into the corresponding Python list structure"""
import tomllib

_d = tomllib.loads("""
mixed = [1, 2.0, "three"]
nested = [[1, 2], [3, 4]]
""")
assert _d["mixed"][0] == 1, f"mixed[0] = {_d['mixed'][0]!r}"
assert _d["mixed"][1] == 2.0, f"mixed[1] = {_d['mixed'][1]!r}"
assert _d["mixed"][2] == "three", f"mixed[2] = {_d['mixed'][2]!r}"
assert _d["nested"] == [[1, 2], [3, 4]], f"nested = {_d['nested']!r}"

print("mixed_and_nested_arrays OK")
