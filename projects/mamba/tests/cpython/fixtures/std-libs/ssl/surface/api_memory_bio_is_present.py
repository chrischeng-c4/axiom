# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_memory_bio_is_present"
# subject = "ssl.MemoryBIO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.MemoryBIO: api_memory_bio_is_present (surface)."""
import ssl

assert hasattr(ssl, "MemoryBIO")
print("api_memory_bio_is_present OK")
