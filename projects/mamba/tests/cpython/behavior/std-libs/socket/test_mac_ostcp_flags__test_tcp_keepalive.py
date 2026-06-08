# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "test_mac_ostcp_flags__test_tcp_keepalive"
# subject = "cpython.test_socket.TestMacOSTCPFlags.test_tcp_keepalive"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_socket.py::TestMacOSTCPFlags::test_tcp_keepalive
"""Auto-ported test: TestMacOSTCPFlags::test_tcp_keepalive (CPython 3.12 oracle)."""


import unittest
from test import support
from test.support import os_helper
from test.support import socket_helper
from test.support import threading_helper
import _thread as thread
import array
import contextlib
import errno
import gc
import io
import itertools
import math
import os
import pickle
import platform
import queue
import random
import re
import select
import signal
import socket
import string
import struct
import sys
import tempfile
import threading
import time
import traceback
from weakref import proxy


try:
    import multiprocessing
except ImportError:
    multiprocessing = False

try:
    import fcntl
except ImportError:
    fcntl = None

support.requires_working_socket(module=True)

HOST = socket_helper.HOST

MSG = 'Michael Gilfix was hereሴ\r\n'.encode('utf-8')

VMADDR_CID_LOCAL = 1

VSOCKPORT = 1234

AIX = platform.system() == 'AIX'

WSL = 'microsoft-standard-WSL' in platform.release()

try:
    import _socket
except ImportError:
    _socket = None

def get_cid():
    if fcntl is None:
        return None
    if not hasattr(socket, 'IOCTL_VM_SOCKETS_GET_LOCAL_CID'):
        return None
    try:
        with open('/dev/vsock', 'rb') as f:
            r = fcntl.ioctl(f, socket.IOCTL_VM_SOCKETS_GET_LOCAL_CID, '    ')
    except OSError:
        return None
    else:
        return struct.unpack('I', r)[0]

def _have_socket_can():
    """Check whether CAN sockets are supported on this host."""
    try:
        s = socket.socket(socket.PF_CAN, socket.SOCK_RAW, socket.CAN_RAW)
    except (AttributeError, OSError):
        return False
    else:
        s.close()
    return True

def _have_socket_can_isotp():
    """Check whether CAN ISOTP sockets are supported on this host."""
    try:
        s = socket.socket(socket.PF_CAN, socket.SOCK_DGRAM, socket.CAN_ISOTP)
    except (AttributeError, OSError):
        return False
    else:
        s.close()
    return True

def _have_socket_can_j1939():
    """Check whether CAN J1939 sockets are supported on this host."""
    try:
        s = socket.socket(socket.PF_CAN, socket.SOCK_DGRAM, socket.CAN_J1939)
    except (AttributeError, OSError):
        return False
    else:
        s.close()
    return True

def _have_socket_rds():
    """Check whether RDS sockets are supported on this host."""
    try:
        s = socket.socket(socket.PF_RDS, socket.SOCK_SEQPACKET, 0)
    except (AttributeError, OSError):
        return False
    else:
        s.close()
    return True

def _have_socket_alg():
    """Check whether AF_ALG sockets are supported on this host."""
    try:
        s = socket.socket(socket.AF_ALG, socket.SOCK_SEQPACKET, 0)
    except (AttributeError, OSError):
        return False
    else:
        s.close()
    return True

def _have_socket_qipcrtr():
    """Check whether AF_QIPCRTR sockets are supported on this host."""
    try:
        s = socket.socket(socket.AF_QIPCRTR, socket.SOCK_DGRAM, 0)
    except (AttributeError, OSError):
        return False
    else:
        s.close()
    return True

def _have_socket_vsock():
    """Check whether AF_VSOCK sockets are supported on this host."""
    cid = get_cid()
    return cid is not None

def _have_socket_bluetooth():
    """Check whether AF_BLUETOOTH sockets are supported on this host."""
    try:
        s = socket.socket(socket.AF_BLUETOOTH, socket.SOCK_STREAM, socket.BTPROTO_RFCOMM)
    except (AttributeError, OSError):
        return False
    else:
        s.close()
    return True

def _have_socket_hyperv():
    """Check whether AF_HYPERV sockets are supported on this host."""
    try:
        s = socket.socket(socket.AF_HYPERV, socket.SOCK_STREAM, socket.HV_PROTOCOL_RAW)
    except (AttributeError, OSError):
        return False
    else:
        s.close()
    return True

@contextlib.contextmanager
def socket_setdefaulttimeout(timeout):
    old_timeout = socket.getdefaulttimeout()
    try:
        socket.setdefaulttimeout(timeout)
        yield
    finally:
        socket.setdefaulttimeout(old_timeout)

HAVE_SOCKET_CAN = _have_socket_can()

HAVE_SOCKET_CAN_ISOTP = _have_socket_can_isotp()

HAVE_SOCKET_CAN_J1939 = _have_socket_can_j1939()

HAVE_SOCKET_RDS = _have_socket_rds()

HAVE_SOCKET_ALG = _have_socket_alg()

HAVE_SOCKET_QIPCRTR = _have_socket_qipcrtr()

HAVE_SOCKET_VSOCK = _have_socket_vsock()

HAVE_SOCKET_UDPLITE = hasattr(socket, 'IPPROTO_UDPLITE')

HAVE_SOCKET_BLUETOOTH = _have_socket_bluetooth()

HAVE_SOCKET_HYPERV = _have_socket_hyperv()

SIZEOF_INT = array.array('i').itemsize

class ThreadableTest:
    """Threadable Test class

    The ThreadableTest class makes it easy to create a threaded
    client/server pair from an existing unit test. To create a
    new threaded class from an existing unit test, use multiple
    inheritance:

        class NewClass (OldClass, ThreadableTest):
            pass

    This class defines two new fixture functions with obvious
    purposes for overriding:

        clientSetUp ()
        clientTearDown ()

    Any new test functions within the class must then define
    tests in pairs, where the test name is preceded with a
    '_' to indicate the client portion of the test. Ex:

        def testFoo(self):
            # Server portion

        def _testFoo(self):
            # Client portion

    Any exceptions raised by the clients during their tests
    are caught and transferred to the main thread to alert
    the testing framework.

    Note, the server setup function cannot call any blocking
    functions that rely on the client thread during setup,
    unless serverExplicitReady() is called just before
    the blocking call (such as in setting up a client/server
    connection and performing the accept() in setUp().
    """

    def __init__(self):
        self.__setUp = self.setUp
        self.setUp = self._setUp

    def serverExplicitReady(self):
        """This method allows the server to explicitly indicate that
        it wants the client thread to proceed. This is useful if the
        server is about to execute a blocking routine that is
        dependent upon the client thread during its setup routine."""
        self.server_ready.set()

    def _setUp(self):
        self.enterContext(threading_helper.wait_threads_exit())
        self.server_ready = threading.Event()
        self.client_ready = threading.Event()
        self.done = threading.Event()
        self.queue = queue.Queue(1)
        self.server_crashed = False

        def raise_queued_exception():
            if self.queue.qsize():
                raise self.queue.get()
        self.addCleanup(raise_queued_exception)
        methodname = self.id()
        i = methodname.rfind('.')
        methodname = methodname[i + 1:]
        test_method = getattr(self, '_' + methodname)
        self.client_thread = thread.start_new_thread(self.clientRun, (test_method,))
        try:
            self.__setUp()
        except:
            self.server_crashed = True
            raise
        finally:
            self.server_ready.set()
        self.client_ready.wait()
        self.addCleanup(self.done.wait)

    def clientRun(self, test_func):
        self.server_ready.wait()
        try:
            self.clientSetUp()
        except BaseException as e:
            self.queue.put(e)
            self.clientTearDown()
            return
        finally:
            self.client_ready.set()
        if self.server_crashed:
            self.clientTearDown()
            return
        if not hasattr(test_func, '__call__'):
            raise TypeError('test_func must be a callable function')
        try:
            test_func()
        except BaseException as e:
            self.queue.put(e)
        finally:
            self.clientTearDown()

    def clientSetUp(self):
        raise NotImplementedError('clientSetUp must be implemented.')

    def clientTearDown(self):
        self.done.set()
        thread.exit()

