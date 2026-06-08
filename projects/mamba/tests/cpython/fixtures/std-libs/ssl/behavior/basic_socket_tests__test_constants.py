# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "basic_socket_tests__test_constants"
# subject = "cpython.test_ssl.BasicSocketTests.test_constants"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ssl.py::BasicSocketTests::test_constants
"""Auto-ported test: BasicSocketTests::test_constants (CPython 3.12 oracle)."""


import sys
import unittest
import unittest.mock
from test import support
from test.support import import_helper
from test.support import os_helper
from test.support import socket_helper
from test.support import threading_helper
from test.support import warnings_helper
from test.support import asyncore
import array
import re
import socket
import select
import struct
import time
import enum
import gc
import http.client
import os
import errno
import pprint
import urllib.request
import threading
import traceback
import weakref
import platform
import sysconfig
import functools
import _ssl
from ssl import TLSVersion, _TLSContentType, _TLSMessageType, _TLSAlertType
from test.ssl_servers import make_https_server


try:
    import ctypes
except ImportError:
    ctypes = None

ssl = import_helper.import_module('ssl')

Py_DEBUG_WIN32 = support.Py_DEBUG and sys.platform == 'win32'

PROTOCOLS = sorted(ssl._PROTOCOL_NAMES)

HOST = socket_helper.HOST

IS_OPENSSL_3_0_0 = ssl.OPENSSL_VERSION_INFO >= (3, 0, 0)

PY_SSL_DEFAULT_CIPHERS = sysconfig.get_config_var('PY_SSL_DEFAULT_CIPHERS')

PROTOCOL_TO_TLS_VERSION = {}

for proto, ver in (('PROTOCOL_SSLv3', 'SSLv3'), ('PROTOCOL_TLSv1', 'TLSv1'), ('PROTOCOL_TLSv1_1', 'TLSv1_1')):
    try:
        proto = getattr(ssl, proto)
        ver = getattr(ssl.TLSVersion, ver)
    except AttributeError:
        continue
    PROTOCOL_TO_TLS_VERSION[proto] = ver

def data_file(*name):
    return os.path.join(os.path.dirname(__file__), 'certdata', *name)

CERTFILE = data_file('keycert.pem')

BYTES_CERTFILE = os.fsencode(CERTFILE)

ONLYCERT = data_file('ssl_cert.pem')

ONLYKEY = data_file('ssl_key.pem')

BYTES_ONLYCERT = os.fsencode(ONLYCERT)

BYTES_ONLYKEY = os.fsencode(ONLYKEY)

CERTFILE_PROTECTED = data_file('keycert.passwd.pem')

ONLYKEY_PROTECTED = data_file('ssl_key.passwd.pem')

KEY_PASSWORD = 'somepass'

CAPATH = data_file('capath')

BYTES_CAPATH = os.fsencode(CAPATH)

CAFILE_NEURONIO = data_file('capath', '4e1295a3.0')

CAFILE_CACERT = data_file('capath', '5ed36f99.0')

CERTFILE_INFO = {'issuer': ((('countryName', 'XY'),), (('localityName', 'Castle Anthrax'),), (('organizationName', 'Python Software Foundation'),), (('commonName', 'localhost'),)), 'notAfter': 'Aug 26 14:23:15 2028 GMT', 'notBefore': 'Aug 29 14:23:15 2018 GMT', 'serialNumber': '98A7CF88C74A32ED', 'subject': ((('countryName', 'XY'),), (('localityName', 'Castle Anthrax'),), (('organizationName', 'Python Software Foundation'),), (('commonName', 'localhost'),)), 'subjectAltName': (('DNS', 'localhost'),), 'version': 3}

CRLFILE = data_file('revocation.crl')

SIGNED_CERTFILE = data_file('keycert3.pem')

SIGNED_CERTFILE_HOSTNAME = 'localhost'

SIGNED_CERTFILE_INFO = {'OCSP': ('http://testca.pythontest.net/testca/ocsp/',), 'caIssuers': ('http://testca.pythontest.net/testca/pycacert.cer',), 'crlDistributionPoints': ('http://testca.pythontest.net/testca/revocation.crl',), 'issuer': ((('countryName', 'XY'),), (('organizationName', 'Python Software Foundation CA'),), (('commonName', 'our-ca-server'),)), 'notAfter': 'Oct 28 14:23:16 2037 GMT', 'notBefore': 'Aug 29 14:23:16 2018 GMT', 'serialNumber': 'CB2D80995A69525C', 'subject': ((('countryName', 'XY'),), (('localityName', 'Castle Anthrax'),), (('organizationName', 'Python Software Foundation'),), (('commonName', 'localhost'),)), 'subjectAltName': (('DNS', 'localhost'),), 'version': 3}

