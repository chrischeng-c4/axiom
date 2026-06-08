# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "api_seek_cur_is_present"
# subject = "io.SEEK_CUR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""io.SEEK_CUR: api_seek_cur_is_present (surface)."""
import io

assert hasattr(io, "SEEK_CUR")
print("api_seek_cur_is_present OK")
