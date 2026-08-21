# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "behavior"
# case = "tokenize_test__test_int"
# subject = "cpython.test_tokenize.TokenizeTest.test_int"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tokenize.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tokenize.py::TokenizeTest::test_int
"""Auto-ported test: TokenizeTest::test_int (CPython 3.12 oracle)."""


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
check_tokenize('0xff <= 255', "    NUMBER     '0xff'        (1, 0) (1, 4)\n    OP         '<='          (1, 5) (1, 7)\n    NUMBER     '255'         (1, 8) (1, 11)\n    ")
check_tokenize('0b10 <= 255', "    NUMBER     '0b10'        (1, 0) (1, 4)\n    OP         '<='          (1, 5) (1, 7)\n    NUMBER     '255'         (1, 8) (1, 11)\n    ")
check_tokenize('0o123 <= 0O123', "    NUMBER     '0o123'       (1, 0) (1, 5)\n    OP         '<='          (1, 6) (1, 8)\n    NUMBER     '0O123'       (1, 9) (1, 14)\n    ")
check_tokenize('1234567 > ~0x15', "    NUMBER     '1234567'     (1, 0) (1, 7)\n    OP         '>'           (1, 8) (1, 9)\n    OP         '~'           (1, 10) (1, 11)\n    NUMBER     '0x15'        (1, 11) (1, 15)\n    ")
check_tokenize('2134568 != 1231515', "    NUMBER     '2134568'     (1, 0) (1, 7)\n    OP         '!='          (1, 8) (1, 10)\n    NUMBER     '1231515'     (1, 11) (1, 18)\n    ")
check_tokenize('(-124561-1) & 200000000', "    OP         '('           (1, 0) (1, 1)\n    OP         '-'           (1, 1) (1, 2)\n    NUMBER     '124561'      (1, 2) (1, 8)\n    OP         '-'           (1, 8) (1, 9)\n    NUMBER     '1'           (1, 9) (1, 10)\n    OP         ')'           (1, 10) (1, 11)\n    OP         '&'           (1, 12) (1, 13)\n    NUMBER     '200000000'   (1, 14) (1, 23)\n    ")
check_tokenize('0xdeadbeef != -1', "    NUMBER     '0xdeadbeef'  (1, 0) (1, 10)\n    OP         '!='          (1, 11) (1, 13)\n    OP         '-'           (1, 14) (1, 15)\n    NUMBER     '1'           (1, 15) (1, 16)\n    ")
check_tokenize('0xdeadc0de & 12345', "    NUMBER     '0xdeadc0de'  (1, 0) (1, 10)\n    OP         '&'           (1, 11) (1, 12)\n    NUMBER     '12345'       (1, 13) (1, 18)\n    ")
check_tokenize('0xFF & 0x15 | 1234', "    NUMBER     '0xFF'        (1, 0) (1, 4)\n    OP         '&'           (1, 5) (1, 6)\n    NUMBER     '0x15'        (1, 7) (1, 11)\n    OP         '|'           (1, 12) (1, 13)\n    NUMBER     '1234'        (1, 14) (1, 18)\n    ")
print("TokenizeTest::test_int: ok")
