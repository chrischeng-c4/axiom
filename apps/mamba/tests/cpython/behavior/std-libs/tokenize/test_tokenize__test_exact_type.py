# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "behavior"
# case = "test_tokenize__test_exact_type"
# subject = "cpython.test_tokenize.TestTokenize.test_exact_type"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tokenize.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tokenize.py::TestTokenize::test_exact_type
"""Auto-ported test: TestTokenize::test_exact_type (CPython 3.12 oracle)."""


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
def assertExactTypeEqual(opstr, *optypes):
    tokens = list(tokenize(BytesIO(opstr.encode('utf-8')).readline))
    num_optypes = len(optypes)

    assert len(tokens) == 3 + num_optypes

    assert tok_name[tokens[0].exact_type] == tok_name[ENCODING]
    for i in range(num_optypes):

        assert tok_name[tokens[i + 1].exact_type] == tok_name[optypes[i]]

    assert tok_name[tokens[1 + num_optypes].exact_type] == tok_name[token.NEWLINE]

    assert tok_name[tokens[2 + num_optypes].exact_type] == tok_name[token.ENDMARKER]
assertExactTypeEqual('()', token.LPAR, token.RPAR)
assertExactTypeEqual('[]', token.LSQB, token.RSQB)
assertExactTypeEqual(':', token.COLON)
assertExactTypeEqual(',', token.COMMA)
assertExactTypeEqual(';', token.SEMI)
assertExactTypeEqual('+', token.PLUS)
assertExactTypeEqual('-', token.MINUS)
assertExactTypeEqual('*', token.STAR)
assertExactTypeEqual('/', token.SLASH)
assertExactTypeEqual('|', token.VBAR)
assertExactTypeEqual('&', token.AMPER)
assertExactTypeEqual('<', token.LESS)
assertExactTypeEqual('>', token.GREATER)
assertExactTypeEqual('=', token.EQUAL)
assertExactTypeEqual('.', token.DOT)
assertExactTypeEqual('%', token.PERCENT)
assertExactTypeEqual('{}', token.LBRACE, token.RBRACE)
assertExactTypeEqual('==', token.EQEQUAL)
assertExactTypeEqual('!=', token.NOTEQUAL)
assertExactTypeEqual('<=', token.LESSEQUAL)
assertExactTypeEqual('>=', token.GREATEREQUAL)
assertExactTypeEqual('~', token.TILDE)
assertExactTypeEqual('^', token.CIRCUMFLEX)
assertExactTypeEqual('<<', token.LEFTSHIFT)
assertExactTypeEqual('>>', token.RIGHTSHIFT)
assertExactTypeEqual('**', token.DOUBLESTAR)
assertExactTypeEqual('+=', token.PLUSEQUAL)
assertExactTypeEqual('-=', token.MINEQUAL)
assertExactTypeEqual('*=', token.STAREQUAL)
assertExactTypeEqual('/=', token.SLASHEQUAL)
assertExactTypeEqual('%=', token.PERCENTEQUAL)
assertExactTypeEqual('&=', token.AMPEREQUAL)
assertExactTypeEqual('|=', token.VBAREQUAL)
assertExactTypeEqual('^=', token.CIRCUMFLEXEQUAL)
assertExactTypeEqual('^=', token.CIRCUMFLEXEQUAL)
assertExactTypeEqual('<<=', token.LEFTSHIFTEQUAL)
assertExactTypeEqual('>>=', token.RIGHTSHIFTEQUAL)
assertExactTypeEqual('**=', token.DOUBLESTAREQUAL)
assertExactTypeEqual('//', token.DOUBLESLASH)
assertExactTypeEqual('//=', token.DOUBLESLASHEQUAL)
assertExactTypeEqual(':=', token.COLONEQUAL)
assertExactTypeEqual('...', token.ELLIPSIS)
assertExactTypeEqual('->', token.RARROW)
assertExactTypeEqual('@', token.AT)
assertExactTypeEqual('@=', token.ATEQUAL)
assertExactTypeEqual('a**2+b**2==c**2', NAME, token.DOUBLESTAR, NUMBER, token.PLUS, NAME, token.DOUBLESTAR, NUMBER, token.EQEQUAL, NAME, token.DOUBLESTAR, NUMBER)
assertExactTypeEqual('{1, 2, 3}', token.LBRACE, token.NUMBER, token.COMMA, token.NUMBER, token.COMMA, token.NUMBER, token.RBRACE)
assertExactTypeEqual('^(x & 0x1)', token.CIRCUMFLEX, token.LPAR, token.NAME, token.AMPER, token.NUMBER, token.RPAR)
print("TestTokenize::test_exact_type: ok")
