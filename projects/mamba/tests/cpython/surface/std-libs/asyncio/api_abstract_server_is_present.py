# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_abstract_server_is_present"
# subject = "asyncio.AbstractServer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.AbstractServer: api_abstract_server_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "AbstractServer")
print("api_abstract_server_is_present OK")
