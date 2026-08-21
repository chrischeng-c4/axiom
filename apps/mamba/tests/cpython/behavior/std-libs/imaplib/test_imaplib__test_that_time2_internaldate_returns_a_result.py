# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "imaplib"
# dimension = "behavior"
# case = "test_imaplib__test_that_time2_internaldate_returns_a_result"
# subject = "cpython.test_imaplib.TestImaplib.test_that_Time2Internaldate_returns_a_result"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_imaplib.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_imaplib.py::TestImaplib::test_that_Time2Internaldate_returns_a_result
"""Auto-ported test: TestImaplib::test_that_Time2Internaldate_returns_a_result (CPython 3.12 oracle)."""


from test import support
from test.support import socket_helper
from contextlib import contextmanager
import imaplib
import os.path
import socketserver
import time
import calendar
import threading
import re
import socket
from test.support import verbose, run_with_tz, run_with_locale, cpython_only, requires_resource
from test.support import hashlib_helper
from test.support import threading_helper
import unittest
from unittest import mock
from datetime import datetime, timezone, timedelta


try:
    import ssl
except ImportError:
    ssl = None

support.requires_working_socket(module=True)

CERTFILE = os.path.join(os.path.dirname(__file__) or os.curdir, 'certdata', 'keycert3.pem')

CAFILE = os.path.join(os.path.dirname(__file__) or os.curdir, 'certdata', 'pycacert.pem')

if ssl:

    class SecureTCPServer(socketserver.TCPServer):

        def get_request(self):
            newsocket, fromaddr = self.socket.accept()
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            context.load_cert_chain(CERTFILE)
            connstream = context.wrap_socket(newsocket, server_side=True)
            return (connstream, fromaddr)
    IMAP4_SSL = imaplib.IMAP4_SSL
else:

    class SecureTCPServer:
        pass
    IMAP4_SSL = None

class SimpleIMAPHandler(socketserver.StreamRequestHandler):
    timeout = support.LOOPBACK_TIMEOUT
    continuation = None
    capabilities = ''

    def setup(self):
        super().setup()
        self.server.is_selected = False
        self.server.logged = None

    def _send(self, message):
        if verbose:
            print('SENT: %r' % message.strip())
        self.wfile.write(message)

    def _send_line(self, message):
        self._send(message + b'\r\n')

    def _send_textline(self, message):
        self._send_line(message.encode('ASCII'))

    def _send_tagged(self, tag, code, message):
        self._send_textline(' '.join((tag, code, message)))

    def handle(self):
        self._send_textline('* OK IMAP4rev1')
        while 1:
            line = b''
            while 1:
                try:
                    part = self.rfile.read(1)
                    if part == b'':
                        return
                    line += part
                except OSError:
                    return
                if line.endswith(b'\r\n'):
                    break
            if verbose:
                print('GOT: %r' % line.strip())
            if self.continuation:
                try:
                    self.continuation.send(line)
                except StopIteration:
                    self.continuation = None
                continue
            splitline = line.decode('ASCII').split()
            tag = splitline[0]
            cmd = splitline[1]
            args = splitline[2:]
            if hasattr(self, 'cmd_' + cmd):
                continuation = getattr(self, 'cmd_' + cmd)(tag, args)
                if continuation:
                    self.continuation = continuation
                    next(continuation)
            else:
                self._send_tagged(tag, 'BAD', cmd + ' unknown')

    def cmd_CAPABILITY(self, tag, args):
        caps = 'IMAP4rev1 ' + self.capabilities if self.capabilities else 'IMAP4rev1'
        self._send_textline('* CAPABILITY ' + caps)
        self._send_tagged(tag, 'OK', 'CAPABILITY completed')

    def cmd_LOGOUT(self, tag, args):
        self.server.logged = None
        self._send_textline('* BYE IMAP4ref1 Server logging out')
        self._send_tagged(tag, 'OK', 'LOGOUT completed')

    def cmd_LOGIN(self, tag, args):
        self.server.logged = args[0]
        self._send_tagged(tag, 'OK', 'LOGIN completed')

    def cmd_SELECT(self, tag, args):
        self.server.is_selected = True
        self._send_line(b'* 2 EXISTS')
        self._send_tagged(tag, 'OK', '[READ-WRITE] SELECT completed.')

    def cmd_UNSELECT(self, tag, args):
        if self.server.is_selected:
            self.server.is_selected = False
            self._send_tagged(tag, 'OK', 'Returned to authenticated state. (Success)')
        else:
            self._send_tagged(tag, 'BAD', 'No mailbox selected')


# --- test body ---
def timevalues():
    return [2000000000, 2000000000.0, time.localtime(2000000000), (2033, 5, 18, 5, 33, 20, -1, -1, -1), (2033, 5, 18, 5, 33, 20, -1, -1, 1), datetime.fromtimestamp(2000000000, timezone(timedelta(0, 2 * 60 * 60))), '"18-May-2033 05:33:20 +0200"']
for t in timevalues():
    imaplib.Time2Internaldate(t)
print("TestImaplib::test_that_Time2Internaldate_returns_a_result: ok")
