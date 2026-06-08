# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_quopri__test_header_encode_non_ascii"
# subject = "cpython.test_email.TestQuopri.test_header_encode_non_ascii"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import re
import time
import base64
import textwrap
from io import StringIO, BytesIO
from itertools import chain
from random import choice
from threading import Thread
import email
import email.policy
import email.utils
from email.charset import Charset
from email.generator import Generator, DecodedGenerator, BytesGenerator
from email.header import Header, decode_header, make_header
from email.headerregistry import HeaderRegistry
from email.message import Message
from email.mime.application import MIMEApplication
from email.mime.audio import MIMEAudio
from email.mime.base import MIMEBase
from email.mime.image import MIMEImage
from email.mime.message import MIMEMessage
from email.mime.multipart import MIMEMultipart
from email.mime.nonmultipart import MIMENonMultipart
from email.mime.text import MIMEText
from email.parser import Parser, HeaderParser
from email import base64mime
from email import encoders
from email import errors
from email import iterators
from email import quoprimime
from email import utils
from email.parser import FeedParser

def _test_header_encode(header, expected_encoded_header, charset=None):
    if charset is None:
        encoded_header = quoprimime.header_encode(header)
    else:
        encoded_header = quoprimime.header_encode(header, charset)
    assert encoded_header == expected_encoded_header

def _test_header_decode(encoded_header, expected_decoded_header):
    decoded_header = quoprimime.header_decode(encoded_header)
    assert decoded_header == expected_decoded_header

def _test_decode(encoded, expected_decoded, eol=None):
    if eol is None:
        decoded = quoprimime.decode(encoded)
    else:
        decoded = quoprimime.decode(encoded, eol=eol)
    assert decoded == expected_decoded

def _test_encode(body, expected_encoded_body, maxlinelen=None, eol=None):
    kwargs = {}
    if maxlinelen is None:
        maxlinelen = 76
    else:
        kwargs['maxlinelen'] = maxlinelen
    if eol is None:
        eol = '\n'
    else:
        kwargs['eol'] = eol
    encoded_body = quoprimime.body_encode(body, **kwargs)
    assert encoded_body == expected_encoded_body
    if eol == '\n' or eol == '\r\n':
        for line in encoded_body.splitlines():
            assert len(line) <= maxlinelen
self_hlit = list(chain(range(ord('a'), ord('z') + 1), range(ord('A'), ord('Z') + 1), range(ord('0'), ord('9') + 1), (c for c in b'!*+-/')))
self_hnon = [c for c in range(256) if c not in self_hlit]
assert len(self_hlit) + len(self_hnon) == 256
self_blit = list(range(ord(' '), ord('~') + 1))
self_blit.append(ord('\t'))
self_blit.remove(ord('='))
self_bnon = [c for c in range(256) if c not in self_blit]
assert len(self_blit) + len(self_bnon) == 256
_test_header_encode(b'hello\xc7there', '=?iso-8859-1?q?hello=C7there?=')

print("TestQuopri::test_header_encode_non_ascii: ok")
