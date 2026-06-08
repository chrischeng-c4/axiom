# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_from_mangling__test_multipart_with_bad_bytes_in_cte"
# subject = "cpython.test_email.TestFromMangling.test_multipart_with_bad_bytes_in_cte"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
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
source = textwrap.dedent('            From: aperson@example.com\n            Content-Type: multipart/mixed; boundary="1"\n            Content-Transfer-Encoding: È\n        ').encode('utf-8')
msg = email.message_from_bytes(source)

print("TestFromMangling::test_multipart_with_bad_bytes_in_cte: ok")
