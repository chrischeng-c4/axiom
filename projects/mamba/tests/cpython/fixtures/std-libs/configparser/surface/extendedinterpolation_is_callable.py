# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "surface"
# case = "extendedinterpolation_is_callable"
# subject = "configparser.ExtendedInterpolation"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""configparser.ExtendedInterpolation: extendedinterpolation_is_callable (surface)."""
import configparser

assert callable(configparser.ExtendedInterpolation)
print("extendedinterpolation_is_callable OK")
