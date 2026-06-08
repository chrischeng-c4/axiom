# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "behavior"
# case = "test_partial_file__test_seek_and_tell"
# subject = "cpython.test_mailbox.TestPartialFile.test_seek_and_tell"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mailbox.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_mailbox.py::TestPartialFile::test_seek_and_tell
"""Auto-ported test: TestPartialFile::test_seek_and_tell (CPython 3.12 oracle)."""


import os
import sys
import time
import stat
import socket
import email
import email.message
import re
import io
import tempfile
from test import support
from test.support import import_helper
from test.support import os_helper
from test.support import socket_helper
import unittest
import textwrap
import mailbox
import glob


if not socket_helper.has_gethostname:
    raise unittest.SkipTest('test requires gethostname()')

class FakeFileLikeObject:

    def __init__(self):
        self.closed = False

    def close(self):
        self.closed = True

class FakeMailBox(mailbox.Mailbox):

    def __init__(self):
        mailbox.Mailbox.__init__(self, '', lambda file: None)
        self.files = [FakeFileLikeObject() for i in range(10)]

    def get_file(self, key):
        return self.files[key]

FROM_ = 'From some.body@dummy.domain  Sat Jul 24 13:43:35 2004\n'

DUMMY_MESSAGE = 'From: some.body@dummy.domain\nTo: me@my.domain\nSubject: Simple Test\n\nThis is a dummy message.\n'

_sample_message = 'Return-Path: <gkj@gregorykjohnson.com>\nX-Original-To: gkj+person@localhost\nDelivered-To: gkj+person@localhost\nReceived: from localhost (localhost [127.0.0.1])\n        by andy.gregorykjohnson.com (Postfix) with ESMTP id 356ED9DD17\n        for <gkj+person@localhost>; Wed, 13 Jul 2005 17:23:16 -0400 (EDT)\nDelivered-To: gkj@sundance.gregorykjohnson.com\nReceived: from localhost [127.0.0.1]\n        by localhost with POP3 (fetchmail-6.2.5)\n        for gkj+person@localhost (single-drop); Wed, 13 Jul 2005 17:23:16 -0400 (EDT)\nReceived: from andy.gregorykjohnson.com (andy.gregorykjohnson.com [64.32.235.228])\n        by sundance.gregorykjohnson.com (Postfix) with ESMTP id 5B056316746\n        for <gkj@gregorykjohnson.com>; Wed, 13 Jul 2005 17:23:11 -0400 (EDT)\nReceived: by andy.gregorykjohnson.com (Postfix, from userid 1000)\n        id 490CD9DD17; Wed, 13 Jul 2005 17:23:11 -0400 (EDT)\nDate: Wed, 13 Jul 2005 17:23:11 -0400\nFrom: "Gregory K. Johnson" <gkj@gregorykjohnson.com>\nTo: gkj@gregorykjohnson.com\nSubject: Sample message\nMessage-ID: <20050713212311.GC4701@andy.gregorykjohnson.com>\nMime-Version: 1.0\nContent-Type: multipart/mixed; boundary="NMuMz9nt05w80d4+"\nContent-Disposition: inline\nUser-Agent: Mutt/1.5.9i\n\n\n--NMuMz9nt05w80d4+\nContent-Type: text/plain; charset=us-ascii\nContent-Disposition: inline\n\nThis is a sample message.\n\n--\nGregory K. Johnson\n\n--NMuMz9nt05w80d4+\nContent-Type: application/octet-stream\nContent-Disposition: attachment; filename="text.gz"\nContent-Transfer-Encoding: base64\n\nH4sICM2D1UIAA3RleHQAC8nILFYAokSFktSKEoW0zJxUPa7wzJIMhZLyfIWczLzUYj0uAHTs\n3FYlAAAA\n\n--NMuMz9nt05w80d4+--\n'

_bytes_sample_message = _sample_message.encode('ascii')

_sample_headers = [('Return-Path', '<gkj@gregorykjohnson.com>'), ('X-Original-To', 'gkj+person@localhost'), ('Delivered-To', 'gkj+person@localhost'), ('Received', 'from localhost (localhost [127.0.0.1])\n        by andy.gregorykjohnson.com (Postfix) with ESMTP id 356ED9DD17\n        for <gkj+person@localhost>; Wed, 13 Jul 2005 17:23:16 -0400 (EDT)'), ('Delivered-To', 'gkj@sundance.gregorykjohnson.com'), ('Received', 'from localhost [127.0.0.1]\n        by localhost with POP3 (fetchmail-6.2.5)\n        for gkj+person@localhost (single-drop); Wed, 13 Jul 2005 17:23:16 -0400 (EDT)'), ('Received', 'from andy.gregorykjohnson.com (andy.gregorykjohnson.com [64.32.235.228])\n        by sundance.gregorykjohnson.com (Postfix) with ESMTP id 5B056316746\n        for <gkj@gregorykjohnson.com>; Wed, 13 Jul 2005 17:23:11 -0400 (EDT)'), ('Received', 'by andy.gregorykjohnson.com (Postfix, from userid 1000)\n        id 490CD9DD17; Wed, 13 Jul 2005 17:23:11 -0400 (EDT)'), ('Date', 'Wed, 13 Jul 2005 17:23:11 -0400'), ('From', '"Gregory K. Johnson" <gkj@gregorykjohnson.com>'), ('To', 'gkj@gregorykjohnson.com'), ('Subject', 'Sample message'), ('Mime-Version', '1.0'), ('Content-Type', 'multipart/mixed; boundary="NMuMz9nt05w80d4+"'), ('Content-Disposition', 'inline'), ('User-Agent', 'Mutt/1.5.9i')]

