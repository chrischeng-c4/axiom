# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtplib"
# dimension = "behavior"
# case = "default_arguments_tests__test_send_message"
# subject = "cpython.test_smtplib.DefaultArgumentsTests.testSendMessage"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_smtplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_smtplib.py::DefaultArgumentsTests::testSendMessage
"""Auto-ported test: DefaultArgumentsTests::testSendMessage (CPython 3.12 oracle)."""


import base64
import email.mime.text
from email.message import EmailMessage
from email.base64mime import body_encode as encode_base64
import email.utils
import hashlib
import hmac
import socket
import smtplib
import io
import re
import sys
import time
import select
import errno
import textwrap
import threading
import unittest
from test import support, mock_socket
from test.support import hashlib_helper
from test.support import socket_helper
from test.support import threading_helper
from test.support import asyncore
from test.support import smtpd
from unittest.mock import Mock


support.requires_working_socket(module=True)

HOST = socket_helper.HOST

if sys.platform == 'darwin':

    def handle_expt(self):
        pass
    smtpd.SMTPChannel.handle_expt = handle_expt

def server(evt, buf, serv):
    serv.listen()
    evt.set()
    try:
        conn, addr = serv.accept()
    except TimeoutError:
        pass
    else:
        n = 500
        while buf and n > 0:
            r, w, e = select.select([], [conn], [])
            if w:
                sent = conn.send(buf)
                buf = buf[sent:]
            n -= 1
        conn.close()
    finally:
        serv.close()
        evt.set()

def debugging_server(serv, serv_evt, client_evt):
    serv_evt.set()
    try:
        if hasattr(select, 'poll'):
            poll_fun = asyncore.poll2
        else:
            poll_fun = asyncore.poll
        n = 1000
        while asyncore.socket_map and n > 0:
            poll_fun(0.01, asyncore.socket_map)
            if client_evt.is_set():
                serv.close()
                break
            n -= 1
    except TimeoutError:
        pass
    finally:
        if not client_evt.is_set():
            time.sleep(0.5)
            serv.close()
        asyncore.close_all()
        serv_evt.set()

MSG_BEGIN = '---------- MESSAGE FOLLOWS ----------\n'

MSG_END = '------------ END MESSAGE ------------\n'

sim_users = {'Mr.A@somewhere.com': 'John A', 'Ms.B@xn--fo-fka.com': 'Sally B', 'Mrs.C@somewhereesle.com': 'Ruth C'}

sim_auth = ('Mr.A@somewhere.com', 'somepassword')

sim_cram_md5_challenge = 'PENCeUxFREJoU0NnbmhNWitOMjNGNndAZWx3b29kLmlubm9zb2Z0LmNvbT4='

sim_lists = {'list-1': ['Mr.A@somewhere.com', 'Mrs.C@somewhereesle.com'], 'list-2': ['Ms.B@xn--fo-fka.com']}

class ResponseException(Exception):
    pass

