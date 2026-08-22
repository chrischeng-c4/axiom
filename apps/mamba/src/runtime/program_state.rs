use std::collections::{HashMap, HashSet};
use std::sync::{Arc, OnceLock, RwLock};
use crate::runtime::closure::{MbClosure, MbParamInfo, ScopedSymbolKey, SymTy};
use crate::runtime::value::MbValue;

/// `ProgramState` represents program-wide, thread-shared symbol/global namespace and closure/cell state.
///
/// It is `Send + Sync`, wrapped in an `Arc`, and uses `RwLock` for `Sync`-safe interior mutability.
pub struct ProgramState {
    pub globals: RwLock<HashMap<String, MbValue>>,
    pub global_ids: RwLock<HashMap<ScopedSymbolKey, MbValue>>,
    pub closures: RwLock<Vec<Option<MbClosure>>>,
    pub cells: RwLock<Vec<Option<MbValue>>>,
    pub func_names: RwLock<HashMap<u64, String>>,
    pub func_qualnames: RwLock<HashMap<u64, String>>,
    pub func_docs: RwLock<HashMap<u64, String>>,
    pub func_modules: RwLock<HashMap<u64, String>>,
    pub func_argcounts: RwLock<HashMap<u64, i64>>,
    pub func_varnames: RwLock<HashMap<u64, Vec<String>>>,
    pub func_flags: RwLock<HashMap<u64, i64>>,
    pub func_freevars: RwLock<HashMap<u64, Vec<(String, i64)>>>,
    pub func_params: RwLock<HashMap<u64, Vec<MbParamInfo>>>,
    pub func_boxed_params: RwLock<HashSet<u64>>,
    pub func_ret_annos: RwLock<HashMap<u64, String>>,
    pub func_lines: RwLock<HashMap<u64, i64>>,
    pub func_files: RwLock<HashMap<u64, String>>,
    pub module_sym_info: RwLock<HashMap<i64, (String, SymTy)>>,
    pub module_func_info: RwLock<HashMap<String, MbValue>>,
}

impl Default for ProgramState {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgramState {
    pub fn new() -> Self {
        Self {
            globals: RwLock::new(HashMap::new()),
            global_ids: RwLock::new(HashMap::new()),
            closures: RwLock::new(Vec::new()),
            cells: RwLock::new(Vec::new()),
            func_names: RwLock::new(HashMap::new()),
            func_qualnames: RwLock::new(HashMap::new()),
            func_docs: RwLock::new(HashMap::new()),
            func_modules: RwLock::new(HashMap::new()),
            func_argcounts: RwLock::new(HashMap::new()),
            func_varnames: RwLock::new(HashMap::new()),
            func_flags: RwLock::new(HashMap::new()),
            func_freevars: RwLock::new(HashMap::new()),
            func_params: RwLock::new(HashMap::new()),
            func_boxed_params: RwLock::new(HashSet::new()),
            func_ret_annos: RwLock::new(HashMap::new()),
            func_lines: RwLock::new(HashMap::new()),
            func_files: RwLock::new(HashMap::new()),
            module_sym_info: RwLock::new(HashMap::new()),
            module_func_info: RwLock::new(HashMap::new()),
        }
    }

    /// Clear all stored values in `ProgramState`.
    /// Used during runtime reset (`cleanup_all_closures`).
    pub fn clear(&self) {
        if let Ok(mut g) = self.globals.write() {
            g.clear();
        }
        if let Ok(mut g_id) = self.global_ids.write() {
            g_id.clear();
        }
        if let Ok(mut c) = self.closures.write() {
            c.clear();
        }
        if let Ok(mut c) = self.cells.write() {
            c.clear();
        }
        if let Ok(mut m) = self.func_names.write() {
            m.clear();
        }
        if let Ok(mut m) = self.func_qualnames.write() {
            m.clear();
        }
        if let Ok(mut m) = self.func_docs.write() {
            m.clear();
        }
        if let Ok(mut m) = self.func_modules.write() {
            m.clear();
        }
        if let Ok(mut m) = self.func_argcounts.write() {
            m.clear();
        }
        if let Ok(mut m) = self.func_varnames.write() {
            m.clear();
        }
        if let Ok(mut m) = self.func_flags.write() {
            m.clear();
        }
        if let Ok(mut m) = self.func_freevars.write() {
            m.clear();
        }
        if let Ok(mut m) = self.func_params.write() {
            m.clear();
        }
        if let Ok(mut m) = self.func_boxed_params.write() {
            m.clear();
        }
        if let Ok(mut m) = self.func_ret_annos.write() {
            m.clear();
        }
        if let Ok(mut m) = self.func_lines.write() {
            m.clear();
        }
        if let Ok(mut m) = self.func_files.write() {
            m.clear();
        }
        if let Ok(mut m) = self.module_sym_info.write() {
            m.clear();
        }
        if let Ok(mut m) = self.module_func_info.write() {
            m.clear();
        }
    }
}

static GLOBAL_PROGRAM_STATE: OnceLock<Arc<ProgramState>> = OnceLock::new();

/// Returns the process-global `ProgramState`.
pub fn current_program_state() -> Arc<ProgramState> {
    global_program_state()
}

/// Returns the lazily initialized process-global `ProgramState`.
pub fn global_program_state() -> Arc<ProgramState> {
    GLOBAL_PROGRAM_STATE
        .get_or_init(|| Arc::new(ProgramState::new()))
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn test_program_state_is_send_and_sync() {
        assert_send_sync::<ProgramState>();
    }
}

#[cfg(test)]
mod program_state_globals_sharing {
    use super::*;
    use std::sync::{Arc, Barrier, Mutex};
    use crate::runtime::closure::{mb_global_set_id, mb_global_get_id_raw};
    use crate::runtime::value::MbValue;

