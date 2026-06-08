# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "api_converter_mapping_is_present"
# subject = "configparser.ConverterMapping"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""configparser.ConverterMapping: api_converter_mapping_is_present (surface)."""
import configparser

assert hasattr(configparser, "ConverterMapping")
print("api_converter_mapping_is_present OK")
