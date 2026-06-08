# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "behavior"
# case = "c_tokenize_test__test_continuation_lines_indentation"
# subject = "cpython.test_tokenize.CTokenizeTest.test_continuation_lines_indentation"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tokenize.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tokenize.py::CTokenizeTest::test_continuation_lines_indentation
"""Auto-ported test: CTokenizeTest::test_continuation_lines_indentation (CPython 3.12 oracle)."""


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
    return [(kind, string) for kind, string, *_ in _generate_tokens_from_c_tokenizer(the_string.readline)]
code = dedent("\n            def fib(n):\n                \\\n            '''Print a Fibonacci series up to n.'''\n                \\\n            a, b = 0, 1\n        ")
check_tokenize(code, '    NAME       \'def\'         (2, 0) (2, 3)\n    NAME       \'fib\'         (2, 4) (2, 7)\n    LPAR       \'(\'           (2, 7) (2, 8)\n    NAME       \'n\'           (2, 8) (2, 9)\n    RPAR       \')\'           (2, 9) (2, 10)\n    COLON      \':\'           (2, 10) (2, 11)\n    NEWLINE    \'\'            (2, 11) (2, 11)\n    INDENT     \'\'            (4, -1) (4, -1)\n    STRING     "\'\'\'Print a Fibonacci series up to n.\'\'\'" (4, 0) (4, 39)\n    NEWLINE    \'\'            (4, 39) (4, 39)\n    NAME       \'a\'           (6, 0) (6, 1)\n    COMMA      \',\'           (6, 1) (6, 2)\n    NAME       \'b\'           (6, 3) (6, 4)\n    EQUAL      \'=\'           (6, 5) (6, 6)\n    NUMBER     \'0\'           (6, 7) (6, 8)\n    COMMA      \',\'           (6, 8) (6, 9)\n    NUMBER     \'1\'           (6, 10) (6, 11)\n    NEWLINE    \'\'            (6, 11) (6, 11)\n    DEDENT     \'\'            (6, -1) (6, -1)\n        ')
code_no_cont = dedent("\n            def fib(n):\n                '''Print a Fibonacci series up to n.'''\n                a, b = 0, 1\n        ")

assert get_tokens(code) == get_tokens(code_no_cont)
code = dedent('\n            pass\n                \\\n\n            pass\n        ')
check_tokenize(code, "    NAME       'pass'        (2, 0) (2, 4)\n    NEWLINE    ''            (2, 4) (2, 4)\n    NAME       'pass'        (5, 0) (5, 4)\n    NEWLINE    ''            (5, 4) (5, 4)\n        ")
code_no_cont = dedent('\n            pass\n            pass\n        ')

assert get_tokens(code) == get_tokens(code_no_cont)
code = dedent('\n            if x:\n                y = 1\n                \\\n                        \\\n                    \\\n                \\\n                foo = 1\n        ')
check_tokenize(code, "    NAME       'if'          (2, 0) (2, 2)\n    NAME       'x'           (2, 3) (2, 4)\n    COLON      ':'           (2, 4) (2, 5)\n    NEWLINE    ''            (2, 5) (2, 5)\n    INDENT     ''            (3, -1) (3, -1)\n    NAME       'y'           (3, 4) (3, 5)\n    EQUAL      '='           (3, 6) (3, 7)\n    NUMBER     '1'           (3, 8) (3, 9)\n    NEWLINE    ''            (3, 9) (3, 9)\n    NAME       'foo'         (8, 4) (8, 7)\n    EQUAL      '='           (8, 8) (8, 9)\n    NUMBER     '1'           (8, 10) (8, 11)\n    NEWLINE    ''            (8, 11) (8, 11)\n    DEDENT     ''            (8, -1) (8, -1)\n        ")
code_no_cont = dedent('\n            if x:\n                y = 1\n                foo = 1\n        ')

assert get_tokens(code) == get_tokens(code_no_cont)
print("CTokenizeTest::test_continuation_lines_indentation: ok")
