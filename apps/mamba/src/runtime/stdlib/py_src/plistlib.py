"""plistlib — generate and parse Apple .plist files (mamba port)."""

import binascii
import datetime
import struct
import itertools
from io import BytesIO

__all__ = [
    "InvalidFileException", "FMT_XML", "FMT_BINARY", "load", "dump",
    "loads", "dumps", "UID"
]


class PlistFormat:
    def __init__(self, name, value):
        self.name = name
        self.value = value

    def __repr__(self):
        return "PlistFormat." + self.name

    def __hash__(self):
        return hash(self.value)

    def __eq__(self, other):
        return isinstance(other, PlistFormat) and other.value == self.value


FMT_XML = PlistFormat("FMT_XML", 1)
FMT_BINARY = PlistFormat("FMT_BINARY", 2)

_UID_MAX = 2 ** 64
# Use binary subtraction rather than unary negation: mamba's JIT inlines unary
# minus as a raw integer negate that mishandles heap big integers.
_INT_MIN = 0 - (2 ** 63)
_INT_MAX = 2 ** 64


class UID:
    def __init__(self, data):
        if not isinstance(data, int):
            raise TypeError("data must be an int")
        if data >= _UID_MAX:
            raise ValueError("UIDs cannot be >= 2**64")
        if data < 0:
            raise ValueError("UIDs must be positive")
        self.data = data

    def __index__(self):
        return self.data

    def __repr__(self):
        return "UID(%s)" % repr(self.data)

    def __reduce__(self):
        return (UID, (self.data,))

    def __eq__(self, other):
        if not isinstance(other, UID):
            return NotImplemented
        return self.data == other.data

    def __hash__(self):
        return hash(self.data)


class InvalidFileException(ValueError):
    def __init__(self, message="Invalid file"):
        ValueError.__init__(self, message)


#
# XML support
#

# Built by explicit concatenation: mamba does not honor a `\` line-continuation
# inside a triple-quoted literal, which would otherwise leave a stray "\\\n".
PLISTHEADER = (
    b'<?xml version="1.0" encoding="UTF-8"?>\n'
    b'<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" '
    b'"http://www.apple.com/DTDs/PropertyList-1.0.dtd">\n'
)

_CTRL_OK = (9, 10, 13)


def _has_ctrl(text):
    for ch in text:
        o = ord(ch)
        if o < 32 and o not in _CTRL_OK:
            return True
    return False


def _escape(text):
    if _has_ctrl(text):
        raise ValueError("strings can't contain control characters; "
                         "use bytes instead")
    text = text.replace("\r\n", "\n")
    text = text.replace("\r", "\n")
    text = text.replace("&", "&amp;")
    text = text.replace("<", "&lt;")
    text = text.replace(">", "&gt;")
    return text


def _date_to_string(d):
    return '%04d-%02d-%02dT%02d:%02d:%02dZ' % (
        d.year, d.month, d.day, d.hour, d.minute, d.second)


def _date_from_string(s):
    s = s.rstrip("Z")
    year = int(s[0:4])
    month = 1
    day = 1
    hour = 0
    minute = 0
    second = 0
    if len(s) >= 7:
        month = int(s[5:7])
    if len(s) >= 10:
        day = int(s[8:10])
    if len(s) >= 13:
        hour = int(s[11:13])
    if len(s) >= 16:
        minute = int(s[14:16])
    if len(s) >= 19:
        second = int(s[17:19])
    return datetime.datetime(year, month, day, hour, minute, second)