SIGNED_CERTFILE2 = data_file('keycert4.pem')

SIGNED_CERTFILE2_HOSTNAME = 'fakehostname'

SIGNED_CERTFILE_ECC = data_file('keycertecc.pem')

SIGNED_CERTFILE_ECC_HOSTNAME = 'localhost-ecc'

SIGNING_CA = data_file('capath', 'ceff1710.0')

ALLSANFILE = data_file('allsans.pem')

IDNSANSFILE = data_file('idnsans.pem')

NOSANFILE = data_file('nosan.pem')

NOSAN_HOSTNAME = 'localhost'

REMOTE_HOST = 'self-signed.pythontest.net'

EMPTYCERT = data_file('nullcert.pem')

BADCERT = data_file('badcert.pem')

NONEXISTINGCERT = data_file('XXXnonexisting.pem')

BADKEY = data_file('badkey.pem')

NOKIACERT = data_file('nokia.pem')

NULLBYTECERT = data_file('nullbytecert.pem')

TALOS_INVALID_CRLDP = data_file('talos-2019-0758.pem')

DHFILE = data_file('ffdh3072.pem')

BYTES_DHFILE = os.fsencode(DHFILE)

OP_NO_COMPRESSION = getattr(ssl, 'OP_NO_COMPRESSION', 0)

OP_SINGLE_DH_USE = getattr(ssl, 'OP_SINGLE_DH_USE', 0)

OP_SINGLE_ECDH_USE = getattr(ssl, 'OP_SINGLE_ECDH_USE', 0)

OP_CIPHER_SERVER_PREFERENCE = getattr(ssl, 'OP_CIPHER_SERVER_PREFERENCE', 0)

OP_ENABLE_MIDDLEBOX_COMPAT = getattr(ssl, 'OP_ENABLE_MIDDLEBOX_COMPAT', 0)

def is_ubuntu():
    try:
        with open('/etc/os-release', encoding='utf-8') as f:
            return 'ubuntu' in f.read()
    except FileNotFoundError:
        return False

if is_ubuntu():

    def seclevel_workaround(*ctxs):
        """"Lower security level to '1' and allow all ciphers for TLS 1.0/1"""
        for ctx in ctxs:
            if hasattr(ctx, 'minimum_version') and ctx.minimum_version <= ssl.TLSVersion.TLSv1_1 and (ctx.security_level > 1):
                ctx.set_ciphers('@SECLEVEL=1:ALL')
else:

    def seclevel_workaround(*ctxs):
        pass

def has_tls_protocol(protocol):
    """Check if a TLS protocol is available and enabled

    :param protocol: enum ssl._SSLMethod member or name
    :return: bool
    """
    if isinstance(protocol, str):
        assert protocol.startswith('PROTOCOL_')
        protocol = getattr(ssl, protocol, None)
        if protocol is None:
            return False
    if protocol in {ssl.PROTOCOL_TLS, ssl.PROTOCOL_TLS_SERVER, ssl.PROTOCOL_TLS_CLIENT}:
        return True
    name = protocol.name
    return has_tls_version(name[len('PROTOCOL_'):])

@functools.lru_cache
def has_tls_version(version):
    """Check if a TLS/SSL version is enabled

    :param version: TLS version name or ssl.TLSVersion member
    :return: bool
    """
    if isinstance(version, str):
        version = ssl.TLSVersion.__members__[version]
    if not getattr(ssl, f'HAS_{version.name}'):
        return False
    if IS_OPENSSL_3_0_0 and version < ssl.TLSVersion.TLSv1_2:
        return False
    ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
    if hasattr(ctx, 'minimum_version') and ctx.minimum_version != ssl.TLSVersion.MINIMUM_SUPPORTED and (version < ctx.minimum_version):
        return False
    if hasattr(ctx, 'maximum_version') and ctx.maximum_version != ssl.TLSVersion.MAXIMUM_SUPPORTED and (version > ctx.maximum_version):
        return False
    return True

