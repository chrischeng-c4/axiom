# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "infer_return_type_single_arg"
# subject = "tempfile._infer_return_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile._infer_return_type: single-argument inference: str -> str, bytes -> bytes, None -> str (the default)"""
import tempfile

infer = tempfile._infer_return_type
assert infer("") is str, "single str -> str"
assert infer(b"") is bytes, "single bytes -> bytes"
assert infer(None) is str, "single None -> str"
print("infer_return_type_single_arg OK")
