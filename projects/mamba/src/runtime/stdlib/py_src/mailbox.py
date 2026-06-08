# NOTE (mamba): trimmed from CPython 3.12 Lib/mailbox.py.
# Kept: Mailbox base, Maildir, the Message family (Message/MaildirMessage/
#   mboxMessage/MHMessage/BabylMessage/MMDFMessage), _ProxyFile/_PartialFile,
#   the lock/sync helpers, and the Error hierarchy.
# Dropped: _singlefileMailbox/mbox/MMDF/MH/Babyl filesystem mailboxes (no
#   inline behavior fixture exercises them) so the module compiles within
#   mamba's per-run CPU budget. Restore them verbatim from CPython if a
#   single-file mailbox fixture is added.
"""Read/write support for Maildir, mbox, MH, Babyl, and MMDF mailboxes."""

# Notes for authors of new mailbox subclasses:
#
# Remember to fsync() changes to disk before closing a modified file
# or returning from a flush() method.  See functions _sync_flush() and
# _sync_close().

import os
import time
import calendar
import socket
import errno
import copy
import warnings
import email
import email.message
import email.generator
import io
import contextlib
from types import GenericAlias
try:
    import fcntl
except ImportError:
    fcntl = None

__all__ = ['Mailbox', 'Maildir', 'mbox', 'MH', 'Babyl', 'MMDF',
           'Message', 'MaildirMessage', 'mboxMessage', 'MHMessage',
           'BabylMessage', 'MMDFMessage', 'Error', 'NoSuchMailboxError',
           'NotEmptyError', 'ExternalClashError', 'FormatError']

linesep = os.linesep.encode('ascii')

