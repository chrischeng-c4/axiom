# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "surface"
# case = "import_token"
# subject = "token"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""token: import_token (surface)."""
import token

assert hasattr(token, "tok_name")
print("import_token OK")