_sample_payloads = ('This is a sample message.\n\n--\nGregory K. Johnson\n', 'H4sICM2D1UIAA3RleHQAC8nILFYAokSFktSKEoW0zJxUPa7wzJIMhZLyfIWczLzUYj0uAHTs\n3FYlAAAA\n')

def tearDownModule():
    support.reap_children()


# --- test body ---
all_mailbox_types = (mailbox.Message, mailbox.MaildirMessage, mailbox.mboxMessage, mailbox.MHMessage, mailbox.BabylMessage, mailbox.MMDFMessage)

def _check_sample(msg):

    assert isinstance(msg, email.message.Message)

    assert isinstance(msg, mailbox.Message)
    for key, value in _sample_headers:

        assert value in msg.get_all(key)

    assert msg.is_multipart()

    assert len(msg.get_payload()) == len(_sample_payloads)
    for i, payload in enumerate(_sample_payloads):
        part = msg.get_payload(i)

        assert isinstance(part, email.message.Message)

        assert not isinstance(part, mailbox.Message)

        assert part.get_payload() == payload

def _delete_recursively(target):
    if os.path.isdir(target):
        os_helper.rmtree(target)
    elif os.path.exists(target):
        os_helper.unlink(target)

def _test_close(proxy):

    assert not proxy.closed
    proxy.close()

    assert proxy.closed
    proxy.close()

    assert proxy.closed

def _test_iteration(proxy):
    linesep = os.linesep.encode()
    proxy.seek(0)
    iterator = iter(proxy)

    assert next(iterator) == b'foo' + linesep

    assert next(iterator) == b'bar' + linesep

    assert next(iterator) == b'fred' + linesep

    assert next(iterator) == b'bob'

    try:
        next(iterator)
        raise AssertionError('expected StopIteration')
    except StopIteration:
        pass

def _test_read(proxy):
    proxy.seek(0)

    assert proxy.read() == b'bar'
    proxy.seek(1)

    assert proxy.read() == b'ar'
    proxy.seek(0)

    assert proxy.read(2) == b'ba'
    proxy.seek(1)

    assert proxy.read(-1) == b'ar'
    proxy.seek(2)

    assert proxy.read(1000) == b'r'

def _test_readline(proxy):
    linesep = os.linesep.encode()
    proxy.seek(0)

    assert proxy.readline() == b'foo' + linesep

    assert proxy.readline() == b'bar' + linesep

    assert proxy.readline() == b'fred' + linesep

    assert proxy.readline() == b'bob'
    proxy.seek(2)

    assert proxy.readline() == b'o' + linesep
    proxy.seek(6 + 2 * len(os.linesep))

    assert proxy.readline() == b'fred' + linesep
    proxy.seek(6 + 2 * len(os.linesep))

    assert proxy.readline(2) == b'fr'

    assert proxy.readline(-10) == b'ed' + linesep

def _test_readlines(proxy):
    linesep = os.linesep.encode()
    proxy.seek(0)

    assert proxy.readlines() == [b'foo' + linesep, b'bar' + linesep, b'fred' + linesep, b'bob']
    proxy.seek(0)

    assert proxy.readlines(2) == [b'foo' + linesep]
    proxy.seek(3 + len(linesep))

    assert proxy.readlines(4 + len(linesep)) == [b'bar' + linesep, b'fred' + linesep]
    proxy.seek(3)

    assert proxy.readlines(1000) == [linesep, b'bar' + linesep, b'fred' + linesep, b'bob']

def _test_seek_and_tell(proxy):
    linesep = os.linesep.encode()
    proxy.seek(3)

    assert proxy.tell() == 3

    assert proxy.read(len(linesep)) == linesep
    proxy.seek(2, 1)

    assert proxy.read(1 + len(linesep)) == b'r' + linesep
    proxy.seek(-3 - len(linesep), 2)

    assert proxy.read(3) == b'bar'
    proxy.seek(2, 0)

    assert proxy.read() == b'o' + linesep + b'bar' + linesep
    proxy.seek(100)

    assert not proxy.read()
self__path = os_helper.TESTFN
self__file = open(self__path, 'wb+')
self__file.write(bytes('(((foo%sbar%s$$$' % (os.linesep, os.linesep), 'ascii'))
_test_seek_and_tell(mailbox._PartialFile(self__file, 3, 9 + 2 * len(os.linesep)))
print("TestPartialFile::test_seek_and_tell: ok")
