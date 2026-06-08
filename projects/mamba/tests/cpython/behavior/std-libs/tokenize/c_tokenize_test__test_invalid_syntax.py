# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "behavior"
# case = "c_tokenize_test__test_invalid_syntax"
# subject = "cpython.test_tokenize.CTokenizeTest.test_invalid_syntax"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tokenize.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tokenize.py::CTokenizeTest::test_invalid_syntax
"""Auto-ported test: CTokenizeTest::test_invalid_syntax (CPython 3.12 oracle)."""


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
    result = stringify_tokens_from_source(_generate_tokens_from_c_tokenizer(f.readline), s)

    assert result == expected.rstrip().splitlines()

def get_tokens(string):
    the_string = StringIO(string)
    return list(_generate_tokens_from_c_tokenizer(the_string.readline))
for case in ['(1+2]', '(1+2}', '{1+2]', '1_', '1.2_', '1e2_', '1e+', '\xa0', '€', '0b12', '0b1_2', '0b2', '0b1_', '0b', '0o18', '0o1_8', '0o8', '0o1_', '0o', '0x1_', '0x', '1_', '012', '1.2_', '1e2_', '1e+', "'sdfsdf", "'''sdfsdf''", '(' * 1000 + 'a' + ')' * 1000, ']']:

    try:
        get_tokens(case)
        raise AssertionError('expected TokenError')
    except TokenError:
        pass
print("CTokenizeTest::test_invalid_syntax: ok")