    #[test]
    fn test_worker_rebind_visible_before_join() {
        let _jit_guard = crate::codegen::cranelift::jit::JIT_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        crate::runtime::closure::push_active_module_name("__main__".to_string());
        let barrier1 = Arc::new(Barrier::new(2));
        let barrier2 = barrier1.clone();
        let sym_id = 999901i64;
        let id_val = MbValue::from_bits(sym_id as u64);

        mb_global_set_id(id_val, MbValue::from_int(10));

        let handle = std::thread::spawn(move || {
            crate::runtime::closure::push_active_module_name("__main__".to_string());
            mb_global_set_id(id_val, MbValue::from_int(42));
            barrier2.wait(); // Signal parent that write is done
            barrier2.wait(); // Wait for parent observation before exiting
        });

        barrier1.wait(); // Wait for worker to write value
        let val = mb_global_get_id_raw(sym_id);
        assert_eq!(
            val.as_int(),
            Some(42),
            "Parent must observe worker rebind while worker is still running!"
        );
        barrier1.wait(); // Release worker
        handle.join().unwrap();
    }

    #[test]
    fn test_two_workers_summing_globals() {
        let _jit_guard = crate::codegen::cranelift::jit::JIT_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        crate::runtime::closure::push_active_module_name("__main__".to_string());
        let sym_id = 999902i64;
        let id_val = MbValue::from_bits(sym_id as u64);
        mb_global_set_id(id_val, MbValue::from_int(0));

        let mutex = Arc::new(Mutex::new(()));
        let m1 = mutex.clone();
        let m2 = mutex.clone();

        let t1 = std::thread::spawn(move || {
            crate::runtime::closure::push_active_module_name("__main__".to_string());
            for _ in 0..100 {
                let _guard = m1.lock().unwrap();
                let cur = mb_global_get_id_raw(sym_id).as_int().unwrap_or(0);
                mb_global_set_id(id_val, MbValue::from_int(cur + 1));
            }
        });

        let t2 = std::thread::spawn(move || {
            crate::runtime::closure::push_active_module_name("__main__".to_string());
            for _ in 0..100 {
                let _guard = m2.lock().unwrap();
                let cur = mb_global_get_id_raw(sym_id).as_int().unwrap_or(0);
                mb_global_set_id(id_val, MbValue::from_int(cur + 1));
            }
        });

        t1.join().unwrap();
        t2.join().unwrap();

        let final_val = mb_global_get_id_raw(sym_id).as_int();
        assert_eq!(
            final_val,
            Some(200),
            "Two workers summing must accumulate to 200, not last-writer-win!"
        );
    }

    #[test]
    fn test_parent_bind_after_spawn_visible_to_worker() {
        let _jit_guard = crate::codegen::cranelift::jit::JIT_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        crate::runtime::closure::push_active_module_name("__main__".to_string());
        let barrier1 = Arc::new(Barrier::new(2));
        let barrier2 = barrier1.clone();
        let sym_id = 999903i64;
        let id_val = MbValue::from_bits(sym_id as u64);

        let handle = std::thread::spawn(move || {
            crate::runtime::closure::push_active_module_name("__main__".to_string());
            barrier2.wait(); // Wait for parent to set global
            let val = mb_global_get_id_raw(sym_id);
            assert_eq!(
                val.as_int(),
                Some(777),
                "Worker must see global bound by parent after worker start!"
            );
        });

        mb_global_set_id(id_val, MbValue::from_int(777));
        barrier1.wait(); // Signal worker to read
        handle.join().unwrap();
    }

    #[test]
    fn test_program_state_cleanup_idempotent() {
        let _jit_guard = crate::codegen::cranelift::jit::JIT_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        use crate::runtime::closure::cleanup_all_closures;
        crate::runtime::closure::push_active_module_name("__main__".to_string());

        let sym_id = 999904i64;
        let id_val = MbValue::from_bits(sym_id as u64);

        mb_global_set_id(id_val, MbValue::from_int(123));
        assert_eq!(mb_global_get_id_raw(sym_id).as_int(), Some(123));

        cleanup_all_closures();
        assert_eq!(mb_global_get_id_raw(sym_id).as_int(), None);

        // Idempotent second call
        cleanup_all_closures();
        assert_eq!(mb_global_get_id_raw(sym_id).as_int(), None);
    }

    #[test]
    fn test_concurrent_import_globals_not_cleared() {
        let _jit_guard = crate::codegen::cranelift::jit::JIT_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        use crate::runtime::closure::{push_active_module_name, pop_active_module_name};

        let sym_id = 999905i64;
        let id_val = MbValue::from_bits(sym_id as u64);

        push_active_module_name("__main__".to_string());
        mb_global_set_id(id_val, MbValue::from_int(555));

        let barrier1 = Arc::new(Barrier::new(2));
        let barrier2 = barrier1.clone();

        let handle = std::thread::spawn(move || {
            push_active_module_name("worker_mod".to_string());
            mb_global_set_id(MbValue::from_bits(111), MbValue::from_int(999));
            barrier2.wait();
            barrier2.wait();
            pop_active_module_name();
        });

        barrier1.wait();
        let val = mb_global_get_id_raw(sym_id);
        assert_eq!(
            val.as_int(),
            Some(555),
            "Main thread's globals must not be wiped by worker thread!"
        );
        barrier1.wait();
        handle.join().unwrap();
        pop_active_module_name();
    }
}
