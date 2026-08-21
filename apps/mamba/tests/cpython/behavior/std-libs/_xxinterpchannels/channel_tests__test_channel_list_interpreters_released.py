# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_xxinterpchannels"
# dimension = "behavior"
# case = "channel_tests__test_channel_list_interpreters_released"
# subject = "cpython.test__xxinterpchannels.ChannelTests.test_channel_list_interpreters_released"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test__xxinterpchannels.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test__xxinterpchannels.py::ChannelTests::test_channel_list_interpreters_released
"""Auto-ported test: ChannelTests::test_channel_list_interpreters_released (CPython 3.12 oracle)."""


from collections import namedtuple
import contextlib
import sys
from textwrap import dedent
import threading
import time
import unittest
from test.support import import_helper
from test.test__xxsubinterpreters import interpreters, _run_output, clean_up_interpreters


channels = import_helper.import_module('_xxinterpchannels')

def run_interp(id, source, **shared):
    _run_interp(id, source, shared)

def _run_interp(id, source, shared, _mainns={}):
    source = dedent(source)
    main = interpreters.get_main()
    if main == id:
        if interpreters.get_current() != main:
            raise RuntimeError
        exec(source, _mainns)
    else:
        interpreters.run_string(id, source, shared)

class Interpreter(namedtuple('Interpreter', 'name id')):

    @classmethod
    def from_raw(cls, raw):
        if isinstance(raw, cls):
            return raw
        elif isinstance(raw, str):
            return cls(raw)
        else:
            raise NotImplementedError

    def __new__(cls, name=None, id=None):
        main = interpreters.get_main()
        if id == main:
            if not name:
                name = 'main'
            elif name != 'main':
                raise ValueError('name mismatch (expected "main", got "{}")'.format(name))
            id = main
        elif id is not None:
            if not name:
                name = 'interp'
            elif name == 'main':
                raise ValueError('name mismatch (unexpected "main")')
            if not isinstance(id, interpreters.InterpreterID):
                id = interpreters.InterpreterID(id)
        elif not name or name == 'main':
            name = 'main'
            id = main
        else:
            id = interpreters.create()
        self = super().__new__(cls, name, id)
        return self

@contextlib.contextmanager
def expect_channel_closed():
    try:
        yield
    except channels.ChannelClosedError:
        pass
    else:
        assert False, 'channel not closed'

class ChannelAction(namedtuple('ChannelAction', 'action end interp')):

    def __new__(cls, action, end=None, interp=None):
        if not end:
            end = 'both'
        if not interp:
            interp = 'main'
        self = super().__new__(cls, action, end, interp)
        return self

    def __init__(self, *args, **kwargs):
        if self.action == 'use':
            if self.end not in ('same', 'opposite', 'send', 'recv'):
                raise ValueError(self.end)
        elif self.action in ('close', 'force-close'):
            if self.end not in ('both', 'same', 'opposite', 'send', 'recv'):
                raise ValueError(self.end)
        else:
            raise ValueError(self.action)
        if self.interp not in ('main', 'same', 'other', 'extra'):
            raise ValueError(self.interp)

    def resolve_end(self, end):
        if self.end == 'same':
            return end
        elif self.end == 'opposite':
            return 'recv' if end == 'send' else 'send'
        else:
            return self.end

    def resolve_interp(self, interp, other, extra):
        if self.interp == 'same':
            return interp
        elif self.interp == 'other':
            if other is None:
                raise RuntimeError
            return other
        elif self.interp == 'extra':
            if extra is None:
                raise RuntimeError
            return extra
        elif self.interp == 'main':
            if interp.name == 'main':
                return interp
            elif other and other.name == 'main':
                return other
            else:
                raise RuntimeError

class ChannelState(namedtuple('ChannelState', 'pending closed')):

    def __new__(cls, pending=0, *, closed=False):
        self = super().__new__(cls, pending, closed)
        return self

    def incr(self):
        return type(self)(self.pending + 1, closed=self.closed)

    def decr(self):
        return type(self)(self.pending - 1, closed=self.closed)

    def close(self, *, force=True):
        if self.closed:
            if not force or self.pending == 0:
                return self
        return type(self)(0 if force else self.pending, closed=True)

def run_action(cid, action, end, state, *, hideclosed=True):
    if state.closed:
        if action == 'use' and end == 'recv' and state.pending:
            expectfail = False
        else:
            expectfail = True
    else:
        expectfail = False
    try:
        result = _run_action(cid, action, end, state)
    except channels.ChannelClosedError:
        if not hideclosed and (not expectfail):
            raise
        result = state.close()
    else:
        if expectfail:
            raise ...
    return result