def requires_tls_version(version):
    """Decorator to skip tests when a required TLS version is not available

    :param version: TLS version name or ssl.TLSVersion member
    :return:
    """

    def decorator(func):

        @functools.wraps(func)
        def wrapper(*args, **kw):
            if not has_tls_version(version):
                raise unittest.SkipTest(f'{version} is not available.')
            else:
                return func(*args, **kw)
        return wrapper
    return decorator

def handle_error(prefix):
    exc_format = ' '.join(traceback.format_exception(sys.exception()))
    if support.verbose:
        sys.stdout.write(prefix + exc_format)

def utc_offset():
    if time.daylight and time.localtime().tm_isdst > 0:
        return -time.altzone
    return -time.timezone

ignore_deprecation = warnings_helper.ignore_warnings(category=DeprecationWarning)

def test_wrap_socket(sock, *, cert_reqs=ssl.CERT_NONE, ca_certs=None, ciphers=None, certfile=None, keyfile=None, **kwargs):
    if not kwargs.get('server_side'):
        kwargs['server_hostname'] = SIGNED_CERTFILE_HOSTNAME
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
    else:
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    if cert_reqs is not None:
        if cert_reqs == ssl.CERT_NONE:
            context.check_hostname = False
        context.verify_mode = cert_reqs
    if ca_certs is not None:
        context.load_verify_locations(ca_certs)
    if certfile is not None or keyfile is not None:
        context.load_cert_chain(certfile, keyfile)
    if ciphers is not None:
        context.set_ciphers(ciphers)
    return context.wrap_socket(sock, **kwargs)

def testing_context(server_cert=SIGNED_CERTFILE, *, server_chain=True):
    """Create context

    client_context, server_context, hostname = testing_context()
    """
    if server_cert == SIGNED_CERTFILE:
        hostname = SIGNED_CERTFILE_HOSTNAME
    elif server_cert == SIGNED_CERTFILE2:
        hostname = SIGNED_CERTFILE2_HOSTNAME
    elif server_cert == NOSANFILE:
        hostname = NOSAN_HOSTNAME
    else:
        raise ValueError(server_cert)
    client_context = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
    client_context.load_verify_locations(SIGNING_CA)
    server_context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    server_context.load_cert_chain(server_cert)
    if server_chain:
        server_context.load_verify_locations(SIGNING_CA)
    return (client_context, server_context, hostname)

def _test_get_server_certificate(test, host, port, cert=None):
    pem = ssl.get_server_certificate((host, port))
    if not pem:
        test.fail('No server certificate on %s:%s!' % (host, port))
    pem = ssl.get_server_certificate((host, port), ca_certs=cert)
    if not pem:
        test.fail('No server certificate on %s:%s!' % (host, port))
    if support.verbose:
        sys.stdout.write('\nVerified certificate for %s:%s is\n%s\n' % (host, port, pem))

def _test_get_server_certificate_fail(test, host, port):
    with warnings_helper.check_no_resource_warning(test):
        try:
            pem = ssl.get_server_certificate((host, port), ca_certs=CERTFILE)
        except ssl.SSLError as x:
            if support.verbose:
                sys.stdout.write('%s\n' % x)
        else:
            test.fail('Got server certificate %s for %s:%s!' % (pem, host, port))

