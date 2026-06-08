"""Helper class to quickly write a loop over all standard input files.

Pure-Python port of CPython 3.12 fileinput, adapted to run under Mamba.

Adaptations from the CPython source (behavior preserved):
  * The "_readline = self._file.readline" instance-attribute-over-method shadow
    trick is replaced with an explicit self._readline_hook attribute that the
    state machine consults directly, because Mamba does not dispatch a call
    through an instance attribute that shadows a class method.
  * os.PathLike is not a real type under Mamba, so the PathLike branch in
    __init__ duck-types on __fspath__ instead of isinstance(files, os.PathLike).

Note: several upstream Mamba bugs (keyword-argument binding for callables
defined in an imported module, cross-module writes to module globals such as
`fileinput._state`, and `builtins.open` reassignment not being honored by a
bare `open()`) currently prevent most of the auto-ported CPython fixtures from
exercising this code, but the implementation itself mirrors CPython 3.12.
"""

import io
import sys, os

__all__ = ["input", "close", "nextfile", "filename", "lineno", "filelineno",
           "fileno", "isfirstline", "isstdin", "FileInput", "hook_compressed",
           "hook_encoded"]

_state = None

def input(files=None, inplace=False, backup="", *, mode="r", openhook=None,
          encoding=None, errors=None):
    """Return an instance of the FileInput class, which can be iterated.

    The parameters are passed to the constructor of the FileInput class.
    The returned instance, in addition to being an iterator,
    keeps global state for the functions of this module,.
    """
    global _state
    if _state and _state._file:
        raise RuntimeError("input() already active")
    _state = FileInput(files, inplace, backup, mode=mode, openhook=openhook,
                       encoding=encoding, errors=errors)
    return _state

def close():
    """Close the sequence."""
    global _state
    state = _state
    _state = None
    if state:
        state.close()

def nextfile():
    """
    Close the current file so that the next iteration will read the first
    line from the next file (if any); lines not read from the file will
    not count towards the cumulative line count. The filename is not
    changed until after the first line of the next file has been read.
    Before the first line has been read, this function has no effect;
    it cannot be used to skip the first file. After the last line of the
    last file has been read, this function has no effect.
    """
    if not _state:
        raise RuntimeError("no active input()")
    return _state.nextfile()

def filename():
    """
    Return the name of the file currently being read.
    Before the first line has been read, returns None.
    """
    if not _state:
        raise RuntimeError("no active input()")
    return _state.filename()

def lineno():
    """
    Return the cumulative line number of the line that has just been read.
    Before the first line has been read, returns 0. After the last line
    of the last file has been read, returns the line number of that line.
    """
    if not _state:
        raise RuntimeError("no active input()")
    return _state.lineno()

def filelineno():
    """
    Return the line number in the current file. Before the first line
    has been read, returns 0. After the last line of the last file has
    been read, returns the line number of that line within the file.
    """
    if not _state:
        raise RuntimeError("no active input()")
    return _state.filelineno()

def fileno():
    """
    Return the file number of the current file. When no file is currently
    opened, returns -1.
    """
    if not _state:
        raise RuntimeError("no active input()")
    return _state.fileno()

def isfirstline():
    """
    Returns true the line just read is the first line of its file,
    otherwise returns false.
    """
    if not _state:
        raise RuntimeError("no active input()")
    return _state.isfirstline()

def isstdin():
    """
    Returns true if the last line was read from sys.stdin,
    otherwise returns false.
    """
    if not _state:
        raise RuntimeError("no active input()")
    return _state.isstdin()


def _is_pathlike(obj):
    # os.PathLike is not a real type under Mamba; duck-type on __fspath__.
    return hasattr(obj, "__fspath__")


