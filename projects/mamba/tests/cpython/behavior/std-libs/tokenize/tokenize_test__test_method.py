# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "behavior"
# case = "tokenize_test__test_method"
# subject = "cpython.test_tokenize.TokenizeTest.test_method"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tokenize.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tokenize.py::TokenizeTest::test_method
"""Auto-ported test: TokenizeTest::test_method (CPython 3.12 oracle)."""


import os
import re
import token
import unittest
from tokenize import tokenize, untokenize, NUMBER, NAME, OP, STRING, ENDMARKER, ENCODING, tok_name, detect_encoding, open as tokenize_open, Untokenizer, generate_tokens, NEWLINE, _generate_tokens_from_c_tokenizer, DEDENT, TokenInfo, TokenError
from io import BytesIO, StringIO
from textwrap import dedent
from unittest import TestCase, mock
from test import support
from test.test_grammar import VALID_UNDERSCORE_LITERALS, INVALID_UNDERSCORE_LITERALS
from test.support import os_helper
from test.support.script_helper import run_test_script, make_script, run_python_until_end


def stringify_tokens_from_source(token_generator, source_string):
    result = []
    num_lines = len(source_string.splitlines())
    missing_trailing_nl = source_string[-1] not in '\r\n'
    for type, token, start, end, line in token_generator:
        if type == ENDMARKER:
            break
        if missing_trailing_nl and type == NEWLINE and (end[0] == num_lines):
            continue
        type = tok_name[type]
        result.append(f'    {type:10} {token!r:13} {start} {end}')
    return result

def decistmt(s):
    result = []
    g = tokenize(BytesIO(s.encode('utf-8')).readline)
    for toknum, tokval, _, _, _ in g:
        if toknum == NUMBER and '.' in tokval:
            result.extend([(NAME, 'Decimal'), (OP, '('), (STRING, repr(tokval)), (OP, ')')])
        else:
            result.append((toknum, tokval))
    return untokenize(result).decode('utf-8').strip()

def contains_ambiguous_backslash(source):
    """Return `True` if the source contains a backslash on a
    line by itself. For example:

    a = (1
        \\
    )

    Code like this cannot be untokenized exactly. This is because
    the tokenizer does not produce any tokens for the line containing
    the backslash and so there is no way to know its indent.
    """
    pattern = re.compile(b'\\n\\s*\\\\\\r?\\n')
    return pattern.search(source) is not None


# --- test body ---
def check_tokenize(s, expected):
    f = BytesIO(s.encode('utf-8'))
    result = stringify_tokens_from_source(tokenize(f.readline), s)

    assert result == ["    ENCODING   'utf-8'       (0, 0) (0, 0)"] + expected.rstrip().splitlines()
check_tokenize('@staticmethod\ndef foo(x,y): pass', "    OP         '@'           (1, 0) (1, 1)\n    NAME       'staticmethod' (1, 1) (1, 13)\n    NEWLINE    '\\n'          (1, 13) (1, 14)\n    NAME       'def'         (2, 0) (2, 3)\n    NAME       'foo'         (2, 4) (2, 7)\n    OP         '('           (2, 7) (2, 8)\n    NAME       'x'           (2, 8) (2, 9)\n    OP         ','           (2, 9) (2, 10)\n    NAME       'y'           (2, 10) (2, 11)\n    OP         ')'           (2, 11) (2, 12)\n    OP         ':'           (2, 12) (2, 13)\n    NAME       'pass'        (2, 14) (2, 18)\n    ")
print("TokenizeTest::test_method: ok")