class ThreadedEchoServer(threading.Thread):

    class ConnectionHandler(threading.Thread):
        """A mildly complicated class, because we want it to work both
        with and without the SSL wrapper around the socket connection, so
        that we can test the STARTTLS functionality."""

        def __init__(self, server, connsock, addr):
            self.server = server
            self.running = False
            self.sock = connsock
            self.addr = addr
            self.sock.setblocking(True)
            self.sslconn = None
            threading.Thread.__init__(self)
            self.daemon = True

        def wrap_conn(self):
            try:
                self.sslconn = self.server.context.wrap_socket(self.sock, server_side=True)
                self.server.selected_alpn_protocols.append(self.sslconn.selected_alpn_protocol())
            except (ConnectionResetError, BrokenPipeError, ConnectionAbortedError) as e:
                self.server.conn_errors.append(str(e))
                if self.server.chatty:
                    handle_error('\n server:  bad connection attempt from ' + repr(self.addr) + ':\n')
                self.running = False
                self.close()
                return False
            except (ssl.SSLError, OSError) as e:
                self.server.conn_errors.append(str(e))
                if self.server.chatty:
                    handle_error('\n server:  bad connection attempt from ' + repr(self.addr) + ':\n')
                if e.errno != errno.EPROTOTYPE and sys.platform != 'darwin':
                    self.running = False
                    self.close()
                return False
            else:
                self.server.shared_ciphers.append(self.sslconn.shared_ciphers())
                if self.server.context.verify_mode == ssl.CERT_REQUIRED:
                    cert = self.sslconn.getpeercert()
                    if support.verbose and self.server.chatty:
                        sys.stdout.write(' client cert is ' + pprint.pformat(cert) + '\n')
                    cert_binary = self.sslconn.getpeercert(True)
                    if support.verbose and self.server.chatty:
                        if cert_binary is None:
                            sys.stdout.write(' client did not provide a cert\n')
                        else:
                            sys.stdout.write(f' cert binary is {len(cert_binary)}b\n')
                cipher = self.sslconn.cipher()
                if support.verbose and self.server.chatty:
                    sys.stdout.write(' server: connection cipher is now ' + str(cipher) + '\n')
                return True

        def read(self):
            if self.sslconn:
                return self.sslconn.read()
            else:
                return self.sock.recv(1024)

        def write(self, bytes):
            if self.sslconn:
                return self.sslconn.write(bytes)
            else:
                return self.sock.send(bytes)

        def close(self):
            if self.sslconn:
                self.sslconn.close()
            else:
                self.sock.close()

        def run(self):
            self.running = True
            if not self.server.starttls_server:
                if not self.wrap_conn():
                    return
            while self.running:
                try:
                    msg = self.read()
                    stripped = msg.strip()
                    if not stripped:
                        self.running = False
                        try:
                            self.sock = self.sslconn.unwrap()
                        except OSError:
                            pass
                        else:
                            self.sslconn = None
                        self.close()
                    elif stripped == b'over':
                        if support.verbose and self.server.connectionchatty:
                            sys.stdout.write(' server: client closed connection\n')
                        self.close()
                        return
                    elif self.server.starttls_server and stripped == b'STARTTLS':
                        if support.verbose and self.server.connectionchatty:
                            sys.stdout.write(' server: read STARTTLS from client, sending OK...\n')
                        self.write(b'OK\n')
                        if not self.wrap_conn():
                            return
                    elif self.server.starttls_server and self.sslconn and (stripped == b'ENDTLS'):
                        if support.verbose and self.server.connectionchatty:
                            sys.stdout.write(' server: read ENDTLS from client, sending OK...\n')
                        self.write(b'OK\n')
                        self.sock = self.sslconn.unwrap()
                        self.sslconn = None
                        if support.verbose and self.server.connectionchatty:
                            sys.stdout.write(' server: connection is now unencrypted...\n')
                    elif stripped == b'CB tls-unique':
                        if support.verbose and self.server.connectionchatty:
                            sys.stdout.write(' server: read CB tls-unique from client, sending our CB data...\n')
                        data = self.sslconn.get_channel_binding('tls-unique')
                        self.write(repr(data).encode('us-ascii') + b'\n')
                    elif stripped == b'PHA':
                        if support.verbose and self.server.connectionchatty:
                            sys.stdout.write(' server: initiating post handshake auth\n')
                        try:
                            self.sslconn.verify_client_post_handshake()
                        except ssl.SSLError as e:
                            self.write(repr(e).encode('us-ascii') + b'\n')
                        else:
                            self.write(b'OK\n')
                    elif stripped == b'HASCERT':
                        if self.sslconn.getpeercert() is not None:
                            self.write(b'TRUE\n')
                        else:
                            self.write(b'FALSE\n')
                    elif stripped == b'GETCERT':
                        cert = self.sslconn.getpeercert()
                        self.write(repr(cert).encode('us-ascii') + b'\n')
                    elif stripped == b'VERIFIEDCHAIN':
                        certs = self.sslconn._sslobj.get_verified_chain()
                        self.write(len(certs).to_bytes(1, 'big') + b'\n')
                    elif stripped == b'UNVERIFIEDCHAIN':
                        certs = self.sslconn._sslobj.get_unverified_chain()
                        self.write(len(certs).to_bytes(1, 'big') + b'\n')
                    else:
                        if support.verbose and self.server.connectionchatty:
                            ctype = self.sslconn and 'encrypted' or 'unencrypted'
                            sys.stdout.write(' server: read %r (%s), sending back %r (%s)...\n' % (msg, ctype, msg.lower(), ctype))
                        self.write(msg.lower())
                except OSError as e:
                    if self.server.chatty and support.verbose:
                        if isinstance(e, ConnectionError):
                            print(f' Connection reset by peer: {self.addr}')
                        else:
                            handle_error('Test server failure:\n')
                    try:
                        self.write(b'ERROR\n')
                    except OSError:
                        pass
                    self.close()
                    self.running = False

    def __init__(self, certificate=None, ssl_version=None, certreqs=None, cacerts=None, chatty=True, connectionchatty=False, starttls_server=False, alpn_protocols=None, ciphers=None, context=None):
        if context:
            self.context = context
        else:
            self.context = ssl.SSLContext(ssl_version if ssl_version is not None else ssl.PROTOCOL_TLS_SERVER)
            self.context.verify_mode = certreqs if certreqs is not None else ssl.CERT_NONE
            if cacerts:
                self.context.load_verify_locations(cacerts)
            if certificate:
                self.context.load_cert_chain(certificate)
            if alpn_protocols:
                self.context.set_alpn_protocols(alpn_protocols)
            if ciphers:
                self.context.set_ciphers(ciphers)
        self.chatty = chatty
        self.connectionchatty = connectionchatty
        self.starttls_server = starttls_server
        self.sock = socket.socket()
        self.port = socket_helper.bind_port(self.sock)
        self.flag = None
        self.active = False
        self.selected_alpn_protocols = []
        self.shared_ciphers = []
        self.conn_errors = []
        threading.Thread.__init__(self)
        self.daemon = True
        self._in_context = False

    def __enter__(self):
        if self._in_context:
            raise ValueError('Re-entering ThreadedEchoServer context')
        self._in_context = True
        self.start(threading.Event())
        self.flag.wait()
        return self

    def __exit__(self, *args):
        assert self._in_context
        self._in_context = False
        self.stop()
        self.join()

    def start(self, flag=None):
        if not self._in_context:
            raise ValueError('ThreadedEchoServer must be used as a context manager')
        self.flag = flag
        threading.Thread.start(self)

    def run(self):
        if not self._in_context:
            raise ValueError('ThreadedEchoServer must be used as a context manager')
        self.sock.settimeout(1.0)
        self.sock.listen(5)
        self.active = True
        if self.flag:
            self.flag.set()
        while self.active:
            try:
                newconn, connaddr = self.sock.accept()
                if support.verbose and self.chatty:
                    sys.stdout.write(' server:  new connection from ' + repr(connaddr) + '\n')
                handler = self.ConnectionHandler(self, newconn, connaddr)
                handler.start()
                handler.join()
            except TimeoutError as e:
                if support.verbose:
                    sys.stdout.write(f' connection timeout {e!r}\n')
            except KeyboardInterrupt:
                self.stop()
            except BaseException as e:
                if support.verbose and self.chatty:
                    sys.stdout.write(' connection handling failed: ' + repr(e) + '\n')
        self.close()

    def close(self):
        if self.sock is not None:
            self.sock.close()
            self.sock = None

    def stop(self):
        self.active = False

