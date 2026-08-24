use std::cell::RefCell;
use std::marker::PhantomData;
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};

static NEXT_CONTEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Opaque ABI-safe token representing an execution context identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ContextHandle(pub NonZeroU64);

impl ContextHandle {
    pub fn new(val: u64) -> Option<Self> {
        NonZeroU64::new(val).map(ContextHandle)
    }

    pub fn as_u64(&self) -> u64 {
        self.0.get()
    }
}

/// Execution phase lifecycle state for an ExecutionContext.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ExecutionPhase {
    Created = 0,
    Installed = 1,
    Executing = 2,
    Quiescing = 3,
    Retired = 4,
    Failed = 5,
}

impl ExecutionPhase {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => ExecutionPhase::Created,
            1 => ExecutionPhase::Installed,
            2 => ExecutionPhase::Executing,
            3 => ExecutionPhase::Quiescing,
            4 => ExecutionPhase::Retired,
            5 => ExecutionPhase::Failed,
            _ => ExecutionPhase::Failed,
        }
    }
}

/// Aggregate root representing execution-local state boundaries.
pub struct ExecutionContext {
    handle: ContextHandle,
    phase: AtomicU8,
    sentinel: AtomicU64,
    pub(crate) capture_buf: RefCell<Option<Vec<u8>>>,
    pub(crate) stdout_redirect: RefCell<Vec<u64>>,
    pub(crate) stderr_redirect: RefCell<Vec<u64>>,
    pub(crate) current_exception: RefCell<Option<super::exception::MbException>>,
    pub(crate) modules: RefCell<std::collections::HashMap<String, super::module::MbModule>>,
    pub(crate) module_value_ptrs: RefCell<std::collections::HashSet<u64>>,
    pub(crate) search_paths: RefCell<Vec<std::path::PathBuf>>,
    pub(crate) module_jit_backends: RefCell<Vec<Box<crate::codegen::cranelift::jit::CraneliftJitBackend>>>,
    pub(crate) script_dir: RefCell<Option<std::path::PathBuf>>,
    pub(crate) current_module_package: RefCell<Option<String>>,
}

impl ExecutionContext {
    pub fn create() -> Self {
        let id = NEXT_CONTEXT_ID.fetch_add(1, Ordering::Relaxed);
        let nz = NonZeroU64::new(id).unwrap_or_else(|| NonZeroU64::new(u64::MAX).unwrap());
        let handle = ContextHandle(nz);
        let initial_sentinel = id * 1000 + 42;
        Self {
            handle,
            phase: AtomicU8::new(ExecutionPhase::Created as u8),
            sentinel: AtomicU64::new(initial_sentinel),
            capture_buf: RefCell::new(None),
            stdout_redirect: RefCell::new(Vec::new()),
            stderr_redirect: RefCell::new(Vec::new()),
            current_exception: RefCell::new(None),
            modules: RefCell::new(std::collections::HashMap::new()),
            module_value_ptrs: RefCell::new(std::collections::HashSet::new()),
            search_paths: RefCell::new(vec![std::path::PathBuf::from(".")]),
            module_jit_backends: RefCell::new(Vec::new()),
            script_dir: RefCell::new(None),
            current_module_package: RefCell::new(None),
        }
    }

    pub fn new() -> Self {
        Self::create()
    }

    pub fn handle(&self) -> ContextHandle {
        self.handle
    }

    pub fn phase(&self) -> ExecutionPhase {
        ExecutionPhase::from_u8(self.phase.load(Ordering::SeqCst))
    }

    pub fn set_phase(&self, phase: ExecutionPhase) {
        self.phase.store(phase as u8, Ordering::SeqCst);
    }

    pub fn sentinel(&self) -> u64 {
        self.sentinel.load(Ordering::SeqCst)
    }

    pub fn set_sentinel(&self, val: u64) {
        self.sentinel.store(val, Ordering::SeqCst);
    }

