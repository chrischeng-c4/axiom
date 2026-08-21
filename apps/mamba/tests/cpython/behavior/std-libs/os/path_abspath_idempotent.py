# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "path_abspath_idempotent"
# subject = "os.path.abspath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.abspath: os.path.abspath of an already-absolute path is idempotent and isabs reports it absolute"""
import os.path

rel = os.path.abspath(".")
assert os.path.isabs(rel), f"abspath is absolute: {rel!r}"
again = os.path.abspath(rel)
assert rel == again, "abspath idempotent on absolute"
print("path_abspath_idempotent OK")
