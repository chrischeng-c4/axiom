# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib2net"
# dimension = "behavior"
# case = "other_network_tests__test_file"
# subject = "cpython.test_urllib2net.OtherNetworkTests.test_file"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib2net.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_urllib2net.py::OtherNetworkTests::test_file
"""Auto-ported test: OtherNetworkTests::test_file (CPython 3.12 oracle)."""


import errno
import unittest
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import ResourceDenied
from test.test_urllib2 import sanepathname2url
import os
import socket
import urllib.error
import urllib.request
import sys


support.requires('network')

def _retry_thrice(func, exc, *args, **kwargs):
    for i in range(3):
        try:
            return func(*args, **kwargs)
        except exc as e:
            last_exc = e
            continue
    raise last_exc

def _wrap_with_retry_thrice(func, exc):

    def wrapped(*args, **kwargs):
        return _retry_thrice(func, exc, *args, **kwargs)
    return wrapped

_urlopen_with_retry = _wrap_with_retry_thrice(urllib.request.urlopen, urllib.error.URLError)

class TransientResource(object):
    """Raise ResourceDenied if an exception is raised while the context manager
    is in effect that matches the specified exception and attributes."""

    def __init__(self, exc, **kwargs):
        self.exc = exc
        self.attrs = kwargs

    def __enter__(self):
        return self

    def __exit__(self, type_=None, value=None, traceback=None):
        """If type_ is a subclass of self.exc and value has attributes matching
        self.attrs, raise ResourceDenied.  Otherwise let the exception
        propagate (if any)."""
        if type_ is not None and issubclass(self.exc, type_):
            for attr, attr_value in self.attrs.items():
                if not hasattr(value, attr):
                    break
                if getattr(value, attr) != attr_value:
                    break
            else:
                raise ResourceDenied('an optional resource is not available')

time_out = TransientResource(OSError, errno=errno.ETIMEDOUT)

socket_peer_reset = TransientResource(OSError, errno=errno.ECONNRESET)

ioerror_peer_reset = TransientResource(OSError, errno=errno.ECONNRESET)


# --- test body ---
def _extra_handlers():
    handlers = []
    cfh = urllib.request.CacheFTPHandler()
    pass
    cfh.setTimeout(1)
    handlers.append(cfh)
    return handlers

def _test_urls(urls, handlers, retry=True):
    import time
    import logging
    debug = logging.getLogger('test_urllib2').debug
    urlopen = urllib.request.build_opener(*handlers).open
    if retry:
        urlopen = _wrap_with_retry_thrice(urlopen, urllib.error.URLError)
    for url in urls:
        if isinstance(url, tuple):
            url, req, expected_err = url
        else:
            req = expected_err = None
        with socket_helper.transient_internet(url):
            try:
                f = urlopen(url, req, support.INTERNET_TIMEOUT)
            except OSError as err:
                if expected_err:
                    msg = "Didn't get expected error(s) %s for %s %s, got %s: %s" % (expected_err, url, req, type(err), err)

                    assert isinstance(err, expected_err)
                else:
                    raise
            else:
                try:
                    with time_out, socket_peer_reset, ioerror_peer_reset:
                        buf = f.read()
                        debug('read %d bytes' % len(buf))
                except TimeoutError:
                    print('<timeout: %s>' % url, file=sys.stderr)
                f.close()
        time.sleep(0.1)
if 0:
    import logging
    logger = logging.getLogger('test_urllib2net')
    logger.addHandler(logging.StreamHandler())
TESTFN = os_helper.TESTFN
f = open(TESTFN, 'w')
try:
    f.write('hi there\n')
    f.close()
    urls = ['file:' + sanepathname2url(os.path.abspath(TESTFN)), ('file:///nonsensename/etc/passwd', None, urllib.error.URLError)]
    _test_urls(urls, _extra_handlers(), retry=True)
finally:
    os.remove(TESTFN)

try:
    urllib.request.urlopen('./relative_path/to/file')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("OtherNetworkTests::test_file: ok")
