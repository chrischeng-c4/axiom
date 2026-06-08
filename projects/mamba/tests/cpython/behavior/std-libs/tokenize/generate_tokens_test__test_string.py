# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "behavior"
# case = "generate_tokens_test__test_string"
# subject = "cpython.test_tokenize.GenerateTokensTest.test_string"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tokenize.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tokenize.py::GenerateTokensTest::test_string
"""Auto-ported test: GenerateTokensTest::test_string (CPython 3.12 oracle)."""


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
check_tokenize('x = \'\'; y = ""', '    NAME       \'x\'           (1, 0) (1, 1)\n    OP         \'=\'           (1, 2) (1, 3)\n    STRING     "\'\'"          (1, 4) (1, 6)\n    OP         \';\'           (1, 6) (1, 7)\n    NAME       \'y\'           (1, 8) (1, 9)\n    OP         \'=\'           (1, 10) (1, 11)\n    STRING     \'""\'          (1, 12) (1, 14)\n    ')
check_tokenize('x = \'"\'; y = "\'"', '    NAME       \'x\'           (1, 0) (1, 1)\n    OP         \'=\'           (1, 2) (1, 3)\n    STRING     \'\\\'"\\\'\'       (1, 4) (1, 7)\n    OP         \';\'           (1, 7) (1, 8)\n    NAME       \'y\'           (1, 9) (1, 10)\n    OP         \'=\'           (1, 11) (1, 12)\n    STRING     \'"\\\'"\'        (1, 13) (1, 16)\n    ')
check_tokenize('x = "doesn\'t "shrink", does it"', '    NAME       \'x\'           (1, 0) (1, 1)\n    OP         \'=\'           (1, 2) (1, 3)\n    STRING     \'"doesn\\\'t "\' (1, 4) (1, 14)\n    NAME       \'shrink\'      (1, 14) (1, 20)\n    STRING     \'", does it"\' (1, 20) (1, 31)\n    ')
check_tokenize("x = 'abc' + 'ABC'", '    NAME       \'x\'           (1, 0) (1, 1)\n    OP         \'=\'           (1, 2) (1, 3)\n    STRING     "\'abc\'"       (1, 4) (1, 9)\n    OP         \'+\'           (1, 10) (1, 11)\n    STRING     "\'ABC\'"       (1, 12) (1, 17)\n    ')
check_tokenize('y = "ABC" + "ABC"', '    NAME       \'y\'           (1, 0) (1, 1)\n    OP         \'=\'           (1, 2) (1, 3)\n    STRING     \'"ABC"\'       (1, 4) (1, 9)\n    OP         \'+\'           (1, 10) (1, 11)\n    STRING     \'"ABC"\'       (1, 12) (1, 17)\n    ')
check_tokenize("x = r'abc' + r'ABC' + R'ABC' + R'ABC'", '    NAME       \'x\'           (1, 0) (1, 1)\n    OP         \'=\'           (1, 2) (1, 3)\n    STRING     "r\'abc\'"      (1, 4) (1, 10)\n    OP         \'+\'           (1, 11) (1, 12)\n    STRING     "r\'ABC\'"      (1, 13) (1, 19)\n    OP         \'+\'           (1, 20) (1, 21)\n    STRING     "R\'ABC\'"      (1, 22) (1, 28)\n    OP         \'+\'           (1, 29) (1, 30)\n    STRING     "R\'ABC\'"      (1, 31) (1, 37)\n    ')
check_tokenize('y = r"abc" + r"ABC" + R"ABC" + R"ABC"', '    NAME       \'y\'           (1, 0) (1, 1)\n    OP         \'=\'           (1, 2) (1, 3)\n    STRING     \'r"abc"\'      (1, 4) (1, 10)\n    OP         \'+\'           (1, 11) (1, 12)\n    STRING     \'r"ABC"\'      (1, 13) (1, 19)\n    OP         \'+\'           (1, 20) (1, 21)\n    STRING     \'R"ABC"\'      (1, 22) (1, 28)\n    OP         \'+\'           (1, 29) (1, 30)\n    STRING     \'R"ABC"\'      (1, 31) (1, 37)\n    ')
check_tokenize("u'abc' + U'abc'", '    STRING     "u\'abc\'"      (1, 0) (1, 6)\n    OP         \'+\'           (1, 7) (1, 8)\n    STRING     "U\'abc\'"      (1, 9) (1, 15)\n    ')
check_tokenize('u"abc" + U"abc"', '    STRING     \'u"abc"\'      (1, 0) (1, 6)\n    OP         \'+\'           (1, 7) (1, 8)\n    STRING     \'U"abc"\'      (1, 9) (1, 15)\n    ')
check_tokenize("b'abc' + B'abc'", '    STRING     "b\'abc\'"      (1, 0) (1, 6)\n    OP         \'+\'           (1, 7) (1, 8)\n    STRING     "B\'abc\'"      (1, 9) (1, 15)\n    ')
check_tokenize('b"abc" + B"abc"', '    STRING     \'b"abc"\'      (1, 0) (1, 6)\n    OP         \'+\'           (1, 7) (1, 8)\n    STRING     \'B"abc"\'      (1, 9) (1, 15)\n    ')
check_tokenize("br'abc' + bR'abc' + Br'abc' + BR'abc'", '    STRING     "br\'abc\'"     (1, 0) (1, 7)\n    OP         \'+\'           (1, 8) (1, 9)\n    STRING     "bR\'abc\'"     (1, 10) (1, 17)\n    OP         \'+\'           (1, 18) (1, 19)\n    STRING     "Br\'abc\'"     (1, 20) (1, 27)\n    OP         \'+\'           (1, 28) (1, 29)\n    STRING     "BR\'abc\'"     (1, 30) (1, 37)\n    ')
check_tokenize('br"abc" + bR"abc" + Br"abc" + BR"abc"', '    STRING     \'br"abc"\'     (1, 0) (1, 7)\n    OP         \'+\'           (1, 8) (1, 9)\n    STRING     \'bR"abc"\'     (1, 10) (1, 17)\n    OP         \'+\'           (1, 18) (1, 19)\n    STRING     \'Br"abc"\'     (1, 20) (1, 27)\n    OP         \'+\'           (1, 28) (1, 29)\n    STRING     \'BR"abc"\'     (1, 30) (1, 37)\n    ')
check_tokenize("rb'abc' + rB'abc' + Rb'abc' + RB'abc'", '    STRING     "rb\'abc\'"     (1, 0) (1, 7)\n    OP         \'+\'           (1, 8) (1, 9)\n    STRING     "rB\'abc\'"     (1, 10) (1, 17)\n    OP         \'+\'           (1, 18) (1, 19)\n    STRING     "Rb\'abc\'"     (1, 20) (1, 27)\n    OP         \'+\'           (1, 28) (1, 29)\n    STRING     "RB\'abc\'"     (1, 30) (1, 37)\n    ')
check_tokenize('rb"abc" + rB"abc" + Rb"abc" + RB"abc"', '    STRING     \'rb"abc"\'     (1, 0) (1, 7)\n    OP         \'+\'           (1, 8) (1, 9)\n    STRING     \'rB"abc"\'     (1, 10) (1, 17)\n    OP         \'+\'           (1, 18) (1, 19)\n    STRING     \'Rb"abc"\'     (1, 20) (1, 27)\n    OP         \'+\'           (1, 28) (1, 29)\n    STRING     \'RB"abc"\'     (1, 30) (1, 37)\n    ')
check_tokenize('"a\\\nde\\\nfg"', '    STRING     \'"a\\\\\\nde\\\\\\nfg"\' (1, 0) (3, 3)\n    ')
check_tokenize('u"a\\\nde"', '    STRING     \'u"a\\\\\\nde"\'  (1, 0) (2, 3)\n    ')
check_tokenize('rb"a\\\nd"', '    STRING     \'rb"a\\\\\\nd"\'  (1, 0) (2, 2)\n    ')
check_tokenize('"""a\\\nb"""', '    STRING     \'"""a\\\\\\nb"""\' (1, 0) (2, 4)\n    ')
check_tokenize('u"""a\\\nb"""', '    STRING     \'u"""a\\\\\\nb"""\' (1, 0) (2, 4)\n    ')
check_tokenize('rb"""a\\\nb\\\nc"""', '    STRING     \'rb"""a\\\\\\nb\\\\\\nc"""\' (1, 0) (3, 4)\n    ')
check_tokenize('f"abc"', '    FSTRING_START \'f"\'          (1, 0) (1, 2)\n    FSTRING_MIDDLE \'abc\'         (1, 2) (1, 5)\n    FSTRING_END \'"\'           (1, 5) (1, 6)\n    ')
check_tokenize('fR"a{b}c"', '    FSTRING_START \'fR"\'         (1, 0) (1, 3)\n    FSTRING_MIDDLE \'a\'           (1, 3) (1, 4)\n    OP         \'{\'           (1, 4) (1, 5)\n    NAME       \'b\'           (1, 5) (1, 6)\n    OP         \'}\'           (1, 6) (1, 7)\n    FSTRING_MIDDLE \'c\'           (1, 7) (1, 8)\n    FSTRING_END \'"\'           (1, 8) (1, 9)\n    ')
check_tokenize('fR"a{{{b!r}}}c"', '    FSTRING_START \'fR"\'         (1, 0) (1, 3)\n    FSTRING_MIDDLE \'a{\'          (1, 3) (1, 5)\n    OP         \'{\'           (1, 6) (1, 7)\n    NAME       \'b\'           (1, 7) (1, 8)\n    OP         \'!\'           (1, 8) (1, 9)\n    NAME       \'r\'           (1, 9) (1, 10)\n    OP         \'}\'           (1, 10) (1, 11)\n    FSTRING_MIDDLE \'}\'           (1, 11) (1, 12)\n    FSTRING_MIDDLE \'c\'           (1, 13) (1, 14)\n    FSTRING_END \'"\'           (1, 14) (1, 15)\n    ')
check_tokenize('f"{{{1+1}}}"', '    FSTRING_START \'f"\'          (1, 0) (1, 2)\n    FSTRING_MIDDLE \'{\'           (1, 2) (1, 3)\n    OP         \'{\'           (1, 4) (1, 5)\n    NUMBER     \'1\'           (1, 5) (1, 6)\n    OP         \'+\'           (1, 6) (1, 7)\n    NUMBER     \'1\'           (1, 7) (1, 8)\n    OP         \'}\'           (1, 8) (1, 9)\n    FSTRING_MIDDLE \'}\'           (1, 9) (1, 10)\n    FSTRING_END \'"\'           (1, 11) (1, 12)\n    ')
check_tokenize('f"""{f\'\'\'{f\'{f"{1+1}"}\'}\'\'\'}"""', '    FSTRING_START \'f"""\'        (1, 0) (1, 4)\n    OP         \'{\'           (1, 4) (1, 5)\n    FSTRING_START "f\'\'\'"        (1, 5) (1, 9)\n    OP         \'{\'           (1, 9) (1, 10)\n    FSTRING_START "f\'"          (1, 10) (1, 12)\n    OP         \'{\'           (1, 12) (1, 13)\n    FSTRING_START \'f"\'          (1, 13) (1, 15)\n    OP         \'{\'           (1, 15) (1, 16)\n    NUMBER     \'1\'           (1, 16) (1, 17)\n    OP         \'+\'           (1, 17) (1, 18)\n    NUMBER     \'1\'           (1, 18) (1, 19)\n    OP         \'}\'           (1, 19) (1, 20)\n    FSTRING_END \'"\'           (1, 20) (1, 21)\n    OP         \'}\'           (1, 21) (1, 22)\n    FSTRING_END "\'"           (1, 22) (1, 23)\n    OP         \'}\'           (1, 23) (1, 24)\n    FSTRING_END "\'\'\'"         (1, 24) (1, 27)\n    OP         \'}\'           (1, 27) (1, 28)\n    FSTRING_END \'"""\'         (1, 28) (1, 31)\n    ')
check_tokenize('f"""     x\nstr(data, encoding={invalid!r})\n"""', '    FSTRING_START \'f"""\'        (1, 0) (1, 4)\n    FSTRING_MIDDLE \'     x\\nstr(data, encoding=\' (1, 4) (2, 19)\n    OP         \'{\'           (2, 19) (2, 20)\n    NAME       \'invalid\'     (2, 20) (2, 27)\n    OP         \'!\'           (2, 27) (2, 28)\n    NAME       \'r\'           (2, 28) (2, 29)\n    OP         \'}\'           (2, 29) (2, 30)\n    FSTRING_MIDDLE \')\\n\'         (2, 30) (3, 0)\n    FSTRING_END \'"""\'         (3, 0) (3, 3)\n    ')
check_tokenize('f"""123456789\nsomething{None}bad"""', '    FSTRING_START \'f"""\'        (1, 0) (1, 4)\n    FSTRING_MIDDLE \'123456789\\nsomething\' (1, 4) (2, 9)\n    OP         \'{\'           (2, 9) (2, 10)\n    NAME       \'None\'        (2, 10) (2, 14)\n    OP         \'}\'           (2, 14) (2, 15)\n    FSTRING_MIDDLE \'bad\'         (2, 15) (2, 18)\n    FSTRING_END \'"""\'         (2, 18) (2, 21)\n    ')
check_tokenize('f"""abc"""', '    FSTRING_START \'f"""\'        (1, 0) (1, 4)\n    FSTRING_MIDDLE \'abc\'         (1, 4) (1, 7)\n    FSTRING_END \'"""\'         (1, 7) (1, 10)\n    ')
check_tokenize('f"abc\\\ndef"', '    FSTRING_START \'f"\'          (1, 0) (1, 2)\n    FSTRING_MIDDLE \'abc\\\\\\ndef\'  (1, 2) (2, 3)\n    FSTRING_END \'"\'           (2, 3) (2, 4)\n    ')
check_tokenize('Rf"abc\\\ndef"', '    FSTRING_START \'Rf"\'         (1, 0) (1, 3)\n    FSTRING_MIDDLE \'abc\\\\\\ndef\'  (1, 3) (2, 3)\n    FSTRING_END \'"\'           (2, 3) (2, 4)\n    ')
check_tokenize("f'some words {a+b:.3f} more words {c+d=} final words'", '    FSTRING_START "f\'"          (1, 0) (1, 2)\n    FSTRING_MIDDLE \'some words \' (1, 2) (1, 13)\n    OP         \'{\'           (1, 13) (1, 14)\n    NAME       \'a\'           (1, 14) (1, 15)\n    OP         \'+\'           (1, 15) (1, 16)\n    NAME       \'b\'           (1, 16) (1, 17)\n    OP         \':\'           (1, 17) (1, 18)\n    FSTRING_MIDDLE \'.3f\'         (1, 18) (1, 21)\n    OP         \'}\'           (1, 21) (1, 22)\n    FSTRING_MIDDLE \' more words \' (1, 22) (1, 34)\n    OP         \'{\'           (1, 34) (1, 35)\n    NAME       \'c\'           (1, 35) (1, 36)\n    OP         \'+\'           (1, 36) (1, 37)\n    NAME       \'d\'           (1, 37) (1, 38)\n    OP         \'=\'           (1, 38) (1, 39)\n    OP         \'}\'           (1, 39) (1, 40)\n    FSTRING_MIDDLE \' final words\' (1, 40) (1, 52)\n    FSTRING_END "\'"           (1, 52) (1, 53)\n    ')
check_tokenize("f'''{\n3\n=}'''", '    FSTRING_START "f\'\'\'"        (1, 0) (1, 4)\n    OP         \'{\'           (1, 4) (1, 5)\n    NL         \'\\n\'          (1, 5) (1, 6)\n    NUMBER     \'3\'           (2, 0) (2, 1)\n    NL         \'\\n\'          (2, 1) (2, 2)\n    OP         \'=\'           (3, 0) (3, 1)\n    OP         \'}\'           (3, 1) (3, 2)\n    FSTRING_END "\'\'\'"         (3, 2) (3, 5)\n    ')
check_tokenize("f'''__{\n    x:a\n}__'''", '    FSTRING_START "f\'\'\'"        (1, 0) (1, 4)\n    FSTRING_MIDDLE \'__\'          (1, 4) (1, 6)\n    OP         \'{\'           (1, 6) (1, 7)\n    NL         \'\\n\'          (1, 7) (1, 8)\n    NAME       \'x\'           (2, 4) (2, 5)\n    OP         \':\'           (2, 5) (2, 6)\n    FSTRING_MIDDLE \'a\\n\'         (2, 6) (3, 0)\n    OP         \'}\'           (3, 0) (3, 1)\n    FSTRING_MIDDLE \'__\'          (3, 1) (3, 3)\n    FSTRING_END "\'\'\'"         (3, 3) (3, 6)\n    ')
check_tokenize("f'''__{\n    x:a\n    b\n     c\n      d\n}__'''", '    FSTRING_START "f\'\'\'"        (1, 0) (1, 4)\n    FSTRING_MIDDLE \'__\'          (1, 4) (1, 6)\n    OP         \'{\'           (1, 6) (1, 7)\n    NL         \'\\n\'          (1, 7) (1, 8)\n    NAME       \'x\'           (2, 4) (2, 5)\n    OP         \':\'           (2, 5) (2, 6)\n    FSTRING_MIDDLE \'a\\n    b\\n     c\\n      d\\n\' (2, 6) (6, 0)\n    OP         \'}\'           (6, 0) (6, 1)\n    FSTRING_MIDDLE \'__\'          (6, 1) (6, 3)\n    FSTRING_END "\'\'\'"         (6, 3) (6, 6)\n    ')
check_tokenize("f'__{\n    x:d\n}__'", '    FSTRING_START "f\'"          (1, 0) (1, 2)\n    FSTRING_MIDDLE \'__\'          (1, 2) (1, 4)\n    OP         \'{\'           (1, 4) (1, 5)\n    NL         \'\\n\'          (1, 5) (1, 6)\n    NAME       \'x\'           (2, 4) (2, 5)\n    OP         \':\'           (2, 5) (2, 6)\n    FSTRING_MIDDLE \'d\'           (2, 6) (2, 7)\n    NL         \'\\n\'          (2, 7) (2, 8)\n    OP         \'}\'           (3, 0) (3, 1)\n    FSTRING_MIDDLE \'__\'          (3, 1) (3, 3)\n    FSTRING_END "\'"           (3, 3) (3, 4)\n    ')
check_tokenize("    '''Autorzy, którzy tą jednostkę mają wpisani jako AKTUALNA -- czyli\n    aktualni pracownicy, obecni pracownicy'''\n", '    INDENT     \'    \'        (1, 0) (1, 4)\n    STRING     "\'\'\'Autorzy, którzy tą jednostkę mają wpisani jako AKTUALNA -- czyli\\n    aktualni pracownicy, obecni pracownicy\'\'\'" (1, 4) (2, 45)\n    NEWLINE    \'\\n\'          (2, 45) (2, 46)\n    DEDENT     \'\'            (3, 0) (3, 0)\n    ')
print("GenerateTokensTest::test_string: ok")
