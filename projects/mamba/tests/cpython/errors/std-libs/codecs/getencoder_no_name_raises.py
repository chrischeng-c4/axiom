# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "errors"
# case = "getencoder_no_name_raises"
# subject = "codecs.getencoder"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.getencoder: getencoder_no_name_raises (errors)."""
import codecs

_raised = False
try:
    codecs.getencoder()
except TypeError:
    _raised = True
assert _raised, "getencoder_no_name_raises: expected TypeError"
print("getencoder_no_name_raises OK")
