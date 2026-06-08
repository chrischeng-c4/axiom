# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "surface"
# case = "unquote_plus_is_callable"
# subject = "urllib.parse.unquote_plus"
# kind = "mechanical"
# xfail = "mamba dotted-import quirk: urllib.parse.unquote_plus -> None (repo-memory project_mamba_dotted_import_quirk)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse.unquote_plus: unquote_plus_is_callable (surface)."""
import urllib.parse

assert callable(urllib.parse.unquote_plus)
print("unquote_plus_is_callable OK")
