# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "behavior"
# case = "generate_tokens_test__test_async"
# subject = "cpython.test_tokenize.GenerateTokensTest.test_async"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tokenize.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tokenize.py::GenerateTokensTest::test_async
"""Auto-ported test: GenerateTokensTest::test_async (CPython 3.12 oracle)."""


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
check_tokenize('async = 1', "    NAME       'async'       (1, 0) (1, 5)\n    OP         '='           (1, 6) (1, 7)\n    NUMBER     '1'           (1, 8) (1, 9)\n    ")
check_tokenize('a = (async = 1)', "    NAME       'a'           (1, 0) (1, 1)\n    OP         '='           (1, 2) (1, 3)\n    OP         '('           (1, 4) (1, 5)\n    NAME       'async'       (1, 5) (1, 10)\n    OP         '='           (1, 11) (1, 12)\n    NUMBER     '1'           (1, 13) (1, 14)\n    OP         ')'           (1, 14) (1, 15)\n    ")
check_tokenize('async()', "    NAME       'async'       (1, 0) (1, 5)\n    OP         '('           (1, 5) (1, 6)\n    OP         ')'           (1, 6) (1, 7)\n    ")
check_tokenize('class async(Bar):pass', "    NAME       'class'       (1, 0) (1, 5)\n    NAME       'async'       (1, 6) (1, 11)\n    OP         '('           (1, 11) (1, 12)\n    NAME       'Bar'         (1, 12) (1, 15)\n    OP         ')'           (1, 15) (1, 16)\n    OP         ':'           (1, 16) (1, 17)\n    NAME       'pass'        (1, 17) (1, 21)\n    ")
check_tokenize('class async:pass', "    NAME       'class'       (1, 0) (1, 5)\n    NAME       'async'       (1, 6) (1, 11)\n    OP         ':'           (1, 11) (1, 12)\n    NAME       'pass'        (1, 12) (1, 16)\n    ")
check_tokenize('await = 1', "    NAME       'await'       (1, 0) (1, 5)\n    OP         '='           (1, 6) (1, 7)\n    NUMBER     '1'           (1, 8) (1, 9)\n    ")
check_tokenize('foo.async', "    NAME       'foo'         (1, 0) (1, 3)\n    OP         '.'           (1, 3) (1, 4)\n    NAME       'async'       (1, 4) (1, 9)\n    ")
check_tokenize('async for a in b: pass', "    NAME       'async'       (1, 0) (1, 5)\n    NAME       'for'         (1, 6) (1, 9)\n    NAME       'a'           (1, 10) (1, 11)\n    NAME       'in'          (1, 12) (1, 14)\n    NAME       'b'           (1, 15) (1, 16)\n    OP         ':'           (1, 16) (1, 17)\n    NAME       'pass'        (1, 18) (1, 22)\n    ")
check_tokenize('async with a as b: pass', "    NAME       'async'       (1, 0) (1, 5)\n    NAME       'with'        (1, 6) (1, 10)\n    NAME       'a'           (1, 11) (1, 12)\n    NAME       'as'          (1, 13) (1, 15)\n    NAME       'b'           (1, 16) (1, 17)\n    OP         ':'           (1, 17) (1, 18)\n    NAME       'pass'        (1, 19) (1, 23)\n    ")
check_tokenize('async.foo', "    NAME       'async'       (1, 0) (1, 5)\n    OP         '.'           (1, 5) (1, 6)\n    NAME       'foo'         (1, 6) (1, 9)\n    ")
check_tokenize('async', "    NAME       'async'       (1, 0) (1, 5)\n    ")
check_tokenize('async\n#comment\nawait', "    NAME       'async'       (1, 0) (1, 5)\n    NEWLINE    '\\n'          (1, 5) (1, 6)\n    COMMENT    '#comment'    (2, 0) (2, 8)\n    NL         '\\n'          (2, 8) (2, 9)\n    NAME       'await'       (3, 0) (3, 5)\n    ")
check_tokenize('async\n...\nawait', "    NAME       'async'       (1, 0) (1, 5)\n    NEWLINE    '\\n'          (1, 5) (1, 6)\n    OP         '...'         (2, 0) (2, 3)\n    NEWLINE    '\\n'          (2, 3) (2, 4)\n    NAME       'await'       (3, 0) (3, 5)\n    ")
check_tokenize('async\nawait', "    NAME       'async'       (1, 0) (1, 5)\n    NEWLINE    '\\n'          (1, 5) (1, 6)\n    NAME       'await'       (2, 0) (2, 5)\n    ")
check_tokenize('foo.async + 1', "    NAME       'foo'         (1, 0) (1, 3)\n    OP         '.'           (1, 3) (1, 4)\n    NAME       'async'       (1, 4) (1, 9)\n    OP         '+'           (1, 10) (1, 11)\n    NUMBER     '1'           (1, 12) (1, 13)\n    ")
check_tokenize('async def foo(): pass', "    NAME       'async'       (1, 0) (1, 5)\n    NAME       'def'         (1, 6) (1, 9)\n    NAME       'foo'         (1, 10) (1, 13)\n    OP         '('           (1, 13) (1, 14)\n    OP         ')'           (1, 14) (1, 15)\n    OP         ':'           (1, 15) (1, 16)\n    NAME       'pass'        (1, 17) (1, 21)\n    ")
check_tokenize('async def foo():\n  def foo(await):\n    await = 1\n  if 1:\n    await\nasync += 1\n', "    NAME       'async'       (1, 0) (1, 5)\n    NAME       'def'         (1, 6) (1, 9)\n    NAME       'foo'         (1, 10) (1, 13)\n    OP         '('           (1, 13) (1, 14)\n    OP         ')'           (1, 14) (1, 15)\n    OP         ':'           (1, 15) (1, 16)\n    NEWLINE    '\\n'          (1, 16) (1, 17)\n    INDENT     '  '          (2, 0) (2, 2)\n    NAME       'def'         (2, 2) (2, 5)\n    NAME       'foo'         (2, 6) (2, 9)\n    OP         '('           (2, 9) (2, 10)\n    NAME       'await'       (2, 10) (2, 15)\n    OP         ')'           (2, 15) (2, 16)\n    OP         ':'           (2, 16) (2, 17)\n    NEWLINE    '\\n'          (2, 17) (2, 18)\n    INDENT     '    '        (3, 0) (3, 4)\n    NAME       'await'       (3, 4) (3, 9)\n    OP         '='           (3, 10) (3, 11)\n    NUMBER     '1'           (3, 12) (3, 13)\n    NEWLINE    '\\n'          (3, 13) (3, 14)\n    DEDENT     ''            (4, 2) (4, 2)\n    NAME       'if'          (4, 2) (4, 4)\n    NUMBER     '1'           (4, 5) (4, 6)\n    OP         ':'           (4, 6) (4, 7)\n    NEWLINE    '\\n'          (4, 7) (4, 8)\n    INDENT     '    '        (5, 0) (5, 4)\n    NAME       'await'       (5, 4) (5, 9)\n    NEWLINE    '\\n'          (5, 9) (5, 10)\n    DEDENT     ''            (6, 0) (6, 0)\n    DEDENT     ''            (6, 0) (6, 0)\n    NAME       'async'       (6, 0) (6, 5)\n    OP         '+='          (6, 6) (6, 8)\n    NUMBER     '1'           (6, 9) (6, 10)\n    NEWLINE    '\\n'          (6, 10) (6, 11)\n    ")
check_tokenize('async def foo():\n  async for i in 1: pass', "    NAME       'async'       (1, 0) (1, 5)\n    NAME       'def'         (1, 6) (1, 9)\n    NAME       'foo'         (1, 10) (1, 13)\n    OP         '('           (1, 13) (1, 14)\n    OP         ')'           (1, 14) (1, 15)\n    OP         ':'           (1, 15) (1, 16)\n    NEWLINE    '\\n'          (1, 16) (1, 17)\n    INDENT     '  '          (2, 0) (2, 2)\n    NAME       'async'       (2, 2) (2, 7)\n    NAME       'for'         (2, 8) (2, 11)\n    NAME       'i'           (2, 12) (2, 13)\n    NAME       'in'          (2, 14) (2, 16)\n    NUMBER     '1'           (2, 17) (2, 18)\n    OP         ':'           (2, 18) (2, 19)\n    NAME       'pass'        (2, 20) (2, 24)\n    DEDENT     ''            (3, 0) (3, 0)\n    ")
check_tokenize('async def foo(async): await', "    NAME       'async'       (1, 0) (1, 5)\n    NAME       'def'         (1, 6) (1, 9)\n    NAME       'foo'         (1, 10) (1, 13)\n    OP         '('           (1, 13) (1, 14)\n    NAME       'async'       (1, 14) (1, 19)\n    OP         ')'           (1, 19) (1, 20)\n    OP         ':'           (1, 20) (1, 21)\n    NAME       'await'       (1, 22) (1, 27)\n    ")
check_tokenize('def f():\n\n  def baz(): pass\n  async def bar(): pass\n\n  await = 2', "    NAME       'def'         (1, 0) (1, 3)\n    NAME       'f'           (1, 4) (1, 5)\n    OP         '('           (1, 5) (1, 6)\n    OP         ')'           (1, 6) (1, 7)\n    OP         ':'           (1, 7) (1, 8)\n    NEWLINE    '\\n'          (1, 8) (1, 9)\n    NL         '\\n'          (2, 0) (2, 1)\n    INDENT     '  '          (3, 0) (3, 2)\n    NAME       'def'         (3, 2) (3, 5)\n    NAME       'baz'         (3, 6) (3, 9)\n    OP         '('           (3, 9) (3, 10)\n    OP         ')'           (3, 10) (3, 11)\n    OP         ':'           (3, 11) (3, 12)\n    NAME       'pass'        (3, 13) (3, 17)\n    NEWLINE    '\\n'          (3, 17) (3, 18)\n    NAME       'async'       (4, 2) (4, 7)\n    NAME       'def'         (4, 8) (4, 11)\n    NAME       'bar'         (4, 12) (4, 15)\n    OP         '('           (4, 15) (4, 16)\n    OP         ')'           (4, 16) (4, 17)\n    OP         ':'           (4, 17) (4, 18)\n    NAME       'pass'        (4, 19) (4, 23)\n    NEWLINE    '\\n'          (4, 23) (4, 24)\n    NL         '\\n'          (5, 0) (5, 1)\n    NAME       'await'       (6, 2) (6, 7)\n    OP         '='           (6, 8) (6, 9)\n    NUMBER     '2'           (6, 10) (6, 11)\n    DEDENT     ''            (7, 0) (7, 0)\n    ")
check_tokenize('async def f():\n\n  def baz(): pass\n  async def bar(): pass\n\n  await = 2', "    NAME       'async'       (1, 0) (1, 5)\n    NAME       'def'         (1, 6) (1, 9)\n    NAME       'f'           (1, 10) (1, 11)\n    OP         '('           (1, 11) (1, 12)\n    OP         ')'           (1, 12) (1, 13)\n    OP         ':'           (1, 13) (1, 14)\n    NEWLINE    '\\n'          (1, 14) (1, 15)\n    NL         '\\n'          (2, 0) (2, 1)\n    INDENT     '  '          (3, 0) (3, 2)\n    NAME       'def'         (3, 2) (3, 5)\n    NAME       'baz'         (3, 6) (3, 9)\n    OP         '('           (3, 9) (3, 10)\n    OP         ')'           (3, 10) (3, 11)\n    OP         ':'           (3, 11) (3, 12)\n    NAME       'pass'        (3, 13) (3, 17)\n    NEWLINE    '\\n'          (3, 17) (3, 18)\n    NAME       'async'       (4, 2) (4, 7)\n    NAME       'def'         (4, 8) (4, 11)\n    NAME       'bar'         (4, 12) (4, 15)\n    OP         '('           (4, 15) (4, 16)\n    OP         ')'           (4, 16) (4, 17)\n    OP         ':'           (4, 17) (4, 18)\n    NAME       'pass'        (4, 19) (4, 23)\n    NEWLINE    '\\n'          (4, 23) (4, 24)\n    NL         '\\n'          (5, 0) (5, 1)\n    NAME       'await'       (6, 2) (6, 7)\n    OP         '='           (6, 8) (6, 9)\n    NUMBER     '2'           (6, 10) (6, 11)\n    DEDENT     ''            (7, 0) (7, 0)\n    ")
print("GenerateTokensTest::test_async: ok")
