# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "send_recv_fds_tests__test_send_and_recv_fds"
# subject = "cpython.test_socket.SendRecvFdsTests.testSendAndRecvFds"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_socket.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_socket.py::SendRecvFdsTests::testSendAndRecvFds
"""Auto-ported test: SendRecvFdsTests::testSendAndRecvFds (CPython 3.12 oracle)."""


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
def close_pipes(pipes):
    for fd1, fd2 in pipes:
        os.close(fd1)
        os.close(fd2)

def close_fds(fds):
    for fd in fds:
        os.close(fd)
pipes = [os.pipe() for _ in range(10)]
pass
fds = [rfd for rfd, wfd in pipes]
sock1, sock2 = socket.socketpair(socket.AF_UNIX, socket.SOCK_STREAM)
with sock1, sock2:
    socket.send_fds(sock1, [MSG], fds)
    msg, fds2, flags, addr = socket.recv_fds(sock2, len(MSG) * 2, len(fds) * 2)
    pass

assert msg == MSG

assert len(fds2) == len(fds)

assert flags == 0
for index, fds in enumerate(pipes):
    rfd, wfd = fds
    os.write(wfd, str(index).encode())
for index, rfd in enumerate(fds2):
    data = os.read(rfd, 100)

    assert data == str(index).encode()
print("SendRecvFdsTests::testSendAndRecvFds: ok")
