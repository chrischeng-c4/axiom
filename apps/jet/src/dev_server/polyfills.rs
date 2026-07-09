// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
// CODEGEN-BEGIN
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Set of Node.js built-in module names that have real browser polyfills.
const POLYFILL_BUILTINS: &[&str] = &[
    "crypto",
    "url",
    "buffer",
    "path",
    "events",
    "util",
    "querystring",
    "process",
    "stream",
];

/// Set of Node.js built-in module names that are stub-only in the browser.
const STUB_BUILTINS: &[&str] = &[
    "fs",
    "child_process",
    "cluster",
    "net",
    "tls",
    "dgram",
    "worker_threads",
    "v8",
    "vm",
    "dns",
    "os",
    "http",
    "http2",
    "https",
    "zlib",
    "readline",
    "tty",
    "assert",
    "constants",
    "module",
    "perf_hooks",
    "string_decoder",
    "sys",
    "timers",
    "domain",
    "punycode",
];

/// All known Node.js builtins (union of polyfill + stub).
fn all_builtins() -> HashSet<&'static str> {
    let mut set = HashSet::new();
    for b in POLYFILL_BUILTINS {
        set.insert(*b);
    }
    for b in STUB_BUILTINS {
        set.insert(*b);
    }
    set
}

/// Detect which Node.js builtin modules are imported in the given source code.
///
/// Scans for `require('builtin')`, `require('node:builtin')`,
/// `from 'builtin'`, and `from 'node:builtin'` patterns.
///
/// Returns a map from builtin name → set of importing package names.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn detect_builtin_imports(
    prebundled_sources: &HashMap<String, String>,
) -> HashMap<String, HashSet<String>> {
    let known = all_builtins();
    let mut result: HashMap<String, HashSet<String>> = HashMap::new();

    for (pkg_name, source) in prebundled_sources {
        // Scan for require('name') and require('node:name')
        for cap in find_require_imports(source) {
            let name = cap.strip_prefix("node:").unwrap_or(&cap);
            if known.contains(name) {
                result
                    .entry(name.to_string())
                    .or_default()
                    .insert(pkg_name.clone());
            }
        }

        // Scan for from 'name' and from 'node:name'
        for cap in find_from_imports(source) {
            let name = cap.strip_prefix("node:").unwrap_or(&cap);
            if known.contains(name) {
                result
                    .entry(name.to_string())
                    .or_default()
                    .insert(pkg_name.clone());
            }
        }
    }

    result
}

/// Simple `require('...')` extractor — returns the string argument.
fn find_require_imports(source: &str) -> Vec<String> {
    let mut results = Vec::new();
    let needle = "require(";
    let mut pos = 0;
    while let Some(idx) = source[pos..].find(needle) {
        let start = pos + idx + needle.len();
        if start >= source.len() {
            break;
        }
        let quote = source.as_bytes()[start];
        if quote == b'\'' || quote == b'"' {
            let inner_start = start + 1;
            if let Some(end) = source[inner_start..].find(quote as char) {
                results.push(source[inner_start..inner_start + end].to_string());
            }
        }
        pos = start;
    }
    results
}

/// Simple `from '...'` / `from "..."` extractor.
fn find_from_imports(source: &str) -> Vec<String> {
    let mut results = Vec::new();
    let needle = "from ";
    let mut pos = 0;
    while let Some(idx) = source[pos..].find(needle) {
        let start = pos + idx + needle.len();
        if start >= source.len() {
            break;
        }
        let quote = source.as_bytes()[start];
        if quote == b'\'' || quote == b'"' {
            let inner_start = start + 1;
            if let Some(end) = source[inner_start..].find(quote as char) {
                results.push(source[inner_start..inner_start + end].to_string());
            }
        }
        pos = start + 1;
    }
    results
}

/// Generate a browser polyfill ESM file for the given builtin module.
///
/// Returns the ESM source code string. For polyfill builtins, generates a
/// real browser-compatible implementation. For stub builtins, generates a
/// warning + empty export.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn generate_polyfill(builtin: &str) -> String {
    match builtin {
        "crypto" => generate_crypto_polyfill(),
        "url" => generate_url_polyfill(),
        "buffer" => generate_buffer_polyfill(),
        "path" => generate_path_polyfill(),
        "events" => generate_events_polyfill(),
        "util" => generate_util_polyfill(),
        "querystring" => generate_querystring_polyfill(),
        "process" => generate_process_polyfill(),
        "stream" => generate_stream_polyfill(),
        _ => String::new(),
    }
}

/// Generate a stub ESM module for a builtin that has no browser equivalent.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn generate_stub(builtin: &str, importing_package: &str) -> String {
    format!(
        r#"// Stub polyfill for '{builtin}' (no browser equivalent)
console.warn("[jet] Warning: '{builtin}' imported by '{importing_package}' — stubbed (no browser equivalent)");
const stub = {{}};
export default stub;
export {{}};
"#,
    )
}