    pub fn bind(&self) -> ContextBindingGuard<'_> {
        let guard = bind_context(self);
        self.set_phase(ExecutionPhase::Installed);
        guard
    }

    /// Idempotent context-local teardown.
    ///
    /// Transitions phase from Created/Installed/Executing/Failed -> Quiescing -> Retired.
    /// Mutates only this context's sentinel and output capture/redirect state.
    pub fn teardown(&self) {
        let current_p = self.phase();
        if current_p == ExecutionPhase::Retired {
            return;
        }
        self.set_phase(ExecutionPhase::Quiescing);
        // Reset context-local sentinel
        self.sentinel.store(0, Ordering::SeqCst);
        // Reset context-local output capture and redirect state
        *self.capture_buf.borrow_mut() = None;
        self.stdout_redirect.borrow_mut().clear();
        self.stderr_redirect.borrow_mut().clear();
        // Reset context-local exception state
        *self.current_exception.borrow_mut() = None;
        // Reset context-local module, search-path, JIT backend, script dir, package state
        self.modules.borrow_mut().clear();
        self.module_value_ptrs.borrow_mut().clear();
        *self.search_paths.borrow_mut() = vec![std::path::PathBuf::from(".")];
        self.module_jit_backends.borrow_mut().clear();
        *self.script_dir.borrow_mut() = None;
        *self.current_module_package.borrow_mut() = None;
        self.set_phase(ExecutionPhase::Retired);
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::create()
    }
}

struct ActiveContextBinding {
    handle: ContextHandle,
    ctx_ptr: *const ExecutionContext,
}

thread_local! {
    static CONTEXT_STACK: RefCell<Vec<ActiveContextBinding>> = const { RefCell::new(Vec::new()) };
}

/// RAII guard that pops the context handle from the TLS stack on drop.
pub struct ContextBindingGuard<'ctx> {
    _ctx: PhantomData<&'ctx ExecutionContext>,
}

impl Drop for ContextBindingGuard<'_> {
    fn drop(&mut self) {
        CONTEXT_STACK.with(|stack| {
            let mut s = stack.borrow_mut();
            let popped = s.pop();
            debug_assert!(popped.is_some(), "Context stack underflow in TLS drop");
        });
    }
}

/// Binds an execution context to the current thread.
///
/// ```compile_fail
/// use mamba::runtime::execution_context::ExecutionContext;
///
/// let escaped_guard = {
///     let ctx = ExecutionContext::create();
///     ctx.bind()
/// };
/// ```
pub fn bind_context(ctx: &ExecutionContext) -> ContextBindingGuard<'_> {
    CONTEXT_STACK.with(|stack| {
        stack.borrow_mut().push(ActiveContextBinding {
            handle: ctx.handle(),
            ctx_ptr: ctx as *const ExecutionContext,
        });
    });
    ContextBindingGuard { _ctx: PhantomData }
}

pub fn try_current_context_handle() -> Option<ContextHandle> {
    CONTEXT_STACK.with(|stack| stack.borrow().last().map(|b| b.handle))
}

pub fn current_context_handle() -> ContextHandle {
    match try_current_context_handle() {
        Some(h) => h,
        None => {
            panic!("No ExecutionContext installed on the current thread");
        }
    }
}

pub fn resolve_current_context_handle() -> ContextHandle {
    current_context_handle()
}

pub fn with_current_context<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&ExecutionContext) -> R,
{
    CONTEXT_STACK.with(|stack| {
        let s = stack.borrow();
        if let Some(binding) = s.last() {
            if !binding.ctx_ptr.is_null() {
                // SAFETY: binding.ctx_ptr is valid for the duration of ContextBindingGuard on this thread.
                unsafe {
                    let ctx = &*binding.ctx_ptr;
                    return Some(f(ctx));
                }
            }
        }
        None
    })
}

#[cfg(test)]
mod execution_context_binding {
    use super::*;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::thread;

    #[test]
    fn test_nested_binding_restores_after_normal_return() {
        let ctx_a = ExecutionContext::create();
        let ctx_b = ExecutionContext::create();

        let _guard_a = ctx_a.bind();
        assert_eq!(current_context_handle(), ctx_a.handle());

        {
            let _guard_b = ctx_b.bind();
            assert_eq!(current_context_handle(), ctx_b.handle());
        }

        assert_eq!(current_context_handle(), ctx_a.handle());
    }

    #[test]
    fn test_nested_binding_restores_after_panic() {
        let ctx_a = ExecutionContext::create();
        let ctx_b = ExecutionContext::create();

        let _guard_a = ctx_a.bind();
        assert_eq!(current_context_handle(), ctx_a.handle());

        let result = catch_unwind(AssertUnwindSafe(|| {
            let _guard_b = ctx_b.bind();
            assert_eq!(current_context_handle(), ctx_b.handle());
            panic!("deliberate panic inside nested context");
        }));

        assert!(result.is_err());
        assert_eq!(current_context_handle(), ctx_a.handle());
    }

