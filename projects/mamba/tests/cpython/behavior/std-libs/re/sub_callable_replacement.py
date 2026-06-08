# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "sub_callable_replacement"
# subject = "re.sub"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
"""re.sub: a callable replacement is invoked per match with the Match object: re.sub(r'\\d+', lambda m: str(int(m.group())*2), 'a1b2c3') is 'a2b4c6'"""
import re

doubled = re.sub(r"\d+", lambda m: str(int(m.group()) * 2), "a1b2c3")
assert doubled == "a2b4c6", f"sub callable = {doubled!r}"

print("sub_callable_replacement OK")