def _encode_base64(s, maxlinelength=76):
    maxbinsize = (maxlinelength // 4) * 3
    pieces = []
    i = 0
    n = len(s)
    while i < n:
        chunk = s[i:i + maxbinsize]
        pieces.append(binascii.b2a_base64(chunk))
        i += maxbinsize
    return b''.join(pieces)


def _decode_base64(s):
    if isinstance(s, str):
        return binascii.a2b_base64(s.encode("utf-8"))
    return binascii.a2b_base64(s)


class _PlistWriter:
    def __init__(self, file, sort_keys=True, skipkeys=False, write_header=True):
        if write_header:
            file.write(PLISTHEADER)
        self.file = file
        self.stack = []
        self._indent_level = 0
        self.indent = b"\t"
        self._sort_keys = sort_keys
        self._skipkeys = skipkeys

    def writeln(self, line):
        if line:
            if isinstance(line, str):
                line = line.encode('utf-8')
            self.file.write(self._indent_level * self.indent)
            self.file.write(line)
        self.file.write(b'\n')

    def begin_element(self, element):
        self.stack.append(element)
        self.writeln("<%s>" % element)
        self._indent_level += 1

    def end_element(self, element):
        self._indent_level -= 1
        self.stack.pop()
        self.writeln("</%s>" % element)

    def simple_element(self, element, value=None):
        if value is not None:
            value = _escape(value)
            self.writeln("<%s>%s</%s>" % (element, value, element))
        else:
            self.writeln("<%s/>" % element)

    def write(self, value):
        self.writeln("<plist version=\"1.0\">")
        self.write_value(value)
        self.writeln("</plist>")

    def write_value(self, value):
        if isinstance(value, str):
            self.simple_element("string", value)
        elif value is True:
            self.simple_element("true")
        elif value is False:
            self.simple_element("false")
        elif isinstance(value, int):
            if _INT_MIN <= value < _INT_MAX:
                self.simple_element("integer", "%d" % value)
            else:
                raise OverflowError(value)
        elif isinstance(value, float):
            self.simple_element("real", repr(value))
        elif isinstance(value, dict):
            self.write_dict(value)
        elif isinstance(value, (bytes, bytearray)):
            self.write_bytes(value)
        elif isinstance(value, datetime.datetime):
            self.simple_element("date", _date_to_string(value))
        elif isinstance(value, (tuple, list)):
            self.write_array(value)
        else:
            raise TypeError("unsupported type: %s" % type(value))

    def write_bytes(self, data):
        self.begin_element("data")
        self._indent_level -= 1
        maxlinelength = 76 - 8 * self._indent_level
        if maxlinelength < 16:
            maxlinelength = 16
        for line in _encode_base64(bytes(data), maxlinelength).split(b"\n"):
            if line:
                self.writeln(line)
        self._indent_level += 1
        self.end_element("data")

    def write_dict(self, d):
        if d:
            self.begin_element("dict")
            if self._sort_keys:
                items = sorted(d.items())
            else:
                items = list(d.items())
            for key, value in items:
                if not isinstance(key, str):
                    if self._skipkeys:
                        continue
                    raise TypeError("keys must be strings")
                self.simple_element("key", key)
                self.write_value(value)
            self.end_element("dict")
        else:
            self.simple_element("dict")

    def write_array(self, array):
        if array:
            self.begin_element("array")
            for value in array:
                self.write_value(value)
            self.end_element("array")
        else:
            self.simple_element("array")


#
# XML parser — minimal tokenizer (no expat dependency)
#

class _PlistParser:
    def __init__(self, dict_type=dict):
        self.stack = []
        self.current_key = None
        self.root = None
        self._dict_type = dict_type
        self.data = []

    def parse(self, fileobj):
        data = fileobj.read()
        if isinstance(data, bytes):
            text = self._decode(data)
        else:
            text = data
        self._tokenize(text)
        return self.root

    def _decode(self, data):
        if data[:3] == b'\xef\xbb\xbf':
            return data[3:].decode('utf-8')
        if data[:2] == b'\xff\xfe':
            return data[2:].decode('utf-16le')
        if data[:2] == b'\xfe\xff':
            return data[2:].decode('utf-16be')
        return data.decode('utf-8')

    def _tokenize(self, text):
        i = 0
        n = len(text)
        chardata = []
        while i < n:
            c = text[i]
            if c == '<':
                if text[i:i + 4] == '<!--':
                    end = text.find('-->', i)
                    i = n if end == -1 else end + 3
                    continue
                if text[i:i + 9] == '<!DOCTYPE':
                    # DOCTYPE may carry an internal subset in [...]; reject
                    # entity declarations, then skip to the matching '>'.
                    bracket = text.find('[', i)
                    gt = text.find('>', i)
                    if bracket != -1 and (gt == -1 or bracket < gt):
                        close = text.find(']', bracket)
                        subset = text[bracket:close + 1] if close != -1 else text[bracket:]
                        if '<!ENTITY' in subset:
                            raise InvalidFileException(
                                "XML entity declarations are not supported in plist files")
                        gt = text.find('>', close if close != -1 else i)
                    i = n if gt == -1 else gt + 1
                    continue
                end = text.find('>', i)
                if end == -1:
                    break
                tag = text[i + 1:end]
                i = end + 1
                if tag.startswith('?') or tag.startswith('!'):
                    if '<!ENTITY' in tag or tag.startswith('!ENTITY'):
                        raise InvalidFileException(
                            "XML entity declarations are not supported in plist files")
                    continue
                if tag.startswith('/'):
                    name = tag[1:].strip()
                    self.data = chardata
                    chardata = []
                    self._handle_end(name)
                elif tag.endswith('/'):
                    name = tag[:-1].strip()
                    self.data = []
                    self._handle_begin(name)
                    self.data = []
                    self._handle_end(name)
                    chardata = []
                else:
                    parts = tag.split()
                    name = parts[0] if parts else tag
                    self.data = []
                    chardata = []
                    self._handle_begin(name)
            else:
                end = text.find('<', i)
                if end == -1:
                    end = n
                chardata.append(text[i:end])
                i = end

    def _handle_begin(self, element):
        handler = getattr(self, "begin_" + element, None)
        if handler is not None:
            handler()

    def _handle_end(self, element):
        handler = getattr(self, "end_" + element, None)
        if handler is not None:
            handler()

    def get_data(self):
        data = ''.join(self.data)
        data = data.replace("&lt;", "<")
        data = data.replace("&gt;", ">")
        data = data.replace("&quot;", '"')
        data = data.replace("&apos;", "'")
        data = data.replace("&amp;", "&")
        self.data = []
        return data

    def add_object(self, value):
        if self.current_key is not None:
            if not isinstance(self.stack[-1], dict):
                raise ValueError("unexpected element")
            self.stack[-1][self.current_key] = value
            self.current_key = None
        elif not self.stack:
            self.root = value
        else:
            if not isinstance(self.stack[-1], list):
                raise ValueError("unexpected element")
            self.stack[-1].append(value)

    def begin_dict(self):
        d = self._dict_type()
        self.add_object(d)
        self.stack.append(d)

    def end_dict(self):
        if self.current_key:
            raise ValueError("missing value for key '%s'" % self.current_key)
        self.stack.pop()

    def end_key(self):
        if self.current_key or not isinstance(self.stack[-1], dict):
            raise ValueError("unexpected key")
        self.current_key = self.get_data()

    def begin_array(self):
        a = []
        self.add_object(a)
        self.stack.append(a)

    def end_array(self):
        self.stack.pop()

    def end_true(self):
        self.add_object(True)

    def end_false(self):
        self.add_object(False)

    def end_integer(self):
        raw = self.get_data()
        if raw.startswith('0x') or raw.startswith('0X'):
            self.add_object(int(raw, 16))
        else:
            self.add_object(int(raw))

    def end_real(self):
        self.add_object(float(self.get_data()))

    def end_string(self):
        self.add_object(self.get_data())

    def end_data(self):
        self.add_object(_decode_base64(self.get_data()))

    def end_date(self):
        self.add_object(_date_from_string(self.get_data()))


def _is_fmt_xml(header):
    prefixes = (b'<?xml', b'<plist')
    for pfx in prefixes:
        if header[:len(pfx)] == pfx:
            return True
    boms = (
        (b'\xef\xbb\xbf', 'utf-8'),
        (b'\xfe\xff', 'utf-16be'),
        (b'\xff\xfe', 'utf-16le'),
    )
    for bom, encoding in boms:
        if header[:len(bom)] != bom:
            continue
        for start in prefixes:
            prefix = bom + start.decode('ascii').encode(encoding)
            if header[:len(prefix)] == prefix:
                return True
    return False


#
# Binary plist
#

_BINARY_FORMAT = {1: 'B', 2: 'H', 4: 'L', 8: 'Q'}
_undefined = object()


class _BinaryPlistParser:
    def __init__(self, dict_type=dict):
        self._dict_type = dict_type

    def parse(self, fp):
        try:
            self._fp = fp
            self._fp.seek(-32, 2)
            trailer = self._fp.read(32)
            if len(trailer) != 32:
                raise InvalidFileException()
            (offset_size, self._ref_size, num_objects, top_object,
             offset_table_offset) = struct.unpack('>6xBBQQQ', trailer)
            self._fp.seek(offset_table_offset)
            self._object_offsets = self._read_ints(num_objects, offset_size)
            self._objects = [_undefined] * num_objects
            return self._read_object(top_object)
        except InvalidFileException:
            raise
        except (OSError, IndexError, struct.error, OverflowError, ValueError):
            raise InvalidFileException()

    def _get_size(self, tokenL):
        if tokenL == 0xF:
            m = self._fp.read(1)[0] & 0x3
            s = 1 << m
            f = '>' + _BINARY_FORMAT[s]
            return struct.unpack(f, self._fp.read(s))[0]
        return tokenL

    def _read_ints(self, n, size):
        data = self._fp.read(size * n)
        if size in _BINARY_FORMAT:
            return struct.unpack('>%d%s' % (n, _BINARY_FORMAT[size]), data)
        if not size or len(data) != size * n:
            raise InvalidFileException()
        result = []
        for i in range(0, size * n, size):
            result.append(int.from_bytes(data[i:i + size], 'big'))
        return tuple(result)

    def _read_refs(self, n):
        return self._read_ints(n, self._ref_size)

    def _read_object(self, ref):
        result = self._objects[ref]
        if result is not _undefined:
            return result
        offset = self._object_offsets[ref]
        self._fp.seek(offset)
        token = self._fp.read(1)[0]
        tokenH = token & 0xF0
        tokenL = token & 0x0F

        if token == 0x00:
            result = None
        elif token == 0x08:
            result = False
        elif token == 0x09:
            result = True
        elif token == 0x0f:
            result = b''
        elif tokenH == 0x10:
            result = int.from_bytes(self._fp.read(1 << tokenL), 'big',
                                    signed=tokenL >= 3)
        elif token == 0x22:
            result = struct.unpack('>f', self._fp.read(4))[0]
        elif token == 0x23:
            result = struct.unpack('>d', self._fp.read(8))[0]
        elif token == 0x33:
            f = struct.unpack('>d', self._fp.read(8))[0]
            result = (datetime.datetime(2001, 1, 1) +
                      datetime.timedelta(seconds=f))
        elif tokenH == 0x40:
            s = self._get_size(tokenL)
            result = self._fp.read(s)
            if len(result) != s:
                raise InvalidFileException()
        elif tokenH == 0x50:
            s = self._get_size(tokenL)
            data = self._fp.read(s)
            if len(data) != s:
                raise InvalidFileException()
            result = data.decode('ascii')
        elif tokenH == 0x60:
            s = self._get_size(tokenL) * 2
            data = self._fp.read(s)
            if len(data) != s:
                raise InvalidFileException()
            result = data.decode('utf-16be')
        elif tokenH == 0x80:
            result = UID(int.from_bytes(self._fp.read(1 + tokenL), 'big'))
        elif tokenH == 0xA0:
            s = self._get_size(tokenL)
            obj_refs = self._read_refs(s)
            result = []
            self._objects[ref] = result
            for x in obj_refs:
                result.append(self._read_object(x))
        elif tokenH == 0xD0:
            s = self._get_size(tokenL)
            key_refs = self._read_refs(s)
            obj_refs = self._read_refs(s)
            result = self._dict_type()
            self._objects[ref] = result
            try:
                for j in range(s):
                    k = self._read_object(key_refs[j])
                    o = self._read_object(obj_refs[j])
                    result[k] = o
            except TypeError:
                raise InvalidFileException()
        else:
            raise InvalidFileException()

        self._objects[ref] = result
        return result


def _count_to_size(count):
    if count < 2 ** 8:
        return 1
    elif count < 2 ** 16:
        return 2
    elif count < 2 ** 32:
        return 4
    else:
        return 8


_scalars = (str, int, float, datetime.datetime, bytes)


class _BinaryPlistWriter:
    def __init__(self, fp, sort_keys=True, skipkeys=False):
        self._fp = fp
        self._sort_keys = sort_keys
        self._skipkeys = skipkeys

    def write(self, value):
        self._objlist = []
        self._objtable = {}
        self._objidtable = {}
        self._flatten(value)
        num_objects = len(self._objlist)
        self._object_offsets = [0] * num_objects
        self._ref_size = _count_to_size(num_objects)
        self._ref_format = _BINARY_FORMAT[self._ref_size]
        self._fp.write(b'bplist00')
        for obj in self._objlist:
            self._write_object(obj)
        top_object = self._getrefnum(value)
        offset_table_offset = self._fp.tell()
        offset_size = _count_to_size(offset_table_offset)
        offset_format = '>' + _BINARY_FORMAT[offset_size] * num_objects
        self._fp.write(struct.pack(offset_format, *self._object_offsets))
        sort_version = 0
        trailer = (sort_version, offset_size, self._ref_size, num_objects,
                   top_object, offset_table_offset)
        self._fp.write(struct.pack('>5xBBBQQQ', *trailer))

    def _flatten(self, value):
        if isinstance(value, UID):
            if (UID, value.data) in self._objtable:
                return
        elif isinstance(value, _scalars):
            if (type(value), value) in self._objtable:
                return
        elif id(value) in self._objidtable:
            return

        refnum = len(self._objlist)
        self._objlist.append(value)
        if isinstance(value, UID):
            self._objtable[(UID, value.data)] = refnum
        elif isinstance(value, _scalars):
            self._objtable[(type(value), value)] = refnum
        else:
            self._objidtable[id(value)] = refnum

        if isinstance(value, dict):
            keys = []
            values = []
            items = value.items()
            if self._sort_keys:
                items = sorted(items)
            for k, v in items:
                if not isinstance(k, str):
                    if self._skipkeys:
                        continue
                    raise TypeError("keys must be strings")
                keys.append(k)
                values.append(v)
            for o in itertools.chain(keys, values):
                self._flatten(o)
        elif isinstance(value, (list, tuple)):
            for o in value:
                self._flatten(o)

    def _getrefnum(self, value):
        if isinstance(value, UID):
            return self._objtable[(UID, value.data)]
        elif isinstance(value, _scalars):
            return self._objtable[(type(value), value)]
        else:
            return self._objidtable[id(value)]

    def _write_size(self, token, size):
        if size < 15:
            self._fp.write(struct.pack('>B', token | size))
        elif size < 2 ** 8:
            self._fp.write(struct.pack('>BBB', token | 0xF, 0x10, size))
        elif size < 2 ** 16:
            self._fp.write(struct.pack('>BBH', token | 0xF, 0x11, size))
        elif size < 2 ** 32:
            self._fp.write(struct.pack('>BBL', token | 0xF, 0x12, size))
        else:
            self._fp.write(struct.pack('>BBQ', token | 0xF, 0x13, size))

    def _write_object(self, value):
        ref = self._getrefnum(value)
        self._object_offsets[ref] = self._fp.tell()
        if value is None:
            self._fp.write(b'\x00')
        elif value is False:
            self._fp.write(b'\x08')
        elif value is True:
            self._fp.write(b'\x09')
        elif isinstance(value, UID):
            if value.data < 0:
                raise ValueError("UIDs must be positive")
            elif value.data < 2 ** 8:
                self._fp.write(struct.pack('>BB', 0x80, value.data))
            elif value.data < 2 ** 16:
                self._fp.write(struct.pack('>BH', 0x81, value.data))
            elif value.data < 2 ** 32:
                self._fp.write(struct.pack('>BL', 0x83, value.data))
            elif value.data < 2 ** 64:
                self._fp.write(struct.pack('>BQ', 0x87, value.data))
            else:
                raise OverflowError(value)
        elif isinstance(value, int):
            if value < 0:
                if value < _INT_MIN:
                    raise OverflowError(value)
                self._fp.write(struct.pack('>Bq', 0x13, value))
            elif value < 2 ** 8:
                self._fp.write(struct.pack('>BB', 0x10, value))
            elif value < 2 ** 16:
                self._fp.write(struct.pack('>BH', 0x11, value))
            elif value < 2 ** 32:
                self._fp.write(struct.pack('>BL', 0x12, value))
            elif value < 2 ** 63:
                self._fp.write(struct.pack('>BQ', 0x13, value))
            elif value < 2 ** 64:
                self._fp.write(b'\x14' + value.to_bytes(16, 'big', signed=True))
            else:
                raise OverflowError(value)
        elif isinstance(value, float):
            self._fp.write(struct.pack('>Bd', 0x23, value))
        elif isinstance(value, datetime.datetime):
            delta = value - datetime.datetime(2001, 1, 1)
            f = delta.total_seconds()
            self._fp.write(struct.pack('>Bd', 0x33, f))
        elif isinstance(value, (bytes, bytearray)):
            self._write_size(0x40, len(value))
            self._fp.write(bytes(value))
        elif isinstance(value, str):
            try:
                t = value.encode('ascii')
                self._write_size(0x50, len(value))
            except UnicodeEncodeError:
                t = value.encode('utf-16be')
                self._write_size(0x60, len(t) // 2)
            self._fp.write(t)
        elif isinstance(value, (list, tuple)):
            refs = []
            for o in value:
                refs.append(self._getrefnum(o))
            s = len(refs)
            self._write_size(0xA0, s)
            self._fp.write(struct.pack('>' + self._ref_format * s, *refs))
        elif isinstance(value, dict):
            keyRefs = []
            valRefs = []
            if self._sort_keys:
                rootItems = sorted(value.items())
            else:
                rootItems = list(value.items())
            for k, v in rootItems:
                if not isinstance(k, str):
                    if self._skipkeys:
                        continue
                    raise TypeError("keys must be strings")
                keyRefs.append(self._getrefnum(k))
                valRefs.append(self._getrefnum(v))
            s = len(keyRefs)
            self._write_size(0xD0, s)
            self._fp.write(struct.pack('>' + self._ref_format * s, *keyRefs))
            self._fp.write(struct.pack('>' + self._ref_format * s, *valRefs))
        else:
            raise TypeError(value)


def _is_fmt_binary(header):
    return header[:8] == b'bplist00'


#
# Public API
#

_FORMATS = {
    FMT_XML: {
        'detect': _is_fmt_xml,
        'parser': _PlistParser,
        'writer': _PlistWriter,
    },
    FMT_BINARY: {
        'detect': _is_fmt_binary,
        'parser': _BinaryPlistParser,
        'writer': _BinaryPlistWriter,
    },
}


# The public entry points take *args and parse positional + a trailing
# keyword-arguments dict by hand. mamba's cross-module call convention forwards
# keyword arguments to an imported module's function as a trailing dict appended
# to *args, and does NOT apply a callee's own parameter defaults across the
# module boundary — so explicit *args parsing is the portable way to honor
# both positional and keyword call styles from the test fixtures.

def _split_args(args, names):
    # Returns a dict name->value, where positional args fill `names` in order
    # and a trailing kwargs dict (the last arg, if a dict whose keys are all in
    # `names`) overrides by name.
    pos = list(args)
    kw = {}
    if pos and isinstance(pos[-1], dict):
        last = pos[-1]
        if last and all(isinstance(k, str) and k in names for k in last):
            kw = last
            pos = pos[:-1]
    out = {}
    for i in range(len(pos)):
        if i < len(names):
            out[names[i]] = pos[i]
    for k in kw:
        out[k] = kw[k]
    return out


def _make_parser(which, dict_type):
    # Direct class instantiation: calling a class through a variable / dict
    # slot does not invoke __init__ correctly under mamba, so branch explicitly.
    if which == 1:
        return _PlistParser(dict_type)
    return _BinaryPlistParser(dict_type)


def _load_impl(fp, fmt, dict_type):
    if fmt is None:
        header = fp.read(32)
        fp.seek(0)
        which = 0
        if _is_fmt_xml(header):
            which = 1
        elif _is_fmt_binary(header):
            which = 2
        if which == 0:
            raise InvalidFileException()
    else:
        if fmt not in _FORMATS:
            raise ValueError("Unsupported format: %r" % (fmt,))
        which = fmt.value
    p = _make_parser(which, dict_type)
    return p.parse(fp)


def load(*args):
    a = _split_args(args, ['fp', 'fmt', 'dict_type'])
    fp = a['fp']
    fmt = a.get('fmt', None)
    dict_type = a.get('dict_type', dict)
    return _load_impl(fp, fmt, dict_type)


def loads(*args):
    a = _split_args(args, ['value', 'fmt', 'dict_type'])
    value = a['value']
    fmt = a.get('fmt', None)
    dict_type = a.get('dict_type', dict)
    fp = BytesIO(value)
    return _load_impl(fp, fmt, dict_type)


def _dump_impl(value, fp, fmt, sort_keys, skipkeys):
    if fmt not in _FORMATS:
        raise ValueError("Unsupported format: %r" % (fmt,))
    # Direct class instantiation (see _make_parser note).
    if fmt.value == 1:
        writer = _PlistWriter(fp, sort_keys, skipkeys)
    else:
        writer = _BinaryPlistWriter(fp, sort_keys, skipkeys)
    writer.write(value)


def dump(*args):
    a = _split_args(args, ['value', 'fp', 'fmt', 'sort_keys', 'skipkeys'])
    value = a['value']
    fp = a['fp']
    fmt = a.get('fmt', FMT_XML)
    sort_keys = a.get('sort_keys', True)
    skipkeys = a.get('skipkeys', False)
    _dump_impl(value, fp, fmt, sort_keys, skipkeys)


def dumps(*args):
    a = _split_args(args, ['value', 'fmt', 'skipkeys', 'sort_keys'])
    value = a['value']
    fmt = a.get('fmt', FMT_XML)
    skipkeys = a.get('skipkeys', False)
    sort_keys = a.get('sort_keys', True)
    fp = BytesIO()
    _dump_impl(value, fp, fmt, sort_keys, skipkeys)
    return fp.getvalue()
