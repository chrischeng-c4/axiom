# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "import_configparser"
# subject = "configparser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""configparser: import_configparser (surface)."""
import configparser

assert hasattr(configparser, "ConfigParser")
print("import_configparser OK")