class AsyncoreEchoServer(threading.Thread):

    class EchoServer(asyncore.dispatcher):

        class ConnectionHandler(asyncore.dispatcher_with_send):

            def __init__(self, conn, certfile):
                self.socket = test_wrap_socket(conn, server_side=True, certfile=certfile, do_handshake_on_connect=False)
                asyncore.dispatcher_with_send.__init__(self, self.socket)
                self._ssl_accepting = True
                self._do_ssl_handshake()

            def readable(self):
                if isinstance(self.socket, ssl.SSLSocket):
                    while self.socket.pending() > 0:
                        self.handle_read_event()
                return True

            def _do_ssl_handshake(self):
                try:
                    self.socket.do_handshake()
                except (ssl.SSLWantReadError, ssl.SSLWantWriteError):
                    return
                except ssl.SSLEOFError:
                    return self.handle_close()
                except ssl.SSLError:
                    raise
                except OSError as err:
                    if err.args[0] == errno.ECONNABORTED:
                        return self.handle_close()
                else:
                    self._ssl_accepting = False

            def handle_read(self):
                if self._ssl_accepting:
                    self._do_ssl_handshake()
                else:
                    data = self.recv(1024)
                    if support.verbose:
                        sys.stdout.write(' server:  read %s from client\n' % repr(data))
                    if not data:
                        self.close()
                    else:
                        self.send(data.lower())

            def handle_close(self):
                self.close()
                if support.verbose:
                    sys.stdout.write(' server:  closed connection %s\n' % self.socket)

            def handle_error(self):
                raise

        def __init__(self, certfile):
            self.certfile = certfile
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.port = socket_helper.bind_port(sock, '')
            asyncore.dispatcher.__init__(self, sock)
            self.listen(5)

        def handle_accepted(self, sock_obj, addr):
            if support.verbose:
                sys.stdout.write(' server:  new connection from %s:%s\n' % addr)
            self.ConnectionHandler(sock_obj, self.certfile)

        def handle_error(self):
            raise

    def __init__(self, certfile):
        self.flag = None
        self.active = False
        self.server = self.EchoServer(certfile)
        self.port = self.server.port
        threading.Thread.__init__(self)
        self.daemon = True

    def __str__(self):
        return '<%s %s>' % (self.__class__.__name__, self.server)

    def __enter__(self):
        self.start(threading.Event())
        self.flag.wait()
        return self

    def __exit__(self, *args):
        if support.verbose:
            sys.stdout.write(' cleanup: stopping server.\n')
        self.stop()
        if support.verbose:
            sys.stdout.write(' cleanup: joining server thread.\n')
        self.join()
        if support.verbose:
            sys.stdout.write(' cleanup: successfully joined.\n')
        asyncore.close_all(ignore_all=True)

    def start(self, flag=None):
        self.flag = flag
        threading.Thread.start(self)

    def run(self):
        self.active = True
        if self.flag:
            self.flag.set()
        while self.active:
            try:
                asyncore.loop(1)
            except:
                pass

    def stop(self):
        self.active = False
        self.server.close()

