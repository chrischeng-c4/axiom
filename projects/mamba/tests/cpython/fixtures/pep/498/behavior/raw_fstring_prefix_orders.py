# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "498"
# dimension = "behavior"
# case = "raw_fstring_prefix_orders"
# subject = "fstring.prefix"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fstring.prefix: both prefix orders combine raw + f: rf'\\n{1}' is '\\\\n1' and fr'{1}\\t' is '1\\\\t'"""
# rf and fr prefixes both yield a raw f-string

assert f"{1}" == "1"
assert rf"\n{1}" == "\\n1"
assert fr"{1}\t" == "1\\t"

print("raw_fstring_prefix_orders OK")
