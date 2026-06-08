# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "behavior"
# case = "test_roundtrip__test_roundtrip"
# subject = "cpython.test_tokenize.TestRoundtrip.test_roundtrip"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tokenize.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tokenize.py::TestRoundtrip::test_roundtrip
"""Auto-ported test: TestRoundtrip::test_roundtrip (CPython 3.12 oracle)."""


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
def check_line_extraction(f):
    if isinstance(f, str):
        code = f.encode('utf-8')
    else:
        code = f.read()
    readline = iter(code.splitlines(keepends=True)).__next__
    for tok in tokenize(readline):
        if tok.type in {ENCODING, ENDMARKER}:
            continue

        assert tok.string == tok.line[tok.start[1]:tok.end[1]]

def check_roundtrip(f):
    """
        Test roundtrip for `untokenize`. `f` is an open file or a string.
        The source code in f is tokenized to both 5- and 2-tuples.
        Both sequences are converted back to source code via
        tokenize.untokenize(), and the latter tokenized again to 2-tuples.
        The test fails if the 3 pair tokenizations do not match.

        If the source code can be untokenized unambiguously, the
        untokenized code must match the original code exactly.

        When untokenize bugs are fixed, untokenize with 5-tuples should
        reproduce code that does not contain a backslash continuation
        following spaces.  A proper test should test this.
        """
    if isinstance(f, str):
        code = f.encode('utf-8')
    else:
        code = f.read()
    readline = iter(code.splitlines(keepends=True)).__next__
    tokens5 = list(tokenize(readline))
    tokens2 = [tok[:2] for tok in tokens5]
    bytes_from2 = untokenize(tokens2)
    readline2 = iter(bytes_from2.splitlines(keepends=True)).__next__
    tokens2_from2 = [tok[:2] for tok in tokenize(readline2)]

    assert tokens2_from2 == tokens2
    bytes_from5 = untokenize(tokens5)
    readline5 = iter(bytes_from5.splitlines(keepends=True)).__next__
    tokens2_from5 = [tok[:2] for tok in tokenize(readline5)]

    assert tokens2_from5 == tokens2
    if not contains_ambiguous_backslash(code):
        code_without_bom = code.removeprefix(b'\xef\xbb\xbf')
        readline = iter(code_without_bom.splitlines(keepends=True)).__next__
        untokenized_code = untokenize(tokenize(readline))

        assert code_without_bom == untokenized_code

def roundtrip(code):
    if isinstance(code, str):
        code = code.encode('utf-8')
    return untokenize(tokenize(BytesIO(code).readline)).decode('utf-8')
check_roundtrip('if x == 1:\n    print(x)\n')
check_roundtrip('# This is a comment\n# This also\n')
check_roundtrip('if x == 1 : \n  print(x)\n')
fn = support.findfile('tokenize_tests.txt', subdir='tokenizedata')
with open(fn, 'rb') as f:
    check_roundtrip(f)
check_roundtrip('if x == 1:\n    # A comment by itself.\n    print(x) # Comment here, too.\n    # Another comment.\nafter_if = True\n')
check_roundtrip("if (x # The comments need to go in the right place\n    == 1):\n    print('x==1')\n")
check_roundtrip('class Test: # A comment here\n  # A comment with weird indent\n  after_com = 5\n  def x(m): return m*5 # a one liner\n  def y(m): # A whitespace after the colon\n     return y*4 # 3-space indent\n')
check_roundtrip("try: import somemodule\nexcept ImportError: # comment\n    print('Can not import' # comment2\n)else:   print('Loaded')\n")
check_roundtrip("f'\\N{EXCLAMATION MARK}'")
check_roundtrip("f'\\\\N{SNAKE}'")
check_roundtrip("f'\\\\N{{SNAKE}}'")
check_roundtrip("f'\\N{SNAKE}'")
check_roundtrip("f'\\\\\\N{SNAKE}'")
check_roundtrip("f'\\\\\\\\\\N{SNAKE}'")
check_roundtrip("f'\\\\\\\\\\\\\\N{SNAKE}'")
check_roundtrip("f'\\\\N{1}'")
check_roundtrip("f'\\\\\\\\N{2}'")
check_roundtrip("f'\\\\\\\\\\\\N{3}'")
check_roundtrip("f'\\\\\\\\\\\\\\\\N{4}'")
check_roundtrip("f'\\\\N{{'")
check_roundtrip("f'\\\\\\\\N{{'")
check_roundtrip("f'\\\\\\\\\\\\N{{'")
check_roundtrip("f'\\\\\\\\\\\\\\\\N{{'")
check_roundtrip("f'\\n{{foo}}'")
check_roundtrip("f'\\\\n{{foo}}'")
check_roundtrip("f'\\\\\\n{{foo}}'")
check_roundtrip("f'\\\\\\\\n{{foo}}'")
check_roundtrip("f'\\t{{foo}}'")
check_roundtrip("f'\\\\t{{foo}}'")
check_roundtrip("f'\\\\\\t{{foo}}'")
check_roundtrip("f'\\\\\\\\t{{foo}}'")
check_roundtrip("rf'\\t{{foo}}'")
check_roundtrip("rf'\\\\t{{foo}}'")
check_roundtrip("rf'\\\\\\t{{foo}}'")
check_roundtrip("rf'\\\\\\\\t{{foo}}'")
check_roundtrip("rf'\\{{foo}}'")
check_roundtrip("f'\\\\{{foo}}'")
check_roundtrip("rf'\\\\\\{{foo}}'")
check_roundtrip("f'\\\\\\\\{{foo}}'")
cases = ['\nif 1:\n    "foo"\n"bar"\n', '\nif 1:\n    ("foo"\n     "bar")\n', '\nif 1:\n    "foo"\n    "bar"\n']
for case in cases:
    check_roundtrip(case)
print("TestRoundtrip::test_roundtrip: ok")
