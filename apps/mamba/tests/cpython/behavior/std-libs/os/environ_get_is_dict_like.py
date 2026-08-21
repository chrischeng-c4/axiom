# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "environ_get_is_dict_like"
# subject = "os.environ"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.environ: os.environ behaves like a dict: .get(PATH) is None or str, and subscript/get are available"""
import os

assert hasattr(os.environ, "__getitem__"), "environ subscriptable"
assert hasattr(os.environ, "get"), "environ has get"
path = os.environ.get("PATH")
assert path is None or isinstance(path, str), f"PATH type = {type(path)!r}"
assert isinstance(os.environ.get("PATH", ""), str), "PATH default is str"
print("environ_get_is_dict_like OK")