def server_params_test(client_context, server_context, indata=b'FOO\n', chatty=True, connectionchatty=False, sni_name=None, session=None):
    """
    Launch a server, connect a client to it and try various reads
    and writes.
    """
    stats = {}
    server = ThreadedEchoServer(context=server_context, chatty=chatty, connectionchatty=False)
    with server:
        with client_context.wrap_socket(socket.socket(), server_hostname=sni_name, session=session) as s:
            s.connect((HOST, server.port))
            for arg in [indata, bytearray(indata), memoryview(indata)]:
                if connectionchatty:
                    if support.verbose:
                        sys.stdout.write(' client:  sending %r...\n' % indata)
                s.write(arg)
                outdata = s.read()
                if connectionchatty:
                    if support.verbose:
                        sys.stdout.write(' client:  read %r\n' % outdata)
                if outdata != indata.lower():
                    raise AssertionError('bad data <<%r>> (%d) received; expected <<%r>> (%d)\n' % (outdata[:20], len(outdata), indata[:20].lower(), len(indata)))
            s.write(b'over\n')
            if connectionchatty:
                if support.verbose:
                    sys.stdout.write(' client:  closing connection.\n')
            stats.update({'compression': s.compression(), 'cipher': s.cipher(), 'peercert': s.getpeercert(), 'client_alpn_protocol': s.selected_alpn_protocol(), 'version': s.version(), 'session_reused': s.session_reused, 'session': s.session})
            s.close()
        stats['server_alpn_protocols'] = server.selected_alpn_protocols
        stats['server_shared_ciphers'] = server.shared_ciphers
    return stats

