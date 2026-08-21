# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "behavior"
# case = "generate_tokens_test__test_comparison"
# subject = "cpython.test_tokenize.GenerateTokensTest.test_comparison"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tokenize.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tokenize.py::GenerateTokensTest::test_comparison
"""Auto-ported test: GenerateTokensTest::test_comparison (CPython 3.12 oracle)."""


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
    f = StringIO(s)
    result = stringify_tokens_from_source(generate_tokens(f.readline), s)

    assert result == expected.rstrip().splitlines()
check_tokenize('if 1 < 1 > 1 == 1 >= 5 <= 0x15 <= 0x12 != 1 and 5 in 1 not in 1 is 1 or 5 is not 1: pass', "    NAME       'if'          (1, 0) (1, 2)\n    NUMBER     '1'           (1, 3) (1, 4)\n    OP         '<'           (1, 5) (1, 6)\n    NUMBER     '1'           (1, 7) (1, 8)\n    OP         '>'           (1, 9) (1, 10)\n    NUMBER     '1'           (1, 11) (1, 12)\n    OP         '=='          (1, 13) (1, 15)\n    NUMBER     '1'           (1, 16) (1, 17)\n    OP         '>='          (1, 18) (1, 20)\n    NUMBER     '5'           (1, 21) (1, 22)\n    OP         '<='          (1, 23) (1, 25)\n    NUMBER     '0x15'        (1, 26) (1, 30)\n    OP         '<='          (1, 31) (1, 33)\n    NUMBER     '0x12'        (1, 34) (1, 38)\n    OP         '!='          (1, 39) (1, 41)\n    NUMBER     '1'           (1, 42) (1, 43)\n    NAME       'and'         (1, 44) (1, 47)\n    NUMBER     '5'           (1, 48) (1, 49)\n    NAME       'in'          (1, 50) (1, 52)\n    NUMBER     '1'           (1, 53) (1, 54)\n    NAME       'not'         (1, 55) (1, 58)\n    NAME       'in'          (1, 59) (1, 61)\n    NUMBER     '1'           (1, 62) (1, 63)\n    NAME       'is'          (1, 64) (1, 66)\n    NUMBER     '1'           (1, 67) (1, 68)\n    NAME       'or'          (1, 69) (1, 71)\n    NUMBER     '5'           (1, 72) (1, 73)\n    NAME       'is'          (1, 74) (1, 76)\n    NAME       'not'         (1, 77) (1, 80)\n    NUMBER     '1'           (1, 81) (1, 82)\n    OP         ':'           (1, 82) (1, 83)\n    NAME       'pass'        (1, 84) (1, 88)\n    ")
print("GenerateTokensTest::test_comparison: ok")
