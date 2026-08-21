# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "errors"
# case = "rmtree_ignore_errors_swallows"
# subject = "shutil.rmtree"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_shutil.py"
# status = "filled"
# ///
"""shutil.rmtree: rmtree(nonexistent, ignore_errors=True) swallows the FileNotFoundError and returns without raising"""
import shutil

# ignore_errors=True must swallow the would-be FileNotFoundError; reaching the
# assert proves no exception escaped.
shutil.rmtree("/no/such/path_to_rmtree", ignore_errors=True)
assert True, "rmtree(ignore_errors=True) returned without raising"
print("rmtree_ignore_errors_swallows OK")
