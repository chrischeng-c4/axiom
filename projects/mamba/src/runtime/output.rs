/// Thread-local output capture for conformance testing.
///
/// When capture is active, `mb_print` and other output functions write to a
/// thread-local buffer instead of stdout. This allows `cargo test` to compare
/// mamba output against golden files without subprocess overhead.
///
/// Generator threads use a shared capture buffer (from generator.rs) since
/// they run on separate OS threads and don't share the caller's thread-local.
use std::cell::RefCell;
use std::io::Write;

use super::rc::MbObject;
use super::value::MbValue;

thread_local! {
    static CAPTURE_BUF: RefCell<Option<Vec<u8>>> = const { RefCell::new(None) };
    /// contextlib.redirect_stdout target stack. Each entry is the NaN-boxed
    /// bits of a writable stream (e.g. an io.StringIO). When non-empty, stdout
    /// output is routed to the top stream via `mb_stringio_write` instead of
    /// the process stdout / capture buffer. Pushed by `redirect_stdout.__enter__`
    /// and popped by `__exit__`.
    static STDOUT_REDIRECT: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
    /// contextlib.redirect_stderr target stack (mirror of STDOUT_REDIRECT for
    /// `sys.stderr` writes).
    static STDERR_REDIRECT: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
}

/// Push a stderr redirect target.
pub fn push_stderr_redirect(target: MbValue) {
    STDERR_REDIRECT.with(|s| s.borrow_mut().push(target.to_bits()));
}

/// Pop the most recent stderr redirect target.
pub fn pop_stderr_redirect() {
    STDERR_REDIRECT.with(|s| {
        s.borrow_mut().pop();
    });
}

/// If a stderr redirect is active, write `s` to the top target stream and
/// return true. Otherwise return false. Used by `print(..., file=sys.stderr)`.
pub fn try_write_stderr_redirect(s: &str) -> bool {
    let target_bits = STDERR_REDIRECT.with(|stk| stk.borrow().last().copied());
    if let Some(bits) = target_bits {
        let target = MbValue::from_bits(bits);
        write_redirect_target(target, s);
        return true;
    }
    false
}

/// Push a stdout redirect target (an io.StringIO-like stream value).
pub fn push_stdout_redirect(target: MbValue) {
    STDOUT_REDIRECT.with(|s| s.borrow_mut().push(target.to_bits()));
}

/// Pop the most recent stdout redirect target.
pub fn pop_stdout_redirect() {
    STDOUT_REDIRECT.with(|s| {
        s.borrow_mut().pop();
    });
}

/// If a stdout redirect is active, write `s` to the top target stream and
/// return true. Otherwise return false.
fn try_write_redirect(s: &str) -> bool {
    let target_bits = STDOUT_REDIRECT.with(|stk| stk.borrow().last().copied());
    if let Some(bits) = target_bits {
        let target = MbValue::from_bits(bits);
        write_redirect_target(target, s);
        return true;
    }
    false
}

fn write_redirect_target(target: MbValue, s: &str) {
    let method = MbValue::from_ptr(MbObject::new_str("write".to_string()));
    let arg = MbValue::from_ptr(MbObject::new_str(s.to_string()));
    let args = MbValue::from_ptr(MbObject::new_list(vec![arg]));
    let _ = super::class::mb_call_method(target, method, args);
}

/// Begin capturing stdout output to an internal buffer.
/// Returns any previously captured content (useful for nested captures).
pub fn begin_capture() -> Option<Vec<u8>> {
    CAPTURE_BUF.with(|buf| buf.borrow_mut().replace(Vec::new()))
}

/// End capturing and return the captured bytes as a UTF-8 string.
/// Restores the previous capture state if `prev` is provided.
pub fn end_capture(prev: Option<Vec<u8>>) -> String {
    // First, flush any shared capture data from generator threads
    super::generator::flush_shared_capture();
    let captured = CAPTURE_BUF.with(|buf| {
        let mut b = buf.borrow_mut();
        let result = b.take().unwrap_or_default();
        *b = prev;
        result
    });
    String::from_utf8(captured)
        .unwrap_or_else(|e| String::from_utf8_lossy(&e.into_bytes()).into_owned())
}

/// Write a string to the capture buffer if active, otherwise to stdout.
/// Returns `true` if written to capture buffer.
///
/// Falls back to the generator shared capture buffer if this thread is a
/// generator thread (no local CAPTURE_BUF but has a shared capture set).
pub fn write_captured(s: &str) -> bool {
    // contextlib.redirect_stdout: route the program's stdout into the active
    // redirect target before any capture/stdout handling.
    if try_write_redirect(s) {
        return true;
    }
    let local = CAPTURE_BUF.with(|buf| {
        let mut b = buf.borrow_mut();
        if let Some(ref mut vec) = *b {
            let _ = vec.write_all(s.as_bytes());
            true
        } else {
            false
        }
    });
    if local {
        return true;
    }
    // Fallback: try generator shared capture buffer
    super::generator::write_shared_capture(s)
}

/// Write a line (with newline) to the capture buffer if active, else stdout.
/// Returns `true` if written to capture buffer.
pub fn writeln_captured(s: &str) -> bool {
    // contextlib.redirect_stdout: route the program's stdout (with the newline
    // print() adds) into the active redirect target.
    if STDOUT_REDIRECT.with(|stk| !stk.borrow().is_empty()) {
        let line = format!("{s}\n");
        return try_write_redirect(&line);
    }
    let local = CAPTURE_BUF.with(|buf| {
        let mut b = buf.borrow_mut();
        if let Some(ref mut vec) = *b {
            let _ = writeln!(vec, "{s}");
            true
        } else {
            false
        }
    });
    if local {
        return true;
    }
    // Fallback: try generator shared capture buffer
    let line = format!("{s}\n");
    super::generator::write_shared_capture(&line)
}

/// Check if capture is currently active.
pub fn is_capturing() -> bool {
    CAPTURE_BUF.with(|buf| buf.borrow().is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_capture() {
        let prev = begin_capture();
        assert!(is_capturing());
        write_captured("hello ");
        writeln_captured("world");
        let output = end_capture(prev);
        assert_eq!(output, "hello world\n");
        assert!(!is_capturing());
    }

    #[test]
    fn test_no_capture() {
        assert!(!is_capturing());
        assert!(!write_captured("ignored"));
        assert!(!writeln_captured("ignored"));
    }

    #[test]
    fn test_nested_capture() {
        let prev1 = begin_capture();
        write_captured("outer ");
        let prev2 = begin_capture();
        write_captured("inner");
        let inner = end_capture(prev2);
        assert_eq!(inner, "inner");
        write_captured("more");
        let outer = end_capture(prev1);
        assert_eq!(outer, "outer more");
    }
}
