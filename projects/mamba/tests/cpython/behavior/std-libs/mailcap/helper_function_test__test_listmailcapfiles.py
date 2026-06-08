# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailcap"
# dimension = "behavior"
# case = "helper_function_test__test_listmailcapfiles"
# subject = "cpython.test_mailcap.HelperFunctionTest.test_listmailcapfiles"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mailcap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_mailcap.py::HelperFunctionTest::test_listmailcapfiles
"""Auto-ported test: HelperFunctionTest::test_listmailcapfiles (CPython 3.12 oracle)."""


import copy
import os
import sys
import test.support
import unittest
from test.support import os_helper
from test.support import warnings_helper


mailcap = warnings_helper.import_deprecated('mailcap')

MAILCAPFILE = test.support.findfile('mailcap.txt')

MAILCAPDICT = {'application/x-movie': [{'compose': 'moviemaker %s', 'x11-bitmap': '"/usr/lib/Zmail/bitmaps/movie.xbm"', 'description': '"Movie"', 'view': 'movieplayer %s', 'lineno': 4}], 'application/*': [{'copiousoutput': '', 'view': 'echo "This is \\"%t\\" but        is 50 \\% Greek to me" \\; cat %s', 'lineno': 5}], 'audio/basic': [{'edit': 'audiocompose %s', 'compose': 'audiocompose %s', 'description': '"An audio fragment"', 'view': 'showaudio %s', 'lineno': 6}], 'video/mpeg': [{'view': 'mpeg_play %s', 'lineno': 13}], 'application/postscript': [{'needsterminal': '', 'view': 'ps-to-terminal %s', 'lineno': 1}, {'compose': 'idraw %s', 'view': 'ps-to-terminal %s', 'lineno': 2}], 'application/x-dvi': [{'view': 'xdvi %s', 'lineno': 3}], 'message/external-body': [{'composetyped': 'extcompose %s', 'description': '"A reference to data stored in an external location"', 'needsterminal': '', 'view': 'showexternal %s %{access-type} %{name} %{site}     %{directory} %{mode} %{server}', 'lineno': 10}], 'text/richtext': [{'test': 'test "`echo     %{charset} | tr \'[A-Z]\' \'[a-z]\'`"  = iso-8859-8', 'copiousoutput': '', 'view': 'shownonascii iso-8859-8 -e richtext -p %s', 'lineno': 11}], 'image/x-xwindowdump': [{'view': 'display %s', 'lineno': 9}], 'audio/*': [{'view': '/usr/local/bin/showaudio %t', 'lineno': 7}], 'video/*': [{'view': 'animate %s', 'lineno': 12}], 'application/frame': [{'print': '"cat %s | lp"', 'view': 'showframe %s', 'lineno': 0}], 'image/rgb': [{'view': 'display %s', 'lineno': 8}]}

MAILCAPDICT_DEPRECATED = copy.deepcopy(MAILCAPDICT)

for entry_list in MAILCAPDICT_DEPRECATED.values():
    for entry in entry_list:
        entry.pop('lineno')


# --- test body ---
mcfiles = mailcap.listmailcapfiles()

assert isinstance(mcfiles, list)
for m in mcfiles:

    assert isinstance(m, str)
with os_helper.EnvironmentVarGuard() as env:
    if 'MAILCAPS' in env:
        env_mailcaps = env['MAILCAPS'].split(os.pathsep)
    else:
        env_mailcaps = ['/testdir1/.mailcap', '/testdir2/mailcap']
        env['MAILCAPS'] = os.pathsep.join(env_mailcaps)
        mcfiles = mailcap.listmailcapfiles()

assert env_mailcaps == mcfiles
print("HelperFunctionTest::test_listmailcapfiles: ok")