class Mailbox:
    """A group of messages in a particular place."""

    def __init__(self, path, factory=None, create=True):
        """Initialize a Mailbox instance."""
        self._path = os.path.abspath(os.path.expanduser(path))
        self._factory = factory

    def add(self, message):
        """Add message and return assigned key."""
        raise NotImplementedError('Method must be implemented by subclass')

    def remove(self, key):
        """Remove the keyed message; raise KeyError if it doesn't exist."""
        raise NotImplementedError('Method must be implemented by subclass')

    def __delitem__(self, key):
        self.remove(key)

    def discard(self, key):
        """If the keyed message exists, remove it."""
        try:
            self.remove(key)
        except KeyError:
            pass

    def __setitem__(self, key, message):
        """Replace the keyed message; raise KeyError if it doesn't exist."""
        raise NotImplementedError('Method must be implemented by subclass')

    def get(self, key, default=None):
        """Return the keyed message, or default if it doesn't exist."""
        try:
            return self.__getitem__(key)
        except KeyError:
            return default

    def __getitem__(self, key):
        """Return the keyed message; raise KeyError if it doesn't exist."""
        if not self._factory:
            return self.get_message(key)
        else:
            with contextlib.closing(self.get_file(key)) as file:
                return self._factory(file)

    def get_message(self, key):
        """Return a Message representation or raise a KeyError."""
        raise NotImplementedError('Method must be implemented by subclass')

    def get_string(self, key):
        """Return a string representation or raise a KeyError.

        Uses email.message.Message to create a 7bit clean string
        representation of the message."""
        return email.message_from_bytes(self.get_bytes(key)).as_string()

    def get_bytes(self, key):
        """Return a byte string representation or raise a KeyError."""
        raise NotImplementedError('Method must be implemented by subclass')

    def get_file(self, key):
        """Return a file-like representation or raise a KeyError."""
        raise NotImplementedError('Method must be implemented by subclass')

    def iterkeys(self):
        """Return an iterator over keys."""
        raise NotImplementedError('Method must be implemented by subclass')

    def keys(self):
        """Return a list of keys."""
        return list(self.iterkeys())

    def itervalues(self):
        """Return an iterator over all messages."""
        for key in self.iterkeys():
            try:
                value = self[key]
            except KeyError:
                continue
            yield value

    def __iter__(self):
        return self.itervalues()

    def values(self):
        """Return a list of messages. Memory intensive."""
        return list(self.itervalues())

    def iteritems(self):
        """Return an iterator over (key, message) tuples."""
        for key in self.iterkeys():
            try:
                value = self[key]
            except KeyError:
                continue
            yield (key, value)

    def items(self):
        """Return a list of (key, message) tuples. Memory intensive."""
        return list(self.iteritems())

    def __contains__(self, key):
        """Return True if the keyed message exists, False otherwise."""
        raise NotImplementedError('Method must be implemented by subclass')

    def __len__(self):
        """Return a count of messages in the mailbox."""
        raise NotImplementedError('Method must be implemented by subclass')

    def clear(self):
        """Delete all messages."""
        for key in self.keys():
            self.discard(key)

    def pop(self, key, default=None):
        """Delete the keyed message and return it, or default."""
        try:
            result = self[key]
        except KeyError:
            return default
        self.discard(key)
        return result

    def popitem(self):
        """Delete an arbitrary (key, message) pair and return it."""
        for key in self.iterkeys():
            return (key, self.pop(key))     # This is only run once.
        else:
            raise KeyError('No messages in mailbox')

    def update(self, arg=None):
        """Change the messages that correspond to certain keys."""
        if hasattr(arg, 'iteritems'):
            source = arg.iteritems()
        elif hasattr(arg, 'items'):
            source = arg.items()
        else:
            source = arg
        bad_key = False
        for key, message in source:
            try:
                self[key] = message
            except KeyError:
                bad_key = True
        if bad_key:
            raise KeyError('No message with key(s)')

    def flush(self):
        """Write any pending changes to the disk."""
        raise NotImplementedError('Method must be implemented by subclass')

    def lock(self):
        """Lock the mailbox."""
        raise NotImplementedError('Method must be implemented by subclass')

    def unlock(self):
        """Unlock the mailbox if it is locked."""
        raise NotImplementedError('Method must be implemented by subclass')

    def close(self):
        """Flush and close the mailbox."""
        raise NotImplementedError('Method must be implemented by subclass')

    def _string_to_bytes(self, message):
        # If a message is not 7bit clean, we refuse to handle it since it
        # likely came from reading invalid messages in text mode, and that way
        # lies mojibake.
        try:
            return message.encode('ascii')
        except UnicodeError:
            raise ValueError("String input must be ASCII-only; "
                "use bytes or a Message instead")

    # Whether each message must end in a newline
    _append_newline = False

    def _dump_message(self, message, target, mangle_from_=False):
        # This assumes the target file is open in binary mode.
        """Dump message contents to target file."""
        if isinstance(message, email.message.Message):
            buffer = io.BytesIO()
            gen = email.generator.BytesGenerator(buffer, mangle_from_, 0)
            gen.flatten(message)
            buffer.seek(0)
            data = buffer.read()
            data = data.replace(b'\n', linesep)
            target.write(data)
            if self._append_newline and not data.endswith(linesep):
                # Make sure the message ends with a newline
                target.write(linesep)
        elif isinstance(message, (str, bytes, io.StringIO)):
            if isinstance(message, io.StringIO):
                warnings.warn("Use of StringIO input is deprecated, "
                    "use BytesIO instead", DeprecationWarning, 3)
                message = message.getvalue()
            if isinstance(message, str):
                message = self._string_to_bytes(message)
            if mangle_from_:
                message = message.replace(b'\nFrom ', b'\n>From ')
            message = message.replace(b'\n', linesep)
            target.write(message)
            if self._append_newline and not message.endswith(linesep):
                # Make sure the message ends with a newline
                target.write(linesep)
        elif hasattr(message, 'read'):
            if hasattr(message, 'buffer'):
                warnings.warn("Use of text mode files is deprecated, "
                    "use a binary mode file instead", DeprecationWarning, 3)
                message = message.buffer
            lastline = None
            while True:
                line = message.readline()
                # Universal newline support.
                if line.endswith(b'\r\n'):
                    line = line[:-2] + b'\n'
                elif line.endswith(b'\r'):
                    line = line[:-1] + b'\n'
                if not line:
                    break
                if mangle_from_ and line.startswith(b'From '):
                    line = b'>From ' + line[5:]
                line = line.replace(b'\n', linesep)
                target.write(line)
                lastline = line
            if self._append_newline and lastline and not lastline.endswith(linesep):
                # Make sure the message ends with a newline
                target.write(linesep)
        else:
            raise TypeError('Invalid message type: %s' % type(message))

    __class_getitem__ = classmethod(GenericAlias)