    #[test]
    fn test_resolving_with_no_context_installed_fails_explicitly() {
        let handle = thread::spawn(|| {
            catch_unwind(AssertUnwindSafe(|| {
                current_context_handle();
            }))
        })
        .join()
        .unwrap();

        assert!(handle.is_err());
    }

    #[test]
    fn test_two_contexts_on_two_threads_each_see_own_handle() {
        let (tx1, rx1) = std::sync::mpsc::channel();
        let (tx2, rx2) = std::sync::mpsc::channel();

        let t1 = thread::spawn(move || {
            let ctx_a = ExecutionContext::create();
            let handle_a = ctx_a.handle();
            let _guard_a = ctx_a.bind();
            assert_eq!(current_context_handle(), handle_a);
            tx1.send(handle_a).unwrap();
        });

        let t2 = thread::spawn(move || {
            let ctx_b = ExecutionContext::create();
            let handle_b = ctx_b.handle();
            let _guard_b = ctx_b.bind();
            assert_eq!(current_context_handle(), handle_b);
            tx2.send(handle_b).unwrap();
        });

        let handle_a = rx1.recv().unwrap();
        let handle_b = rx2.recv().unwrap();
        assert_ne!(handle_a, handle_b);

        t1.join().unwrap();
        t2.join().unwrap();
    }

    #[test]
    fn test_teardown_leaves_other_sentinel_intact() {
        let ctx_a = ExecutionContext::create();
        let ctx_b = ExecutionContext::create();

        ctx_a.set_sentinel(111);
        ctx_b.set_sentinel(222);

        ctx_a.teardown();

        assert_eq!(ctx_a.sentinel(), 0);
        assert_eq!(ctx_a.phase(), ExecutionPhase::Retired);
        assert_eq!(ctx_b.sentinel(), 222);
        assert_ne!(ctx_b.phase(), ExecutionPhase::Retired);
    }

    #[test]
    fn test_teardown_is_idempotent() {
        let ctx = ExecutionContext::create();
        ctx.set_sentinel(999);

        ctx.teardown();
        assert_eq!(ctx.sentinel(), 0);
        assert_eq!(ctx.phase(), ExecutionPhase::Retired);

        // Second teardown call should be a no-op and not panic
        ctx.teardown();
        assert_eq!(ctx.sentinel(), 0);
        assert_eq!(ctx.phase(), ExecutionPhase::Retired);
    }

    #[test]
    fn test_teardown_after_partial_setup_or_failed() {
        // Teardown after partial setup (Created phase)
        let ctx_created = ExecutionContext::create();
        assert_eq!(ctx_created.phase(), ExecutionPhase::Created);
        ctx_created.teardown();
        assert_eq!(ctx_created.phase(), ExecutionPhase::Retired);

        // Teardown after Failed phase
        let ctx_failed = ExecutionContext::create();
        ctx_failed.set_phase(ExecutionPhase::Failed);
        ctx_failed.teardown();
        assert_eq!(ctx_failed.phase(), ExecutionPhase::Retired);
    }
}

#[cfg(test)]
mod execution_context_module_isolation {
    use super::*;
    use crate::codegen::cranelift::jit::CraneliftJitBackend;
    use crate::runtime::module::{
        mb_module_register, with_current_module_package, with_module_jit_backends,
        with_module_value_ptrs, with_modules, with_search_paths, with_script_dir, MbModule,
    };
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_module_isolation_two_contexts_distinct_modules() {
        let ctx_a = ExecutionContext::create();
        let ctx_b = ExecutionContext::create();

        {
            let _guard_a = ctx_a.bind();
            let mut attrs_a = HashMap::new();
            attrs_a.insert("val_a".to_string(), super::super::value::MbValue::from_int(100));
            mb_module_register("mod_a", attrs_a);

            assert!(with_modules(|m| m.borrow().contains_key("mod_a")));
            assert!(with_modules(|m| !m.borrow().contains_key("mod_b")));
        }

        {
            let _guard_b = ctx_b.bind();
            let mut attrs_b = HashMap::new();
            attrs_b.insert("val_b".to_string(), super::super::value::MbValue::from_int(200));
            mb_module_register("mod_b", attrs_b);

            // Discriminator assertion: would FAIL pre-migration on a single thread.
            assert!(with_modules(|m| m.borrow().contains_key("mod_b")));
            assert!(with_modules(|m| !m.borrow().contains_key("mod_a")));
        }

        {
            let _guard_a = ctx_a.bind();
            assert!(with_modules(|m| m.borrow().contains_key("mod_a")));
            assert!(with_modules(|m| !m.borrow().contains_key("mod_b")));
        }
    }