def skipWithClientIf(condition, reason):
    """Skip decorated test if condition is true, add client_skip decorator.

    If the decorated object is not a class, sets its attribute
    "client_skip" to a decorator which will return an empty function
    if the test is to be skipped, or the original function if it is
    not.  This can be used to avoid running the client part of a
    skipped test when using ThreadableTest.
    """

    def client_pass(*args, **kwargs):
        pass

    def skipdec(obj):
        retval = unittest.skip(reason)(obj)
        if not isinstance(obj, type):
            retval.client_skip = lambda f: client_pass
        return retval

    def noskipdec(obj):
        if not (isinstance(obj, type) or hasattr(obj, 'client_skip')):
            obj.client_skip = lambda f: f
        return obj
    return skipdec if condition else noskipdec

def requireAttrs(obj, *attributes):
    """Skip decorated test if obj is missing any of the given attributes.

    Sets client_skip attribute as skipWithClientIf() does.
    """
    missing = [name for name in attributes if not hasattr(obj, name)]
    return skipWithClientIf(missing, "don't have " + ', '.join((name for name in missing)))

def requireSocket(*args):
    """Skip decorated test if a socket cannot be created with given arguments.

    When an argument is given as a string, will use the value of that
    attribute of the socket module, or skip the test if it doesn't
    exist.  Sets client_skip attribute as skipWithClientIf() does.
    """
    err = None
    missing = [obj for obj in args if isinstance(obj, str) and (not hasattr(socket, obj))]
    if missing:
        err = "don't have " + ', '.join((name for name in missing))
    else:
        callargs = [getattr(socket, obj) if isinstance(obj, str) else obj for obj in args]
        try:
            s = socket.socket(*callargs)
        except OSError as e:
            err = str(e)
        else:
            s.close()
    return skipWithClientIf(err is not None, "can't create socket({0}): {1}".format(', '.join((str(o) for o in args)), err))

class SendrecvmsgBase:
    fail_timeout = support.LOOPBACK_TIMEOUT

    def setUp(self):
        self.misc_event = threading.Event()
        super().setUp()

    def sendToServer(self, msg):
        return self.cli_sock.send(msg)
    sendmsg_to_server_defaults = ()

    def sendmsgToServer(self, *args):
        return self.cli_sock.sendmsg(*args + self.sendmsg_to_server_defaults[len(args):])

    def doRecvmsg(self, sock, bufsize, *args):
        result = sock.recvmsg(bufsize, *args)
        self.registerRecvmsgResult(result)
        return result

    def registerRecvmsgResult(self, result):
        pass

    def checkRecvmsgAddress(self, addr1, addr2):
        self.assertEqual(addr1, addr2)
    msg_flags_common_unset = 0
    for name in ('MSG_CTRUNC', 'MSG_OOB'):
        msg_flags_common_unset |= getattr(socket, name, 0)
    msg_flags_common_set = 0
    msg_flags_eor_indicator = 0
    msg_flags_non_eor_indicator = 0

    def checkFlags(self, flags, eor=None, checkset=0, checkunset=0, ignore=0):
        defaultset = self.msg_flags_common_set
        defaultunset = self.msg_flags_common_unset
        if eor:
            defaultset |= self.msg_flags_eor_indicator
            defaultunset |= self.msg_flags_non_eor_indicator
        elif eor is not None:
            defaultset |= self.msg_flags_non_eor_indicator
            defaultunset |= self.msg_flags_eor_indicator
        defaultset &= ~checkunset
        defaultunset &= ~checkset
        checkset |= defaultset
        checkunset |= defaultunset
        inboth = checkset & checkunset & ~ignore
        if inboth:
            raise Exception('contradictory set, unset requirements for flags {0:#x}'.format(inboth))
        mask = (checkset | checkunset) & ~ignore
        self.assertEqual(flags & mask, checkset & mask)

class RecvmsgIntoMixin(SendrecvmsgBase):

    def doRecvmsg(self, sock, bufsize, *args):
        buf = bytearray(bufsize)
        result = sock.recvmsg_into([buf], *args)
        self.registerRecvmsgResult(result)
        self.assertGreaterEqual(result[0], 0)
        self.assertLessEqual(result[0], bufsize)
        return (bytes(buf[:result[0]]),) + result[1:]

class SendrecvmsgDgramFlagsBase(SendrecvmsgBase):

    @property
    def msg_flags_non_eor_indicator(self):
        return super().msg_flags_non_eor_indicator | socket.MSG_TRUNC

class SendrecvmsgSCTPFlagsBase(SendrecvmsgBase):

    @property
    def msg_flags_eor_indicator(self):
        return super().msg_flags_eor_indicator | socket.MSG_EOR

class SendrecvmsgConnectionlessBase(SendrecvmsgBase):

    @property
    def serv_sock(self):
        return self.serv

    @property
    def cli_sock(self):
        return self.cli

    @property
    def sendmsg_to_server_defaults(self):
        return ([], [], 0, self.serv_addr)

    def sendToServer(self, msg):
        return self.cli_sock.sendto(msg, self.serv_addr)

class SendrecvmsgConnectedBase(SendrecvmsgBase):

    @property
    def serv_sock(self):
        return self.cli_conn

    @property
    def cli_sock(self):
        return self.serv_conn

    def checkRecvmsgAddress(self, addr1, addr2):
        pass

class SendrecvmsgServerTimeoutBase(SendrecvmsgBase):

    def setUp(self):
        super().setUp()
        self.serv_sock.settimeout(self.fail_timeout)

