# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "api_conversion_syntax_is_present"
# subject = "decimal.ConversionSyntax"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""decimal.ConversionSyntax: api_conversion_syntax_is_present (surface)."""
import decimal

assert hasattr(decimal, "ConversionSyntax")
print("api_conversion_syntax_is_present OK")