    #[test]
    fn test_search_paths_isolation_across_contexts() {
        let ctx_a = ExecutionContext::create();
        let ctx_b = ExecutionContext::create();

        {
            let _guard_a = ctx_a.bind();
            with_search_paths(|sp| sp.borrow_mut().push(PathBuf::from("/path/ctx_a")));
            assert!(with_search_paths(|sp| sp.borrow().contains(&PathBuf::from("/path/ctx_a"))));
        }

        {
            let _guard_b = ctx_b.bind();
            assert!(with_search_paths(|sp| !sp.borrow().contains(&PathBuf::from("/path/ctx_a"))));
            with_search_paths(|sp| sp.borrow_mut().push(PathBuf::from("/path/ctx_b")));
            assert!(with_search_paths(|sp| sp.borrow().contains(&PathBuf::from("/path/ctx_b"))));
        }

        {
            let _guard_a = ctx_a.bind();
            assert!(with_search_paths(|sp| sp.borrow().contains(&PathBuf::from("/path/ctx_a"))));
            assert!(with_search_paths(|sp| !sp.borrow().contains(&PathBuf::from("/path/ctx_b"))));
        }
    }

    #[test]
    fn test_script_dir_and_package_isolation_across_contexts() {
        let ctx_a = ExecutionContext::create();
        let ctx_b = ExecutionContext::create();

        {
            let _guard_a = ctx_a.bind();
            with_script_dir(|sd| *sd.borrow_mut() = Some(PathBuf::from("/script/dir_a")));
            with_current_module_package(|cp| *cp.borrow_mut() = Some("pkg_a".to_string()));

            assert_eq!(with_script_dir(|sd| sd.borrow().clone()), Some(PathBuf::from("/script/dir_a")));
            assert_eq!(with_current_module_package(|cp| cp.borrow().clone()), Some("pkg_a".to_string()));
        }

        {
            let _guard_b = ctx_b.bind();
            assert_eq!(with_script_dir(|sd| sd.borrow().clone()), None);
            assert_eq!(with_current_module_package(|cp| cp.borrow().clone()), None);

            with_script_dir(|sd| *sd.borrow_mut() = Some(PathBuf::from("/script/dir_b")));
            with_current_module_package(|cp| *cp.borrow_mut() = Some("pkg_b".to_string()));
            assert_eq!(with_script_dir(|sd| sd.borrow().clone()), Some(PathBuf::from("/script/dir_b")));
            assert_eq!(with_current_module_package(|cp| cp.borrow().clone()), Some("pkg_b".to_string()));
        }

        {
            let _guard_a = ctx_a.bind();
            assert_eq!(with_script_dir(|sd| sd.borrow().clone()), Some(PathBuf::from("/script/dir_a")));
            assert_eq!(with_current_module_package(|cp| cp.borrow().clone()), Some("pkg_a".to_string()));
        }
    }

    #[test]
    fn test_module_jit_backends_isolation_asymmetric_counts() {
        let ctx_a = ExecutionContext::create();
        let ctx_b = ExecutionContext::create();

        {
            let _guard_a = ctx_a.bind();
            let b1 = CraneliftJitBackend::new().expect("backend 1");
            with_module_jit_backends(|b| b.borrow_mut().push(Box::new(b1)));
            assert_eq!(with_module_jit_backends(|b| b.borrow().len()), 1);
        }

        {
            let _guard_b = ctx_b.bind();
            let b2 = CraneliftJitBackend::new().expect("backend 2");
            let b3 = CraneliftJitBackend::new().expect("backend 3");
            with_module_jit_backends(|b| {
                let mut mut_b = b.borrow_mut();
                mut_b.push(Box::new(b2));
                mut_b.push(Box::new(b3));
            });
            assert_eq!(with_module_jit_backends(|b| b.borrow().len()), 2);
        }

        {
            let _guard_a = ctx_a.bind();
            assert_eq!(with_module_jit_backends(|b| b.borrow().len()), 1);
        }

        ctx_a.teardown();

        {
            let _guard_b = ctx_b.bind();
            assert_eq!(with_module_jit_backends(|b| b.borrow().len()), 2);
        }
    }

