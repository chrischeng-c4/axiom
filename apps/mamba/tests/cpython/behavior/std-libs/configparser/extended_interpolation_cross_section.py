# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "configparser"
# dimension = "behavior"
# case = "extended_interpolation_cross_section"
# subject = "configparser.ExtendedInterpolation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_configparser.py"
# status = "filled"
# ///
"""configparser.ExtendedInterpolation: ExtendedInterpolation resolves ${section:option} cross-section references (${paths:home}/bob -> /home/bob)"""
import configparser

ext = configparser.ConfigParser(interpolation=configparser.ExtendedInterpolation())
ext.read_string("[paths]\nhome = /home\n[user]\ndir = ${paths:home}/bob\n")
assert ext.get("user", "dir") == "/home/bob", f"extended = {ext.get('user', 'dir')!r}"

print("extended_interpolation_cross_section OK")
