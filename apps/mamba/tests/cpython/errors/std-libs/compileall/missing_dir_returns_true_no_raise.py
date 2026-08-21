# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "errors"
# case = "missing_dir_returns_true_no_raise"
# subject = "compileall.compile_dir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""compileall.compile_dir: compile_dir on a non-existent directory does NOT raise; it returns True (no files means no failures)"""
import compileall

# A missing directory yields no work, so compile_dir reports success (True)
# without raising rather than treating the absent tree as a failure.
result = compileall.compile_dir("/no/such/dir", quiet=2)
assert result is True, result
print("missing_dir_returns_true_no_raise OK")
