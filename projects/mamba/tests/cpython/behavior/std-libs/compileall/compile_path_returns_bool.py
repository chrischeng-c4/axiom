# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "behavior"
# case = "compile_path_returns_bool"
# subject = "compileall.compile_path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_path: compile_path walks sys.path entries and returns a bool/int verdict without raising"""
import compileall

# compile_path walks sys.path; some entries may be unwriteable, so the verdict
# is True or False depending on the environment, but it never raises and is
# always a bool/int.
ok = compileall.compile_path(quiet=2)
assert isinstance(ok, (bool, int)), type(ok)
print("compile_path_returns_bool OK")