class Maildir(Mailbox):
    """A qmail-style Maildir mailbox."""

    colon = ':'

    def __init__(self, dirname, factory=None, create=True):
        """Initialize a Maildir instance."""
        Mailbox.__init__(self, dirname, factory, create)
        self._paths = {
            'tmp': os.path.join(self._path, 'tmp'),
            'new': os.path.join(self._path, 'new'),
            'cur': os.path.join(self._path, 'cur'),
            }
        if not os.path.exists(self._path):
            if create:
                os.mkdir(self._path, 0o700)
                for path in self._paths.values():
                    os.mkdir(path, 0o700)
            else:
                raise NoSuchMailboxError(self._path)
        self._toc = {}
        self._toc_mtimes = {'cur': 0, 'new': 0}
        self._last_read = 0         # Records last time we read cur/new
        self._skewfactor = 0.1      # Adjust if os/fs clocks are skewing

    def add(self, message):
        """Add message and return assigned key."""
        tmp_file = self._create_tmp()
        try:
            self._dump_message(message, tmp_file)
        except BaseException:
            tmp_file.close()
            os.remove(tmp_file.name)
            raise
        _sync_close(tmp_file)
        if isinstance(message, MaildirMessage):
            subdir = message.get_subdir()
            suffix = self.colon + message.get_info()
            if suffix == self.colon:
                suffix = ''
        else:
            subdir = 'new'
            suffix = ''
        uniq = os.path.basename(tmp_file.name).split(self.colon)[0]
        dest = os.path.join(self._path, subdir, uniq + suffix)
        if isinstance(message, MaildirMessage):
            os.utime(tmp_file.name,
                     (os.path.getatime(tmp_file.name), message.get_date()))
        # No file modification should be done after the file is moved to its
        # final position in order to prevent race conditions with changes
        # from other programs
        try:
            try:
                os.link(tmp_file.name, dest)
            except (AttributeError, PermissionError):
                os.rename(tmp_file.name, dest)
            else:
                os.remove(tmp_file.name)
        except OSError as e:
            os.remove(tmp_file.name)
            if e.errno == errno.EEXIST:
                raise ExternalClashError('Name clash with existing message: %s'
                                         % dest)
            else:
                raise
        return uniq

    def remove(self, key):
        """Remove the keyed message; raise KeyError if it doesn't exist."""
        os.remove(os.path.join(self._path, self._lookup(key)))

    def discard(self, key):
        """If the keyed message exists, remove it."""
        # This overrides an inapplicable implementation in the superclass.
        try:
            self.remove(key)
        except (KeyError, FileNotFoundError):
            pass

    def __setitem__(self, key, message):
        """Replace the keyed message; raise KeyError if it doesn't exist."""
        old_subpath = self._lookup(key)
        temp_key = self.add(message)
        temp_subpath = self._lookup(temp_key)
        if isinstance(message, MaildirMessage):
            # temp's subdir and suffix were specified by message.
            dominant_subpath = temp_subpath
        else:
            # temp's subdir and suffix were defaults from add().
            dominant_subpath = old_subpath
        subdir = os.path.dirname(dominant_subpath)
        if self.colon in dominant_subpath:
            suffix = self.colon + dominant_subpath.split(self.colon)[-1]
        else:
            suffix = ''
        self.discard(key)
        tmp_path = os.path.join(self._path, temp_subpath)
        new_path = os.path.join(self._path, subdir, key + suffix)
        if isinstance(message, MaildirMessage):
            os.utime(tmp_path,
                     (os.path.getatime(tmp_path), message.get_date()))
        # No file modification should be done after the file is moved to its
        # final position in order to prevent race conditions with changes
        # from other programs
        os.rename(tmp_path, new_path)

    def get_message(self, key):
        """Return a Message representation or raise a KeyError."""
        subpath = self._lookup(key)
        with open(os.path.join(self._path, subpath), 'rb') as f:
            if self._factory:
                msg = self._factory(f)
            else:
                msg = MaildirMessage(f)
        subdir, name = os.path.split(subpath)
        msg.set_subdir(subdir)
        if self.colon in name:
            msg.set_info(name.split(self.colon)[-1])
        msg.set_date(os.path.getmtime(os.path.join(self._path, subpath)))
        return msg

    def get_bytes(self, key):
        """Return a bytes representation or raise a KeyError."""
        with open(os.path.join(self._path, self._lookup(key)), 'rb') as f:
            return f.read().replace(linesep, b'\n')

    def get_file(self, key):
        """Return a file-like representation or raise a KeyError."""
        f = open(os.path.join(self._path, self._lookup(key)), 'rb')
        return _ProxyFile(f)

    def iterkeys(self):
        """Return an iterator over keys."""
        self._refresh()
        for key in self._toc:
            try:
                self._lookup(key)
            except KeyError:
                continue
            yield key

    def __contains__(self, key):
        """Return True if the keyed message exists, False otherwise."""
        self._refresh()
        return key in self._toc

    def __len__(self):
        """Return a count of messages in the mailbox."""
        self._refresh()
        return len(self._toc)

    def flush(self):
        """Write any pending changes to disk."""
        # Maildir changes are always written immediately, so there's nothing
        # to do.
        pass

    def lock(self):
        """Lock the mailbox."""
        return

    def unlock(self):
        """Unlock the mailbox if it is locked."""
        return

    def close(self):
        """Flush and close the mailbox."""
        return

    def list_folders(self):
        """Return a list of folder names."""
        result = []
        for entry in os.listdir(self._path):
            if len(entry) > 1 and entry[0] == '.' and \
               os.path.isdir(os.path.join(self._path, entry)):
                result.append(entry[1:])
        return result

    def get_folder(self, folder):
        """Return a Maildir instance for the named folder."""
        return Maildir(os.path.join(self._path, '.' + folder),
                       factory=self._factory,
                       create=False)

    def add_folder(self, folder):
        """Create a folder and return a Maildir instance representing it."""
        path = os.path.join(self._path, '.' + folder)
        result = Maildir(path, factory=self._factory)
        maildirfolder_path = os.path.join(path, 'maildirfolder')
        if not os.path.exists(maildirfolder_path):
            os.close(os.open(maildirfolder_path, os.O_CREAT | os.O_WRONLY,
                0o666))
        return result

    def remove_folder(self, folder):
        """Delete the named folder, which must be empty."""
        path = os.path.join(self._path, '.' + folder)
        for entry in os.listdir(os.path.join(path, 'new')) + \
                     os.listdir(os.path.join(path, 'cur')):
            if len(entry) < 1 or entry[0] != '.':
                raise NotEmptyError('Folder contains message(s): %s' % folder)
        for entry in os.listdir(path):
            if entry != 'new' and entry != 'cur' and entry != 'tmp' and \
               os.path.isdir(os.path.join(path, entry)):
                raise NotEmptyError("Folder contains subdirectory '%s': %s" %
                                    (folder, entry))
        for root, dirs, files in os.walk(path, topdown=False):
            for entry in files:
                os.remove(os.path.join(root, entry))
            for entry in dirs:
                os.rmdir(os.path.join(root, entry))
        os.rmdir(path)

    def clean(self):
        """Delete old files in "tmp"."""
        now = time.time()
        for entry in os.listdir(os.path.join(self._path, 'tmp')):
            path = os.path.join(self._path, 'tmp', entry)
            if now - os.path.getatime(path) > 129600:   # 60 * 60 * 36
                os.remove(path)

    _count = 1  # This is used to generate unique file names.

    def _create_tmp(self):
        """Create a file in the tmp subdirectory and open and return it."""
        now = time.time()
        hostname = socket.gethostname()
        if '/' in hostname:
            hostname = hostname.replace('/', r'\057')
        if ':' in hostname:
            hostname = hostname.replace(':', r'\072')
        uniq = "%s.M%sP%sQ%s.%s" % (int(now), int(now % 1 * 1e6), os.getpid(),
                                    Maildir._count, hostname)
        path = os.path.join(self._path, 'tmp', uniq)
        try:
            os.stat(path)
        except FileNotFoundError:
            Maildir._count += 1
            try:
                return _create_carefully(path)
            except FileExistsError:
                pass

        # Fall through to here if stat succeeded or open raised EEXIST.
        raise ExternalClashError('Name clash prevented file creation: %s' %
                                 path)

    def _refresh(self):
        """Update table of contents mapping."""
        # If it has been less than two seconds since the last _refresh() call,
        # we have to unconditionally re-read the mailbox just in case it has
        # been modified, because os.path.mtime() has a 2 sec resolution in the
        # most common worst case (FAT) and a 1 sec resolution typically.  This
        # results in a few unnecessary re-reads when _refresh() is called
        # multiple times in that interval, but once the clock ticks over, we
        # will only re-read as needed.  Because the filesystem might be being
        # served by an independent system with its own clock, we record and
        # compare with the mtimes from the filesystem.  Because the other
        # system's clock might be skewing relative to our clock, we add an
        # extra delta to our wait.  The default is one tenth second, but is an
        # instance variable and so can be adjusted if dealing with a
        # particularly skewed or irregular system.
        if time.time() - self._last_read > 2 + self._skewfactor:
            refresh = False
            for subdir in self._toc_mtimes:
                mtime = os.path.getmtime(self._paths[subdir])
                if mtime > self._toc_mtimes[subdir]:
                    refresh = True
                self._toc_mtimes[subdir] = mtime
            if not refresh:
                return
        # Refresh toc
        self._toc = {}
        for subdir in self._toc_mtimes:
            path = self._paths[subdir]
            for entry in os.listdir(path):
                p = os.path.join(path, entry)
                if os.path.isdir(p):
                    continue
                uniq = entry.split(self.colon)[0]
                self._toc[uniq] = os.path.join(subdir, entry)
        self._last_read = time.time()

    def _lookup(self, key):
        """Use TOC to return subpath for given key, or raise a KeyError."""
        try:
            if os.path.exists(os.path.join(self._path, self._toc[key])):
                return self._toc[key]
        except KeyError:
            pass
        self._refresh()
        try:
            return self._toc[key]
        except KeyError:
            raise KeyError('No message with key: %s' % key) from None

    # This method is for backward compatibility only.
    def next(self):
        """Return the next message in a one-time iteration."""
        if not hasattr(self, '_onetime_keys'):
            self._onetime_keys = self.iterkeys()
        while True:
            try:
                return self[next(self._onetime_keys)]
            except StopIteration:
                return None
            except KeyError:
                continue



