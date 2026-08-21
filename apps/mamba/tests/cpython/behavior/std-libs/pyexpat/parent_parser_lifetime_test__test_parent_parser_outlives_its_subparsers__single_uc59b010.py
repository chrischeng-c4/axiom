# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyexpat"
# dimension = "behavior"
# case = "parent_parser_lifetime_test__test_parent_parser_outlives_its_subparsers__single_uc59b010"
# subject = "cpython.test_pyexpat.ParentParserLifetimeTest.test_parent_parser_outlives_its_subparsers__single"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pyexpat.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import os
import platform
import sys
import sysconfig
import traceback
from io import BytesIO
from xml.parsers import expat
from xml.parsers.expat import errors
parser = expat.ParserCreate()
subparser = parser.ExternalEntityParserCreate(None)
del parser

print("ParentParserLifetimeTest::test_parent_parser_outlives_its_subparsers__single: ok")