/// Check if a builtin has a real polyfill (vs stub).
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn has_polyfill(builtin: &str) -> bool {
    POLYFILL_BUILTINS.contains(&builtin)
}

/// Write all needed polyfill files to the `.jet/` directory.
///
/// `detected` — map from builtin name → set of importing package names
/// `jet_dir`  — path to `node_modules/.jet/`
///
/// Returns the list of builtin names that were written (for importmap).
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn write_polyfills(detected: &HashMap<String, HashSet<String>>, jet_dir: &Path) -> Vec<String> {
    let mut written = Vec::new();

    // If stream is imported, ensure events polyfill is also generated
    // because generate_stream_polyfill() imports from ./polyfill-events.mjs.
    let mut detected = detected.clone();
    if detected.contains_key("stream") && !detected.contains_key("events") {
        let stream_importers = detected["stream"].clone();
        detected.insert("events".to_string(), stream_importers);
    }

    for (builtin, importers) in &detected {
        let filename = format!("polyfill-{}.mjs", builtin);
        let filepath = jet_dir.join(&filename);

        let content = if has_polyfill(builtin) {
            generate_polyfill(builtin)
        } else {
            // Pick the first importer for the warning message
            let importer = importers
                .iter()
                .next()
                .map(|s| s.as_str())
                .unwrap_or("unknown");
            tracing::warn!(
                "[jet] Warning: '{}' imported by '{}' — stubbed (no browser equivalent)",
                builtin,
                importer
            );
            generate_stub(builtin, importer)
        };

        if let Err(e) = std::fs::write(&filepath, &content) {
            tracing::error!("Failed to write polyfill {}: {}", filename, e);
            continue;
        }

        written.push(builtin.clone());
    }

    written
}

// ─── Polyfill generators ──────────────────────────────────────────────────────

fn generate_crypto_polyfill() -> String {
    r#"// Browser polyfill for 'crypto'
const cryptoPolyfill = globalThis.crypto || {};

export function randomUUID() {
  if (cryptoPolyfill.randomUUID) {
    return cryptoPolyfill.randomUUID();
  }
  // Fallback UUID v4
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
    var r = Math.random() * 16 | 0;
    var v = c === 'x' ? r : (r & 0x3 | 0x8);
    return v.toString(16);
  });
}

export function getRandomValues(array) {
  if (cryptoPolyfill.getRandomValues) {
    return cryptoPolyfill.getRandomValues(array);
  }
  for (var i = 0; i < array.length; i++) {
    array[i] = Math.floor(Math.random() * 256);
  }
  return array;
}

export const subtle = cryptoPolyfill.subtle || {};

export default cryptoPolyfill;
"#
    .to_string()
}

fn generate_url_polyfill() -> String {
    r#"// Browser polyfill for 'url'
export const URL = globalThis.URL;
export const URLSearchParams = globalThis.URLSearchParams;

export function parse(urlStr) {
  try {
    var u = new URL(urlStr);
    return {
      protocol: u.protocol,
      hostname: u.hostname,
      port: u.port,
      pathname: u.pathname,
      search: u.search,
      hash: u.hash,
      host: u.host,
      href: u.href,
    };
  } catch (e) {
    return { href: urlStr };
  }
}

export function format(urlObj) {
  if (typeof urlObj === 'string') return urlObj;
  return (urlObj.protocol || '') + '//' + (urlObj.hostname || '') +
    (urlObj.port ? ':' + urlObj.port : '') +
    (urlObj.pathname || '/') +
    (urlObj.search || '') +
    (urlObj.hash || '');
}

export function resolve(from, to) {
  try {
    return new URL(to, from).href;
  } catch (e) {
    return to;
  }
}

export default { URL, URLSearchParams, parse, format, resolve };
"#
    .to_string()
}

