# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailbox"
# dimension = "behavior"
# case = "test_partial_file__test_initialize"
# subject = "cpython.test_mailbox.TestPartialFile.test_initialize"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_mailbox.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_mailbox.py::TestPartialFile::test_initialize
"""Auto-ported test: TestPartialFile::test_initialize (CPython 3.12 oracle)."""


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
self__path = os_helper.TESTFN
self__file = open(self__path, 'wb+')
self__file.write(bytes('foo' + os.linesep + 'bar', 'ascii'))
pos = self__file.tell()
proxy = mailbox._PartialFile(self__file, 2, 5)

assert proxy.tell() == 0

assert self__file.tell() == pos
print("TestPartialFile::test_initialize: ok")
