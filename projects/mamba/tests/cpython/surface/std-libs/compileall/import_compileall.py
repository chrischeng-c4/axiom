# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "surface"
# case = "import_compileall"
# subject = "compileall"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""compileall: import_compileall (surface)."""
import compileall

assert hasattr(compileall, "compile_dir")
print("import_compileall OK")
