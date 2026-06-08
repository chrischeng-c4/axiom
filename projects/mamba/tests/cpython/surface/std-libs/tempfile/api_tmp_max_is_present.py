# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "api_tmp_max_is_present"
# subject = "tempfile.TMP_MAX"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tempfile.TMP_MAX: api_tmp_max_is_present (surface)."""
import tempfile

assert hasattr(tempfile, "TMP_MAX")
print("api_tmp_max_is_present OK")
