# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "errors"
# case = "missing_file_returns_true_no_raise"
# subject = "compileall.compile_file"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""compileall.compile_file: compile_file on a non-existent .py path does NOT raise; it reports success (True) because there is nothing to fail at"""
import compileall

# A missing source path is not an error: nothing to compile means nothing to
# fail, so compile_file returns a truthy verdict without raising.
result = compileall.compile_file("/no/such/file.py", quiet=2)
assert result is True, result
print("missing_file_returns_true_no_raise OK")