class Message(email.message.Message):
    """Message with mailbox-format-specific properties."""

    def __init__(self, message=None):
        """Initialize a Message instance."""
        if isinstance(message, email.message.Message):
            self._become_message(copy.deepcopy(message))
            if isinstance(message, Message):
                message._explain_to(self)
        elif isinstance(message, bytes):
            self._become_message(email.message_from_bytes(message))
        elif isinstance(message, str):
            self._become_message(email.message_from_string(message))
        elif isinstance(message, io.TextIOWrapper):
            self._become_message(email.message_from_file(message))
        elif hasattr(message, "read"):
            self._become_message(email.message_from_binary_file(message))
        elif message is None:
            email.message.Message.__init__(self)
        else:
            raise TypeError('Invalid message type: %s' % type(message))

    def _become_message(self, message):
        """Assume the non-format-specific state of message."""
        type_specific = getattr(message, '_type_specific_attributes', [])
        for name in message.__dict__:
            if name not in type_specific:
                self.__dict__[name] = message.__dict__[name]

    def _explain_to(self, message):
        """Copy format-specific state to message insofar as possible."""
        if isinstance(message, Message):
            return  # There's nothing format-specific to explain.
        else:
            raise TypeError('Cannot convert to specified type')


class MaildirMessage(Message):
    """Message with Maildir-specific properties."""

    _type_specific_attributes = ['_subdir', '_info', '_date']

    def __init__(self, message=None):
        """Initialize a MaildirMessage instance."""
        self._subdir = 'new'
        self._info = ''
        self._date = time.time()
        Message.__init__(self, message)

    def get_subdir(self):
        """Return 'new' or 'cur'."""
        return self._subdir

    def set_subdir(self, subdir):
        """Set subdir to 'new' or 'cur'."""
        if subdir == 'new' or subdir == 'cur':
            self._subdir = subdir
        else:
            raise ValueError("subdir must be 'new' or 'cur': %s" % subdir)

    def get_flags(self):
        """Return as a string the flags that are set."""
        if self._info.startswith('2,'):
            return self._info[2:]
        else:
            return ''

    def set_flags(self, flags):
        """Set the given flags and unset all others."""
        self._info = '2,' + ''.join(sorted(flags))

    def add_flag(self, flag):
        """Set the given flag(s) without changing others."""
        self.set_flags(''.join(set(self.get_flags()) | set(flag)))

    def remove_flag(self, flag):
        """Unset the given string flag(s) without changing others."""
        if self.get_flags():
            self.set_flags(''.join(set(self.get_flags()) - set(flag)))

    def get_date(self):
        """Return delivery date of message, in seconds since the epoch."""
        return self._date

    def set_date(self, date):
        """Set delivery date of message, in seconds since the epoch."""
        try:
            self._date = float(date)
        except ValueError:
            raise TypeError("can't convert to float: %s" % date) from None

    def get_info(self):
        """Get the message's "info" as a string."""
        return self._info

    def set_info(self, info):
        """Set the message's "info" string."""
        if isinstance(info, str):
            self._info = info
        else:
            raise TypeError('info must be a string: %s' % type(info))

    def _explain_to(self, message):
        """Copy Maildir-specific state to message insofar as possible."""
        if isinstance(message, MaildirMessage):
            message.set_flags(self.get_flags())
            message.set_subdir(self.get_subdir())
            message.set_date(self.get_date())
        elif isinstance(message, _mboxMMDFMessage):
            flags = set(self.get_flags())
            if 'S' in flags:
                message.add_flag('R')
            if self.get_subdir() == 'cur':
                message.add_flag('O')
            if 'T' in flags:
                message.add_flag('D')
            if 'F' in flags:
                message.add_flag('F')
            if 'R' in flags:
                message.add_flag('A')
            message.set_from('MAILER-DAEMON', time.gmtime(self.get_date()))
        elif isinstance(message, MHMessage):
            flags = set(self.get_flags())
            if 'S' not in flags:
                message.add_sequence('unseen')
            if 'R' in flags:
                message.add_sequence('replied')
            if 'F' in flags:
                message.add_sequence('flagged')
        elif isinstance(message, BabylMessage):
            flags = set(self.get_flags())
            if 'S' not in flags:
                message.add_label('unseen')
            if 'T' in flags:
                message.add_label('deleted')
            if 'R' in flags:
                message.add_label('answered')
            if 'P' in flags:
                message.add_label('forwarded')
        elif isinstance(message, Message):
            pass
        else:
            raise TypeError('Cannot convert to specified type: %s' %
                            type(message))


