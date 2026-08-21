# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_from_mangling__test_mangled_from_with_bad_bytes"
# subject = "cpython.test_email.TestFromMangling.test_mangled_from_with_bad_bytes"
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
self_msg = Message()
self_msg['From'] = 'aaa@bbb.org'
self_msg.set_payload('From the desk of A.A.A.:\nBlah blah blah\n')
source = textwrap.dedent('            Content-Type: text/plain; charset="utf-8"\n            MIME-Version: 1.0\n            Content-Transfer-Encoding: 8bit\n            From: aaa@bbb.org\n\n        ').encode('utf-8')
msg = email.message_from_bytes(source + b'From R\xc3\xb6lli\n')
b = BytesIO()
g = BytesGenerator(b, mangle_from_=True)
g.flatten(msg)
assert b.getvalue() == source + b'>From R\xc3\xb6lli\n'

print("TestFromMangling::test_mangled_from_with_bad_bytes: ok")
