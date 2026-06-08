# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "behavior"
# case = "tokenize_test__test_unary"
# subject = "cpython.test_tokenize.TokenizeTest.test_unary"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tokenize.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tokenize.py::TokenizeTest::test_unary
"""Auto-ported test: TokenizeTest::test_unary (CPython 3.12 oracle)."""


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
check_tokenize('~1 ^ 1 & 1 |1 ^ -1', "    OP         '~'           (1, 0) (1, 1)\n    NUMBER     '1'           (1, 1) (1, 2)\n    OP         '^'           (1, 3) (1, 4)\n    NUMBER     '1'           (1, 5) (1, 6)\n    OP         '&'           (1, 7) (1, 8)\n    NUMBER     '1'           (1, 9) (1, 10)\n    OP         '|'           (1, 11) (1, 12)\n    NUMBER     '1'           (1, 12) (1, 13)\n    OP         '^'           (1, 14) (1, 15)\n    OP         '-'           (1, 16) (1, 17)\n    NUMBER     '1'           (1, 17) (1, 18)\n    ")
check_tokenize('-1*1/1+1*1//1 - ---1**1', "    OP         '-'           (1, 0) (1, 1)\n    NUMBER     '1'           (1, 1) (1, 2)\n    OP         '*'           (1, 2) (1, 3)\n    NUMBER     '1'           (1, 3) (1, 4)\n    OP         '/'           (1, 4) (1, 5)\n    NUMBER     '1'           (1, 5) (1, 6)\n    OP         '+'           (1, 6) (1, 7)\n    NUMBER     '1'           (1, 7) (1, 8)\n    OP         '*'           (1, 8) (1, 9)\n    NUMBER     '1'           (1, 9) (1, 10)\n    OP         '//'          (1, 10) (1, 12)\n    NUMBER     '1'           (1, 12) (1, 13)\n    OP         '-'           (1, 14) (1, 15)\n    OP         '-'           (1, 16) (1, 17)\n    OP         '-'           (1, 17) (1, 18)\n    OP         '-'           (1, 18) (1, 19)\n    NUMBER     '1'           (1, 19) (1, 20)\n    OP         '**'          (1, 20) (1, 22)\n    NUMBER     '1'           (1, 22) (1, 23)\n    ")
print("TokenizeTest::test_unary: ok")
