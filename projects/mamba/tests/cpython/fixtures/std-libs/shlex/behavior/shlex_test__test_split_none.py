# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "behavior"
# case = "shlex_test__test_split_none"
# subject = "cpython.test_shlex.ShlexTest.testSplitNone"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_shlex.py::ShlexTest::testSplitNone
"""Auto-ported test: ShlexTest::testSplitNone (CPython 3.12 oracle)."""


import io
import itertools
import shlex
import string
import unittest


data = 'x|x|\nfoo bar|foo|bar|\n foo bar|foo|bar|\n foo bar |foo|bar|\nfoo   bar    bla     fasel|foo|bar|bla|fasel|\nx y  z              xxxx|x|y|z|xxxx|\n\\x bar|\\|x|bar|\n\\ x bar|\\|x|bar|\n\\ bar|\\|bar|\nfoo \\x bar|foo|\\|x|bar|\nfoo \\ x bar|foo|\\|x|bar|\nfoo \\ bar|foo|\\|bar|\nfoo "bar" bla|foo|"bar"|bla|\n"foo" "bar" "bla"|"foo"|"bar"|"bla"|\n"foo" bar "bla"|"foo"|bar|"bla"|\n"foo" bar bla|"foo"|bar|bla|\nfoo \'bar\' bla|foo|\'bar\'|bla|\n\'foo\' \'bar\' \'bla\'|\'foo\'|\'bar\'|\'bla\'|\n\'foo\' bar \'bla\'|\'foo\'|bar|\'bla\'|\n\'foo\' bar bla|\'foo\'|bar|bla|\nblurb foo"bar"bar"fasel" baz|blurb|foo"bar"bar"fasel"|baz|\nblurb foo\'bar\'bar\'fasel\' baz|blurb|foo\'bar\'bar\'fasel\'|baz|\n""|""|\n\'\'|\'\'|\nfoo "" bar|foo|""|bar|\nfoo \'\' bar|foo|\'\'|bar|\nfoo "" "" "" bar|foo|""|""|""|bar|\nfoo \'\' \'\' \'\' bar|foo|\'\'|\'\'|\'\'|bar|\n\\""|\\|""|\n"\\"|"\\"|\n"foo\\ bar"|"foo\\ bar"|\n"foo\\\\ bar"|"foo\\\\ bar"|\n"foo\\\\ bar\\"|"foo\\\\ bar\\"|\n"foo\\\\" bar\\""|"foo\\\\"|bar|\\|""|\n"foo\\\\ bar\\" dfadf"|"foo\\\\ bar\\"|dfadf"|\n"foo\\\\\\ bar\\" dfadf"|"foo\\\\\\ bar\\"|dfadf"|\n"foo\\\\\\x bar\\" dfadf"|"foo\\\\\\x bar\\"|dfadf"|\n"foo\\x bar\\" dfadf"|"foo\\x bar\\"|dfadf"|\n\\\'\'|\\|\'\'|\n\'foo\\ bar\'|\'foo\\ bar\'|\n\'foo\\\\ bar\'|\'foo\\\\ bar\'|\n"foo\\\\\\x bar\\" df\'a\\ \'df\'|"foo\\\\\\x bar\\"|df\'a|\\|\'df\'|\n\\"foo"|\\|"foo"|\n\\"foo"\\x|\\|"foo"|\\|x|\n"foo\\x"|"foo\\x"|\n"foo\\ "|"foo\\ "|\nfoo\\ xx|foo|\\|xx|\nfoo\\ x\\x|foo|\\|x|\\|x|\nfoo\\ x\\x\\""|foo|\\|x|\\|x|\\|""|\n"foo\\ x\\x"|"foo\\ x\\x"|\n"foo\\ x\\x\\\\"|"foo\\ x\\x\\\\"|\n"foo\\ x\\x\\\\""foobar"|"foo\\ x\\x\\\\"|"foobar"|\n"foo\\ x\\x\\\\"\\\'\'"foobar"|"foo\\ x\\x\\\\"|\\|\'\'|"foobar"|\n"foo\\ x\\x\\\\"\\\'"fo\'obar"|"foo\\ x\\x\\\\"|\\|\'"fo\'|obar"|\n"foo\\ x\\x\\\\"\\\'"fo\'obar" \'don\'\\\'\'t\'|"foo\\ x\\x\\\\"|\\|\'"fo\'|obar"|\'don\'|\\|\'\'|t\'|\n\'foo\\ bar\'|\'foo\\ bar\'|\n\'foo\\\\ bar\'|\'foo\\\\ bar\'|\nfoo\\ bar|foo|\\|bar|\nfoo#bar\\nbaz|foobaz|\n:-) ;-)|:|-|)|;|-|)|\náéíóú|á|é|í|ó|ú|\n'