    #[test]
    fn test_teardown_leaves_other_context_jit_sentinel_intact() {
        let ctx_a = ExecutionContext::create();
        let ctx_b = ExecutionContext::create();

        {
            let _guard_a = ctx_a.bind();
            with_modules(|m| {
                m.borrow_mut().insert(
                    "sentinel_mod_a".to_string(),
                    MbModule {
                        name: "sentinel_mod_a".to_string(),
                        file: None,
                        attrs: HashMap::new(),
                        is_package: false,
                        cached_value: None,
                    },
                );
            });
            with_module_value_ptrs(|s| { s.borrow_mut().insert(1001); });
            let backend_a = CraneliftJitBackend::new().expect("jit backend a");
            with_module_jit_backends(|b| b.borrow_mut().push(Box::new(backend_a)));
        }

        {
            let _guard_b = ctx_b.bind();
            with_modules(|m| {
                m.borrow_mut().insert(
                    "sentinel_mod_b".to_string(),
                    MbModule {
                        name: "sentinel_mod_b".to_string(),
                        file: None,
                        attrs: HashMap::new(),
                        is_package: false,
                        cached_value: None,
                    },
                );
            });
            with_module_value_ptrs(|s| { s.borrow_mut().insert(2002); });
            let backend_b = CraneliftJitBackend::new().expect("jit backend b");
            with_module_jit_backends(|b| b.borrow_mut().push(Box::new(backend_b)));
        }

        // Teardown ctx_a
        ctx_a.teardown();

        {
            let _guard_b = ctx_b.bind();
            assert!(with_modules(|m| m.borrow().contains_key("sentinel_mod_b")));
            assert!(with_module_value_ptrs(|s| s.borrow().contains(&2002)));
            assert_eq!(with_module_jit_backends(|b| b.borrow().len()), 1);
        }
    }

    #[test]
    fn test_teardown_is_idempotent_and_does_not_disturb_other_context() {
        let ctx_a = ExecutionContext::create();
        let ctx_b = ExecutionContext::create();

        {
            let _guard_a = ctx_a.bind();
            with_search_paths(|sp| sp.borrow_mut().push(PathBuf::from("/path/a")));
        }
        {
            let _guard_b = ctx_b.bind();
            with_search_paths(|sp| sp.borrow_mut().push(PathBuf::from("/path/b")));
        }

        ctx_a.teardown();
        ctx_a.teardown();

        {
            let _guard_b = ctx_b.bind();
            assert!(with_search_paths(|sp| sp.borrow().contains(&PathBuf::from("/path/b"))));
        }
    }

    #[test]
    fn test_fallback_path_with_no_context_bound() {
        assert!(with_current_context(|_| ()).is_none());

        with_search_paths(|sp| sp.borrow_mut().push(PathBuf::from("/fallback/path")));
        assert!(with_search_paths(|sp| sp.borrow().contains(&PathBuf::from("/fallback/path"))));

        with_script_dir(|sd| *sd.borrow_mut() = Some(PathBuf::from("/fallback/script")));
        assert_eq!(with_script_dir(|sd| sd.borrow().clone()), Some(PathBuf::from("/fallback/script")));

        with_script_dir(|sd| *sd.borrow_mut() = None);
        with_search_paths(|sp| {
            sp.borrow_mut().clear();
            sp.borrow_mut().push(PathBuf::from("."));
        });
    }

    #[test]
    fn test_module_jit_backends_fallback_and_seam() {
        assert!(with_current_context(|_| ()).is_none());
        let backend = CraneliftJitBackend::new().expect("fallback backend");
        with_module_jit_backends(|b| b.borrow_mut().push(Box::new(backend)));
        assert_eq!(with_module_jit_backends(|b| b.borrow().len()), 1);
        with_module_jit_backends(|b| b.borrow_mut().clear());
        assert_eq!(with_module_jit_backends(|b| b.borrow().len()), 0);
    }
}

