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

use super::execution_context::with_current_context;
use super::rc::MbObject;
use super::value::MbValue;

thread_local! {
    static FALLBACK_CAPTURE_BUF: RefCell<Option<Vec<u8>>> = const { RefCell::new(None) };
    /// contextlib.redirect_stdout target stack. Each entry is the NaN-boxed
    /// bits of a writable stream (e.g. an io.StringIO). When non-empty, stdout
    /// output is routed to the top stream via `mb_stringio_write` instead of
    /// the process stdout / capture buffer. Pushed by `redirect_stdout.__enter__`
    /// and popped by `__exit__`.
    static FALLBACK_STDOUT_REDIRECT: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
    /// contextlib.redirect_stderr target stack (mirror of STDOUT_REDIRECT for
    /// `sys.stderr` writes).
    static FALLBACK_STDERR_REDIRECT: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
}

/// Push a stderr redirect target.
pub fn push_stderr_redirect(target: MbValue) {
    if with_current_context(|ctx| ctx.stderr_redirect.borrow_mut().push(target.to_bits())).is_none() {
        FALLBACK_STDERR_REDIRECT.with(|s| s.borrow_mut().push(target.to_bits()));
    }
}

/// Pop the most recent stderr redirect target.
pub fn pop_stderr_redirect() {
    if with_current_context(|ctx| ctx.stderr_redirect.borrow_mut().pop()).is_none() {
        FALLBACK_STDERR_REDIRECT.with(|s| {
            s.borrow_mut().pop();
        });
    }
}

/// If a stderr redirect is active, write `s` to the top target stream and
/// return true. Otherwise return false. Used by `print(..., file=sys.stderr)`.
pub fn try_write_stderr_redirect(s: &str) -> bool {
    let target_bits = with_current_context(|ctx| ctx.stderr_redirect.borrow().last().copied())
        .unwrap_or_else(|| FALLBACK_STDERR_REDIRECT.with(|stk| stk.borrow().last().copied()));
    if let Some(bits) = target_bits {
        let target = MbValue::from_bits(bits);
        write_redirect_target(target, s);
        return true;
    }
    false
}

/// Push a stdout redirect target (an io.StringIO-like stream value).
pub fn push_stdout_redirect(target: MbValue) {
    if with_current_context(|ctx| ctx.stdout_redirect.borrow_mut().push(target.to_bits())).is_none() {
        FALLBACK_STDOUT_REDIRECT.with(|s| s.borrow_mut().push(target.to_bits()));
    }
}

/// Pop the most recent stdout redirect target.
pub fn pop_stdout_redirect() {
    if with_current_context(|ctx| ctx.stdout_redirect.borrow_mut().pop()).is_none() {
        FALLBACK_STDOUT_REDIRECT.with(|s| {
            s.borrow_mut().pop();
        });
    }
}

/// If a stdout redirect is active, write `s` to the top target stream and
/// return true. Otherwise return false.
fn try_write_redirect(s: &str) -> bool {
    let target_bits = with_current_context(|ctx| ctx.stdout_redirect.borrow().last().copied())
        .unwrap_or_else(|| FALLBACK_STDOUT_REDIRECT.with(|stk| stk.borrow().last().copied()));
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
    with_current_context(|ctx| ctx.capture_buf.borrow_mut().replace(Vec::new()))
        .unwrap_or_else(|| FALLBACK_CAPTURE_BUF.with(|buf| buf.borrow_mut().replace(Vec::new())))
}

