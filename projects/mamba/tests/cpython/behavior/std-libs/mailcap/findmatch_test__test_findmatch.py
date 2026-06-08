# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailcap"
# dimension = "behavior"
# case = "findmatch_test__test_findmatch"
# subject = "cpython.test_mailcap.FindmatchTest.test_findmatch"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mailcap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_mailcap.py::FindmatchTest::test_findmatch
"""Auto-ported test: FindmatchTest::test_findmatch (CPython 3.12 oracle)."""


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
def _run_cases(cases):
    for c in cases:

        assert mailcap.findmatch(*c[0], **c[1]) == c[2]
c = MAILCAPDICT
fname = 'foo.txt'
plist = ['access-type=default', 'name=john', 'site=python.org', 'directory=/tmp', 'mode=foo', 'server=bar']
audio_basic_entry = {'edit': 'audiocompose %s', 'compose': 'audiocompose %s', 'description': '"An audio fragment"', 'view': 'showaudio %s', 'lineno': 6}
audio_entry = {'view': '/usr/local/bin/showaudio %t', 'lineno': 7}
video_entry = {'view': 'animate %s', 'lineno': 12}
message_entry = {'composetyped': 'extcompose %s', 'description': '"A reference to data stored in an external location"', 'needsterminal': '', 'view': 'showexternal %s %{access-type} %{name} %{site}     %{directory} %{mode} %{server}', 'lineno': 10}
cases = [([{}, 'video/mpeg'], {}, (None, None)), ([c, 'foo/bar'], {}, (None, None)), ([c, 'video/mpeg'], {}, ('animate /dev/null', video_entry)), ([c, 'audio/basic', 'edit'], {}, ('audiocompose /dev/null', audio_basic_entry)), ([c, 'audio/basic', 'compose'], {}, ('audiocompose /dev/null', audio_basic_entry)), ([c, 'audio/basic', 'description'], {}, ('"An audio fragment"', audio_basic_entry)), ([c, 'audio/basic', 'foobar'], {}, (None, None)), ([c, 'video/*'], {'filename': fname}, ('animate %s' % fname, video_entry)), ([c, 'audio/basic', 'compose'], {'filename': fname}, ('audiocompose %s' % fname, audio_basic_entry)), ([c, 'audio/basic'], {'key': 'description', 'filename': fname}, ('"An audio fragment"', audio_basic_entry)), ([c, 'audio/wav'], {'filename': fname}, ('/usr/local/bin/showaudio audio/wav', audio_entry)), ([c, 'message/external-body'], {'plist': plist}, ('showexternal /dev/null default john python.org     /tmp foo bar', message_entry))]
_run_cases(cases)
print("FindmatchTest::test_findmatch: ok")
