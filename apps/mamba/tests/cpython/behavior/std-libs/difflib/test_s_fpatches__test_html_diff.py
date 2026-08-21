# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "test_s_fpatches__test_html_diff"
# subject = "cpython.test_difflib.TestSFpatches.test_html_diff"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_difflib.py::TestSFpatches::test_html_diff
"""Auto-ported test: TestSFpatches::test_html_diff (CPython 3.12 oracle)."""


import difflib
from test.support import findfile
import unittest
import doctest
import sys


patch914575_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than implicit.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_to1 = '\n   1. Beautiful is better than ugly.\n   3.   Simple is better than complex.\n   4. Complicated is better than complex.\n   5. Flat is better than nested.\n'

patch914575_nonascii_from1 = '\n   1. Beautiful is beTTer than ugly.\n   2. Explicit is better than ımplıcıt.\n   3. Simple is better than complex.\n   4. Complex is better than complicated.\n'

patch914575_nonascii_to1 = '\n   1. Beautiful is better than ügly.\n   3.   Sımple is better than complex.\n   4. Complicated is better than cömplex.\n   5. Flat is better than nested.\n'

patch914575_from2 = '\n\t\tLine 1: preceded by from:[tt] to:[ssss]\n  \t\tLine 2: preceded by from:[sstt] to:[sssst]\n  \t \tLine 3: preceded by from:[sstst] to:[ssssss]\nLine 4:  \thas from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\t\n'

patch914575_to2 = '\n    Line 1: preceded by from:[tt] to:[ssss]\n    \tLine 2: preceded by from:[sstt] to:[sssst]\n      Line 3: preceded by from:[sstst] to:[ssssss]\nLine 4:   has from:[sst] to:[sss] after :\nLine 5: has from:[t] to:[ss] at end\n'

patch914575_from3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2\nline 3\nline 4   changed\nline 5   changed\nline 6   changed\nline 7\nline 8  subtracted\nline 9\n1234567890123456789012345689012345\nshort line\njust fits in!!\njust fits in two lines yup!!\nthe end'

patch914575_to3 = 'line 0\n1234567890123456789012345689012345\nline 1\nline 2    added\nline 3\nline 4   chanGEd\nline 5a  chanGed\nline 6a  changEd\nline 7\nline 8\nline 9\n1234567890\nanother long line that needs to be wrapped\njust fitS in!!\njust fits in two lineS yup!!\nthe end'

def setUpModule():
    difflib.HtmlDiff._default_prefix = 0

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(difflib))
    return tests


# --- test body ---
f1a = (patch914575_from1 + '123\n' * 10) * 3
t1a = (patch914575_to1 + '123\n' * 10) * 3
f1b = '456\n' * 10 + f1a
t1b = '456\n' * 10 + t1a
f1a = f1a.splitlines()
t1a = t1a.splitlines()
f1b = f1b.splitlines()
t1b = t1b.splitlines()
f2 = patch914575_from2.splitlines()
t2 = patch914575_to2.splitlines()
f3 = patch914575_from3
t3 = patch914575_to3
i = difflib.HtmlDiff()
j = difflib.HtmlDiff(tabsize=2)
k = difflib.HtmlDiff(wrapcolumn=14)
full = i.make_file(f1a, t1a, 'from', 'to', context=False, numlines=5)
tables = '\n'.join(['<h2>Context (first diff within numlines=5(default))</h2>', i.make_table(f1a, t1a, 'from', 'to', context=True), '<h2>Context (first diff after numlines=5(default))</h2>', i.make_table(f1b, t1b, 'from', 'to', context=True), '<h2>Context (numlines=6)</h2>', i.make_table(f1a, t1a, 'from', 'to', context=True, numlines=6), '<h2>Context (numlines=0)</h2>', i.make_table(f1a, t1a, 'from', 'to', context=True, numlines=0), '<h2>Same Context</h2>', i.make_table(f1a, f1a, 'from', 'to', context=True), '<h2>Same Full</h2>', i.make_table(f1a, f1a, 'from', 'to', context=False), '<h2>Empty Context</h2>', i.make_table([], [], 'from', 'to', context=True), '<h2>Empty Full</h2>', i.make_table([], [], 'from', 'to', context=False), '<h2>tabsize=2</h2>', j.make_table(f2, t2), '<h2>tabsize=default</h2>', i.make_table(f2, t2), '<h2>Context (wrapcolumn=14,numlines=0)</h2>', k.make_table(f3.splitlines(), t3.splitlines(), context=True, numlines=0), '<h2>wrapcolumn=14,splitlines()</h2>', k.make_table(f3.splitlines(), t3.splitlines()), '<h2>wrapcolumn=14,splitlines(True)</h2>', k.make_table(f3.splitlines(True), t3.splitlines(True))])
actual = full.replace('</body>', '\n%s\n</body>' % tables)
with open(findfile('test_difflib_expect.html'), encoding='utf-8') as fp:

    assert actual == fp.read()
print("TestSFpatches::test_html_diff: ok")