fn generate_buffer_polyfill() -> String {
    r#"// Browser polyfill for 'buffer'
class Buffer extends Uint8Array {
  static from(value, encoding) {
    if (typeof value === 'string') {
      var encoder = new TextEncoder();
      var bytes = encoder.encode(value);
      return new Buffer(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    }
    if (value instanceof ArrayBuffer) {
      return new Buffer(value);
    }
    if (ArrayBuffer.isView(value)) {
      return new Buffer(value.buffer, value.byteOffset, value.byteLength);
    }
    if (Array.isArray(value)) {
      return new Buffer(new Uint8Array(value).buffer);
    }
    return new Buffer(0);
  }

  static alloc(size, fill) {
    var buf = new Buffer(size);
    if (fill !== undefined) {
      buf.fill(typeof fill === 'number' ? fill : 0);
    }
    return buf;
  }

  static isBuffer(obj) {
    return obj instanceof Buffer;
  }

  static concat(list, totalLength) {
    if (totalLength === undefined) {
      totalLength = list.reduce(function(acc, buf) { return acc + buf.length; }, 0);
    }
    var result = Buffer.alloc(totalLength);
    var offset = 0;
    for (var buf of list) {
      result.set(buf, offset);
      offset += buf.length;
    }
    return result;
  }

  toString(encoding) {
    var decoder = new TextDecoder(encoding === 'utf8' || encoding === 'utf-8' ? 'utf-8' : encoding || 'utf-8');
    return decoder.decode(this);
  }

  toJSON() {
    return { type: 'Buffer', data: Array.from(this) };
  }
}

export { Buffer };
export default { Buffer };
"#
    .to_string()
}

fn generate_path_polyfill() -> String {
    r#"// Browser polyfill for 'path' (POSIX)
export var sep = '/';
export var delimiter = ':';

export function join() {
  var parts = [];
  for (var i = 0; i < arguments.length; i++) {
    if (arguments[i]) parts.push(arguments[i]);
  }
  return normalize(parts.join('/'));
}

export function resolve() {
  var resolved = '';
  for (var i = arguments.length - 1; i >= 0; i--) {
    var path = arguments[i];
    if (!path) continue;
    resolved = path + '/' + resolved;
    if (path.charAt(0) === '/') break;
  }
  return normalize('/' + resolved);
}

export function normalize(path) {
  var parts = path.split('/');
  var result = [];
  for (var part of parts) {
    if (part === '..') {
      result.pop();
    } else if (part !== '.' && part !== '') {
      result.push(part);
    }
  }
  var normalized = result.join('/');
  if (path.charAt(0) === '/') normalized = '/' + normalized;
  return normalized || '.';
}

export function dirname(path) {
  var idx = path.lastIndexOf('/');
  if (idx === -1) return '.';
  if (idx === 0) return '/';
  return path.substring(0, idx);
}

export function basename(path, ext) {
  var base = path.substring(path.lastIndexOf('/') + 1);
  if (ext && base.endsWith(ext)) {
    base = base.substring(0, base.length - ext.length);
  }
  return base;
}

export function extname(path) {
  var base = basename(path);
  var idx = base.lastIndexOf('.');
  if (idx <= 0) return '';
  return base.substring(idx);
}

export function isAbsolute(path) {
  return path.charAt(0) === '/';
}

export function relative(from, to) {
  from = resolve(from);
  to = resolve(to);
  if (from === to) return '';
  var fromParts = from.split('/').filter(Boolean);
  var toParts = to.split('/').filter(Boolean);
  var common = 0;
  while (common < fromParts.length && common < toParts.length && fromParts[common] === toParts[common]) {
    common++;
  }
  var ups = fromParts.length - common;
  var result = [];
  for (var i = 0; i < ups; i++) result.push('..');
  result.push.apply(result, toParts.slice(common));
  return result.join('/');
}

export var posix = { sep, delimiter, join, resolve, normalize, dirname, basename, extname, isAbsolute, relative };

export default { sep, delimiter, join, resolve, normalize, dirname, basename, extname, isAbsolute, relative, posix };
"#
    .to_string()
}

fn generate_events_polyfill() -> String {
    r#"// Browser polyfill for 'events'
export class EventEmitter {
  constructor() {
    this._events = {};
    this._maxListeners = 10;
  }

  on(event, listener) {
    if (!this._events[event]) this._events[event] = [];
    this._events[event].push(listener);
    return this;
  }

  addListener(event, listener) {
    return this.on(event, listener);
  }

  once(event, listener) {
    var self = this;
    function wrapper() {
      self.removeListener(event, wrapper);
      listener.apply(this, arguments);
    }
    wrapper._original = listener;
    return this.on(event, wrapper);
  }

  emit(event) {
    var listeners = this._events[event];
    if (!listeners || listeners.length === 0) return false;
    var args = Array.prototype.slice.call(arguments, 1);
    for (var fn of listeners.slice()) {
      fn.apply(this, args);
    }
    return true;
  }

  removeListener(event, listener) {
    var listeners = this._events[event];
    if (!listeners) return this;
    this._events[event] = listeners.filter(function(fn) {
      return fn !== listener && fn._original !== listener;
    });
    return this;
  }

  off(event, listener) {
    return this.removeListener(event, listener);
  }

  removeAllListeners(event) {
    if (event) {
      delete this._events[event];
    } else {
      this._events = {};
    }
    return this;
  }

  listeners(event) {
    return (this._events[event] || []).slice();
  }

  listenerCount(event) {
    return (this._events[event] || []).length;
  }

  setMaxListeners(n) {
    this._maxListeners = n;
    return this;
  }
}

export default EventEmitter;
"#
    .to_string()
}

fn generate_util_polyfill() -> String {
    r#"// Browser polyfill for 'util' (partial)
export function inspect(obj, opts) {
  try {
    return JSON.stringify(obj, null, 2);
  } catch (e) {
    return String(obj);
  }
}

export function promisify(fn) {
  return function() {
    var args = Array.prototype.slice.call(arguments);
    return new Promise(function(resolve, reject) {
      args.push(function(err, result) {
        if (err) reject(err);
        else resolve(result);
      });
      fn.apply(null, args);
    });
  };
}

export function inherits(ctor, superCtor) {
  ctor.prototype = Object.create(superCtor.prototype, {
    constructor: { value: ctor, writable: true, configurable: true }
  });
}

export function deprecate(fn, msg) {
  var warned = false;
  return function() {
    if (!warned) {
      console.warn('DeprecationWarning: ' + msg);
      warned = true;
    }
    return fn.apply(this, arguments);
  };
}

export function format() {
  var args = Array.prototype.slice.call(arguments);
  var fmt = args.shift();
  if (typeof fmt !== 'string') return args.map(String).join(' ');
  return fmt.replace(/%[sdj%]/g, function(match) {
    if (match === '%%') return '%';
    var arg = args.shift();
    if (match === '%s') return String(arg);
    if (match === '%d') return Number(arg);
    if (match === '%j') return JSON.stringify(arg);
    return match;
  });
}

export default { inspect, promisify, inherits, deprecate, format };
"#
    .to_string()
}

fn generate_querystring_polyfill() -> String {
    r#"// Browser polyfill for 'querystring'
export function stringify(obj, sep, eq) {
  sep = sep || '&';
  eq = eq || '=';
  var params = new URLSearchParams();
  for (var key in obj) {
    if (Object.prototype.hasOwnProperty.call(obj, key)) {
      params.set(key, obj[key]);
    }
  }
  return params.toString();
}

export function parse(str, sep, eq) {
  var params = new URLSearchParams(str);
  var result = {};
  params.forEach(function(value, key) {
    result[key] = value;
  });
  return result;
}

export function escape(str) {
  return encodeURIComponent(str);
}

export function unescape(str) {
  return decodeURIComponent(str);
}

export default { stringify, parse, escape, unescape };
"#
    .to_string()
}

fn generate_process_polyfill() -> String {
    r#"// Browser polyfill for 'process'
var process = {
  env: { NODE_ENV: 'development' },
  browser: true,
  argv: [],
  version: '',
  versions: {},
  platform: 'browser',
  cwd: function() { return '/'; },
  nextTick: function(fn) {
    var args = Array.prototype.slice.call(arguments, 1);
    Promise.resolve().then(function() { fn.apply(null, args); });
  },
  stdout: { write: function(s) { console.log(s); } },
  stderr: { write: function(s) { console.error(s); } },
  exit: function() {},
  on: function() { return process; },
  off: function() { return process; },
  emit: function() { return false; },
};

export default process;
export var env = process.env;
export var browser = true;
export var argv = process.argv;
export var nextTick = process.nextTick;
"#
    .to_string()
}

fn generate_stream_polyfill() -> String {
    r#"// Browser polyfill for 'stream' (Web Streams API wrapper)
import EventEmitter from './polyfill-events.mjs';

export class Readable extends EventEmitter {
  constructor(opts) {
    super();
    this.readable = true;
    this._read = (opts && opts.read) || function() {};
  }
  read() { return null; }
  pipe(dest) {
    this.on('data', function(chunk) { dest.write(chunk); });
    this.on('end', function() { if (dest.end) dest.end(); });
    return dest;
  }
  destroy() { this.emit('close'); }
}

export class Writable extends EventEmitter {
  constructor(opts) {
    super();
    this.writable = true;
    this._write = (opts && opts.write) || function(chunk, enc, cb) { cb(); };
  }
  write(chunk) { this.emit('data', chunk); return true; }
  end() { this.emit('finish'); this.emit('end'); }
  destroy() { this.emit('close'); }
}

export class Duplex extends EventEmitter {
  constructor() {
    super();
    this.readable = true;
    this.writable = true;
  }
  read() { return null; }
  write(chunk) { this.emit('data', chunk); return true; }
  end() { this.emit('finish'); this.emit('end'); }
  pipe(dest) {
    this.on('data', function(chunk) { dest.write(chunk); });
    this.on('end', function() { if (dest.end) dest.end(); });
    return dest;
  }
}

export class Transform extends Duplex {
  constructor(opts) {
    super();
    this._transform = (opts && opts.transform) || function(chunk, enc, cb) { cb(null, chunk); };
  }
}

export class PassThrough extends Transform {}

export default { Readable, Writable, Duplex, Transform, PassThrough };
"#
    .to_string()
}

#[cfg(test)]
#[path = "polyfills_tests.rs"]
mod tests;
// CODEGEN-END
