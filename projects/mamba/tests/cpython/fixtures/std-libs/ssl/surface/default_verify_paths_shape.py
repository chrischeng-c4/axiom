# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "default_verify_paths_shape"
# subject = "ssl.get_default_verify_paths"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.get_default_verify_paths: get_default_verify_paths() returns a record exposing cafile and capath attributes"""
import ssl

_paths = ssl.get_default_verify_paths()
assert hasattr(_paths, "cafile"), "cafile attr"
assert hasattr(_paths, "capath"), "capath attr"

print("default_verify_paths_shape OK")