class SendmsgTests(SendrecvmsgServerTimeoutBase):

    def testSendmsg(self):
        self.assertEqual(self.serv_sock.recv(len(MSG)), MSG)

    def _testSendmsg(self):
        self.assertEqual(self.sendmsgToServer([MSG]), len(MSG))

    def testSendmsgDataGenerator(self):
        self.assertEqual(self.serv_sock.recv(len(MSG)), MSG)

    def _testSendmsgDataGenerator(self):
        self.assertEqual(self.sendmsgToServer((o for o in [MSG])), len(MSG))

    def testSendmsgAncillaryGenerator(self):
        self.assertEqual(self.serv_sock.recv(len(MSG)), MSG)

    def _testSendmsgAncillaryGenerator(self):
        self.assertEqual(self.sendmsgToServer([MSG], (o for o in [])), len(MSG))

    def testSendmsgArray(self):
        self.assertEqual(self.serv_sock.recv(len(MSG)), MSG)

    def _testSendmsgArray(self):
        self.assertEqual(self.sendmsgToServer([array.array('B', MSG)]), len(MSG))

    def testSendmsgGather(self):
        self.assertEqual(self.serv_sock.recv(len(MSG)), MSG)

    def _testSendmsgGather(self):
        self.assertEqual(self.sendmsgToServer([MSG[:3], MSG[3:]]), len(MSG))

    def testSendmsgBadArgs(self):
        self.assertEqual(self.serv_sock.recv(1000), b'done')

    def _testSendmsgBadArgs(self):
        self.assertRaises(TypeError, self.cli_sock.sendmsg)
        self.assertRaises(TypeError, self.sendmsgToServer, b'not in an iterable')
        self.assertRaises(TypeError, self.sendmsgToServer, object())
        self.assertRaises(TypeError, self.sendmsgToServer, [object()])
        self.assertRaises(TypeError, self.sendmsgToServer, [MSG, object()])
        self.assertRaises(TypeError, self.sendmsgToServer, [MSG], object())
        self.assertRaises(TypeError, self.sendmsgToServer, [MSG], [], object())
        self.assertRaises(TypeError, self.sendmsgToServer, [MSG], [], 0, object())
        self.sendToServer(b'done')

    def testSendmsgBadCmsg(self):
        self.assertEqual(self.serv_sock.recv(1000), b'done')

    def _testSendmsgBadCmsg(self):
        self.assertRaises(TypeError, self.sendmsgToServer, [MSG], [object()])
        self.assertRaises(TypeError, self.sendmsgToServer, [MSG], [(object(), 0, b'data')])
        self.assertRaises(TypeError, self.sendmsgToServer, [MSG], [(0, object(), b'data')])
        self.assertRaises(TypeError, self.sendmsgToServer, [MSG], [(0, 0, object())])
        self.assertRaises(TypeError, self.sendmsgToServer, [MSG], [(0, 0)])
        self.assertRaises(TypeError, self.sendmsgToServer, [MSG], [(0, 0, b'data', 42)])
        self.sendToServer(b'done')

    @requireAttrs(socket, 'CMSG_SPACE')
    def testSendmsgBadMultiCmsg(self):
        self.assertEqual(self.serv_sock.recv(1000), b'done')

    @testSendmsgBadMultiCmsg.client_skip
    def _testSendmsgBadMultiCmsg(self):
        self.assertRaises(TypeError, self.sendmsgToServer, [MSG], [0, 0, b''])
        self.assertRaises(TypeError, self.sendmsgToServer, [MSG], [(0, 0, b''), object()])
        self.sendToServer(b'done')

    def testSendmsgExcessCmsgReject(self):
        self.assertEqual(self.serv_sock.recv(1000), b'done')

    def _testSendmsgExcessCmsgReject(self):
        if not hasattr(socket, 'CMSG_SPACE'):
            with self.assertRaises(OSError) as cm:
                self.sendmsgToServer([MSG], [(0, 0, b''), (0, 0, b'')])
            self.assertIsNone(cm.exception.errno)
        self.sendToServer(b'done')

    def testSendmsgAfterClose(self):
        pass

    def _testSendmsgAfterClose(self):
        self.cli_sock.close()
        self.assertRaises(OSError, self.sendmsgToServer, [MSG])

class SendmsgStreamTests(SendmsgTests):

    def testSendmsgExplicitNoneAddr(self):
        self.assertEqual(self.serv_sock.recv(len(MSG)), MSG)

    def _testSendmsgExplicitNoneAddr(self):
        self.assertEqual(self.sendmsgToServer([MSG], [], 0, None), len(MSG))

    def testSendmsgTimeout(self):
        self.assertEqual(self.serv_sock.recv(512), b'a' * 512)
        self.assertTrue(self.misc_event.wait(timeout=self.fail_timeout))

    def _testSendmsgTimeout(self):
        try:
            self.cli_sock.settimeout(0.03)
            try:
                while True:
                    self.sendmsgToServer([b'a' * 512])
            except TimeoutError:
                pass
            except OSError as exc:
                if exc.errno != errno.ENOMEM:
                    raise
            else:
                self.fail('TimeoutError not raised')
        finally:
            self.misc_event.set()

    @skipWithClientIf(sys.platform not in {'linux'}, 'MSG_DONTWAIT not known to work on this platform when sending')
    def testSendmsgDontWait(self):
        self.assertEqual(self.serv_sock.recv(512), b'a' * 512)
        self.assertTrue(self.misc_event.wait(timeout=self.fail_timeout))

    @testSendmsgDontWait.client_skip
    def _testSendmsgDontWait(self):
        try:
            with self.assertRaises(OSError) as cm:
                while True:
                    self.sendmsgToServer([b'a' * 512], [], socket.MSG_DONTWAIT)
            self.assertIn(cm.exception.errno, (errno.EAGAIN, errno.EWOULDBLOCK, errno.ENOMEM))
        finally:
            self.misc_event.set()

class SendmsgConnectionlessTests(SendmsgTests):

    def testSendmsgNoDestAddr(self):
        pass

    def _testSendmsgNoDestAddr(self):
        self.assertRaises(OSError, self.cli_sock.sendmsg, [MSG])
        self.assertRaises(OSError, self.cli_sock.sendmsg, [MSG], [], 0, None)

class RecvmsgGenericTests(SendrecvmsgBase):

    def testRecvmsg(self):
        msg, ancdata, flags, addr = self.doRecvmsg(self.serv_sock, len(MSG))
        self.assertEqual(msg, MSG)
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.assertEqual(ancdata, [])
        self.checkFlags(flags, eor=True)

    def _testRecvmsg(self):
        self.sendToServer(MSG)

    def testRecvmsgExplicitDefaults(self):
        msg, ancdata, flags, addr = self.doRecvmsg(self.serv_sock, len(MSG), 0, 0)
        self.assertEqual(msg, MSG)
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.assertEqual(ancdata, [])
        self.checkFlags(flags, eor=True)

    def _testRecvmsgExplicitDefaults(self):
        self.sendToServer(MSG)

    def testRecvmsgShorter(self):
        msg, ancdata, flags, addr = self.doRecvmsg(self.serv_sock, len(MSG) + 42)
        self.assertEqual(msg, MSG)
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.assertEqual(ancdata, [])
        self.checkFlags(flags, eor=True)

    def _testRecvmsgShorter(self):
        self.sendToServer(MSG)

    def testRecvmsgTrunc(self):
        msg, ancdata, flags, addr = self.doRecvmsg(self.serv_sock, len(MSG) - 3)
        self.assertEqual(msg, MSG[:-3])
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.assertEqual(ancdata, [])
        self.checkFlags(flags, eor=False)

    def _testRecvmsgTrunc(self):
        self.sendToServer(MSG)

    def testRecvmsgShortAncillaryBuf(self):
        msg, ancdata, flags, addr = self.doRecvmsg(self.serv_sock, len(MSG), 1)
        self.assertEqual(msg, MSG)
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.assertEqual(ancdata, [])
        self.checkFlags(flags, eor=True)

    def _testRecvmsgShortAncillaryBuf(self):
        self.sendToServer(MSG)

    def testRecvmsgLongAncillaryBuf(self):
        msg, ancdata, flags, addr = self.doRecvmsg(self.serv_sock, len(MSG), 10240)
        self.assertEqual(msg, MSG)
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.assertEqual(ancdata, [])
        self.checkFlags(flags, eor=True)

    def _testRecvmsgLongAncillaryBuf(self):
        self.sendToServer(MSG)

    def testRecvmsgAfterClose(self):
        self.serv_sock.close()
        self.assertRaises(OSError, self.doRecvmsg, self.serv_sock, 1024)

    def _testRecvmsgAfterClose(self):
        pass

    def testRecvmsgTimeout(self):
        try:
            self.serv_sock.settimeout(0.03)
            self.assertRaises(TimeoutError, self.doRecvmsg, self.serv_sock, len(MSG))
        finally:
            self.misc_event.set()

    def _testRecvmsgTimeout(self):
        self.assertTrue(self.misc_event.wait(timeout=self.fail_timeout))

    @requireAttrs(socket, 'MSG_PEEK')
    def testRecvmsgPeek(self):
        msg, ancdata, flags, addr = self.doRecvmsg(self.serv_sock, len(MSG) - 3, 0, socket.MSG_PEEK)
        self.assertEqual(msg, MSG[:-3])
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.assertEqual(ancdata, [])
        self.checkFlags(flags, eor=False, ignore=getattr(socket, 'MSG_TRUNC', 0))
        msg, ancdata, flags, addr = self.doRecvmsg(self.serv_sock, len(MSG), 0, socket.MSG_PEEK)
        self.assertEqual(msg, MSG)
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.assertEqual(ancdata, [])
        self.checkFlags(flags, eor=True)
        msg, ancdata, flags, addr = self.doRecvmsg(self.serv_sock, len(MSG))
        self.assertEqual(msg, MSG)
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.assertEqual(ancdata, [])
        self.checkFlags(flags, eor=True)

    @testRecvmsgPeek.client_skip
    def _testRecvmsgPeek(self):
        self.sendToServer(MSG)

    @requireAttrs(socket.socket, 'sendmsg')
    def testRecvmsgFromSendmsg(self):
        self.serv_sock.settimeout(self.fail_timeout)
        msg, ancdata, flags, addr = self.doRecvmsg(self.serv_sock, len(MSG))
        self.assertEqual(msg, MSG)
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.assertEqual(ancdata, [])
        self.checkFlags(flags, eor=True)

    @testRecvmsgFromSendmsg.client_skip
    def _testRecvmsgFromSendmsg(self):
        self.assertEqual(self.sendmsgToServer([MSG[:3], MSG[3:]]), len(MSG))