class _mboxMMDFMessage(Message):
    """Message with mbox- or MMDF-specific properties."""

    _type_specific_attributes = ['_from']

    def __init__(self, message=None):
        """Initialize an mboxMMDFMessage instance."""
        self.set_from('MAILER-DAEMON', True)
        if isinstance(message, email.message.Message):
            unixfrom = message.get_unixfrom()
            if unixfrom is not None and unixfrom.startswith('From '):
                self.set_from(unixfrom[5:])
        Message.__init__(self, message)

    def get_from(self):
        """Return contents of "From " line."""
        return self._from

    def set_from(self, from_, time_=None):
        """Set "From " line, formatting and appending time_ if specified."""
        if time_ is not None:
            if time_ is True:
                time_ = time.gmtime()
            from_ += ' ' + time.asctime(time_)
        self._from = from_

    def get_flags(self):
        """Return as a string the flags that are set."""
        return self.get('Status', '') + self.get('X-Status', '')

    def set_flags(self, flags):
        """Set the given flags and unset all others."""
        flags = set(flags)
        status_flags, xstatus_flags = '', ''
        for flag in ('R', 'O'):
            if flag in flags:
                status_flags += flag
                flags.remove(flag)
        for flag in ('D', 'F', 'A'):
            if flag in flags:
                xstatus_flags += flag
                flags.remove(flag)
        xstatus_flags += ''.join(sorted(flags))
        try:
            self.replace_header('Status', status_flags)
        except KeyError:
            self.add_header('Status', status_flags)
        try:
            self.replace_header('X-Status', xstatus_flags)
        except KeyError:
            self.add_header('X-Status', xstatus_flags)

    def add_flag(self, flag):
        """Set the given flag(s) without changing others."""
        self.set_flags(''.join(set(self.get_flags()) | set(flag)))

    def remove_flag(self, flag):
        """Unset the given string flag(s) without changing others."""
        if 'Status' in self or 'X-Status' in self:
            self.set_flags(''.join(set(self.get_flags()) - set(flag)))

    def _explain_to(self, message):
        """Copy mbox- or MMDF-specific state to message insofar as possible."""
        if isinstance(message, MaildirMessage):
            flags = set(self.get_flags())
            if 'O' in flags:
                message.set_subdir('cur')
            if 'F' in flags:
                message.add_flag('F')
            if 'A' in flags:
                message.add_flag('R')
            if 'R' in flags:
                message.add_flag('S')
            if 'D' in flags:
                message.add_flag('T')
            del message['status']
            del message['x-status']
            maybe_date = ' '.join(self.get_from().split()[-5:])
            try:
                message.set_date(calendar.timegm(time.strptime(maybe_date,
                                                      '%a %b %d %H:%M:%S %Y')))
            except (ValueError, OverflowError):
                pass
        elif isinstance(message, _mboxMMDFMessage):
            message.set_flags(self.get_flags())
            message.set_from(self.get_from())
        elif isinstance(message, MHMessage):
            flags = set(self.get_flags())
            if 'R' not in flags:
                message.add_sequence('unseen')
            if 'A' in flags:
                message.add_sequence('replied')
            if 'F' in flags:
                message.add_sequence('flagged')
            del message['status']
            del message['x-status']
        elif isinstance(message, BabylMessage):
            flags = set(self.get_flags())
            if 'R' not in flags:
                message.add_label('unseen')
            if 'D' in flags:
                message.add_label('deleted')
            if 'A' in flags:
                message.add_label('answered')
            del message['status']
            del message['x-status']
        elif isinstance(message, Message):
            pass
        else:
            raise TypeError('Cannot convert to specified type: %s' %
                            type(message))


