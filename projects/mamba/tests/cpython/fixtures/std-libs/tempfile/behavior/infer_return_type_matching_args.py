# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "infer_return_type_matching_args"
# subject = "tempfile._infer_return_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile._infer_return_type: multiple matching arguments keep the shared type: str+str -> str, bytes+bytes -> bytes"""
import tempfile

infer = tempfile._infer_return_type
assert infer("", "") is str, "str + str -> str"
assert infer(b"", b"") is bytes, "bytes + bytes -> bytes"
print("infer_return_type_matching_args OK")