def _run_action(cid, action, end, state):
    if action == 'use':
        if end == 'send':
            channels.send(cid, b'spam')
            return state.incr()
        elif end == 'recv':
            if not state.pending:
                try:
                    channels.recv(cid)
                except channels.ChannelEmptyError:
                    return state
                else:
                    raise Exception('expected ChannelEmptyError')
            else:
                channels.recv(cid)
                return state.decr()
        else:
            raise ValueError(end)
    elif action == 'close':
        kwargs = {}
        if end in ('recv', 'send'):
            kwargs[end] = True
        channels.close(cid, **kwargs)
        return state.close()
    elif action == 'force-close':
        kwargs = {'force': True}
        if end in ('recv', 'send'):
            kwargs[end] = True
        channels.close(cid, **kwargs)
        return state.close(force=True)
    else:
        raise ValueError(action)

def clean_up_channels():
    for cid in channels.list_all():
        try:
            channels.destroy(cid)
        except channels.ChannelNotFoundError:
            pass

class ChannelCloseFixture(namedtuple('ChannelCloseFixture', 'end interp other extra creator')):
    QUICK = False

    def __new__(cls, end, interp, other, extra, creator):
        assert end in ('send', 'recv')
        if cls.QUICK:
            known = {}
        else:
            interp = Interpreter.from_raw(interp)
            other = Interpreter.from_raw(other)
            extra = Interpreter.from_raw(extra)
            known = {interp.name: interp, other.name: other, extra.name: extra}
        if not creator:
            creator = 'same'
        self = super().__new__(cls, end, interp, other, extra, creator)
        self._prepped = set()
        self._state = ChannelState()
        self._known = known
        return self

    @property
    def state(self):
        return self._state

    @property
    def cid(self):
        try:
            return self._cid
        except AttributeError:
            creator = self._get_interpreter(self.creator)
            self._cid = self._new_channel(creator)
            return self._cid

    def get_interpreter(self, interp):
        interp = self._get_interpreter(interp)
        self._prep_interpreter(interp)
        return interp

    def expect_closed_error(self, end=None):
        if end is None:
            end = self.end
        if end == 'recv' and self.state.closed == 'send':
            return False
        return bool(self.state.closed)

    def prep_interpreter(self, interp):
        self._prep_interpreter(interp)

    def record_action(self, action, result):
        self._state = result

    def clean_up(self):
        clean_up_interpreters()
        clean_up_channels()

    def _new_channel(self, creator):
        if creator.name == 'main':
            return channels.create()
        else:
            ch = channels.create()
            run_interp(creator.id, f'\n                import _xxsubinterpreters\n                cid = _xxsubchannels.create()\n                # We purposefully send back an int to avoid tying the\n                # channel to the other interpreter.\n                _xxsubchannels.send({ch}, int(cid))\n                del _xxsubinterpreters\n                ')
            self._cid = channels.recv(ch)
        return self._cid

    def _get_interpreter(self, interp):
        if interp in ('same', 'interp'):
            return self.interp
        elif interp == 'other':
            return self.other
        elif interp == 'extra':
            return self.extra
        else:
            name = interp
            try:
                interp = self._known[name]
            except KeyError:
                interp = self._known[name] = Interpreter(name)
            return interp

    def _prep_interpreter(self, interp):
        if interp.id in self._prepped:
            return
        self._prepped.add(interp.id)
        if interp.name == 'main':
            return
        run_interp(interp.id, f'\n            import _xxinterpchannels as channels\n            import test.test__xxinterpchannels as helpers\n            ChannelState = helpers.ChannelState\n            try:\n                cid\n            except NameError:\n                cid = channels._channel_id({self.cid})\n            ')


# --- test body ---
"""Test listing channel interpreters with a released channel."""
interp0 = interpreters.get_main()
interp1 = interpreters.create()
interp2 = interpreters.create()
cid = channels.create()
channels.send(cid, 'data')
_run_output(interp1, dedent(f'\n            import _xxinterpchannels as _channels\n            obj = _channels.recv({cid})\n            '))
channels.send(cid, 'data')
_run_output(interp2, dedent(f'\n            import _xxinterpchannels as _channels\n            obj = _channels.recv({cid})\n            '))
send_interps = channels.list_interpreters(cid, send=True)
recv_interps = channels.list_interpreters(cid, send=False)

assert len(send_interps) == 1

assert len(recv_interps) == 2
channels.release(cid, send=True)
send_interps = channels.list_interpreters(cid, send=True)
recv_interps = channels.list_interpreters(cid, send=False)

assert len(send_interps) == 0

assert len(recv_interps) == 2
_run_output(interp2, dedent(f'\n            import _xxinterpchannels as _channels\n            _channels.release({cid})\n            '))
send_interps = channels.list_interpreters(cid, send=True)
recv_interps = channels.list_interpreters(cid, send=False)

assert len(send_interps) == 0

assert recv_interps == [interp1]
print("ChannelTests::test_channel_list_interpreters_released: ok")