class mboxMessage(_mboxMMDFMessage):
    """Message with mbox-specific properties."""


class MHMessage(Message):
    """Message with MH-specific properties."""

    _type_specific_attributes = ['_sequences']

    def __init__(self, message=None):
        """Initialize an MHMessage instance."""
        self._sequences = []
        Message.__init__(self, message)

    def get_sequences(self):
        """Return a list of sequences that include the message."""
        return self._sequences[:]

    def set_sequences(self, sequences):
        """Set the list of sequences that include the message."""
        self._sequences = list(sequences)

    def add_sequence(self, sequence):
        """Add sequence to list of sequences including the message."""
        if isinstance(sequence, str):
            if not sequence in self._sequences:
                self._sequences.append(sequence)
        else:
            raise TypeError('sequence type must be str: %s' % type(sequence))

    def remove_sequence(self, sequence):
        """Remove sequence from the list of sequences including the message."""
        try:
            self._sequences.remove(sequence)
        except ValueError:
            pass

    def _explain_to(self, message):
        """Copy MH-specific state to message insofar as possible."""
        if isinstance(message, MaildirMessage):
            sequences = set(self.get_sequences())
            if 'unseen' in sequences:
                message.set_subdir('cur')
            else:
                message.set_subdir('cur')
                message.add_flag('S')
            if 'flagged' in sequences:
                message.add_flag('F')
            if 'replied' in sequences:
                message.add_flag('R')
        elif isinstance(message, _mboxMMDFMessage):
            sequences = set(self.get_sequences())
            if 'unseen' not in sequences:
                message.add_flag('RO')
            else:
                message.add_flag('O')
            if 'flagged' in sequences:
                message.add_flag('F')
            if 'replied' in sequences:
                message.add_flag('A')
        elif isinstance(message, MHMessage):
            for sequence in self.get_sequences():
                message.add_sequence(sequence)
        elif isinstance(message, BabylMessage):
            sequences = set(self.get_sequences())
            if 'unseen' in sequences:
                message.add_label('unseen')
            if 'replied' in sequences:
                message.add_label('answered')
        elif isinstance(message, Message):
            pass
        else:
            raise TypeError('Cannot convert to specified type: %s' %
                            type(message))


