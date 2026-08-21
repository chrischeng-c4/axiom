# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "modules_caches_imported"
# subject = "sys.modules"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.modules: sys.modules caches imported modules by name: sys.modules['math'] is the math object and sys.modules['os'] is the os object"""
import sys
import math
import os

assert sys.modules["math"] is math, "sys.modules[math] is math"
assert sys.modules["os"] is os, "sys.modules[os] is os"
print("modules_caches_imported OK")