class RecvmsgGenericStreamTests(RecvmsgGenericTests):

    def testRecvmsgEOF(self):
        msg, ancdata, flags, addr = self.doRecvmsg(self.serv_sock, 1024)
        self.assertEqual(msg, b'')
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.assertEqual(ancdata, [])
        self.checkFlags(flags, eor=None)

    def _testRecvmsgEOF(self):
        self.cli_sock.close()

    def testRecvmsgOverflow(self):
        seg1, ancdata, flags, addr = self.doRecvmsg(self.serv_sock, len(MSG) - 3)
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.assertEqual(ancdata, [])
        self.checkFlags(flags, eor=False)
        seg2, ancdata, flags, addr = self.doRecvmsg(self.serv_sock, 1024)
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.assertEqual(ancdata, [])
        self.checkFlags(flags, eor=True)
        msg = seg1 + seg2
        self.assertEqual(msg, MSG)

    def _testRecvmsgOverflow(self):
        self.sendToServer(MSG)

class RecvmsgTests(RecvmsgGenericTests):

    def testRecvmsgBadArgs(self):
        self.assertRaises(TypeError, self.serv_sock.recvmsg)
        self.assertRaises(ValueError, self.serv_sock.recvmsg, -1, 0, 0)
        self.assertRaises(ValueError, self.serv_sock.recvmsg, len(MSG), -1, 0)
        self.assertRaises(TypeError, self.serv_sock.recvmsg, [bytearray(10)], 0, 0)
        self.assertRaises(TypeError, self.serv_sock.recvmsg, object(), 0, 0)
        self.assertRaises(TypeError, self.serv_sock.recvmsg, len(MSG), object(), 0)
        self.assertRaises(TypeError, self.serv_sock.recvmsg, len(MSG), 0, object())
        msg, ancdata, flags, addr = self.serv_sock.recvmsg(len(MSG), 0, 0)
        self.assertEqual(msg, MSG)
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.assertEqual(ancdata, [])
        self.checkFlags(flags, eor=True)

    def _testRecvmsgBadArgs(self):
        self.sendToServer(MSG)

class RecvmsgIntoTests(RecvmsgIntoMixin, RecvmsgGenericTests):

    def testRecvmsgIntoBadArgs(self):
        buf = bytearray(len(MSG))
        self.assertRaises(TypeError, self.serv_sock.recvmsg_into)
        self.assertRaises(TypeError, self.serv_sock.recvmsg_into, len(MSG), 0, 0)
        self.assertRaises(TypeError, self.serv_sock.recvmsg_into, buf, 0, 0)
        self.assertRaises(TypeError, self.serv_sock.recvmsg_into, [object()], 0, 0)
        self.assertRaises(TypeError, self.serv_sock.recvmsg_into, [b"I'm not writable"], 0, 0)
        self.assertRaises(TypeError, self.serv_sock.recvmsg_into, [buf, object()], 0, 0)
        self.assertRaises(ValueError, self.serv_sock.recvmsg_into, [buf], -1, 0)
        self.assertRaises(TypeError, self.serv_sock.recvmsg_into, [buf], object(), 0)
        self.assertRaises(TypeError, self.serv_sock.recvmsg_into, [buf], 0, object())
        nbytes, ancdata, flags, addr = self.serv_sock.recvmsg_into([buf], 0, 0)
        self.assertEqual(nbytes, len(MSG))
        self.assertEqual(buf, bytearray(MSG))
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.assertEqual(ancdata, [])
        self.checkFlags(flags, eor=True)

    def _testRecvmsgIntoBadArgs(self):
        self.sendToServer(MSG)

    def testRecvmsgIntoGenerator(self):
        buf = bytearray(len(MSG))
        nbytes, ancdata, flags, addr = self.serv_sock.recvmsg_into((o for o in [buf]))
        self.assertEqual(nbytes, len(MSG))
        self.assertEqual(buf, bytearray(MSG))
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.assertEqual(ancdata, [])
        self.checkFlags(flags, eor=True)

    def _testRecvmsgIntoGenerator(self):
        self.sendToServer(MSG)

    def testRecvmsgIntoArray(self):
        buf = array.array('B', [0] * len(MSG))
        nbytes, ancdata, flags, addr = self.serv_sock.recvmsg_into([buf])
        self.assertEqual(nbytes, len(MSG))
        self.assertEqual(buf.tobytes(), MSG)
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.assertEqual(ancdata, [])
        self.checkFlags(flags, eor=True)

    def _testRecvmsgIntoArray(self):
        self.sendToServer(MSG)

    def testRecvmsgIntoScatter(self):
        b1 = bytearray(b'----')
        b2 = bytearray(b'0123456789')
        b3 = bytearray(b'--------------')
        nbytes, ancdata, flags, addr = self.serv_sock.recvmsg_into([b1, memoryview(b2)[2:9], b3])
        self.assertEqual(nbytes, len(b'Mary had a little lamb'))
        self.assertEqual(b1, bytearray(b'Mary'))
        self.assertEqual(b2, bytearray(b'01 had a 9'))
        self.assertEqual(b3, bytearray(b'little lamb---'))
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.assertEqual(ancdata, [])
        self.checkFlags(flags, eor=True)

    def _testRecvmsgIntoScatter(self):
        self.sendToServer(b'Mary had a little lamb')