def try_protocol_combo(server_protocol, client_protocol, expect_success, certsreqs=None, server_options=0, client_options=0):
    """
    Try to SSL-connect using *client_protocol* to *server_protocol*.
    If *expect_success* is true, assert that the connection succeeds,
    if it's false, assert that the connection fails.
    Also, if *expect_success* is a string, assert that it is the protocol
    version actually used by the connection.
    """
    if certsreqs is None:
        certsreqs = ssl.CERT_NONE
    certtype = {ssl.CERT_NONE: 'CERT_NONE', ssl.CERT_OPTIONAL: 'CERT_OPTIONAL', ssl.CERT_REQUIRED: 'CERT_REQUIRED'}[certsreqs]
    if support.verbose:
        formatstr = expect_success and ' %s->%s %s\n' or ' {%s->%s} %s\n'
        sys.stdout.write(formatstr % (ssl.get_protocol_name(client_protocol), ssl.get_protocol_name(server_protocol), certtype))
    with warnings_helper.check_warnings():
        client_context = ssl.SSLContext(client_protocol)
        client_context.options |= client_options
        server_context = ssl.SSLContext(server_protocol)
        server_context.options |= server_options
    min_version = PROTOCOL_TO_TLS_VERSION.get(client_protocol, None)
    if min_version is not None and hasattr(server_context, 'minimum_version') and (server_protocol == ssl.PROTOCOL_TLS) and (server_context.minimum_version > min_version):
        with warnings_helper.check_warnings():
            server_context.minimum_version = min_version
    if client_context.protocol == ssl.PROTOCOL_TLS:
        client_context.set_ciphers('ALL')
    seclevel_workaround(server_context, client_context)
    for ctx in (client_context, server_context):
        ctx.verify_mode = certsreqs
        ctx.load_cert_chain(SIGNED_CERTFILE)
        ctx.load_verify_locations(SIGNING_CA)
    try:
        stats = server_params_test(client_context, server_context, chatty=False, connectionchatty=False)
    except ssl.SSLError:
        if expect_success:
            raise
    except OSError as e:
        if expect_success or e.errno != errno.ECONNRESET:
            raise
    else:
        if not expect_success:
            raise AssertionError('Client protocol %s succeeded with server protocol %s!' % (ssl.get_protocol_name(client_protocol), ssl.get_protocol_name(server_protocol)))
        elif expect_success is not True and expect_success != stats['version']:
            raise AssertionError('version mismatch: expected %r, got %r' % (expect_success, stats['version']))

def supports_kx_alias(ctx, aliases):
    for cipher in ctx.get_ciphers():
        for alias in aliases:
            if f'Kx={alias}' in cipher['description']:
                return True
    return False

HAS_KEYLOG = hasattr(ssl.SSLContext, 'keylog_filename')

requires_keylog = unittest.skipUnless(HAS_KEYLOG, 'test requires OpenSSL 1.1.1 with keylog callback')

def set_socket_so_linger_on_with_zero_timeout(sock):
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_LINGER, struct.pack('ii', 1, 0))

def setUpModule():
    if support.verbose:
        plats = {'Mac': platform.mac_ver, 'Windows': platform.win32_ver}
        for name, func in plats.items():
            plat = func()
            if plat and plat[0]:
                plat = '%s %r' % (name, plat)
                break
        else:
            plat = repr(platform.platform())
        print('test_ssl: testing with %r %r' % (ssl.OPENSSL_VERSION, ssl.OPENSSL_VERSION_INFO))
        print('          under %s' % plat)
        print('          HAS_SNI = %r' % ssl.HAS_SNI)
        print('          OP_ALL = 0x%8x' % ssl.OP_ALL)
        try:
            print('          OP_NO_TLSv1_1 = 0x%8x' % ssl.OP_NO_TLSv1_1)
        except AttributeError:
            pass
    for filename in [CERTFILE, BYTES_CERTFILE, ONLYCERT, ONLYKEY, BYTES_ONLYCERT, BYTES_ONLYKEY, SIGNED_CERTFILE, SIGNED_CERTFILE2, SIGNING_CA, BADCERT, BADKEY, EMPTYCERT]:
        if not os.path.exists(filename):
            raise support.TestFailed("Can't read certificate file %r" % filename)
    thread_info = threading_helper.threading_setup()
    unittest.addModuleCleanup(threading_helper.threading_cleanup, *thread_info)


# --- test body ---
ssl.CERT_NONE
ssl.CERT_OPTIONAL
ssl.CERT_REQUIRED
ssl.OP_CIPHER_SERVER_PREFERENCE
ssl.OP_SINGLE_DH_USE
ssl.OP_SINGLE_ECDH_USE
ssl.OP_NO_COMPRESSION

assert ssl.HAS_SNI == True

assert ssl.HAS_ECDH == True

assert ssl.HAS_TLSv1_2 == True

assert ssl.HAS_TLSv1_3 == True
ssl.OP_NO_SSLv2
ssl.OP_NO_SSLv3
ssl.OP_NO_TLSv1
ssl.OP_NO_TLSv1_3
ssl.OP_NO_TLSv1_1
ssl.OP_NO_TLSv1_2

assert ssl.PROTOCOL_TLS == ssl.PROTOCOL_SSLv23
print("BasicSocketTests::test_constants: ok")