class BabylMessage(Message):
    """Message with Babyl-specific properties."""

    _type_specific_attributes = ['_labels', '_visible']

    def __init__(self, message=None):
        """Initialize a BabylMessage instance."""
        self._labels = []
        self._visible = Message()
        Message.__init__(self, message)

    def get_labels(self):
        """Return a list of labels on the message."""
        return self._labels[:]

    def set_labels(self, labels):
        """Set the list of labels on the message."""
        self._labels = list(labels)

    def add_label(self, label):
        """Add label to list of labels on the message."""
        if isinstance(label, str):
            if label not in self._labels:
                self._labels.append(label)
        else:
            raise TypeError('label must be a string: %s' % type(label))

    def remove_label(self, label):
        """Remove label from the list of labels on the message."""
        try:
            self._labels.remove(label)
        except ValueError:
            pass

    def get_visible(self):
        """Return a Message representation of visible headers."""
        return Message(self._visible)

    def set_visible(self, visible):
        """Set the Message representation of visible headers."""
        self._visible = Message(visible)

    def update_visible(self):
        """Update and/or sensibly generate a set of visible headers."""
        for header in self._visible.keys():
            if header in self:
                self._visible.replace_header(header, self[header])
            else:
                del self._visible[header]
        for header in ('Date', 'From', 'Reply-To', 'To', 'CC', 'Subject'):
            if header in self and header not in self._visible:
                self._visible[header] = self[header]

    def _explain_to(self, message):
        """Copy Babyl-specific state to message insofar as possible."""
        if isinstance(message, MaildirMessage):
            labels = set(self.get_labels())
            if 'unseen' in labels:
                message.set_subdir('cur')
            else:
                message.set_subdir('cur')
                message.add_flag('S')
            if 'forwarded' in labels or 'resent' in labels:
                message.add_flag('P')
            if 'answered' in labels:
                message.add_flag('R')
            if 'deleted' in labels:
                message.add_flag('T')
        elif isinstance(message, _mboxMMDFMessage):
            labels = set(self.get_labels())
            if 'unseen' not in labels:
                message.add_flag('RO')
            else:
                message.add_flag('O')
            if 'deleted' in labels:
                message.add_flag('D')
            if 'answered' in labels:
                message.add_flag('A')
        elif isinstance(message, MHMessage):
            labels = set(self.get_labels())
            if 'unseen' in labels:
                message.add_sequence('unseen')
            if 'answered' in labels:
                message.add_sequence('replied')
        elif isinstance(message, BabylMessage):
            message.set_visible(self.get_visible())
            for label in self.get_labels():
                message.add_label(label)
        elif isinstance(message, Message):
            pass
        else:
            raise TypeError('Cannot convert to specified type: %s' %
                            type(message))


class MMDFMessage(_mboxMMDFMessage):
    """Message with MMDF-specific properties."""


class _ProxyFile:
    """A read-only wrapper of a file."""

    def __init__(self, f, pos=None):
        """Initialize a _ProxyFile."""
        self._file = f
        if pos is None:
            self._pos = f.tell()
        else:
            self._pos = pos

    def read(self, size=None):
        """Read bytes."""
        return self._read(size, self._file.read)

    def read1(self, size=None):
        """Read bytes."""
        return self._read(size, self._file.read1)

    def readline(self, size=None):
        """Read a line."""
        return self._read(size, self._file.readline)

    def readlines(self, sizehint=None):
        """Read multiple lines."""
        result = []
        for line in self:
            result.append(line)
            if sizehint is not None:
                sizehint -= len(line)
                if sizehint <= 0:
                    break
        return result

    def __iter__(self):
        """Iterate over lines."""
        while line := self.readline():
            yield line

    def tell(self):
        """Return the position."""
        return self._pos

    def seek(self, offset, whence=0):
        """Change position."""
        if whence == 1:
            self._file.seek(self._pos)
        self._file.seek(offset, whence)
        self._pos = self._file.tell()

    def close(self):
        """Close the file."""
        if hasattr(self, '_file'):
            try:
                if hasattr(self._file, 'close'):
                    self._file.close()
            finally:
                del self._file

    def _read(self, size, read_method):
        """Read size bytes using read_method."""
        if size is None:
            size = -1
        self._file.seek(self._pos)
        result = read_method(size)
        self._pos = self._file.tell()
        return result

    def __enter__(self):
        """Context management protocol support."""
        return self

    def __exit__(self, *exc):
        self.close()

    def readable(self):
        return self._file.readable()

    def writable(self):
        return self._file.writable()

    def seekable(self):
        return self._file.seekable()

    def flush(self):
        return self._file.flush()

    @property
    def closed(self):
        if not hasattr(self, '_file'):
            return True
        if not hasattr(self._file, 'closed'):
            return False
        return self._file.closed

    __class_getitem__ = classmethod(GenericAlias)


