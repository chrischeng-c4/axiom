# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyexpat"
# dimension = "behavior"
# case = "parent_parser_lifetime_test__test_parent_parser_outlives_its_subparsers__multiple_ucd7a6c3"
# subject = "cpython.test_pyexpat.ParentParserLifetimeTest.test_parent_parser_outlives_its_subparsers__multiple"
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
subparser_one = parser.ExternalEntityParserCreate(None)
subparser_two = parser.ExternalEntityParserCreate(None)
del parser

print("ParentParserLifetimeTest::test_parent_parser_outlives_its_subparsers__multiple: ok")