class FileInput:
    """FileInput([files[, inplace[, backup]]], *, mode=None, openhook=None)

    Class FileInput is the implementation of the module; its methods
    filename(), lineno(), fileline(), isfirstline(), isstdin(), fileno(),
    nextfile() and close() correspond to the functions of the same name
    in the module.
    In addition it has a readline() method which returns the next
    input line, and a __getitem__() method which implements the
    sequence behavior. The sequence must be accessed in strictly
    sequential order; random access and readline() cannot be mixed.
    """

    def __init__(self, files=None, inplace=False, backup="", *,
                 mode="r", openhook=None, encoding=None, errors=None):
        if isinstance(files, str):
            files = (files,)
        elif _is_pathlike(files):
            files = (os.fspath(files), )
        else:
            if files is None:
                files = sys.argv[1:]
            if not files:
                files = ('-',)
            else:
                files = tuple(files)
        self._files = files
        self._inplace = inplace
        self._backup = backup
        self._savestdout = None
        self._output = None
        self._filename = None
        self._startlineno = 0
        self._filelineno = 0
        self._file = None
        self._isstdin = False
        self._backupfilename = None
        self._encoding = encoding
        self._errors = errors
        # Explicit replacement for the CPython "self._readline =
        # self._file.readline" method-shadowing trick.
        self._readline_hook = None

        # restrict mode argument to reading modes
        if mode not in ('r', 'rb'):
            raise ValueError("FileInput opening mode must be 'r' or 'rb'")
        self._mode = mode
        self._write_mode = mode.replace('r', 'w')
        if openhook:
            if inplace:
                raise ValueError("FileInput cannot use an opening hook in inplace mode")
            if not callable(openhook):
                raise ValueError("FileInput openhook must be callable")
        self._openhook = openhook

    def __del__(self):
        self.close()

    def close(self):
        try:
            self.nextfile()
        finally:
            self._files = ()

    def __enter__(self):
        return self

    def __exit__(self, type, value, traceback):
        self.close()

    def __iter__(self):
        return self

    def __next__(self):
        while True:
            line = self._readline()
            if line:
                self._filelineno += 1
                return line
            if not self._file:
                raise StopIteration
            self.nextfile()
            # repeat with next file

    def nextfile(self):
        savestdout = self._savestdout
        self._savestdout = None
        if savestdout:
            sys.stdout = savestdout

        output = self._output
        self._output = None
        try:
            if output:
                output.close()
        finally:
            file = self._file
            self._file = None
            # Restore FileInput._readline (drop any active per-file hook).
            self._readline_hook = None
            try:
                if file and not self._isstdin:
                    file.close()
            finally:
                backupfilename = self._backupfilename
                self._backupfilename = None
                if backupfilename and not self._backup:
                    try: os.unlink(backupfilename)
                    except OSError: pass

                self._isstdin = False

    def readline(self):
        while True:
            line = self._readline()
            if line:
                self._filelineno += 1
                return line
            if not self._file:
                return line
            self.nextfile()
            # repeat with next file

    def _readline(self):
        # If a per-file readline hook is active, use it directly.
        if self._readline_hook is not None:
            return self._readline_hook()
        if not self._files:
            if 'b' in self._mode:
                return b''
            else:
                return ''
        self._filename = self._files[0]
        self._files = self._files[1:]
        self._startlineno = self.lineno()
        self._filelineno = 0
        self._file = None
        self._isstdin = False
        self._backupfilename = 0

        # EncodingWarning is emitted in __init__() already
        if "b" not in self._mode:
            encoding = self._encoding or "locale"
        else:
            encoding = None

        if self._filename == '-':
            self._filename = '<stdin>'
            if 'b' in self._mode:
                self._file = getattr(sys.stdin, 'buffer', sys.stdin)
            else:
                self._file = sys.stdin
            self._isstdin = True
        else:
            if self._inplace:
                self._backupfilename = (
                    os.fspath(self._filename) + (self._backup or ".bak"))
                try:
                    os.unlink(self._backupfilename)
                except OSError:
                    pass
                # The next few lines may raise OSError
                os.rename(self._filename, self._backupfilename)
                self._file = open(self._backupfilename, self._mode,
                                  encoding=encoding, errors=self._errors)
                try:
                    perm = os.fstat(self._file.fileno()).st_mode
                except OSError:
                    self._output = open(self._filename, self._write_mode,
                                        encoding=encoding, errors=self._errors)
                else:
                    mode = os.O_CREAT | os.O_WRONLY | os.O_TRUNC
                    if hasattr(os, 'O_BINARY'):
                        mode |= os.O_BINARY

                    fd = os.open(self._filename, mode, perm)
                    self._output = os.fdopen(fd, self._write_mode,
                                             encoding=encoding, errors=self._errors)
                    try:
                        os.chmod(self._filename, perm)
                    except OSError:
                        pass
                self._savestdout = sys.stdout
                sys.stdout = self._output
            else:
                # This may raise OSError
                if self._openhook:
                    # Custom hooks made previous to Python 3.10 didn't have
                    # encoding argument
                    if self._encoding is None:
                        self._file = self._openhook(self._filename, self._mode)
                    else:
                        self._file = self._openhook(
                            self._filename, self._mode, encoding=self._encoding, errors=self._errors)
                else:
                    self._file = open(self._filename, self._mode, encoding=encoding, errors=self._errors)
        self._readline_hook = self._file.readline  # hide FileInput._readline
        return self._readline_hook()

    def filename(self):
        return self._filename

    def lineno(self):
        return self._startlineno + self._filelineno

    def filelineno(self):
        return self._filelineno

    def fileno(self):
        if self._file:
            try:
                return self._file.fileno()
            except ValueError:
                return -1
        else:
            return -1

    def isfirstline(self):
        return self._filelineno == 1

    def isstdin(self):
        return self._isstdin


def hook_compressed(filename, mode, *, encoding=None, errors=None):
    if encoding is None and "b" not in mode:  # EncodingWarning is emitted in FileInput() already.
        encoding = "locale"
    ext = os.path.splitext(filename)[1]
    if ext == '.gz':
        import gzip
        stream = gzip.open(filename, mode)
    elif ext == '.bz2':
        import bz2
        stream = bz2.BZ2File(filename, mode)
    else:
        return open(filename, mode, encoding=encoding, errors=errors)

    # gzip and bz2 are binary mode by default.
    if "b" not in mode:
        stream = io.TextIOWrapper(stream, encoding=encoding, errors=errors)
    return stream


def hook_encoded(encoding, errors=None):
    def openhook(filename, mode):
        return open(filename, mode, encoding=encoding, errors=errors)
    return openhook