posix_data = 'x|x|\nfoo bar|foo|bar|\n foo bar|foo|bar|\n foo bar |foo|bar|\nfoo   bar    bla     fasel|foo|bar|bla|fasel|\nx y  z              xxxx|x|y|z|xxxx|\n\\x bar|x|bar|\n\\ x bar| x|bar|\n\\ bar| bar|\nfoo \\x bar|foo|x|bar|\nfoo \\ x bar|foo| x|bar|\nfoo \\ bar|foo| bar|\nfoo "bar" bla|foo|bar|bla|\n"foo" "bar" "bla"|foo|bar|bla|\n"foo" bar "bla"|foo|bar|bla|\n"foo" bar bla|foo|bar|bla|\nfoo \'bar\' bla|foo|bar|bla|\n\'foo\' \'bar\' \'bla\'|foo|bar|bla|\n\'foo\' bar \'bla\'|foo|bar|bla|\n\'foo\' bar bla|foo|bar|bla|\nblurb foo"bar"bar"fasel" baz|blurb|foobarbarfasel|baz|\nblurb foo\'bar\'bar\'fasel\' baz|blurb|foobarbarfasel|baz|\n""||\n\'\'||\nfoo "" bar|foo||bar|\nfoo \'\' bar|foo||bar|\nfoo "" "" "" bar|foo||||bar|\nfoo \'\' \'\' \'\' bar|foo||||bar|\n\\"|"|\n"\\""|"|\n"foo\\ bar"|foo\\ bar|\n"foo\\\\ bar"|foo\\ bar|\n"foo\\\\ bar\\""|foo\\ bar"|\n"foo\\\\" bar\\"|foo\\|bar"|\n"foo\\\\ bar\\" dfadf"|foo\\ bar" dfadf|\n"foo\\\\\\ bar\\" dfadf"|foo\\\\ bar" dfadf|\n"foo\\\\\\x bar\\" dfadf"|foo\\\\x bar" dfadf|\n"foo\\x bar\\" dfadf"|foo\\x bar" dfadf|\n\\\'|\'|\n\'foo\\ bar\'|foo\\ bar|\n\'foo\\\\ bar\'|foo\\\\ bar|\n"foo\\\\\\x bar\\" df\'a\\ \'df"|foo\\\\x bar" df\'a\\ \'df|\n\\"foo|"foo|\n\\"foo\\x|"foox|\n"foo\\x"|foo\\x|\n"foo\\ "|foo\\ |\nfoo\\ xx|foo xx|\nfoo\\ x\\x|foo xx|\nfoo\\ x\\x\\"|foo xx"|\n"foo\\ x\\x"|foo\\ x\\x|\n"foo\\ x\\x\\\\"|foo\\ x\\x\\|\n"foo\\ x\\x\\\\""foobar"|foo\\ x\\x\\foobar|\n"foo\\ x\\x\\\\"\\\'"foobar"|foo\\ x\\x\\\'foobar|\n"foo\\ x\\x\\\\"\\\'"fo\'obar"|foo\\ x\\x\\\'fo\'obar|\n"foo\\ x\\x\\\\"\\\'"fo\'obar" \'don\'\\\'\'t\'|foo\\ x\\x\\\'fo\'obar|don\'t|\n"foo\\ x\\x\\\\"\\\'"fo\'obar" \'don\'\\\'\'t\' \\\\|foo\\ x\\x\\\'fo\'obar|don\'t|\\|\n\'foo\\ bar\'|foo\\ bar|\n\'foo\\\\ bar\'|foo\\\\ bar|\nfoo\\ bar|foo bar|\nfoo#bar\\nbaz|foo|baz|\n:-) ;-)|:-)|;-)|\náéíóú|áéíóú|\n'

if not getattr(shlex, 'split', None):
    for methname in dir(ShlexTest):
        if methname.startswith('test') and methname != 'testCompat':
            delattr(ShlexTest, methname)


# --- test body ---
def oldSplit(s):
    ret = []
    lex = shlex.shlex(io.StringIO(s))
    tok = lex.get_token()
    while tok:
        ret.append(tok)
        tok = lex.get_token()
    return ret

def splitTest(data, comments):
    for i in range(len(data)):
        l = shlex.split(data[i][0], comments=comments)

        assert l == data[i][1:]
self_data = [x.split('|')[:-1] for x in data.splitlines()]
self_posix_data = [x.split('|')[:-1] for x in posix_data.splitlines()]
for item in self_data:
    item[0] = item[0].replace('\\n', '\n')
for item in self_posix_data:
    item[0] = item[0].replace('\\n', '\n')
try:
    shlex.split(None)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("ShlexTest::testSplitNone: ok")
