# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_m_i_m_e_application__test_binary_body_with_encode_quopri"
# subject = "cpython.test_email.TestMIMEApplication.test_binary_body_with_encode_quopri"
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
bytesdata = b'\xfa\xfb\xfc\xfd\xfe\xff '
msg = MIMEApplication(bytesdata, _encoder=encoders.encode_quopri)
assert msg.get_payload() == '=FA=FB=FC=FD=FE=FF=20'
assert msg.get_payload(decode=True) == bytesdata
assert msg['Content-Transfer-Encoding'] == 'quoted-printable'
s = BytesIO()
g = BytesGenerator(s)
g.flatten(msg)
wireform = s.getvalue()
msg2 = email.message_from_bytes(wireform)
assert msg.get_payload() == '=FA=FB=FC=FD=FE=FF=20'
assert msg2.get_payload(decode=True) == bytesdata
assert msg2['Content-Transfer-Encoding'] == 'quoted-printable'

print("TestMIMEApplication::test_binary_body_with_encode_quopri: ok")
