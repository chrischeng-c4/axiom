# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "import_asyncio"
# subject = "asyncio"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio: import_asyncio (surface)."""
import asyncio

assert hasattr(asyncio, "run")
print("import_asyncio OK")
