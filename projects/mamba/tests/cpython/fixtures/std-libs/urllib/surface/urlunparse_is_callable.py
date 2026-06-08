# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "surface"
# case = "urlunparse_is_callable"
# subject = "urllib.parse.urlunparse"
# kind = "mechanical"
# xfail = "mamba dotted-import quirk: urllib.parse.urlunparse -> None (repo-memory project_mamba_dotted_import_quirk)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse.urlunparse: urlunparse_is_callable (surface)."""
import urllib.parse

assert callable(urllib.parse.urlunparse)
print("urlunparse_is_callable OK")
