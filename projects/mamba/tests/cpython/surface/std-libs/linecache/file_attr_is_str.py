# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "linecache"
# dimension = "surface"
# case = "file_attr_is_str"
# subject = "linecache.__file__"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""linecache.__file__: file_attr_is_str (surface)."""
import linecache

assert type(linecache.__file__).__name__ == "str"
print("file_attr_is_str OK")