/// End capturing and return the captured bytes as a UTF-8 string.
/// Restores the previous capture state if `prev` is provided.
pub fn end_capture(prev: Option<Vec<u8>>) -> String {
    // First, flush any shared capture data from generator threads
    super::generator::flush_shared_capture();
    let captured = if with_current_context(|_| ()).is_some() {
        with_current_context(|ctx| {
            let mut b = ctx.capture_buf.borrow_mut();
            let result = b.take().unwrap_or_default();
            *b = prev;
            result
        }).unwrap()
    } else {
        FALLBACK_CAPTURE_BUF.with(|buf| {
            let mut b = buf.borrow_mut();
            let result = b.take().unwrap_or_default();
            *b = prev;
            result
        })
    };
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
    let local = with_current_context(|ctx| {
        let mut b = ctx.capture_buf.borrow_mut();
        if let Some(ref mut vec) = *b {
            let _ = vec.write_all(s.as_bytes());
            true
        } else {
            false
        }
    }).unwrap_or_else(|| {
        FALLBACK_CAPTURE_BUF.with(|buf| {
            let mut b = buf.borrow_mut();
            if let Some(ref mut vec) = *b {
                let _ = vec.write_all(s.as_bytes());
                true
            } else {
                false
            }
        })
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
    let stdout_redirect_active = with_current_context(|ctx| !ctx.stdout_redirect.borrow().is_empty())
        .unwrap_or_else(|| FALLBACK_STDOUT_REDIRECT.with(|stk| !stk.borrow().is_empty()));

    if stdout_redirect_active {
        let line = format!("{s}\n");
        return try_write_redirect(&line);
    }
    let local = with_current_context(|ctx| {
        let mut b = ctx.capture_buf.borrow_mut();
        if let Some(ref mut vec) = *b {
            let _ = writeln!(vec, "{s}");
            true
        } else {
            false
        }
    }).unwrap_or_else(|| {
        FALLBACK_CAPTURE_BUF.with(|buf| {
            let mut b = buf.borrow_mut();
            if let Some(ref mut vec) = *b {
                let _ = writeln!(vec, "{s}");
                true
            } else {
                false
            }
        })
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
    with_current_context(|ctx| ctx.capture_buf.borrow().is_some())
        .unwrap_or_else(|| FALLBACK_CAPTURE_BUF.with(|buf| buf.borrow().is_some()))
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

#[cfg(test)]
mod execution_context_output_isolation {
    use super::*;
    use crate::runtime::execution_context::{ExecutionContext, ExecutionPhase};

    #[test]
    fn test_interleaved_stdout_capture_one_thread() {
        let ctx_a = ExecutionContext::create();
        let ctx_b = ExecutionContext::create();

        let _guard_a = ctx_a.bind();
        let prev_a = begin_capture();
        write_captured("a1");

        {
            let _guard_b = ctx_b.bind();
            let prev_b = begin_capture();
            write_captured("b1");
            write_captured("b2");
            let out_b = end_capture(prev_b);
            assert_eq!(out_b, "b1b2");
        }

        write_captured("a2");
        let out_a = end_capture(prev_a);
        assert_eq!(out_a, "a1a2");
    }

    #[test]
    fn test_interleaved_redirect_stdout_one_thread() {
        let ctx_a = ExecutionContext::create();
        let ctx_b = ExecutionContext::create();

        let target_a = MbValue::from_bits(0x1111_2222_3333_4444);
        let target_b = MbValue::from_bits(0x5555_6666_7777_8888);

        let _guard_a = ctx_a.bind();
        push_stdout_redirect(target_a);

        {
            let _guard_b = ctx_b.bind();
            push_stdout_redirect(target_b);

            with_current_context(|ctx| {
                assert_eq!(ctx.stdout_redirect.borrow().as_slice(), &[target_b.to_bits()]);
            });
            pop_stdout_redirect();
            with_current_context(|ctx| {
                assert!(ctx.stdout_redirect.borrow().is_empty());
            });
        }

        with_current_context(|ctx| {
            assert_eq!(ctx.stdout_redirect.borrow().as_slice(), &[target_a.to_bits()]);
        });
        pop_stdout_redirect();
        with_current_context(|ctx| {
            assert!(ctx.stdout_redirect.borrow().is_empty());
        });
    }

    #[test]
    fn test_interleaved_redirect_stderr_one_thread() {
        let ctx_a = ExecutionContext::create();
        let ctx_b = ExecutionContext::create();

        let target_a = MbValue::from_bits(0xaaaa_bbbb_cccc_dddd);
        let target_b = MbValue::from_bits(0x1122_3344_5566_7788);

        let _guard_a = ctx_a.bind();
        push_stderr_redirect(target_a);

        {
            let _guard_b = ctx_b.bind();
            push_stderr_redirect(target_b);

            with_current_context(|ctx| {
                assert_eq!(ctx.stderr_redirect.borrow().as_slice(), &[target_b.to_bits()]);
            });
            pop_stderr_redirect();
            with_current_context(|ctx| {
                assert!(ctx.stderr_redirect.borrow().is_empty());
            });
        }

        with_current_context(|ctx| {
            assert_eq!(ctx.stderr_redirect.borrow().as_slice(), &[target_a.to_bits()]);
        });
        pop_stderr_redirect();
        with_current_context(|ctx| {
            assert!(ctx.stderr_redirect.borrow().is_empty());
        });
    }

    #[test]
    fn test_nested_capture_within_one_context() {
        let ctx = ExecutionContext::create();
        let _guard = ctx.bind();

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

    #[test]
    fn test_teardown_leaves_other_context_capture_untouched() {
        let ctx_a = ExecutionContext::create();
        let ctx_b = ExecutionContext::create();

        {
            let _guard_a = ctx_a.bind();
            begin_capture();
            write_captured("data_a");
        }

        {
            let _guard_b = ctx_b.bind();
            begin_capture();
            write_captured("data_b");
        }

        // Teardown ctx_a
        ctx_a.teardown();

        // Check ctx_b state is untouched
        {
            let _guard_b = ctx_b.bind();
            assert!(is_capturing());
            let out_b = end_capture(None);
            assert_eq!(out_b, "data_b");
        }
    }

    #[test]
    fn test_teardown_cleans_capture_state_idempotently() {
        let ctx = ExecutionContext::create();

        {
            let _guard = ctx.bind();
            begin_capture();
            write_captured("temporary");
            push_stdout_redirect(MbValue::from_bits(12345));
            push_stderr_redirect(MbValue::from_bits(67890));
        }

        ctx.teardown();

        assert!(ctx.capture_buf.borrow().is_none());
        assert!(ctx.stdout_redirect.borrow().is_empty());
        assert!(ctx.stderr_redirect.borrow().is_empty());
        assert_eq!(ctx.phase(), ExecutionPhase::Retired);

        // Second teardown call should be idempotent no-op
        ctx.teardown();

        assert!(ctx.capture_buf.borrow().is_none());
        assert!(ctx.stdout_redirect.borrow().is_empty());
        assert!(ctx.stderr_redirect.borrow().is_empty());
        assert_eq!(ctx.phase(), ExecutionPhase::Retired);
    }
}

