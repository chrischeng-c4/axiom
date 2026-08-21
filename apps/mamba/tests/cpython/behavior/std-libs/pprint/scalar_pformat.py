# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "scalar_pformat"
# subject = "pprint.pformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.pformat: pformat renders scalars verbatim: 42->'42', 'hello'->"'hello'", None->'None', True->'True'"""
import pprint

# Scalars render exactly as their repr; a short scalar never wraps.
assert pprint.pformat(42) == "42"
assert pprint.pformat("hello") == "'hello'"
assert pprint.pformat(None) == "None"
assert pprint.pformat(True) == "True"
print("scalar_pformat OK")
