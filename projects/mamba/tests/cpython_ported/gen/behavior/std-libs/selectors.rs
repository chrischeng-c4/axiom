use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/selectors/context_manager_registers_and_closes.py`.
#[test]
fn test_gen_behavior_std_libs_selectors_context_manager_registers_and_closes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "context_manager_registers_and_closes"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: DefaultSelector used as a context manager registers inside the with-block (get_map length 1) and is usable for the block's duration"""
import selectors
import socket

_s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
with selectors.DefaultSelector() as _sel:
    assert isinstance(_sel, selectors.BaseSelector), "with-target is a selector"
    _sel.register(_s, selectors.EVENT_READ)
    assert len(_sel.get_map()) == 1, "registered inside the context manager"
_s.close()
print("context_manager_registers_and_closes OK")
"###);
    assert_output(&out, r###"context_manager_registers_and_closes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/selectors/default_selector_test_case__test_fileno.py`.
#[test]
fn test_gen_behavior_std_libs_selectors_default_selector_test_case__test_fileno() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "default_selector_test_case__test_fileno"
# subject = "cpython.test_selectors.DefaultSelectorTestCase.test_fileno"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_selectors.py::DefaultSelectorTestCase::test_fileno
"""Auto-ported test: DefaultSelectorTestCase::test_fileno (CPython 3.12 oracle)."""


import errno
import os
import random
import selectors
import signal
import socket
import sys
from test import support
from test.support import os_helper
from test.support import socket_helper
from time import sleep
import unittest
import unittest.mock
import tempfile
from time import monotonic as time


try:
    import resource
except ImportError:
    resource = None

if support.is_emscripten or support.is_wasi:
    raise unittest.SkipTest('Cannot create socketpair on Emscripten/WASI.')

if hasattr(socket, 'socketpair'):
    socketpair = socket.socketpair
else:

    def socketpair(family=socket.AF_INET, type=socket.SOCK_STREAM, proto=0):
        with socket.socket(family, type, proto) as l:
            l.bind((socket_helper.HOST, 0))
            l.listen()
            c = socket.socket(family, type, proto)
            try:
                c.connect(l.getsockname())
                caddr = c.getsockname()
                while True:
                    a, addr = l.accept()
                    if addr == caddr:
                        return (c, a)
                    a.close()
            except OSError:
                c.close()
                raise

def find_ready_matching(ready, flag):
    match = []
    for key, events in ready:
        if events & flag:
            match.append(key.fileobj)
    return match

def tearDownModule():
    support.reap_children()


# --- test body ---
SELECTOR = selectors.DefaultSelector

def make_socketpair():
    rd, wr = socketpair()
    pass
    pass
    return (rd, wr)
s = SELECTOR()
pass
if hasattr(s, 'fileno'):
    fd = s.fileno()

    assert isinstance(fd, int)

    assert fd >= 0
