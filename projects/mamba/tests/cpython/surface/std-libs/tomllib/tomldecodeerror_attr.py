# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "surface"
# case = "tomldecodeerror_attr"
# subject = "tomllib"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tomllib: tomldecodeerror_attr (surface)."""
import tomllib

assert hasattr(tomllib, "TOMLDecodeError")
print("tomldecodeerror_attr OK")
