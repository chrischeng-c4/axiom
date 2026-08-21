# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyexpat"
# dimension = "behavior"
# case = "parent_parser_lifetime_test__test_parent_parser_outlives_its_subparsers__chain_uc5e41d0"
# subject = "cpython.test_pyexpat.ParentParserLifetimeTest.test_parent_parser_outlives_its_subparsers__chain"
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
subsubparser = subparser.ExternalEntityParserCreate(None)
del parser
del subparser

print("ParentParserLifetimeTest::test_parent_parser_outlives_its_subparsers__chain: ok")