print("DefaultSelectorTestCase::test_fileno: ok")
"###);
    assert_output(&out, r###"DefaultSelectorTestCase::test_fileno: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/selectors/get_map_returns_registered_keys.py`.
#[test]
fn test_gen_behavior_std_libs_selectors_get_map_returns_registered_keys() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "get_map_returns_registered_keys"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: get_map() exposes the registered fd->SelectorKey mapping; the registered socket's fd is present and maps back to its key"""
import selectors
import socket

_s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
with selectors.DefaultSelector() as _sel:
    _key = _sel.register(_s, selectors.EVENT_READ, data="payload")
    _map = _sel.get_map()
    assert _key.fd in _map, "registered fd must be a key in get_map()"
    assert _map[_key.fd] is _key, "get_map() must map the fd back to its SelectorKey"
    assert _map[_key.fd].data == "payload", "mapped key carries the registered data"
_s.close()
print("get_map_returns_registered_keys OK")
"###);
    assert_output(&out, r###"get_map_returns_registered_keys OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/selectors/kqueue_selector_test_case__test_fileno.py`.
#[test]
fn test_gen_behavior_std_libs_selectors_kqueue_selector_test_case__test_fileno() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "kqueue_selector_test_case__test_fileno"
# subject = "cpython.test_selectors.KqueueSelectorTestCase.test_fileno"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_selectors.py::KqueueSelectorTestCase::test_fileno
"""Auto-ported test: KqueueSelectorTestCase::test_fileno (CPython 3.12 oracle)."""


import errno
import os
import random
import selectors
import signal
import socket
import sys
from test import support
from test.support import os_helper
from test.support import socket_helper
from time import sleep
import unittest
import unittest.mock
import tempfile
from time import monotonic as time


try:
    import resource
except ImportError:
    resource = None

if support.is_emscripten or support.is_wasi:
    raise unittest.SkipTest('Cannot create socketpair on Emscripten/WASI.')

if hasattr(socket, 'socketpair'):
    socketpair = socket.socketpair
else:

    def socketpair(family=socket.AF_INET, type=socket.SOCK_STREAM, proto=0):
        with socket.socket(family, type, proto) as l:
            l.bind((socket_helper.HOST, 0))
            l.listen()
            c = socket.socket(family, type, proto)
            try:
                c.connect(l.getsockname())
                caddr = c.getsockname()
                while True:
                    a, addr = l.accept()
                    if addr == caddr:
                        return (c, a)
                    a.close()
            except OSError:
                c.close()
                raise

def find_ready_matching(ready, flag):
    match = []
    for key, events in ready:
        if events & flag:
            match.append(key.fileobj)
    return match

def tearDownModule():
    support.reap_children()


# --- test body ---
SELECTOR = getattr(selectors, 'KqueueSelector', None)

def make_socketpair():
    rd, wr = socketpair()
    pass
    pass
    return (rd, wr)
s = SELECTOR()
pass
if hasattr(s, 'fileno'):
    fd = s.fileno()

    assert isinstance(fd, int)

    assert fd >= 0
print("KqueueSelectorTestCase::test_fileno: ok")
"###);
    assert_output(&out, r###"KqueueSelectorTestCase::test_fileno: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/selectors/modify_changes_events.py`.
#[test]
fn test_gen_behavior_std_libs_selectors_modify_changes_events() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "modify_changes_events"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: modify() updates the registered events mask in place and returns the updated SelectorKey, observable via get_key()"""
import selectors
import socket

_s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
with selectors.DefaultSelector() as _sel:
    _sel.register(_s, selectors.EVENT_READ)
    _key2 = _sel.modify(_s, selectors.EVENT_READ | selectors.EVENT_WRITE)
    assert _key2.events == (selectors.EVENT_READ | selectors.EVENT_WRITE), f"modified events = {_key2.events!r}"
    assert _sel.get_key(_s).events == (selectors.EVENT_READ | selectors.EVENT_WRITE), "get_key reflects the modify"
_s.close()
print("modify_changes_events OK")
"###);
    assert_output(&out, r###"modify_changes_events OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/selectors/poll_selector_test_case__test_fileno.py`.
#[test]
fn test_gen_behavior_std_libs_selectors_poll_selector_test_case__test_fileno() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "poll_selector_test_case__test_fileno"
# subject = "cpython.test_selectors.PollSelectorTestCase.test_fileno"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_selectors.py::PollSelectorTestCase::test_fileno
"""Auto-ported test: PollSelectorTestCase::test_fileno (CPython 3.12 oracle)."""


import errno
import os
import random
import selectors
import signal
import socket
import sys
from test import support
from test.support import os_helper
from test.support import socket_helper
from time import sleep
import unittest
import unittest.mock
import tempfile
from time import monotonic as time


try:
    import resource
except ImportError:
    resource = None

if support.is_emscripten or support.is_wasi:
    raise unittest.SkipTest('Cannot create socketpair on Emscripten/WASI.')

if hasattr(socket, 'socketpair'):
    socketpair = socket.socketpair
else:

    def socketpair(family=socket.AF_INET, type=socket.SOCK_STREAM, proto=0):
        with socket.socket(family, type, proto) as l:
            l.bind((socket_helper.HOST, 0))
            l.listen()
            c = socket.socket(family, type, proto)
            try:
                c.connect(l.getsockname())
                caddr = c.getsockname()
                while True:
                    a, addr = l.accept()
                    if addr == caddr:
                        return (c, a)
                    a.close()
            except OSError:
                c.close()
                raise

def find_ready_matching(ready, flag):
    match = []
    for key, events in ready:
        if events & flag:
            match.append(key.fileobj)
    return match

def tearDownModule():
    support.reap_children()


# --- test body ---
SELECTOR = getattr(selectors, 'PollSelector', None)

def make_socketpair():
    rd, wr = socketpair()
    pass
    pass
    return (rd, wr)
s = SELECTOR()
pass
if hasattr(s, 'fileno'):
    fd = s.fileno()

    assert isinstance(fd, int)

    assert fd >= 0
print("PollSelectorTestCase::test_fileno: ok")
"###);
    assert_output(&out, r###"PollSelectorTestCase::test_fileno: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/selectors/register_returns_selector_key.py`.
#[test]
fn test_gen_behavior_std_libs_selectors_register_returns_selector_key() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "register_returns_selector_key"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: register() returns a SelectorKey whose fileobj is the socket, fd is an int, events is the requested mask, and data is the passed-in object"""
import selectors
import socket

_s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
with selectors.DefaultSelector() as _sel:
    _key = _sel.register(_s, selectors.EVENT_READ | selectors.EVENT_WRITE, data=42)
    assert isinstance(_key, selectors.SelectorKey), f"register() must return SelectorKey, got {type(_key)!r}"
    assert _key.fileobj is _s, "key.fileobj must be the registered socket"
    assert isinstance(_key.fd, int), f"key.fd must be int, got {type(_key.fd)!r}"
    assert _key.events == (selectors.EVENT_READ | selectors.EVENT_WRITE), f"key.events = {_key.events!r}"
    assert _key.data == 42, f"key.data = {_key.data!r}"
_s.close()
print("register_returns_selector_key OK")
"###);
    assert_output(&out, r###"register_returns_selector_key OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/selectors/select_detects_writable_socket.py`.
#[test]
fn test_gen_behavior_std_libs_selectors_select_detects_writable_socket() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "select_detects_writable_socket"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: a freshly connected socketpair endpoint registered for EVENT_WRITE is reported writable by select() with EVENT_WRITE in the mask"""
import selectors
import socket

_a, _b = socket.socketpair()
with selectors.DefaultSelector() as _sel:
    _sel.register(_a, selectors.EVENT_WRITE)
    _ready = _sel.select(timeout=0.5)
    assert len(_ready) >= 1, f"connected endpoint should be writable, got {_ready!r}"
    _key, _mask = _ready[0]
    assert _mask & selectors.EVENT_WRITE, "ready mask must carry EVENT_WRITE"
    assert _key.fileobj is _a, "ready key's fileobj must be the registered socket"
_a.close()
_b.close()
print("select_detects_writable_socket OK")
"###);
    assert_output(&out, r###"select_detects_writable_socket OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/selectors/select_selector_test_case__test_fileno.py`.
#[test]
fn test_gen_behavior_std_libs_selectors_select_selector_test_case__test_fileno() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "select_selector_test_case__test_fileno"
# subject = "cpython.test_selectors.SelectSelectorTestCase.test_fileno"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_selectors.py::SelectSelectorTestCase::test_fileno
"""Auto-ported test: SelectSelectorTestCase::test_fileno (CPython 3.12 oracle)."""


import errno
import os
import random
import selectors
import signal
import socket
import sys
from test import support
from test.support import os_helper
from test.support import socket_helper
from time import sleep
import unittest
import unittest.mock
import tempfile
from time import monotonic as time


try:
    import resource
except ImportError:
    resource = None

if support.is_emscripten or support.is_wasi:
    raise unittest.SkipTest('Cannot create socketpair on Emscripten/WASI.')

if hasattr(socket, 'socketpair'):
    socketpair = socket.socketpair
else:

    def socketpair(family=socket.AF_INET, type=socket.SOCK_STREAM, proto=0):
        with socket.socket(family, type, proto) as l:
            l.bind((socket_helper.HOST, 0))
            l.listen()
            c = socket.socket(family, type, proto)
            try:
                c.connect(l.getsockname())
                caddr = c.getsockname()
                while True:
                    a, addr = l.accept()
                    if addr == caddr:
                        return (c, a)
                    a.close()
            except OSError:
                c.close()
                raise

def find_ready_matching(ready, flag):
    match = []
    for key, events in ready:
        if events & flag:
            match.append(key.fileobj)
    return match

def tearDownModule():
    support.reap_children()


# --- test body ---
SELECTOR = selectors.SelectSelector

def make_socketpair():
    rd, wr = socketpair()
    pass
    pass
    return (rd, wr)
s = SELECTOR()
pass
if hasattr(s, 'fileno'):
    fd = s.fileno()

    assert isinstance(fd, int)

    assert fd >= 0
print("SelectSelectorTestCase::test_fileno: ok")
"###);
    assert_output(&out, r###"SelectSelectorTestCase::test_fileno: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/selectors/unregister_clears_map.py`.
#[test]
fn test_gen_behavior_std_libs_selectors_unregister_clears_map() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "unregister_clears_map"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: unregister() removes the fd so get_map() is empty afterward"""
import selectors
import socket

_s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
with selectors.DefaultSelector() as _sel:
    _sel.register(_s, selectors.EVENT_READ)
    assert len(_sel.get_map()) == 1, "map has one entry after register"
    _sel.unregister(_s)
    assert len(_sel.get_map()) == 0, f"map must be empty after unregister, got {len(_sel.get_map())}"
_s.close()
print("unregister_clears_map OK")
"###);
    assert_output(&out, r###"unregister_clears_map OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/selectors/unregister_then_register_reuses_fd.py`.
#[test]
fn test_gen_behavior_std_libs_selectors_unregister_then_register_reuses_fd() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "behavior"
# case = "unregister_then_register_reuses_fd"
# subject = "selectors.DefaultSelector"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_selectors.py"
# status = "filled"
# ///
"""selectors.DefaultSelector: after unregister(), the same socket can be re-registered with a different events mask"""
import selectors
import socket

_s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
with selectors.DefaultSelector() as _sel:
    _sel.register(_s, selectors.EVENT_READ)
    _sel.unregister(_s)
    _key = _sel.register(_s, selectors.EVENT_WRITE)
    assert _key.events == selectors.EVENT_WRITE, f"re-registered events = {_key.events!r}"
_s.close()
print("unregister_then_register_reuses_fd OK")
"###);
    assert_output(&out, r###"unregister_then_register_reuses_fd OK
"###);
}