class SCMRightsTest(SendrecvmsgServerTimeoutBase):
    badfd = -21845

    def newFDs(self, n):
        fds = []
        for i in range(n):
            fd, path = tempfile.mkstemp()
            self.addCleanup(os.unlink, path)
            self.addCleanup(os.close, fd)
            os.write(fd, str(i).encode())
            fds.append(fd)
        return fds

    def checkFDs(self, fds):
        for n, fd in enumerate(fds):
            os.lseek(fd, 0, os.SEEK_SET)
            self.assertEqual(os.read(fd, 1024), str(n).encode())

    def registerRecvmsgResult(self, result):
        self.addCleanup(self.closeRecvmsgFDs, result)

    def closeRecvmsgFDs(self, recvmsg_result):
        for cmsg_level, cmsg_type, cmsg_data in recvmsg_result[1]:
            if cmsg_level == socket.SOL_SOCKET and cmsg_type == socket.SCM_RIGHTS:
                fds = array.array('i')
                fds.frombytes(cmsg_data[:len(cmsg_data) - len(cmsg_data) % fds.itemsize])
                for fd in fds:
                    os.close(fd)

    def createAndSendFDs(self, n):
        self.assertEqual(self.sendmsgToServer([MSG], [(socket.SOL_SOCKET, socket.SCM_RIGHTS, array.array('i', self.newFDs(n)))]), len(MSG))

    def checkRecvmsgFDs(self, numfds, result, maxcmsgs=1, ignoreflags=0):
        msg, ancdata, flags, addr = result
        self.assertEqual(msg, MSG)
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.checkFlags(flags, eor=True, checkunset=socket.MSG_CTRUNC, ignore=ignoreflags)
        self.assertIsInstance(ancdata, list)
        self.assertLessEqual(len(ancdata), maxcmsgs)
        fds = array.array('i')
        for item in ancdata:
            self.assertIsInstance(item, tuple)
            cmsg_level, cmsg_type, cmsg_data = item
            self.assertEqual(cmsg_level, socket.SOL_SOCKET)
            self.assertEqual(cmsg_type, socket.SCM_RIGHTS)
            self.assertIsInstance(cmsg_data, bytes)
            self.assertEqual(len(cmsg_data) % SIZEOF_INT, 0)
            fds.frombytes(cmsg_data)
        self.assertEqual(len(fds), numfds)
        self.checkFDs(fds)

    def testFDPassSimple(self):
        self.checkRecvmsgFDs(1, self.doRecvmsg(self.serv_sock, len(MSG), 10240))

    def _testFDPassSimple(self):
        self.assertEqual(self.sendmsgToServer([MSG], [(socket.SOL_SOCKET, socket.SCM_RIGHTS, array.array('i', self.newFDs(1)).tobytes())]), len(MSG))

    def testMultipleFDPass(self):
        self.checkRecvmsgFDs(4, self.doRecvmsg(self.serv_sock, len(MSG), 10240))

    def _testMultipleFDPass(self):
        self.createAndSendFDs(4)

    @requireAttrs(socket, 'CMSG_SPACE')
    def testFDPassCMSG_SPACE(self):
        self.checkRecvmsgFDs(4, self.doRecvmsg(self.serv_sock, len(MSG), socket.CMSG_SPACE(4 * SIZEOF_INT)))

    @testFDPassCMSG_SPACE.client_skip
    def _testFDPassCMSG_SPACE(self):
        self.createAndSendFDs(4)

    def testFDPassCMSG_LEN(self):
        self.checkRecvmsgFDs(1, self.doRecvmsg(self.serv_sock, len(MSG), socket.CMSG_LEN(4 * SIZEOF_INT)), ignoreflags=socket.MSG_CTRUNC)

    def _testFDPassCMSG_LEN(self):
        self.createAndSendFDs(1)

    @unittest.skipIf(sys.platform == 'darwin', 'skipping, see issue #12958')
    @unittest.skipIf(AIX, 'skipping, see issue #22397')
    @requireAttrs(socket, 'CMSG_SPACE')
    def testFDPassSeparate(self):
        self.checkRecvmsgFDs(2, self.doRecvmsg(self.serv_sock, len(MSG), 10240), maxcmsgs=2)

    @testFDPassSeparate.client_skip
    @unittest.skipIf(sys.platform == 'darwin', 'skipping, see issue #12958')
    @unittest.skipIf(AIX, 'skipping, see issue #22397')
    def _testFDPassSeparate(self):
        fd0, fd1 = self.newFDs(2)
        self.assertEqual(self.sendmsgToServer([MSG], [(socket.SOL_SOCKET, socket.SCM_RIGHTS, array.array('i', [fd0])), (socket.SOL_SOCKET, socket.SCM_RIGHTS, array.array('i', [fd1]))]), len(MSG))

    @unittest.skipIf(sys.platform == 'darwin', 'skipping, see issue #12958')
    @unittest.skipIf(AIX, 'skipping, see issue #22397')
    @requireAttrs(socket, 'CMSG_SPACE')
    def testFDPassSeparateMinSpace(self):
        num_fds = 2
        self.checkRecvmsgFDs(num_fds, self.doRecvmsg(self.serv_sock, len(MSG), socket.CMSG_SPACE(SIZEOF_INT) + socket.CMSG_LEN(SIZEOF_INT * num_fds)), maxcmsgs=2, ignoreflags=socket.MSG_CTRUNC)

    @testFDPassSeparateMinSpace.client_skip
    @unittest.skipIf(sys.platform == 'darwin', 'skipping, see issue #12958')
    @unittest.skipIf(AIX, 'skipping, see issue #22397')
    def _testFDPassSeparateMinSpace(self):
        fd0, fd1 = self.newFDs(2)
        self.assertEqual(self.sendmsgToServer([MSG], [(socket.SOL_SOCKET, socket.SCM_RIGHTS, array.array('i', [fd0])), (socket.SOL_SOCKET, socket.SCM_RIGHTS, array.array('i', [fd1]))]), len(MSG))

    def sendAncillaryIfPossible(self, msg, ancdata):
        try:
            nbytes = self.sendmsgToServer([msg], ancdata)
        except OSError as e:
            self.assertIsInstance(e.errno, int)
            nbytes = self.sendmsgToServer([msg])
        self.assertEqual(nbytes, len(msg))

    @unittest.skipIf(sys.platform == 'darwin', 'see issue #24725')
    def testFDPassEmpty(self):
        self.checkRecvmsgFDs(0, self.doRecvmsg(self.serv_sock, len(MSG), 10240), ignoreflags=socket.MSG_CTRUNC)

    def _testFDPassEmpty(self):
        self.sendAncillaryIfPossible(MSG, [(socket.SOL_SOCKET, socket.SCM_RIGHTS, b'')])

    def testFDPassPartialInt(self):
        msg, ancdata, flags, addr = self.doRecvmsg(self.serv_sock, len(MSG), 10240)
        self.assertEqual(msg, MSG)
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.checkFlags(flags, eor=True, ignore=socket.MSG_CTRUNC)
        self.assertLessEqual(len(ancdata), 1)
        for cmsg_level, cmsg_type, cmsg_data in ancdata:
            self.assertEqual(cmsg_level, socket.SOL_SOCKET)
            self.assertEqual(cmsg_type, socket.SCM_RIGHTS)
            self.assertLess(len(cmsg_data), SIZEOF_INT)

    def _testFDPassPartialInt(self):
        self.sendAncillaryIfPossible(MSG, [(socket.SOL_SOCKET, socket.SCM_RIGHTS, array.array('i', [self.badfd]).tobytes()[:-1])])

    @requireAttrs(socket, 'CMSG_SPACE')
    def testFDPassPartialIntInMiddle(self):
        msg, ancdata, flags, addr = self.doRecvmsg(self.serv_sock, len(MSG), 10240)
        self.assertEqual(msg, MSG)
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.checkFlags(flags, eor=True, ignore=socket.MSG_CTRUNC)
        self.assertLessEqual(len(ancdata), 2)
        fds = array.array('i')
        for cmsg_level, cmsg_type, cmsg_data in ancdata:
            self.assertEqual(cmsg_level, socket.SOL_SOCKET)
            self.assertEqual(cmsg_type, socket.SCM_RIGHTS)
            fds.frombytes(cmsg_data[:len(cmsg_data) - len(cmsg_data) % fds.itemsize])
        self.assertLessEqual(len(fds), 2)
        self.checkFDs(fds)

    @testFDPassPartialIntInMiddle.client_skip
    def _testFDPassPartialIntInMiddle(self):
        fd0, fd1 = self.newFDs(2)
        self.sendAncillaryIfPossible(MSG, [(socket.SOL_SOCKET, socket.SCM_RIGHTS, array.array('i', [fd0, self.badfd]).tobytes()[:-1]), (socket.SOL_SOCKET, socket.SCM_RIGHTS, array.array('i', [fd1]))])

    def checkTruncatedHeader(self, result, ignoreflags=0):
        msg, ancdata, flags, addr = result
        self.assertEqual(msg, MSG)
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.assertEqual(ancdata, [])
        self.checkFlags(flags, eor=True, checkset=socket.MSG_CTRUNC, ignore=ignoreflags)

    def testCmsgTruncNoBufSize(self):
        self.checkTruncatedHeader(self.doRecvmsg(self.serv_sock, len(MSG)), ignoreflags=socket.MSG_CTRUNC)

    def _testCmsgTruncNoBufSize(self):
        self.createAndSendFDs(1)

    def testCmsgTrunc0(self):
        self.checkTruncatedHeader(self.doRecvmsg(self.serv_sock, len(MSG), 0), ignoreflags=socket.MSG_CTRUNC)

    def _testCmsgTrunc0(self):
        self.createAndSendFDs(1)

    def testCmsgTrunc1(self):
        self.checkTruncatedHeader(self.doRecvmsg(self.serv_sock, len(MSG), 1))

    def _testCmsgTrunc1(self):
        self.createAndSendFDs(1)

    def testCmsgTrunc2Int(self):
        self.checkTruncatedHeader(self.doRecvmsg(self.serv_sock, len(MSG), SIZEOF_INT * 2))

    def _testCmsgTrunc2Int(self):
        self.createAndSendFDs(1)

    def testCmsgTruncLen0Minus1(self):
        self.checkTruncatedHeader(self.doRecvmsg(self.serv_sock, len(MSG), socket.CMSG_LEN(0) - 1))

    def _testCmsgTruncLen0Minus1(self):
        self.createAndSendFDs(1)

    def checkTruncatedArray(self, ancbuf, maxdata, mindata=0):
        msg, ancdata, flags, addr = self.doRecvmsg(self.serv_sock, len(MSG), ancbuf)
        self.assertEqual(msg, MSG)
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.checkFlags(flags, eor=True, checkset=socket.MSG_CTRUNC)
        if mindata == 0 and ancdata == []:
            return
        self.assertEqual(len(ancdata), 1)
        cmsg_level, cmsg_type, cmsg_data = ancdata[0]
        self.assertEqual(cmsg_level, socket.SOL_SOCKET)
        self.assertEqual(cmsg_type, socket.SCM_RIGHTS)
        self.assertGreaterEqual(len(cmsg_data), mindata)
        self.assertLessEqual(len(cmsg_data), maxdata)
        fds = array.array('i')
        fds.frombytes(cmsg_data[:len(cmsg_data) - len(cmsg_data) % fds.itemsize])
        self.checkFDs(fds)

    def testCmsgTruncLen0(self):
        self.checkTruncatedArray(ancbuf=socket.CMSG_LEN(0), maxdata=0)

    def _testCmsgTruncLen0(self):
        self.createAndSendFDs(1)

    def testCmsgTruncLen0Plus1(self):
        self.checkTruncatedArray(ancbuf=socket.CMSG_LEN(0) + 1, maxdata=1)

    def _testCmsgTruncLen0Plus1(self):
        self.createAndSendFDs(2)

    def testCmsgTruncLen1(self):
        self.checkTruncatedArray(ancbuf=socket.CMSG_LEN(SIZEOF_INT), maxdata=SIZEOF_INT)

    def _testCmsgTruncLen1(self):
        self.createAndSendFDs(2)

    def testCmsgTruncLen2Minus1(self):
        self.checkTruncatedArray(ancbuf=socket.CMSG_LEN(2 * SIZEOF_INT) - 1, maxdata=2 * SIZEOF_INT - 1)

    def _testCmsgTruncLen2Minus1(self):
        self.createAndSendFDs(2)

