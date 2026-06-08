# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgi"
# dimension = "behavior"
# case = "cgi_tests__test_fieldstorage_multipart_w3c"
# subject = "cpython.test_cgi.CgiTests.test_fieldstorage_multipart_w3c"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cgi.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cgi.py::CgiTests::test_fieldstorage_multipart_w3c
"""Auto-ported test: CgiTests::test_fieldstorage_multipart_w3c (CPython 3.12 oracle)."""


import os
import sys
import tempfile
import unittest
from collections import namedtuple
from io import StringIO, BytesIO
from test import support
from test.support import warnings_helper


cgi = warnings_helper.import_deprecated('cgi')

class HackedSysModule:
    argv = []
    stdin = sys.stdin

cgi.sys = HackedSysModule()

class ComparableException:

    def __init__(self, err):
        self.err = err

    def __str__(self):
        return str(self.err)

    def __eq__(self, anExc):
        if not isinstance(anExc, Exception):
            return NotImplemented
        return self.err.__class__ == anExc.__class__ and self.err.args == anExc.args

    def __getattr__(self, attr):
        return getattr(self.err, attr)

def do_test(buf, method):
    env = {}
    if method == 'GET':
        fp = None
        env['REQUEST_METHOD'] = 'GET'
        env['QUERY_STRING'] = buf
    elif method == 'POST':
        fp = BytesIO(buf.encode('latin-1'))
        env['REQUEST_METHOD'] = 'POST'
        env['CONTENT_TYPE'] = 'application/x-www-form-urlencoded'
        env['CONTENT_LENGTH'] = str(len(buf))
    else:
        raise ValueError('unknown method: %s' % method)
    try:
        return cgi.parse(fp, env, strict_parsing=1)
    except Exception as err:
        return ComparableException(err)

parse_strict_test_cases = [('', {}), ('&', ValueError("bad query field: ''")), ('&&', ValueError("bad query field: ''")), ('=', {}), ('=&=', {}), ('=a', {'': ['a']}), ('&=a', ValueError("bad query field: ''")), ('=a&', ValueError("bad query field: ''")), ('=&a', ValueError("bad query field: 'a'")), ('b=a', {'b': ['a']}), ('b+=a', {'b ': ['a']}), ('a=b=a', {'a': ['b=a']}), ('a=+b=a', {'a': [' b=a']}), ('&b=a', ValueError("bad query field: ''")), ('b&=a', ValueError("bad query field: 'b'")), ('a=a+b&b=b+c', {'a': ['a b'], 'b': ['b c']}), ('a=a+b&a=b+a', {'a': ['a b', 'b a']}), ('x=1&y=2.0&z=2-3.%2b0', {'x': ['1'], 'y': ['2.0'], 'z': ['2-3.+0']}), ('Hbc5161168c542333633315dee1182227:key_store_seqid=400006&cuyer=r&view=bustomer&order_id=0bb2e248638833d48cb7fed300000f1b&expire=964546263&lobale=en-US&kid=130003.300038&ss=env', {'Hbc5161168c542333633315dee1182227:key_store_seqid': ['400006'], 'cuyer': ['r'], 'expire': ['964546263'], 'kid': ['130003.300038'], 'lobale': ['en-US'], 'order_id': ['0bb2e248638833d48cb7fed300000f1b'], 'ss': ['env'], 'view': ['bustomer']}), ('group_id=5470&set=custom&_assigned_to=31392&_status=1&_category=100&SUBMIT=Browse', {'SUBMIT': ['Browse'], '_assigned_to': ['31392'], '_category': ['100'], '_status': ['1'], 'group_id': ['5470'], 'set': ['custom']})]

def norm(seq):
    return sorted(seq, key=repr)

def first_elts(list):
    return [p[0] for p in list]

def first_second_elts(list):
    return [(p[0], p[1][0]) for p in list]

def gen_result(data, environ):
    encoding = 'latin-1'
    fake_stdin = BytesIO(data.encode(encoding))
    fake_stdin.seek(0)
    form = cgi.FieldStorage(fp=fake_stdin, environ=environ, encoding=encoding)
    result = {}
    for k, v in dict(form).items():
        result[k] = isinstance(v, list) and form.getlist(k) or v.value
    return result

BOUNDARY = '---------------------------721837373350705526688164684'

POSTDATA = '-----------------------------721837373350705526688164684\nContent-Disposition: form-data; name="id"\n\n1234\n-----------------------------721837373350705526688164684\nContent-Disposition: form-data; name="title"\n\n\n-----------------------------721837373350705526688164684\nContent-Disposition: form-data; name="file"; filename="test.txt"\nContent-Type: text/plain\n\nTesting 123.\n\n-----------------------------721837373350705526688164684\nContent-Disposition: form-data; name="submit"\n\n Add \n-----------------------------721837373350705526688164684--\n'

POSTDATA_NON_ASCII = '-----------------------------721837373350705526688164684\nContent-Disposition: form-data; name="id"\n\nçñ\x80\n-----------------------------721837373350705526688164684\n'

BOUNDARY_W3 = 'AaB03x'

POSTDATA_W3 = '--AaB03x\nContent-Disposition: form-data; name="submit-name"\n\nLarry\n--AaB03x\nContent-Disposition: form-data; name="files"\nContent-Type: multipart/mixed; boundary=BbC04y\n\n--BbC04y\nContent-Disposition: file; filename="file1.txt"\nContent-Type: text/plain\n\n... contents of file1.txt ...\n--BbC04y\nContent-Disposition: file; filename="file2.gif"\nContent-Type: image/gif\nContent-Transfer-Encoding: binary\n\n...contents of file2.gif...\n--BbC04y--\n--AaB03x--\n'


# --- test body ---
env = {'REQUEST_METHOD': 'POST', 'CONTENT_TYPE': 'multipart/form-data; boundary={}'.format(BOUNDARY_W3), 'CONTENT_LENGTH': str(len(POSTDATA_W3))}
fp = BytesIO(POSTDATA_W3.encode('latin-1'))
fs = cgi.FieldStorage(fp, environ=env, encoding='latin-1')

assert len(fs.list) == 2

assert fs.list[0].name == 'submit-name'

assert fs.list[0].value == 'Larry'

assert fs.list[1].name == 'files'
files = fs.list[1].value

assert len(files) == 2
expect = [{'name': None, 'filename': 'file1.txt', 'value': b'... contents of file1.txt ...'}, {'name': None, 'filename': 'file2.gif', 'value': b'...contents of file2.gif...'}]
for x in range(len(files)):
    for k, exp in expect[x].items():
        got = getattr(files[x], k)

        assert got == exp
print("CgiTests::test_fieldstorage_multipart_w3c: ok")
