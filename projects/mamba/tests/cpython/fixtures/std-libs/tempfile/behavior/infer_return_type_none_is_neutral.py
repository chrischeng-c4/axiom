# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "infer_return_type_none_is_neutral"
# subject = "tempfile._infer_return_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile._infer_return_type: None is neutral and combines with either concrete type: None+str -> str, bytes+None -> bytes, None+None -> str"""
import tempfile

infer = tempfile._infer_return_type
assert infer(None, "") is str, "None + str -> str"
assert infer("", None) is str, "str + None -> str"
assert infer(None, None) is str, "None + None -> str"
assert infer(b"", None) is bytes, "bytes + None -> bytes"
assert infer(None, b"") is bytes, "None + bytes -> bytes"
print("infer_return_type_none_is_neutral OK")