class RFC3542AncillaryTest(SendrecvmsgServerTimeoutBase):
    hop_limit = 2
    traffic_class = -1

    def ancillaryMapping(self, ancdata):
        d = {}
        for cmsg_level, cmsg_type, cmsg_data in ancdata:
            self.assertNotIn((cmsg_level, cmsg_type), d)
            d[cmsg_level, cmsg_type] = cmsg_data
        return d

    def checkHopLimit(self, ancbufsize, maxhop=255, ignoreflags=0):
        self.serv_sock.setsockopt(socket.IPPROTO_IPV6, socket.IPV6_RECVHOPLIMIT, 1)
        self.misc_event.set()
        msg, ancdata, flags, addr = self.doRecvmsg(self.serv_sock, len(MSG), ancbufsize)
        self.assertEqual(msg, MSG)
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.checkFlags(flags, eor=True, checkunset=socket.MSG_CTRUNC, ignore=ignoreflags)
        self.assertEqual(len(ancdata), 1)
        self.assertIsInstance(ancdata[0], tuple)
        cmsg_level, cmsg_type, cmsg_data = ancdata[0]
        self.assertEqual(cmsg_level, socket.IPPROTO_IPV6)
        self.assertEqual(cmsg_type, socket.IPV6_HOPLIMIT)
        self.assertIsInstance(cmsg_data, bytes)
        self.assertEqual(len(cmsg_data), SIZEOF_INT)
        a = array.array('i')
        a.frombytes(cmsg_data)
        self.assertGreaterEqual(a[0], 0)
        self.assertLessEqual(a[0], maxhop)

    @requireAttrs(socket, 'IPV6_RECVHOPLIMIT', 'IPV6_HOPLIMIT')
    def testRecvHopLimit(self):
        self.checkHopLimit(ancbufsize=10240)

    @testRecvHopLimit.client_skip
    def _testRecvHopLimit(self):
        self.assertTrue(self.misc_event.wait(timeout=self.fail_timeout))
        self.sendToServer(MSG)

    @requireAttrs(socket, 'CMSG_SPACE', 'IPV6_RECVHOPLIMIT', 'IPV6_HOPLIMIT')
    def testRecvHopLimitCMSG_SPACE(self):
        self.checkHopLimit(ancbufsize=socket.CMSG_SPACE(SIZEOF_INT))

    @testRecvHopLimitCMSG_SPACE.client_skip
    def _testRecvHopLimitCMSG_SPACE(self):
        self.assertTrue(self.misc_event.wait(timeout=self.fail_timeout))
        self.sendToServer(MSG)

    @requireAttrs(socket.socket, 'sendmsg')
    @requireAttrs(socket, 'IPV6_RECVHOPLIMIT', 'IPV6_HOPLIMIT')
    def testSetHopLimit(self):
        self.checkHopLimit(ancbufsize=10240, maxhop=self.hop_limit)

    @testSetHopLimit.client_skip
    def _testSetHopLimit(self):
        self.assertTrue(self.misc_event.wait(timeout=self.fail_timeout))
        self.assertEqual(self.sendmsgToServer([MSG], [(socket.IPPROTO_IPV6, socket.IPV6_HOPLIMIT, array.array('i', [self.hop_limit]))]), len(MSG))

    def checkTrafficClassAndHopLimit(self, ancbufsize, maxhop=255, ignoreflags=0):
        self.serv_sock.setsockopt(socket.IPPROTO_IPV6, socket.IPV6_RECVHOPLIMIT, 1)
        self.serv_sock.setsockopt(socket.IPPROTO_IPV6, socket.IPV6_RECVTCLASS, 1)
        self.misc_event.set()
        msg, ancdata, flags, addr = self.doRecvmsg(self.serv_sock, len(MSG), ancbufsize)
        self.assertEqual(msg, MSG)
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.checkFlags(flags, eor=True, checkunset=socket.MSG_CTRUNC, ignore=ignoreflags)
        self.assertEqual(len(ancdata), 2)
        ancmap = self.ancillaryMapping(ancdata)
        tcdata = ancmap[socket.IPPROTO_IPV6, socket.IPV6_TCLASS]
        self.assertEqual(len(tcdata), SIZEOF_INT)
        a = array.array('i')
        a.frombytes(tcdata)
        self.assertGreaterEqual(a[0], 0)
        self.assertLessEqual(a[0], 255)
        hldata = ancmap[socket.IPPROTO_IPV6, socket.IPV6_HOPLIMIT]
        self.assertEqual(len(hldata), SIZEOF_INT)
        a = array.array('i')
        a.frombytes(hldata)
        self.assertGreaterEqual(a[0], 0)
        self.assertLessEqual(a[0], maxhop)

    @requireAttrs(socket, 'IPV6_RECVHOPLIMIT', 'IPV6_HOPLIMIT', 'IPV6_RECVTCLASS', 'IPV6_TCLASS')
    def testRecvTrafficClassAndHopLimit(self):
        self.checkTrafficClassAndHopLimit(ancbufsize=10240)

    @testRecvTrafficClassAndHopLimit.client_skip
    def _testRecvTrafficClassAndHopLimit(self):
        self.assertTrue(self.misc_event.wait(timeout=self.fail_timeout))
        self.sendToServer(MSG)

    @requireAttrs(socket, 'CMSG_SPACE', 'IPV6_RECVHOPLIMIT', 'IPV6_HOPLIMIT', 'IPV6_RECVTCLASS', 'IPV6_TCLASS')
    def testRecvTrafficClassAndHopLimitCMSG_SPACE(self):
        self.checkTrafficClassAndHopLimit(ancbufsize=socket.CMSG_SPACE(SIZEOF_INT) * 2)

    @testRecvTrafficClassAndHopLimitCMSG_SPACE.client_skip
    def _testRecvTrafficClassAndHopLimitCMSG_SPACE(self):
        self.assertTrue(self.misc_event.wait(timeout=self.fail_timeout))
        self.sendToServer(MSG)

    @requireAttrs(socket.socket, 'sendmsg')
    @requireAttrs(socket, 'CMSG_SPACE', 'IPV6_RECVHOPLIMIT', 'IPV6_HOPLIMIT', 'IPV6_RECVTCLASS', 'IPV6_TCLASS')
    def testSetTrafficClassAndHopLimit(self):
        self.checkTrafficClassAndHopLimit(ancbufsize=10240, maxhop=self.hop_limit)

    @testSetTrafficClassAndHopLimit.client_skip
    def _testSetTrafficClassAndHopLimit(self):
        self.assertTrue(self.misc_event.wait(timeout=self.fail_timeout))
        self.assertEqual(self.sendmsgToServer([MSG], [(socket.IPPROTO_IPV6, socket.IPV6_TCLASS, array.array('i', [self.traffic_class])), (socket.IPPROTO_IPV6, socket.IPV6_HOPLIMIT, array.array('i', [self.hop_limit]))]), len(MSG))

    @requireAttrs(socket.socket, 'sendmsg')
    @requireAttrs(socket, 'CMSG_SPACE', 'IPV6_RECVHOPLIMIT', 'IPV6_HOPLIMIT', 'IPV6_RECVTCLASS', 'IPV6_TCLASS')
    def testOddCmsgSize(self):
        self.checkTrafficClassAndHopLimit(ancbufsize=10240, maxhop=self.hop_limit)

    @testOddCmsgSize.client_skip
    def _testOddCmsgSize(self):
        self.assertTrue(self.misc_event.wait(timeout=self.fail_timeout))
        try:
            nbytes = self.sendmsgToServer([MSG], [(socket.IPPROTO_IPV6, socket.IPV6_TCLASS, array.array('i', [self.traffic_class]).tobytes() + b'\x00'), (socket.IPPROTO_IPV6, socket.IPV6_HOPLIMIT, array.array('i', [self.hop_limit]))])
        except OSError as e:
            self.assertIsInstance(e.errno, int)
            nbytes = self.sendmsgToServer([MSG], [(socket.IPPROTO_IPV6, socket.IPV6_TCLASS, array.array('i', [self.traffic_class])), (socket.IPPROTO_IPV6, socket.IPV6_HOPLIMIT, array.array('i', [self.hop_limit]))])
            self.assertEqual(nbytes, len(MSG))

    def checkHopLimitTruncatedHeader(self, ancbufsize, ignoreflags=0):
        self.serv_sock.setsockopt(socket.IPPROTO_IPV6, socket.IPV6_RECVHOPLIMIT, 1)
        self.misc_event.set()
        args = () if ancbufsize is None else (ancbufsize,)
        msg, ancdata, flags, addr = self.doRecvmsg(self.serv_sock, len(MSG), *args)
        self.assertEqual(msg, MSG)
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.assertEqual(ancdata, [])
        self.checkFlags(flags, eor=True, checkset=socket.MSG_CTRUNC, ignore=ignoreflags)

    @requireAttrs(socket, 'IPV6_RECVHOPLIMIT', 'IPV6_HOPLIMIT')
    def testCmsgTruncNoBufSize(self):
        self.checkHopLimitTruncatedHeader(ancbufsize=None, ignoreflags=socket.MSG_CTRUNC)

    @testCmsgTruncNoBufSize.client_skip
    def _testCmsgTruncNoBufSize(self):
        self.assertTrue(self.misc_event.wait(timeout=self.fail_timeout))
        self.sendToServer(MSG)

    @requireAttrs(socket, 'IPV6_RECVHOPLIMIT', 'IPV6_HOPLIMIT')
    def testSingleCmsgTrunc0(self):
        self.checkHopLimitTruncatedHeader(ancbufsize=0, ignoreflags=socket.MSG_CTRUNC)

    @testSingleCmsgTrunc0.client_skip
    def _testSingleCmsgTrunc0(self):
        self.assertTrue(self.misc_event.wait(timeout=self.fail_timeout))
        self.sendToServer(MSG)

    @requireAttrs(socket, 'IPV6_RECVHOPLIMIT', 'IPV6_HOPLIMIT')
    def testSingleCmsgTrunc1(self):
        self.checkHopLimitTruncatedHeader(ancbufsize=1)

    @testSingleCmsgTrunc1.client_skip
    def _testSingleCmsgTrunc1(self):
        self.assertTrue(self.misc_event.wait(timeout=self.fail_timeout))
        self.sendToServer(MSG)

    @requireAttrs(socket, 'IPV6_RECVHOPLIMIT', 'IPV6_HOPLIMIT')
    def testSingleCmsgTrunc2Int(self):
        self.checkHopLimitTruncatedHeader(ancbufsize=2 * SIZEOF_INT)

    @testSingleCmsgTrunc2Int.client_skip
    def _testSingleCmsgTrunc2Int(self):
        self.assertTrue(self.misc_event.wait(timeout=self.fail_timeout))
        self.sendToServer(MSG)

    @requireAttrs(socket, 'IPV6_RECVHOPLIMIT', 'IPV6_HOPLIMIT')
    def testSingleCmsgTruncLen0Minus1(self):
        self.checkHopLimitTruncatedHeader(ancbufsize=socket.CMSG_LEN(0) - 1)

    @testSingleCmsgTruncLen0Minus1.client_skip
    def _testSingleCmsgTruncLen0Minus1(self):
        self.assertTrue(self.misc_event.wait(timeout=self.fail_timeout))
        self.sendToServer(MSG)

    @requireAttrs(socket, 'IPV6_RECVHOPLIMIT', 'IPV6_HOPLIMIT')
    def testSingleCmsgTruncInData(self):
        self.serv_sock.setsockopt(socket.IPPROTO_IPV6, socket.IPV6_RECVHOPLIMIT, 1)
        self.misc_event.set()
        msg, ancdata, flags, addr = self.doRecvmsg(self.serv_sock, len(MSG), socket.CMSG_LEN(SIZEOF_INT) - 1)
        self.assertEqual(msg, MSG)
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.checkFlags(flags, eor=True, checkset=socket.MSG_CTRUNC)
        self.assertLessEqual(len(ancdata), 1)
        if ancdata:
            cmsg_level, cmsg_type, cmsg_data = ancdata[0]
            self.assertEqual(cmsg_level, socket.IPPROTO_IPV6)
            self.assertEqual(cmsg_type, socket.IPV6_HOPLIMIT)
            self.assertLess(len(cmsg_data), SIZEOF_INT)

    @testSingleCmsgTruncInData.client_skip
    def _testSingleCmsgTruncInData(self):
        self.assertTrue(self.misc_event.wait(timeout=self.fail_timeout))
        self.sendToServer(MSG)

    def checkTruncatedSecondHeader(self, ancbufsize, ignoreflags=0):
        self.serv_sock.setsockopt(socket.IPPROTO_IPV6, socket.IPV6_RECVHOPLIMIT, 1)
        self.serv_sock.setsockopt(socket.IPPROTO_IPV6, socket.IPV6_RECVTCLASS, 1)
        self.misc_event.set()
        msg, ancdata, flags, addr = self.doRecvmsg(self.serv_sock, len(MSG), ancbufsize)
        self.assertEqual(msg, MSG)
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.checkFlags(flags, eor=True, checkset=socket.MSG_CTRUNC, ignore=ignoreflags)
        self.assertEqual(len(ancdata), 1)
        cmsg_level, cmsg_type, cmsg_data = ancdata[0]
        self.assertEqual(cmsg_level, socket.IPPROTO_IPV6)
        self.assertIn(cmsg_type, {socket.IPV6_TCLASS, socket.IPV6_HOPLIMIT})
        self.assertEqual(len(cmsg_data), SIZEOF_INT)
        a = array.array('i')
        a.frombytes(cmsg_data)
        self.assertGreaterEqual(a[0], 0)
        self.assertLessEqual(a[0], 255)

    @requireAttrs(socket, 'CMSG_SPACE', 'IPV6_RECVHOPLIMIT', 'IPV6_HOPLIMIT', 'IPV6_RECVTCLASS', 'IPV6_TCLASS')
    def testSecondCmsgTrunc0(self):
        self.checkTruncatedSecondHeader(socket.CMSG_SPACE(SIZEOF_INT), ignoreflags=socket.MSG_CTRUNC)

    @testSecondCmsgTrunc0.client_skip
    def _testSecondCmsgTrunc0(self):
        self.assertTrue(self.misc_event.wait(timeout=self.fail_timeout))
        self.sendToServer(MSG)

    @requireAttrs(socket, 'CMSG_SPACE', 'IPV6_RECVHOPLIMIT', 'IPV6_HOPLIMIT', 'IPV6_RECVTCLASS', 'IPV6_TCLASS')
    def testSecondCmsgTrunc1(self):
        self.checkTruncatedSecondHeader(socket.CMSG_SPACE(SIZEOF_INT) + 1)

    @testSecondCmsgTrunc1.client_skip
    def _testSecondCmsgTrunc1(self):
        self.assertTrue(self.misc_event.wait(timeout=self.fail_timeout))
        self.sendToServer(MSG)

    @requireAttrs(socket, 'CMSG_SPACE', 'IPV6_RECVHOPLIMIT', 'IPV6_HOPLIMIT', 'IPV6_RECVTCLASS', 'IPV6_TCLASS')
    def testSecondCmsgTrunc2Int(self):
        self.checkTruncatedSecondHeader(socket.CMSG_SPACE(SIZEOF_INT) + 2 * SIZEOF_INT)

    @testSecondCmsgTrunc2Int.client_skip
    def _testSecondCmsgTrunc2Int(self):
        self.assertTrue(self.misc_event.wait(timeout=self.fail_timeout))
        self.sendToServer(MSG)

    @requireAttrs(socket, 'CMSG_SPACE', 'IPV6_RECVHOPLIMIT', 'IPV6_HOPLIMIT', 'IPV6_RECVTCLASS', 'IPV6_TCLASS')
    def testSecondCmsgTruncLen0Minus1(self):
        self.checkTruncatedSecondHeader(socket.CMSG_SPACE(SIZEOF_INT) + socket.CMSG_LEN(0) - 1)

    @testSecondCmsgTruncLen0Minus1.client_skip
    def _testSecondCmsgTruncLen0Minus1(self):
        self.assertTrue(self.misc_event.wait(timeout=self.fail_timeout))
        self.sendToServer(MSG)

    @requireAttrs(socket, 'CMSG_SPACE', 'IPV6_RECVHOPLIMIT', 'IPV6_HOPLIMIT', 'IPV6_RECVTCLASS', 'IPV6_TCLASS')
    def testSecondCmsgTruncInData(self):
        self.serv_sock.setsockopt(socket.IPPROTO_IPV6, socket.IPV6_RECVHOPLIMIT, 1)
        self.serv_sock.setsockopt(socket.IPPROTO_IPV6, socket.IPV6_RECVTCLASS, 1)
        self.misc_event.set()
        msg, ancdata, flags, addr = self.doRecvmsg(self.serv_sock, len(MSG), socket.CMSG_SPACE(SIZEOF_INT) + socket.CMSG_LEN(SIZEOF_INT) - 1)
        self.assertEqual(msg, MSG)
        self.checkRecvmsgAddress(addr, self.cli_addr)
        self.checkFlags(flags, eor=True, checkset=socket.MSG_CTRUNC)
        cmsg_types = {socket.IPV6_TCLASS, socket.IPV6_HOPLIMIT}
        cmsg_level, cmsg_type, cmsg_data = ancdata.pop(0)
        self.assertEqual(cmsg_level, socket.IPPROTO_IPV6)
        cmsg_types.remove(cmsg_type)
        self.assertEqual(len(cmsg_data), SIZEOF_INT)
        a = array.array('i')
        a.frombytes(cmsg_data)
        self.assertGreaterEqual(a[0], 0)
        self.assertLessEqual(a[0], 255)
        if ancdata:
            cmsg_level, cmsg_type, cmsg_data = ancdata.pop(0)
            self.assertEqual(cmsg_level, socket.IPPROTO_IPV6)
            cmsg_types.remove(cmsg_type)
            self.assertLess(len(cmsg_data), SIZEOF_INT)
        self.assertEqual(ancdata, [])

    @testSecondCmsgTruncInData.client_skip
    def _testSecondCmsgTruncInData(self):
        self.assertTrue(self.misc_event.wait(timeout=self.fail_timeout))
        self.sendToServer(MSG)

