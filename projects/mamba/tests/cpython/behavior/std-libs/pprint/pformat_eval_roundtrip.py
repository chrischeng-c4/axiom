# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "pformat_eval_roundtrip"
# subject = "pprint.pformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.pformat: pformat output is round-trippable for readable values: eval(pformat('hello'))=='hello' and eval(pformat((1,2,3)))==(1,2,3)"""
import pprint

# For readable values the rendered text is a valid literal: eval reconstructs
# the original object exactly.
assert eval(pprint.pformat("hello")) == "hello"
assert eval(pprint.pformat((1, 2, 3))) == (1, 2, 3)
print("pformat_eval_roundtrip OK")