class SimSMTPChannel(smtpd.SMTPChannel):
    quit_response = None
    mail_response = None
    rcpt_response = None
    data_response = None
    rcpt_count = 0
    rset_count = 0
    disconnect = 0
    AUTH = 99
    authenticated_user = None

    def __init__(self, extra_features, *args, **kw):
        self._extrafeatures = ''.join(['250-{0}\r\n'.format(x) for x in extra_features])
        super(SimSMTPChannel, self).__init__(*args, **kw)

    def found_terminator(self):
        if self.smtp_state == self.AUTH:
            line = self._emptystring.join(self.received_lines)
            print('Data:', repr(line), file=smtpd.DEBUGSTREAM)
            self.received_lines = []
            try:
                self.auth_object(line)
            except ResponseException as e:
                self.smtp_state = self.COMMAND
                self.push('%s %s' % (e.smtp_code, e.smtp_error))
            return
        super().found_terminator()

    def smtp_AUTH(self, arg):
        if not self.seen_greeting:
            self.push('503 Error: send EHLO first')
            return
        if not self.extended_smtp or 'AUTH' not in self._extrafeatures:
            self.push('500 Error: command "AUTH" not recognized')
            return
        if self.authenticated_user is not None:
            self.push('503 Bad sequence of commands: already authenticated')
            return
        args = arg.split()
        if len(args) not in [1, 2]:
            self.push('501 Syntax: AUTH <mechanism> [initial-response]')
            return
        auth_object_name = '_auth_%s' % args[0].lower().replace('-', '_')
        try:
            self.auth_object = getattr(self, auth_object_name)
        except AttributeError:
            self.push('504 Command parameter not implemented: unsupported  authentication mechanism {!r}'.format(auth_object_name))
            return
        self.smtp_state = self.AUTH
        self.auth_object(args[1] if len(args) == 2 else None)

    def _authenticated(self, user, valid):
        if valid:
            self.authenticated_user = user
            self.push('235 Authentication Succeeded')
        else:
            self.push('535 Authentication credentials invalid')
        self.smtp_state = self.COMMAND

    def _decode_base64(self, string):
        return base64.decodebytes(string.encode('ascii')).decode('utf-8')

    def _auth_plain(self, arg=None):
        if arg is None:
            self.push('334 ')
        else:
            logpass = self._decode_base64(arg)
            try:
                *_, user, password = logpass.split('\x00')
            except ValueError as e:
                self.push('535 Splitting response {!r} into user and password failed: {}'.format(logpass, e))
                return
            self._authenticated(user, password == sim_auth[1])

    def _auth_login(self, arg=None):
        if arg is None:
            self.push('334 VXNlcm5hbWU6')
        elif not hasattr(self, '_auth_login_user'):
            self._auth_login_user = self._decode_base64(arg)
            self.push('334 UGFzc3dvcmQ6')
        else:
            password = self._decode_base64(arg)
            self._authenticated(self._auth_login_user, password == sim_auth[1])
            del self._auth_login_user

    def _auth_buggy(self, arg=None):
        self.push('334 QnVHZ1liVWdHeQ==')

    def _auth_cram_md5(self, arg=None):
        if arg is None:
            self.push('334 {}'.format(sim_cram_md5_challenge))
        else:
            logpass = self._decode_base64(arg)
            try:
                user, hashed_pass = logpass.split()
            except ValueError as e:
                self.push('535 Splitting response {!r} into user and password failed: {}'.format(logpass, e))
                return False
            valid_hashed_pass = hmac.HMAC(sim_auth[1].encode('ascii'), self._decode_base64(sim_cram_md5_challenge).encode('ascii'), 'md5').hexdigest()
            self._authenticated(user, hashed_pass == valid_hashed_pass)

    def smtp_EHLO(self, arg):
        resp = '250-testhost\r\n250-EXPN\r\n250-SIZE 20000000\r\n250-STARTTLS\r\n250-DELIVERBY\r\n'
        resp = resp + self._extrafeatures + '250 HELP'
        self.push(resp)
        self.seen_greeting = arg
        self.extended_smtp = True

    def smtp_VRFY(self, arg):
        if arg in sim_users:
            self.push('250 %s %s' % (sim_users[arg], smtplib.quoteaddr(arg)))
        else:
            self.push('550 No such user: %s' % arg)

    def smtp_EXPN(self, arg):
        list_name = arg.lower()
        if list_name in sim_lists:
            user_list = sim_lists[list_name]
            for n, user_email in enumerate(user_list):
                quoted_addr = smtplib.quoteaddr(user_email)
                if n < len(user_list) - 1:
                    self.push('250-%s %s' % (sim_users[user_email], quoted_addr))
                else:
                    self.push('250 %s %s' % (sim_users[user_email], quoted_addr))
        else:
            self.push('550 No access for you!')

    def smtp_QUIT(self, arg):
        if self.quit_response is None:
            super(SimSMTPChannel, self).smtp_QUIT(arg)
        else:
            self.push(self.quit_response)
            self.close_when_done()

    def smtp_MAIL(self, arg):
        if self.mail_response is None:
            super().smtp_MAIL(arg)
        else:
            self.push(self.mail_response)
            if self.disconnect:
                self.close_when_done()

    def smtp_RCPT(self, arg):
        if self.rcpt_response is None:
            super().smtp_RCPT(arg)
            return
        self.rcpt_count += 1
        self.push(self.rcpt_response[self.rcpt_count - 1])

    def smtp_RSET(self, arg):
        self.rset_count += 1
        super().smtp_RSET(arg)

    def smtp_DATA(self, arg):
        if self.data_response is None:
            super().smtp_DATA(arg)
        else:
            self.push(self.data_response)

    def handle_error(self):
        raise

class SimSMTPServer(smtpd.SMTPServer):
    channel_class = SimSMTPChannel

    def __init__(self, *args, **kw):
        self._extra_features = []
        self._addresses = {}
        smtpd.SMTPServer.__init__(self, *args, **kw)

    def handle_accepted(self, conn, addr):
        self._SMTPchannel = self.channel_class(self._extra_features, self, conn, addr, decode_data=self._decode_data)

    def process_message(self, peer, mailfrom, rcpttos, data):
        self._addresses['from'] = mailfrom
        self._addresses['tos'] = rcpttos

    def add_feature(self, feature):
        self._extra_features.append(feature)

    def handle_error(self):
        raise

class SimSMTPUTF8Server(SimSMTPServer):

    def __init__(self, *args, **kw):
        self._extra_features = ['SMTPUTF8', '8BITMIME']
        smtpd.SMTPServer.__init__(self, *args, **kw)

    def handle_accepted(self, conn, addr):
        self._SMTPchannel = self.channel_class(self._extra_features, self, conn, addr, decode_data=self._decode_data, enable_SMTPUTF8=self.enable_SMTPUTF8)

    def process_message(self, peer, mailfrom, rcpttos, data, mail_options=None, rcpt_options=None):
        self.last_peer = peer
        self.last_mailfrom = mailfrom
        self.last_rcpttos = rcpttos
        self.last_message = data
        self.last_mail_options = mail_options
        self.last_rcpt_options = rcpt_options

EXPECTED_RESPONSE = encode_base64(b'\x00psu\x00doesnotexist', eol='')

class SimSMTPAUTHInitialResponseChannel(SimSMTPChannel):

    def smtp_AUTH(self, arg):
        args = arg.split()
        if args[0].lower() == 'plain':
            if len(args) == 2:
                if args[1] == EXPECTED_RESPONSE:
                    self.push('235 Ok')
                    return
        self.push('571 Bad authentication')

class SimSMTPAUTHInitialResponseServer(SimSMTPServer):
    channel_class = SimSMTPAUTHInitialResponseChannel


# --- test body ---
self_msg = EmailMessage()
self_msg['From'] = 'Páolo <főo@bar.com>'
self_smtp = smtplib.SMTP()
self_smtp.ehlo = Mock(return_value=(200, 'OK'))
self_smtp.has_extn, self_smtp.sendmail = (Mock(), Mock())
expected_mail_options = ('SMTPUTF8', 'BODY=8BITMIME')
self_smtp.send_message(self_msg)
self_smtp.send_message(self_msg)

assert self_smtp.sendmail.call_args_list[0][0][3] == expected_mail_options

assert self_smtp.sendmail.call_args_list[1][0][3] == expected_mail_options
print("DefaultArgumentsTests::testSendMessage: ok")
