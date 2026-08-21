# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "quopri"
# dimension = "behavior"
# case = "quopri_test_case__test_decode"
# subject = "cpython.test_quopri.QuopriTestCase.test_decode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_quopri.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_quopri.py::QuopriTestCase::test_decode
"""Auto-ported test: QuopriTestCase::test_decode (CPython 3.12 oracle)."""


import unittest
import sys, io, subprocess
import quopri
from test import support


ENCSAMPLE = b"Here's a bunch of special=20\n\n=A1=A2=A3=A4=A5=A6=A7=A8=A9\n=AA=AB=AC=AD=AE=AF=B0=B1=B2=B3\n=B4=B5=B6=B7=B8=B9=BA=BB=BC=BD=BE\n=BF=C0=C1=C2=C3=C4=C5=C6\n=C7=C8=C9=CA=CB=CC=CD=CE=CF\n=D0=D1=D2=D3=D4=D5=D6=D7\n=D8=D9=DA=DB=DC=DD=DE=DF\n=E0=E1=E2=E3=E4=E5=E6=E7\n=E8=E9=EA=EB=EC=ED=EE=EF\n=F0=F1=F2=F3=F4=F5=F6=F7\n=F8=F9=FA=FB=FC=FD=FE=FF\n\ncharacters... have fun!\n"

DECSAMPLE = b"Here's a bunch of special \n" + b'\n\xa1\xa2\xa3\xa4\xa5\xa6\xa7\xa8\xa9\n\xaa\xab\xac\xad\xae\xaf\xb0\xb1\xb2\xb3\n\xb4\xb5\xb6\xb7\xb8\xb9\xba\xbb\xbc\xbd\xbe\n\xbf\xc0\xc1\xc2\xc3\xc4\xc5\xc6\n\xc7\xc8\xc9\xca\xcb\xcc\xcd\xce\xcf\n\xd0\xd1\xd2\xd3\xd4\xd5\xd6\xd7\n\xd8\xd9\xda\xdb\xdc\xdd\xde\xdf\n\xe0\xe1\xe2\xe3\xe4\xe5\xe6\xe7\n\xe8\xe9\xea\xeb\xec\xed\xee\xef\n\xf0\xf1\xf2\xf3\xf4\xf5\xf6\xf7\n\xf8\xf9\xfa\xfb\xfc\xfd\xfe\xff\n\ncharacters... have fun!\n'

def withpythonimplementation(testfunc):

    def newtest(self):
        testfunc(self)
        if quopri.b2a_qp is not None or quopri.a2b_qp is not None:
            oldencode = quopri.b2a_qp
            olddecode = quopri.a2b_qp
            try:
                quopri.b2a_qp = None
                quopri.a2b_qp = None
                testfunc(self)
            finally:
                quopri.b2a_qp = oldencode
                quopri.a2b_qp = olddecode
    newtest.__name__ = testfunc.__name__
    return newtest


# --- test body ---
STRINGS = ((b'hello', b'hello'), (b'hello\n        there\n        world', b'hello\n        there\n        world'), (b'hello\n        there\n        world\n', b'hello\n        there\n        world\n'), (b'\x81\x82\x83', b'=81=82=83'), (b'hello ', b'hello=20'), (b'hello\t', b'hello=09'), (b'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\xd8\xd9\xda\xdb\xdc\xdd\xde\xdfxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx', b'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx=D8=D9=DA=DB=DC=DD=DE=DFx=\nxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx'), (b'yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy', b'yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy'), (b'zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz', b'zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz=\nzz'), (b'zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz', b'zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz=\nzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz'), (b'yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy\nzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz', b'yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy=\nyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy\nzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz'), (DECSAMPLE, ENCSAMPLE))
ESTRINGS = ((b'hello world', b'hello=20world'), (b'hello\tworld', b'hello=09world'))
HSTRINGS = ((b'hello world', b'hello_world'), (b'hello_world', b'hello=5Fworld'))
for p, e in STRINGS:
    infp = io.BytesIO(e)
    outfp = io.BytesIO()
    quopri.decode(infp, outfp)

    assert outfp.getvalue() == p
print("QuopriTestCase::test_decode: ok")
