# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "policy"
# dimension = "behavior"
# case = "test_concrete_policies__test_header_store_parse_rejects_newlines_uc79caa6"
# subject = "cpython.test_policy.TestConcretePolicies.test_header_store_parse_rejects_newlines"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_policy.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import io
import types
import textwrap
import email.errors
import email.policy
import email.parser
import email.generator
import email.message
from email import headerregistry
instance = email.policy.EmailPolicy()
try:
    instance.header_store_parse('From', 'spam\negg@foo.py')
    raise AssertionError('assertRaises: no raise')
except ValueError:
    pass

print("TestConcretePolicies::test_header_store_parse_rejects_newlines: ok")