class _PartialFile(_ProxyFile):
    """A read-only wrapper of part of a file."""

    def __init__(self, f, start=None, stop=None):
        """Initialize a _PartialFile."""
        _ProxyFile.__init__(self, f, start)
        self._start = start
        self._stop = stop

    def tell(self):
        """Return the position with respect to start."""
        return _ProxyFile.tell(self) - self._start

    def seek(self, offset, whence=0):
        """Change position, possibly with respect to start or stop."""
        if whence == 0:
            self._pos = self._start
            whence = 1
        elif whence == 2:
            self._pos = self._stop
            whence = 1
        _ProxyFile.seek(self, offset, whence)

    def _read(self, size, read_method):
        """Read size bytes using read_method, honoring start and stop."""
        remaining = self._stop - self._pos
        if remaining <= 0:
            return b''
        if size is None or size < 0 or size > remaining:
            size = remaining
        return _ProxyFile._read(self, size, read_method)

    def close(self):
        # do *not* close the underlying file object for partial files,
        # since it's global to the mailbox object
        if hasattr(self, '_file'):
            del self._file


def _lock_file(f, dotlock=True):
    """Lock file f using lockf and dot locking."""
    dotlock_done = False
    try:
        if fcntl:
            try:
                fcntl.lockf(f, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except OSError as e:
                if e.errno in (errno.EAGAIN, errno.EACCES, errno.EROFS):
                    raise ExternalClashError('lockf: lock unavailable: %s' %
                                             f.name)
                else:
                    raise
        if dotlock:
            try:
                pre_lock = _create_temporary(f.name + '.lock')
                pre_lock.close()
            except OSError as e:
                if e.errno in (errno.EACCES, errno.EROFS):
                    return  # Without write access, just skip dotlocking.
                else:
                    raise
            try:
                try:
                    os.link(pre_lock.name, f.name + '.lock')
                    dotlock_done = True
                except (AttributeError, PermissionError):
                    os.rename(pre_lock.name, f.name + '.lock')
                    dotlock_done = True
                else:
                    os.unlink(pre_lock.name)
            except FileExistsError:
                os.remove(pre_lock.name)
                raise ExternalClashError('dot lock unavailable: %s' %
                                         f.name)
    except:
        if fcntl:
            fcntl.lockf(f, fcntl.LOCK_UN)
        if dotlock_done:
            os.remove(f.name + '.lock')
        raise

def _unlock_file(f):
    """Unlock file f using lockf and dot locking."""
    if fcntl:
        fcntl.lockf(f, fcntl.LOCK_UN)
    if os.path.exists(f.name + '.lock'):
        os.remove(f.name + '.lock')

def _create_carefully(path):
    """Create a file if it doesn't exist and open for reading and writing."""
    fd = os.open(path, os.O_CREAT | os.O_EXCL | os.O_RDWR, 0o666)
    try:
        return open(path, 'rb+')
    finally:
        os.close(fd)

def _create_temporary(path):
    """Create a temp file based on path and open for reading and writing."""
    return _create_carefully('%s.%s.%s.%s' % (path, int(time.time()),
                                              socket.gethostname(),
                                              os.getpid()))

def _sync_flush(f):
    """Ensure changes to file f are physically on disk."""
    f.flush()
    if hasattr(os, 'fsync'):
        os.fsync(f.fileno())

def _sync_close(f):
    """Close file f, ensuring all changes are physically on disk."""
    _sync_flush(f)
    f.close()


class Error(Exception):
    """Raised for module-specific errors."""

class NoSuchMailboxError(Error):
    """The specified mailbox does not exist and won't be created."""

class NotEmptyError(Error):
    """The specified mailbox is not empty and deletion was requested."""

class ExternalClashError(Error):
    """Another process caused an action to fail."""

class FormatError(Error):
    """A file appears to have an invalid format."""