class InterruptedTimeoutBase:

    def setUp(self):
        super().setUp()
        orig_alrm_handler = signal.signal(signal.SIGALRM, lambda signum, frame: 1 / 0)
        self.addCleanup(signal.signal, signal.SIGALRM, orig_alrm_handler)
    timeout = support.LOOPBACK_TIMEOUT
    if hasattr(signal, 'setitimer'):
        alarm_time = 0.05

        def setAlarm(self, seconds):
            signal.setitimer(signal.ITIMER_REAL, seconds)
    else:
        alarm_time = 2

        def setAlarm(self, seconds):
            signal.alarm(seconds)

class NetworkConnectionTest(object):
    """Prove network connection."""

    def clientSetUp(self):
        self.cli = socket.create_connection((HOST, self.port))
        self.serv_conn = self.cli

TIPC_STYPE = 2000

TIPC_LOWER = 200

TIPC_UPPER = 210

def isTipcAvailable():
    """Check if the TIPC module is loaded

    The TIPC module is not loaded automatically on Ubuntu and probably
    other Linux distros.
    """
    if not hasattr(socket, 'AF_TIPC'):
        return False
    try:
        f = open('/proc/modules', encoding='utf-8')
    except (FileNotFoundError, IsADirectoryError, PermissionError):
        return False
    with f:
        for line in f:
            if line.startswith('tipc '):
                return True
    return False

def setUpModule():
    thread_info = threading_helper.threading_setup()
    unittest.addModuleCleanup(threading_helper.threading_cleanup, *thread_info)


# --- test body ---

assert socket.TCP_KEEPALIVE
print("TestMacOSTCPFlags::test_tcp_keepalive: ok")
