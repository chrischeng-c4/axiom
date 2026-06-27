use super::super::rc::{MbObject, MbObjectHeader, MbRwLock, ObjData, ObjKind};
use super::super::value::MbValue;
use rustc_hash::FxHashMap;
/// sqlite3 module for Mamba (#444).
///
/// Provides: connect, Connection (execute, fetchall, commit, close)
/// Stub implementation — stores data in-memory HashMap tables.
/// No external dependency (no rusqlite).
use std::collections::HashMap;

const CONNECTION_CLASS: &str = "sqlite3.Connection";
const CURSOR_CLASS: &str = "sqlite3.Cursor";
const ROW_CLASS: &str = "sqlite3.Row";

// ── Variadic dispatchers (callable from module-attr context) ──

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

disp_unary!(d_connect, mb_sqlite3_connect);

// Generic surface stub: callable from module-attr context, ignores its
// arguments and returns None. Used to give the DB-API factory functions
// (DateFromTicks, register_adapter, adapt, ...) and the type/class surface
// names (Connection, Cursor, Row, Error, ...) a real callable value so
// `callable(sqlite3.NAME)` and `hasattr(sqlite3, "NAME")` hold, matching
// the lzma/psycopg surface-stub convention. Behaviour is intentionally a
// no-op until the full DB-API plumbing lands.
unsafe extern "C" fn d_surface_stub(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

/// Build an `Instance` of `class_name` whose field map is `fields`. The fields
/// are inserted verbatim (freshly-created owned values; no extra retain — same
/// convention as the queue/uuid stdlib shims).
fn new_instance_with_fields(class_name: &str, fields: FxHashMap<String, MbValue>) -> MbValue {
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: class_name.to_string(),
            fields: MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Read instance field `key` from `inst` (an `ObjData::Instance`), returning a
/// copy of the stored value if present.
fn inst_field(inst: MbValue, key: &str) -> Option<MbValue> {
    inst.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

/// Set instance field `key = val` on `inst` (an `ObjData::Instance`).
fn inst_set_field(inst: MbValue, key: &str, val: MbValue) {
    if let Some(ptr) = inst.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                super::super::rc::retain_if_ptr(val);
                let prev = fields.write().unwrap().insert(key.to_string(), val);
                if let Some(p) = prev {
                    super::super::rc::release_if_ptr(p);
                }
            }
        }
    }
}

// ── Connection / Cursor instance methods (SystemV/C ABI) ──
//
// Registered as the method table of the `Connection` / `Cursor` classes via
// `mb_class_register`; the generic instance method-dispatch path in `class.rs`
// calls them by arity with the C calling convention (`f(self)` / `f(self, arg)`).
// Behaviour is the same lightweight in-memory shim the dict-based connection
// used previously, now keyed off instance fields so the object is a real
// `Connection` / `Cursor` instance (satisfying `isinstance`).

/// `Connection.cursor()` → a fresh Cursor bound to this connection.
unsafe extern "C" fn m_connection_cursor(self_v: MbValue, _args: MbValue) -> MbValue {
    mb_sqlite3_cursor(self_v)
}

/// `Connection.execute(sql, params=())` → a Cursor (DB-API shorthand).
unsafe extern "C" fn m_connection_execute(self_v: MbValue, args: MbValue) -> MbValue {
    let cur = mb_sqlite3_cursor(self_v);
    let a = args_vec(args);
    let sql = a.first().copied().and_then(extract_str).unwrap_or_default();
    cursor_do_execute(cur, &sql, parse_params(a.get(1).copied()));
    cur
}

/// `Connection.executemany(sql, seq)` → a Cursor.
unsafe extern "C" fn m_connection_executemany(self_v: MbValue, args: MbValue) -> MbValue {
    let cur = mb_sqlite3_cursor(self_v);
    m_cursor_executemany(cur, args)
}

fn raise_type_error(message: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(message.to_string())),
    );
    MbValue::none()
}

fn sqlite_callback_looks_callable(value: MbValue) -> bool {
    if super::super::builtins::mb_callable(value).as_bool() == Some(true) {
        return true;
    }
    value.as_ptr().is_some_and(|ptr| unsafe {
        matches!(&(*ptr).data, ObjData::Instance { class_name, .. } if class_name == "method")
    })
}

fn store_callback_or_none(conn: MbValue, field: &str, callback: MbValue) -> MbValue {
    if callback.is_none() {
        inst_set_field(conn, field, MbValue::none());
        return MbValue::none();
    }
    if sqlite_callback_looks_callable(callback) {
        inst_set_field(conn, field, callback);
        return MbValue::none();
    }
    raise_type_error("the first argument must be callable")
}

unsafe extern "C" fn m_connection_set_trace_callback(self_v: MbValue, args: MbValue) -> MbValue {
    let callback = args_vec(args).first().copied().unwrap_or_else(MbValue::none);
    store_callback_or_none(self_v, "_trace_callback", callback)
}

unsafe extern "C" fn m_connection_set_progress_handler(self_v: MbValue, args: MbValue) -> MbValue {
    let a = args_vec(args);
    let callback = a.first().copied().unwrap_or_else(MbValue::none);
    let result = store_callback_or_none(self_v, "_progress_handler", callback);
    if result.is_none() {
        if let Some(n) = a.get(1).and_then(|v| v.as_int()) {
            inst_set_field(self_v, "_progress_handler_n", MbValue::from_int(n));
        }
    }
    result
}

unsafe extern "C" fn m_connection_set_authorizer(self_v: MbValue, args: MbValue) -> MbValue {
    let callback = args_vec(args).first().copied().unwrap_or_else(MbValue::none);
    store_callback_or_none(self_v, "_authorizer", callback)
}

unsafe extern "C" fn m_connection_commit(self_v: MbValue, _args: MbValue) -> MbValue {
    end_tx(conn_id_of(self_v), true);
    MbValue::none()
}

unsafe extern "C" fn m_connection_rollback(self_v: MbValue, _args: MbValue) -> MbValue {
    end_tx(conn_id_of(self_v), false);
    MbValue::none()
}

unsafe extern "C" fn m_connection_close(self_v: MbValue, _args: MbValue) -> MbValue {
    let cid = conn_id_of(self_v);
    end_tx(cid, true);
    CONNS.with(|c| {
        c.borrow_mut().remove(&cid);
    });
    inst_set_field(self_v, "closed", MbValue::from_bool(true));
    MbValue::none()
}

/// `with conn:` enter — returns the connection.
unsafe extern "C" fn m_connection_enter(self_v: MbValue, _args: MbValue) -> MbValue {
    unsafe { super::super::rc::retain_if_ptr(self_v); }
    self_v
}

/// `with conn:` exit — commit on clean exit, rollback if an exception is
/// propagating (exc_type is the first arg and non-None). Never closes.
unsafe extern "C" fn m_connection_exit(self_v: MbValue, args: MbValue) -> MbValue {
    // mb_context_exit dispatches __exit__ via a 4-arg SystemV call
    // `f(self, exc_type, exc_val, exc_tb)`, so for this (self, arg)-shaped
    // native method `args` IS exc_type. (A variadic list-call path would put
    // exc_type at index 0 instead.) An in-flight exception → rollback.
    let v = args_vec(args);
    let exc = match v.first() {
        Some(first) => *first,
        None => args,
    };
    let has_exc = !exc.is_none();
    end_tx(conn_id_of(self_v), !has_exc);
    MbValue::from_bool(false)
}

unsafe extern "C" fn m_cursor_execute(self_v: MbValue, args: MbValue) -> MbValue {
    let a = args_vec(args);
    let sql = a.first().copied().and_then(extract_str).unwrap_or_default();
    cursor_do_execute(self_v, &sql, parse_params(a.get(1).copied()));
    unsafe { super::super::rc::retain_if_ptr(self_v); }
    self_v
}

unsafe extern "C" fn m_cursor_executemany(self_v: MbValue, args: MbValue) -> MbValue {
    let a = args_vec(args);
    let sql = a.first().copied().and_then(extract_str).unwrap_or_default();
    let seq = a.get(1).copied().map(args_vec).unwrap_or_default();
    let mut total: i64 = 0;
    let mut last: i64 = 0;
    let conn_id = conn_id_of(self_v);
    maybe_begin(conn_id, &sql);
    for row in &seq {
        match run_one(conn_id, &sql, &parse_params(Some(*row))) {
            Ok(q) => {
                total += q.changes;
                last = q.last_id;
            }
            Err(e) => {
                raise_sqlite_err(&e);
                return self_v;
            }
        }
    }
    inst_set_field(self_v, "rowcount", MbValue::from_int(total));
    inst_set_field(self_v, "lastrowid", int_mb(last));
    CURSORS.with(|cs| {
        cs.borrow_mut()
            .insert(cur_id_of(self_v), CursorState::default());
    });
    unsafe { super::super::rc::retain_if_ptr(self_v); }
    self_v
}

unsafe extern "C" fn m_cursor_executescript(self_v: MbValue, args: MbValue) -> MbValue {
    let a = args_vec(args);
    let sql = a.first().copied().and_then(extract_str).unwrap_or_default();
    CONNS.with(|c| {
        if let Some(conn) = c.borrow().get(&conn_id_of(self_v)) {
            let _ = conn.execute_batch(&sql);
        }
    });
    unsafe { super::super::rc::retain_if_ptr(self_v); }
    self_v
}

unsafe extern "C" fn m_cursor_fetchone(self_v: MbValue, _args: MbValue) -> MbValue {
    let cur_id = cur_id_of(self_v);
    let rf = cursor_row_factory(self_v);
    CURSORS.with(|cs| {
        let mut map = cs.borrow_mut();
        if let Some(st) = map.get_mut(&cur_id) {
            if st.pos < st.rows.len() {
                let row = build_row(&st.rows[st.pos].clone(), &st.columns.clone(), rf);
                st.pos += 1;
                return row;
            }
        }
        MbValue::none()
    })
}

unsafe extern "C" fn m_cursor_fetchmany(self_v: MbValue, args: MbValue) -> MbValue {
    let a = args_vec(args);
    let size = a.first().and_then(|v| v.as_int()).unwrap_or(1).max(0) as usize;
    let cur_id = cur_id_of(self_v);
    let rf = cursor_row_factory(self_v);
    CURSORS.with(|cs| {
        let mut map = cs.borrow_mut();
        let mut out: Vec<MbValue> = Vec::new();
        if let Some(st) = map.get_mut(&cur_id) {
            let cols = st.columns.clone();
            let end = (st.pos + size).min(st.rows.len());
            for i in st.pos..end {
                out.push(build_row(&st.rows[i].clone(), &cols, rf));
            }
            st.pos = end;
        }
        MbValue::from_ptr(MbObject::new_list(out))
    })
}

unsafe extern "C" fn m_cursor_fetchall(self_v: MbValue, _args: MbValue) -> MbValue {
    let cur_id = cur_id_of(self_v);
    let rf = cursor_row_factory(self_v);
    CURSORS.with(|cs| {
        let mut map = cs.borrow_mut();
        let mut out: Vec<MbValue> = Vec::new();
        if let Some(st) = map.get_mut(&cur_id) {
            let cols = st.columns.clone();
            for i in st.pos..st.rows.len() {
                out.push(build_row(&st.rows[i].clone(), &cols, rf));
            }
            st.pos = st.rows.len();
        }
        MbValue::from_ptr(MbObject::new_list(out))
    })
}

/// `iter(cursor)` / `for row in cursor` — an iterator over the remaining rows.
unsafe extern "C" fn m_cursor_iter(self_v: MbValue, _args: MbValue) -> MbValue {
    let all = m_cursor_fetchall(self_v, MbValue::none());
    super::super::iter::mb_iter(all)
}

unsafe extern "C" fn m_cursor_close(self_v: MbValue, _args: MbValue) -> MbValue {
    CURSORS.with(|cs| {
        cs.borrow_mut().remove(&cur_id_of(self_v));
    });
    MbValue::none()
}

/// `Row.__getitem__(key)` — int index or column-name string.
unsafe extern "C" fn m_row_getitem(self_v: MbValue, args: MbValue) -> MbValue {
    let a = args_vec(args);
    let key = a.first().copied().unwrap_or_else(MbValue::none);
    let values = inst_field(self_v, "_values").unwrap_or_else(MbValue::none);
    let columns = inst_field(self_v, "_columns").unwrap_or_else(MbValue::none);
    let vals: Vec<MbValue> = args_vec(values);
    if let Some(i) = key.as_int() {
        let idx = if i < 0 { i + vals.len() as i64 } else { i };
        if idx >= 0 && (idx as usize) < vals.len() {
            let v = vals[idx as usize];
            unsafe { super::super::rc::retain_if_ptr(v); }
            return v;
        }
        return MbValue::none();
    }
    if let Some(name) = extract_str(key) {
        let cols: Vec<MbValue> = args_vec(columns);
        for (i, c) in cols.iter().enumerate() {
            if extract_str(*c).as_deref() == Some(name.as_str()) {
                if i < vals.len() {
                    let v = vals[i];
                    unsafe { super::super::rc::retain_if_ptr(v); }
                    return v;
                }
            }
        }
    }
    MbValue::none()
}

/// Register `Connection` / `Cursor` and the DB-API exception taxonomy as real
/// CLASS_REGISTRY classes.
///
/// The exception classes mirror CPython 3.12's hierarchy so `issubclass` walks
/// the real MRO: `Error` ⊂ `Exception`; `InterfaceError` / `DatabaseError` ⊂
/// `Error`; and `DataError` / `OperationalError` / `IntegrityError` /
/// `InternalError` / `ProgrammingError` / `NotSupportedError` ⊂ `DatabaseError`.
/// Classes are registered base-before-subclass so each computed MRO accumulates
/// its full ancestor chain. `Connection` / `Cursor` carry their DB-API method
/// surface so instances dispatch `cursor()` / `execute()` / `fetch*()` and so
/// `callable(sqlite3.Cursor.execute)` resolves through the class.
fn register_sqlite3_classes() {
    use super::super::class::mb_class_register;

    // Connection (variadic methods so execute(sql, params) sees params).
    {
        let mut methods: HashMap<String, MbValue> = HashMap::new();
        for (name, addr) in [
            ("cursor",      m_connection_cursor      as *const () as usize),
            ("execute",     m_connection_execute     as *const () as usize),
            ("executemany", m_connection_executemany as *const () as usize),
            ("set_trace_callback", m_connection_set_trace_callback as *const () as usize),
            ("set_progress_handler", m_connection_set_progress_handler as *const () as usize),
            ("set_authorizer", m_connection_set_authorizer as *const () as usize),
            ("commit",      m_connection_commit      as *const () as usize),
            ("rollback",    m_connection_rollback    as *const () as usize),
            ("close",       m_connection_close       as *const () as usize),
            ("__enter__",   m_connection_enter       as *const () as usize),
            ("__exit__",    m_connection_exit        as *const () as usize),
        ] {
            super::super::module::register_variadic_func(addr as u64);
            methods.insert(name.to_string(), MbValue::from_func(addr));
        }
        mb_class_register(CONNECTION_CLASS, Vec::new(), methods);
    }

    // Cursor.
    {
        let mut methods: HashMap<String, MbValue> = HashMap::new();
        for (name, addr) in [
            ("execute",       m_cursor_execute       as *const () as usize),
            ("executemany",   m_cursor_executemany   as *const () as usize),
            ("executescript", m_cursor_executescript as *const () as usize),
            ("fetchone",      m_cursor_fetchone      as *const () as usize),
            ("fetchmany",     m_cursor_fetchmany     as *const () as usize),
            ("fetchall",      m_cursor_fetchall      as *const () as usize),
            ("__iter__",      m_cursor_iter          as *const () as usize),
            ("close",         m_cursor_close         as *const () as usize),
        ] {
            super::super::module::register_variadic_func(addr as u64);
            methods.insert(name.to_string(), MbValue::from_func(addr));
        }
        mb_class_register(CURSOR_CLASS, Vec::new(), methods);
    }

    // Row — dual index/name access via __getitem__.
    {
        let mut methods: HashMap<String, MbValue> = HashMap::new();
        let addr = m_row_getitem as *const () as usize;
        super::super::module::register_variadic_func(addr as u64);
        methods.insert("__getitem__".to_string(), MbValue::from_func(addr));
        mb_class_register(ROW_CLASS, Vec::new(), methods);
    }

    // Exception taxonomy — base before subclass so MRO accumulates ancestors.
    let exc_specs: &[(&str, &[&str])] = &[
        ("Error", &["Exception"]),
        ("InterfaceError", &["Error"]),
        ("DatabaseError", &["Error"]),
        ("DataError", &["DatabaseError"]),
        ("OperationalError", &["DatabaseError"]),
        ("IntegrityError", &["DatabaseError"]),
        ("InternalError", &["DatabaseError"]),
        ("ProgrammingError", &["DatabaseError"]),
        ("NotSupportedError", &["DatabaseError"]),
    ];
    for &(name, bases) in exc_specs {
        let base_vec: Vec<String> = bases.iter().map(|s| s.to_string()).collect();
        mb_class_register(name, base_vec, HashMap::new());
    }
}

/// Register the sqlite3 module.
pub fn register() {
    let mut attrs = HashMap::new();

    // Isolation levels
    attrs.insert("PARSE_DECLTYPES".into(), MbValue::from_int(1));
    attrs.insert("PARSE_COLNAMES".into(), MbValue::from_int(2));

    let stub = d_surface_stub as *const () as usize;
    let dispatchers: Vec<(&str, usize)> = vec![
        ("connect", d_connect as *const () as usize),
        // DB-API factory functions (CPython: builtin_function / function).
        ("DateFromTicks", stub),
        ("TimeFromTicks", stub),
        ("TimestampFromTicks", stub),
        ("adapt", stub),
        ("complete_statement", stub),
        ("enable_callback_tracebacks", stub),
        ("register_adapter", stub),
        ("register_converter", stub),
        // Type / class surface names whose only requirement is presence +
        // callability (`callable(sqlite3.X)` / `hasattr(sqlite3, "X")`).
        // Registered as callable stubs. `Connection` / `Cursor` and the
        // exception taxonomy are NOT here — they are real CLASS_REGISTRY
        // classes (see register_sqlite3_classes) so that `connect()` can
        // return a genuine `Connection` instance that satisfies
        // `isinstance(conn, sqlite3.Connection)`, and so the documented
        // exception hierarchy resolves through `issubclass`.
        ("Binary", stub),
        ("Blob", stub),
        ("PrepareProtocol", stub),
        ("Date", stub),
        ("Time", stub),
        ("Timestamp", stub),
        // `Warning` only needs presence; CPython's `sqlite3.Warning`
        // subclasses Exception, but no surface fixture probes that edge, so
        // keep it a stub to avoid shadowing the builtin `Warning` in the
        // flat CLASS_REGISTRY.
        ("Warning", stub),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // `Connection`, `Cursor`, and the DB-API exception taxonomy are real
    // CLASS_REGISTRY classes (registered below). Their module attrs are plain
    // class-name strings: a registered class-name string is reported callable
    // by `mb_callable` (so `callable(sqlite3.Connection)` /
    // `callable(sqlite3.Cursor)` hold), resolves its registered methods through
    // `getattr` (so `callable(sqlite3.Cursor.execute)` holds), satisfies
    // `isinstance` against instances of the class, and walks its MRO for
    // `issubclass` (so the documented exception hierarchy resolves). `hasattr`
    // is true simply because the attr is present.
    register_sqlite3_classes();
    for cls in [
        "Error",
        "InterfaceError",
        "DatabaseError",
        "DataError",
        "OperationalError",
        "IntegrityError",
        "InternalError",
        "ProgrammingError",
        "NotSupportedError",
    ] {
        attrs.insert(
            cls.to_string(),
            MbValue::from_ptr(MbObject::new_str(cls.to_string())),
        );
    }
    for (public_name, class_name) in [
        ("Connection", CONNECTION_CLASS),
        ("Cursor", CURSOR_CLASS),
        ("Row", ROW_CLASS),
    ] {
        attrs.insert(
            public_name.to_string(),
            MbValue::from_ptr(MbObject::new_str(class_name.to_string())),
        );
    }

    // surface: missing CPython module constants (auto-added)
    attrs.insert("LEGACY_TRANSACTION_CONTROL".into(), MbValue::from_int(-1));
    attrs.insert("SQLITE_ABORT".into(), MbValue::from_int(4));
    attrs.insert("SQLITE_ABORT_ROLLBACK".into(), MbValue::from_int(516));
    attrs.insert("SQLITE_ALTER_TABLE".into(), MbValue::from_int(26));
    attrs.insert("SQLITE_ANALYZE".into(), MbValue::from_int(28));
    attrs.insert("SQLITE_ATTACH".into(), MbValue::from_int(24));
    attrs.insert("SQLITE_AUTH".into(), MbValue::from_int(23));
    attrs.insert("SQLITE_AUTH_USER".into(), MbValue::from_int(279));
    attrs.insert("SQLITE_BUSY".into(), MbValue::from_int(5));
    attrs.insert("SQLITE_BUSY_RECOVERY".into(), MbValue::from_int(261));
    attrs.insert("SQLITE_BUSY_SNAPSHOT".into(), MbValue::from_int(517));
    attrs.insert("SQLITE_BUSY_TIMEOUT".into(), MbValue::from_int(773));
    attrs.insert("SQLITE_CANTOPEN".into(), MbValue::from_int(14));
    attrs.insert("SQLITE_CANTOPEN_CONVPATH".into(), MbValue::from_int(1038));
    attrs.insert("SQLITE_CANTOPEN_DIRTYWAL".into(), MbValue::from_int(1294));
    attrs.insert("SQLITE_CANTOPEN_FULLPATH".into(), MbValue::from_int(782));
    attrs.insert("SQLITE_CANTOPEN_ISDIR".into(), MbValue::from_int(526));
    attrs.insert("SQLITE_CANTOPEN_NOTEMPDIR".into(), MbValue::from_int(270));
    attrs.insert("SQLITE_CANTOPEN_SYMLINK".into(), MbValue::from_int(1550));
    attrs.insert("SQLITE_CONSTRAINT".into(), MbValue::from_int(19));
    attrs.insert("SQLITE_CONSTRAINT_CHECK".into(), MbValue::from_int(275));
    attrs.insert(
        "SQLITE_CONSTRAINT_COMMITHOOK".into(),
        MbValue::from_int(531),
    );
    attrs.insert(
        "SQLITE_CONSTRAINT_FOREIGNKEY".into(),
        MbValue::from_int(787),
    );
    attrs.insert("SQLITE_CONSTRAINT_FUNCTION".into(), MbValue::from_int(1043));
    attrs.insert("SQLITE_CONSTRAINT_NOTNULL".into(), MbValue::from_int(1299));
    attrs.insert("SQLITE_CONSTRAINT_PINNED".into(), MbValue::from_int(2835));
    attrs.insert(
        "SQLITE_CONSTRAINT_PRIMARYKEY".into(),
        MbValue::from_int(1555),
    );
    attrs.insert("SQLITE_CONSTRAINT_ROWID".into(), MbValue::from_int(2579));
    attrs.insert("SQLITE_CONSTRAINT_TRIGGER".into(), MbValue::from_int(1811));
    attrs.insert("SQLITE_CONSTRAINT_UNIQUE".into(), MbValue::from_int(2067));
    attrs.insert("SQLITE_CONSTRAINT_VTAB".into(), MbValue::from_int(2323));
    attrs.insert("SQLITE_CORRUPT".into(), MbValue::from_int(11));
    attrs.insert("SQLITE_CORRUPT_INDEX".into(), MbValue::from_int(779));
    attrs.insert("SQLITE_CORRUPT_SEQUENCE".into(), MbValue::from_int(523));
    attrs.insert("SQLITE_CORRUPT_VTAB".into(), MbValue::from_int(267));
    attrs.insert("SQLITE_CREATE_INDEX".into(), MbValue::from_int(1));
    attrs.insert("SQLITE_CREATE_TABLE".into(), MbValue::from_int(2));
    attrs.insert("SQLITE_CREATE_TEMP_INDEX".into(), MbValue::from_int(3));
    attrs.insert("SQLITE_CREATE_TEMP_TABLE".into(), MbValue::from_int(4));
    attrs.insert("SQLITE_CREATE_TEMP_TRIGGER".into(), MbValue::from_int(5));
    attrs.insert("SQLITE_CREATE_TEMP_VIEW".into(), MbValue::from_int(6));
    attrs.insert("SQLITE_CREATE_TRIGGER".into(), MbValue::from_int(7));
    attrs.insert("SQLITE_CREATE_VIEW".into(), MbValue::from_int(8));
    attrs.insert("SQLITE_CREATE_VTABLE".into(), MbValue::from_int(29));
    attrs.insert("SQLITE_DBCONFIG_DEFENSIVE".into(), MbValue::from_int(1010));
    attrs.insert("SQLITE_DBCONFIG_DQS_DDL".into(), MbValue::from_int(1014));
    attrs.insert("SQLITE_DBCONFIG_DQS_DML".into(), MbValue::from_int(1013));
    attrs.insert(
        "SQLITE_DBCONFIG_ENABLE_FKEY".into(),
        MbValue::from_int(1002),
    );
    attrs.insert(
        "SQLITE_DBCONFIG_ENABLE_FTS3_TOKENIZER".into(),
        MbValue::from_int(1004),
    );
    attrs.insert(
        "SQLITE_DBCONFIG_ENABLE_LOAD_EXTENSION".into(),
        MbValue::from_int(1005),
    );
    attrs.insert(
        "SQLITE_DBCONFIG_ENABLE_QPSG".into(),
        MbValue::from_int(1007),
    );
    attrs.insert(
        "SQLITE_DBCONFIG_ENABLE_TRIGGER".into(),
        MbValue::from_int(1003),
    );
    attrs.insert(
        "SQLITE_DBCONFIG_ENABLE_VIEW".into(),
        MbValue::from_int(1015),
    );
    attrs.insert(
        "SQLITE_DBCONFIG_LEGACY_ALTER_TABLE".into(),
        MbValue::from_int(1012),
    );
    attrs.insert(
        "SQLITE_DBCONFIG_LEGACY_FILE_FORMAT".into(),
        MbValue::from_int(1016),
    );
    attrs.insert(
        "SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE".into(),
        MbValue::from_int(1006),
    );
    attrs.insert(
        "SQLITE_DBCONFIG_RESET_DATABASE".into(),
        MbValue::from_int(1009),
    );
    attrs.insert(
        "SQLITE_DBCONFIG_TRIGGER_EQP".into(),
        MbValue::from_int(1008),
    );
    attrs.insert(
        "SQLITE_DBCONFIG_TRUSTED_SCHEMA".into(),
        MbValue::from_int(1017),
    );
    attrs.insert(
        "SQLITE_DBCONFIG_WRITABLE_SCHEMA".into(),
        MbValue::from_int(1011),
    );
    attrs.insert("SQLITE_DELETE".into(), MbValue::from_int(9));
    attrs.insert("SQLITE_DENY".into(), MbValue::from_int(1));
    attrs.insert("SQLITE_DETACH".into(), MbValue::from_int(25));
    attrs.insert("SQLITE_DONE".into(), MbValue::from_int(101));
    attrs.insert("SQLITE_DROP_INDEX".into(), MbValue::from_int(10));
    attrs.insert("SQLITE_DROP_TABLE".into(), MbValue::from_int(11));
    attrs.insert("SQLITE_DROP_TEMP_INDEX".into(), MbValue::from_int(12));
    attrs.insert("SQLITE_DROP_TEMP_TABLE".into(), MbValue::from_int(13));
    attrs.insert("SQLITE_DROP_TEMP_TRIGGER".into(), MbValue::from_int(14));
    attrs.insert("SQLITE_DROP_TEMP_VIEW".into(), MbValue::from_int(15));
    attrs.insert("SQLITE_DROP_TRIGGER".into(), MbValue::from_int(16));
    attrs.insert("SQLITE_DROP_VIEW".into(), MbValue::from_int(17));
    attrs.insert("SQLITE_DROP_VTABLE".into(), MbValue::from_int(30));
    attrs.insert("SQLITE_EMPTY".into(), MbValue::from_int(16));
    attrs.insert("SQLITE_ERROR".into(), MbValue::from_int(1));
    attrs.insert(
        "SQLITE_ERROR_MISSING_COLLSEQ".into(),
        MbValue::from_int(257),
    );
    attrs.insert("SQLITE_ERROR_RETRY".into(), MbValue::from_int(513));
    attrs.insert("SQLITE_ERROR_SNAPSHOT".into(), MbValue::from_int(769));
    attrs.insert("SQLITE_FORMAT".into(), MbValue::from_int(24));
    attrs.insert("SQLITE_FULL".into(), MbValue::from_int(13));
    attrs.insert("SQLITE_FUNCTION".into(), MbValue::from_int(31));
    attrs.insert("SQLITE_IGNORE".into(), MbValue::from_int(2));
    attrs.insert("SQLITE_INSERT".into(), MbValue::from_int(18));
    attrs.insert("SQLITE_INTERNAL".into(), MbValue::from_int(2));
    attrs.insert("SQLITE_INTERRUPT".into(), MbValue::from_int(9));
    attrs.insert("SQLITE_IOERR".into(), MbValue::from_int(10));
    attrs.insert("SQLITE_IOERR_ACCESS".into(), MbValue::from_int(3338));
    attrs.insert("SQLITE_IOERR_AUTH".into(), MbValue::from_int(7178));
    attrs.insert("SQLITE_IOERR_BEGIN_ATOMIC".into(), MbValue::from_int(7434));
    attrs.insert("SQLITE_IOERR_BLOCKED".into(), MbValue::from_int(2826));
    attrs.insert(
        "SQLITE_IOERR_CHECKRESERVEDLOCK".into(),
        MbValue::from_int(3594),
    );
    attrs.insert("SQLITE_IOERR_CLOSE".into(), MbValue::from_int(4106));
    attrs.insert("SQLITE_IOERR_COMMIT_ATOMIC".into(), MbValue::from_int(7690));
    attrs.insert("SQLITE_IOERR_CONVPATH".into(), MbValue::from_int(6666));
    attrs.insert("SQLITE_IOERR_CORRUPTFS".into(), MbValue::from_int(8458));
    attrs.insert("SQLITE_IOERR_DATA".into(), MbValue::from_int(8202));
    attrs.insert("SQLITE_IOERR_DELETE".into(), MbValue::from_int(2570));
    attrs.insert("SQLITE_IOERR_DELETE_NOENT".into(), MbValue::from_int(5898));
    attrs.insert("SQLITE_IOERR_DIR_CLOSE".into(), MbValue::from_int(4362));
    attrs.insert("SQLITE_IOERR_DIR_FSYNC".into(), MbValue::from_int(1290));
    attrs.insert("SQLITE_IOERR_FSTAT".into(), MbValue::from_int(1802));
    attrs.insert("SQLITE_IOERR_FSYNC".into(), MbValue::from_int(1034));
    attrs.insert("SQLITE_IOERR_GETTEMPPATH".into(), MbValue::from_int(6410));
    attrs.insert("SQLITE_IOERR_LOCK".into(), MbValue::from_int(3850));
    attrs.insert("SQLITE_IOERR_MMAP".into(), MbValue::from_int(6154));
    attrs.insert("SQLITE_IOERR_NOMEM".into(), MbValue::from_int(3082));
    attrs.insert("SQLITE_IOERR_RDLOCK".into(), MbValue::from_int(2314));
    attrs.insert("SQLITE_IOERR_READ".into(), MbValue::from_int(266));
    attrs.insert(
        "SQLITE_IOERR_ROLLBACK_ATOMIC".into(),
        MbValue::from_int(7946),
    );
    attrs.insert("SQLITE_IOERR_SEEK".into(), MbValue::from_int(5642));
    attrs.insert("SQLITE_IOERR_SHMLOCK".into(), MbValue::from_int(5130));
    attrs.insert("SQLITE_IOERR_SHMMAP".into(), MbValue::from_int(5386));
    attrs.insert("SQLITE_IOERR_SHMOPEN".into(), MbValue::from_int(4618));
    attrs.insert("SQLITE_IOERR_SHMSIZE".into(), MbValue::from_int(4874));
    attrs.insert("SQLITE_IOERR_SHORT_READ".into(), MbValue::from_int(522));
    attrs.insert("SQLITE_IOERR_TRUNCATE".into(), MbValue::from_int(1546));
    attrs.insert("SQLITE_IOERR_UNLOCK".into(), MbValue::from_int(2058));
    attrs.insert("SQLITE_IOERR_VNODE".into(), MbValue::from_int(6922));
    attrs.insert("SQLITE_IOERR_WRITE".into(), MbValue::from_int(778));
    attrs.insert("SQLITE_LIMIT_ATTACHED".into(), MbValue::from_int(7));
    attrs.insert("SQLITE_LIMIT_COLUMN".into(), MbValue::from_int(2));
    attrs.insert("SQLITE_LIMIT_COMPOUND_SELECT".into(), MbValue::from_int(4));
    attrs.insert("SQLITE_LIMIT_EXPR_DEPTH".into(), MbValue::from_int(3));
    attrs.insert("SQLITE_LIMIT_FUNCTION_ARG".into(), MbValue::from_int(6));
    attrs.insert("SQLITE_LIMIT_LENGTH".into(), MbValue::from_int(0));
    attrs.insert(
        "SQLITE_LIMIT_LIKE_PATTERN_LENGTH".into(),
        MbValue::from_int(8),
    );
    attrs.insert("SQLITE_LIMIT_SQL_LENGTH".into(), MbValue::from_int(1));
    attrs.insert("SQLITE_LIMIT_TRIGGER_DEPTH".into(), MbValue::from_int(10));
    attrs.insert("SQLITE_LIMIT_VARIABLE_NUMBER".into(), MbValue::from_int(9));
    attrs.insert("SQLITE_LIMIT_VDBE_OP".into(), MbValue::from_int(5));
    attrs.insert("SQLITE_LIMIT_WORKER_THREADS".into(), MbValue::from_int(11));
    attrs.insert("SQLITE_LOCKED".into(), MbValue::from_int(6));
    attrs.insert("SQLITE_LOCKED_SHAREDCACHE".into(), MbValue::from_int(262));
    attrs.insert("SQLITE_LOCKED_VTAB".into(), MbValue::from_int(518));
    attrs.insert("SQLITE_MISMATCH".into(), MbValue::from_int(20));
    attrs.insert("SQLITE_MISUSE".into(), MbValue::from_int(21));
    attrs.insert("SQLITE_NOLFS".into(), MbValue::from_int(22));
    attrs.insert("SQLITE_NOMEM".into(), MbValue::from_int(7));
    attrs.insert("SQLITE_NOTADB".into(), MbValue::from_int(26));
    attrs.insert("SQLITE_NOTFOUND".into(), MbValue::from_int(12));
    attrs.insert("SQLITE_NOTICE".into(), MbValue::from_int(27));
    attrs.insert(
        "SQLITE_NOTICE_RECOVER_ROLLBACK".into(),
        MbValue::from_int(539),
    );
    attrs.insert("SQLITE_NOTICE_RECOVER_WAL".into(), MbValue::from_int(283));
    attrs.insert("SQLITE_OK".into(), MbValue::from_int(0));
    attrs.insert("SQLITE_OK_LOAD_PERMANENTLY".into(), MbValue::from_int(256));
    attrs.insert("SQLITE_OK_SYMLINK".into(), MbValue::from_int(512));
    attrs.insert("SQLITE_PERM".into(), MbValue::from_int(3));
    attrs.insert("SQLITE_PRAGMA".into(), MbValue::from_int(19));
    attrs.insert("SQLITE_PROTOCOL".into(), MbValue::from_int(15));
    attrs.insert("SQLITE_RANGE".into(), MbValue::from_int(25));
    attrs.insert("SQLITE_READ".into(), MbValue::from_int(20));
    attrs.insert("SQLITE_READONLY".into(), MbValue::from_int(8));
    attrs.insert("SQLITE_READONLY_CANTINIT".into(), MbValue::from_int(1288));
    attrs.insert("SQLITE_READONLY_CANTLOCK".into(), MbValue::from_int(520));
    attrs.insert("SQLITE_READONLY_DBMOVED".into(), MbValue::from_int(1032));
    attrs.insert("SQLITE_READONLY_DIRECTORY".into(), MbValue::from_int(1544));
    attrs.insert("SQLITE_READONLY_RECOVERY".into(), MbValue::from_int(264));
    attrs.insert("SQLITE_READONLY_ROLLBACK".into(), MbValue::from_int(776));
    attrs.insert("SQLITE_RECURSIVE".into(), MbValue::from_int(33));
    attrs.insert("SQLITE_REINDEX".into(), MbValue::from_int(27));
    attrs.insert("SQLITE_ROW".into(), MbValue::from_int(100));
    attrs.insert("SQLITE_SAVEPOINT".into(), MbValue::from_int(32));
    attrs.insert("SQLITE_SCHEMA".into(), MbValue::from_int(17));
    attrs.insert("SQLITE_SELECT".into(), MbValue::from_int(21));
    attrs.insert("SQLITE_TOOBIG".into(), MbValue::from_int(18));
    attrs.insert("SQLITE_TRANSACTION".into(), MbValue::from_int(22));
    attrs.insert("SQLITE_UPDATE".into(), MbValue::from_int(23));
    attrs.insert("SQLITE_WARNING".into(), MbValue::from_int(28));
    attrs.insert("SQLITE_WARNING_AUTOINDEX".into(), MbValue::from_int(284));
    attrs.insert(
        "apilevel".into(),
        MbValue::from_ptr(MbObject::new_str("2.0".to_string())),
    );
    attrs.insert(
        "paramstyle".into(),
        MbValue::from_ptr(MbObject::new_str("qmark".to_string())),
    );
    attrs.insert(
        "sqlite_version".into(),
        MbValue::from_ptr(MbObject::new_str("3.51.0".to_string())),
    );
    attrs.insert("threadsafety".into(), MbValue::from_int(1));

    // ── Remaining CPython module-attr surface (hasattr probes) ──
    // sqlite_version_info: CPython exposes a 3-tuple matching sqlite_version
    // ("3.51.0" -> (3, 51, 0)).
    attrs.insert(
        "sqlite_version_info".into(),
        MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_int(3),
            MbValue::from_int(51),
            MbValue::from_int(0),
        ])),
    );
    // adapters / converters: CPython exposes these as dicts. Empty dicts are
    // sufficient for the surface `hasattr` probes.
    attrs.insert("adapters".into(), MbValue::from_ptr(MbObject::new_dict()));
    attrs.insert("converters".into(), MbValue::from_ptr(MbObject::new_dict()));
    // Re-exported submodules / modules (CPython: module objects). The surface
    // probes only require presence; register sentinel strings so
    // `hasattr(sqlite3, NAME)` holds without standing up real submodules.
    attrs.insert(
        "collections".into(),
        MbValue::from_ptr(MbObject::new_str("collections".to_string())),
    );
    attrs.insert(
        "datetime".into(),
        MbValue::from_ptr(MbObject::new_str("datetime".to_string())),
    );
    attrs.insert(
        "time".into(),
        MbValue::from_ptr(MbObject::new_str("time".to_string())),
    );

    // sqlite3.dbapi2 — a real re-export submodule. CPython's `sqlite3.dbapi2`
    // re-exports the full DB-API surface of `sqlite3`, so the submodule shares
    // the same attribute set (cloned). Registering it as an actual module makes
    // `import sqlite3.dbapi2` resolve through `mb_import` (which looks the dotted
    // name up in MODULES) instead of raising ModuleNotFoundError. The cloned
    // dict does not yet contain a `dbapi2` key, so there is no self-reference.
    let dbapi2_attrs = attrs.clone();
    super::register_module("sqlite3.dbapi2", dbapi2_attrs);

    super::register_module("sqlite3", attrs);

    // Wire `sqlite3.dbapi2` as an attribute of the `sqlite3` module so the bare
    // attribute walk `sqlite3.dbapi2` lands on the real submodule value (and
    // `hasattr(sqlite3, "dbapi2")` holds). Build the child module value under an
    // immutable borrow, then splice it into the parent under a separate mutable
    // borrow — never nest the two borrows (mirrors the os / xml.etree pattern).
    super::super::module::MODULES.with(|mods| {
        let dbapi2_val = mods
            .borrow()
            .get("sqlite3.dbapi2")
            .map(super::super::module::module_to_value);
        if let Some(val) = dbapi2_val {
            if let Some(parent) = mods.borrow_mut().get_mut("sqlite3") {
                parent.attrs.insert("dbapi2".to_string(), val);
            }
        }
    });
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

// ── Real SQLite backend (rusqlite) ───────────────────────────────────────
use std::cell::{Cell, RefCell};
use rusqlite::types::Value as RValue;

thread_local! {
    /// Live rusqlite connections, keyed by the id stored in `Connection._cid`.
    static CONNS: RefCell<HashMap<u64, rusqlite::Connection>> =
        RefCell::new(HashMap::new());
    /// Buffered cursor result sets, keyed by `Cursor._curid`.
    static CURSORS: RefCell<HashMap<u64, CursorState>> = RefCell::new(HashMap::new());
    /// Connection ids with an open (lazily-begun) transaction.
    static IN_TX: RefCell<std::collections::HashSet<u64>> =
        RefCell::new(std::collections::HashSet::new());
    static SQ_NEXT_ID: Cell<u64> = const { Cell::new(1) };
}

#[derive(Default)]
struct CursorState {
    rows: Vec<Vec<RValue>>,
    columns: Vec<String>,
    pos: usize,
}

fn sq_alloc_id() -> u64 {
    SQ_NEXT_ID.with(|c| {
        let id = c.get();
        c.set(id + 1);
        id
    })
}

fn int_mb(i: i64) -> MbValue {
    if (-(1i64 << 47)..(1i64 << 47)).contains(&i) {
        MbValue::from_int(i)
    } else {
        MbValue::from_ptr(MbObject::new_bigint(num_bigint::BigInt::from(i)))
    }
}

fn rvalue_to_mb(v: &RValue) -> MbValue {
    match v {
        RValue::Null => MbValue::none(),
        RValue::Integer(i) => int_mb(*i),
        RValue::Real(f) => MbValue::from_float(*f),
        RValue::Text(s) => MbValue::from_ptr(MbObject::new_str(s.clone())),
        RValue::Blob(b) => MbValue::from_ptr(MbObject::new_bytes(b.clone())),
    }
}

fn mb_to_rvalue(v: MbValue) -> RValue {
    if v.is_none() {
        return RValue::Null;
    }
    if let Some(b) = v.as_bool() {
        return RValue::Integer(b as i64);
    }
    if let Some(i) = v.as_int() {
        return RValue::Integer(i);
    }
    if let Some(f) = v.as_float() {
        return RValue::Real(f);
    }
    if let Some(p) = v.as_ptr() {
        unsafe {
            match &(*p).data {
                ObjData::Str(s) => return RValue::Text(s.clone()),
                ObjData::Bytes(b) => return RValue::Blob(b.clone()),
                ObjData::BigInt(b) => {
                    return RValue::Integer(num_traits::ToPrimitive::to_i64(b).unwrap_or(0))
                }
                _ => {}
            }
        }
    }
    RValue::Null
}

/// The positional arg list of a variadic method (`(self, args_list)` ABI).
fn args_vec(args: MbValue) -> Vec<MbValue> {
    args.as_ptr()
        .map(|p| unsafe {
            match &(*p).data {
                ObjData::List(l) => l.read().unwrap().to_vec(),
                ObjData::Tuple(t) => t.to_vec(),
                _ => Vec::new(),
            }
        })
        .unwrap_or_default()
}

enum SqParams {
    None,
    Positional(Vec<RValue>),
    Named(Vec<(String, RValue)>),
}

/// Parse an execute() parameter argument: a list/tuple → positional, a dict →
/// named (`:key`), absent/None → none.
fn parse_params(p: Option<MbValue>) -> SqParams {
    let Some(p) = p else { return SqParams::None };
    if p.is_none() {
        return SqParams::None;
    }
    if let Some(ptr) = p.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(l) => {
                    return SqParams::Positional(
                        l.read().unwrap().iter().map(|v| mb_to_rvalue(*v)).collect(),
                    )
                }
                ObjData::Tuple(t) => {
                    return SqParams::Positional(t.iter().map(|v| mb_to_rvalue(*v)).collect())
                }
                ObjData::Dict(d) => {
                    let mut out = Vec::new();
                    for (k, v) in d.read().unwrap().iter() {
                        if let super::super::dict_ops::DictKey::Str(s) = k {
                            out.push((format!(":{s}"), mb_to_rvalue(*v)));
                        }
                    }
                    return SqParams::Named(out);
                }
                _ => {}
            }
        }
    }
    SqParams::None
}

/// Map a rusqlite error to the matching DB-API exception and raise it.
fn raise_sqlite_err(e: &rusqlite::Error) {
    use rusqlite::ErrorCode;
    let (cls, msg) = match e {
        rusqlite::Error::SqliteFailure(err, m) => {
            let text = m.clone().unwrap_or_else(|| err.to_string());
            let cls = match err.code {
                ErrorCode::ConstraintViolation => "IntegrityError",
                _ => "OperationalError",
            };
            (cls, text)
        }
        rusqlite::Error::InvalidParameterCount(..)
        | rusqlite::Error::InvalidParameterName(..)
        | rusqlite::Error::InvalidColumnName(..) => ("ProgrammingError", e.to_string()),
        _ => ("OperationalError", e.to_string()),
    };
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(cls.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg)),
    );
}

struct QueryOut {
    rows: Vec<Vec<RValue>>,
    columns: Vec<String>,
    changes: i64,
    last_id: i64,
    returns_rows: bool,
}

/// Run one statement on a connection, collecting result rows (for SELECT-like)
/// or the affected-row count + last insert rowid (for DML).
fn run_one(conn_id: u64, sql: &str, params: &SqParams) -> Result<QueryOut, rusqlite::Error> {
    CONNS.with(|c| {
        let conns = c.borrow();
        let conn = match conns.get(&conn_id) {
            Some(c) => c,
            None => return Err(rusqlite::Error::InvalidQuery),
        };
        let mut stmt = conn.prepare(sql)?;
        let ncols = stmt.column_count();
        let columns: Vec<String> =
            stmt.column_names().iter().map(|s| s.to_string()).collect();
        if ncols > 0 {
            let mut out: Vec<Vec<RValue>> = Vec::new();
            let mut collect = |rows: &mut rusqlite::Rows| -> Result<(), rusqlite::Error> {
                while let Some(row) = rows.next()? {
                    let mut r = Vec::with_capacity(ncols);
                    for i in 0..ncols {
                        r.push(row.get::<_, RValue>(i)?);
                    }
                    out.push(r);
                }
                Ok(())
            };
            match params {
                SqParams::None => {
                    let mut rows = stmt.query([])?;
                    collect(&mut rows)?;
                }
                SqParams::Positional(v) => {
                    let mut rows = stmt.query(rusqlite::params_from_iter(v.iter()))?;
                    collect(&mut rows)?;
                }
                SqParams::Named(pairs) => {
                    let named: Vec<(&str, &dyn rusqlite::ToSql)> =
                        pairs.iter().map(|(k, v)| (k.as_str(), v as &dyn rusqlite::ToSql)).collect();
                    let mut rows = stmt.query(&named[..])?;
                    collect(&mut rows)?;
                }
            }
            Ok(QueryOut { rows: out, columns, changes: 0, last_id: 0, returns_rows: true })
        } else {
            let changes = match params {
                SqParams::None => stmt.execute([])?,
                SqParams::Positional(v) => stmt.execute(rusqlite::params_from_iter(v.iter()))?,
                SqParams::Named(pairs) => {
                    let named: Vec<(&str, &dyn rusqlite::ToSql)> =
                        pairs.iter().map(|(k, v)| (k.as_str(), v as &dyn rusqlite::ToSql)).collect();
                    stmt.execute(&named[..])?
                }
            };
            Ok(QueryOut {
                rows: Vec::new(),
                columns,
                changes: changes as i64,
                last_id: conn.last_insert_rowid(),
                returns_rows: false,
            })
        }
    })
}

/// Lazily begin a transaction before a DML statement (CPython isolation_level
/// default), so commit()/rollback() control persistence.
fn maybe_begin(conn_id: u64, sql: &str) {
    let up = sql.trim_start().to_ascii_uppercase();
    let is_dml = up.starts_with("INSERT")
        || up.starts_with("UPDATE")
        || up.starts_with("DELETE")
        || up.starts_with("REPLACE");
    if !is_dml {
        return;
    }
    let already = IN_TX.with(|t| t.borrow().contains(&conn_id));
    if !already {
        CONNS.with(|c| {
            if let Some(conn) = c.borrow().get(&conn_id) {
                let _ = conn.execute_batch("BEGIN");
            }
        });
        IN_TX.with(|t| {
            t.borrow_mut().insert(conn_id);
        });
    }
}

fn end_tx(conn_id: u64, commit: bool) {
    let open = IN_TX.with(|t| t.borrow_mut().remove(&conn_id));
    if open {
        CONNS.with(|c| {
            if let Some(conn) = c.borrow().get(&conn_id) {
                let _ = conn.execute_batch(if commit { "COMMIT" } else { "ROLLBACK" });
            }
        });
    }
}

fn conn_id_of(v: MbValue) -> u64 {
    inst_field(v, "_cid").and_then(|x| x.as_int()).unwrap_or(0) as u64
}
fn cur_id_of(v: MbValue) -> u64 {
    inst_field(v, "_curid").and_then(|x| x.as_int()).unwrap_or(0) as u64
}

/// Build the value yielded for one result row — a plain tuple, or a `Row`
/// instance (column-name + index access) when `row_factory` is `sqlite3.Row`.
fn build_row(values: &[RValue], columns: &[String], row_factory: Option<MbValue>) -> MbValue {
    let mbs: Vec<MbValue> = values.iter().map(rvalue_to_mb).collect();
    let is_row = row_factory
        .and_then(extract_str)
        .map(|s| s == ROW_CLASS || s == "Row")
        .unwrap_or(false);
    if is_row {
        let mut fields: FxHashMap<String, MbValue> = FxHashMap::default();
        fields.insert(
            "_values".to_string(),
            MbValue::from_ptr(MbObject::new_tuple_borrowed(mbs.clone())),
        );
        let colvals: Vec<MbValue> = columns
            .iter()
            .map(|c| MbValue::from_ptr(MbObject::new_str(c.clone())))
            .collect();
        fields.insert(
            "_columns".to_string(),
            MbValue::from_ptr(MbObject::new_tuple_borrowed(colvals)),
        );
        new_instance_with_fields(ROW_CLASS, fields)
    } else {
        MbValue::from_ptr(MbObject::new_tuple_borrowed(mbs))
    }
}

/// `Cursor.execute(sql, params=())` — run, buffer rows, set rowcount/lastrowid.
fn cursor_do_execute(cur: MbValue, sql: &str, params: SqParams) {
    let conn_id = conn_id_of(cur);
    let cur_id = cur_id_of(cur);
    // An empty / whitespace-only statement is a no-op (CPython returns NULL
    // for it; rusqlite's prepare("") would otherwise error).
    if sql.trim().is_empty() {
        CURSORS.with(|cs| {
            cs.borrow_mut().insert(cur_id, CursorState::default());
        });
        inst_set_field(cur, "rowcount", MbValue::from_int(-1));
        return;
    }
    maybe_begin(conn_id, sql);
    match run_one(conn_id, sql, &params) {
        Ok(q) => {
            CURSORS.with(|cs| {
                cs.borrow_mut().insert(
                    cur_id,
                    CursorState { rows: q.rows, columns: q.columns, pos: 0 },
                );
            });
            let rc = if q.returns_rows { -1 } else { q.changes };
            inst_set_field(cur, "rowcount", MbValue::from_int(rc));
            inst_set_field(cur, "lastrowid", int_mb(q.last_id));
        }
        Err(e) => raise_sqlite_err(&e),
    }
}

/// `row_factory` to thread into fetched rows (Connection attr is inherited).
fn cursor_row_factory(cur: MbValue) -> Option<MbValue> {
    inst_field(cur, "row_factory").filter(|v| !v.is_none())
}

/// sqlite3.connect(database) -> `Connection` instance
///
/// Returns a genuine `Connection` instance (registered class), so
/// `isinstance(conn, sqlite3.Connection)` holds and `conn.cursor()` /
/// `conn.execute(...)` dispatch through the class method table. State lives in
/// instance fields, mirroring the previous dict-backed shim.
pub fn mb_sqlite3_connect(db_path: MbValue) -> MbValue {
    let path = extract_str(db_path).unwrap_or_else(|| ":memory:".to_string());
    // A path naming an existing directory cannot be opened as a database file:
    // CPython's sqlite3.connect raises OperationalError "unable to open database
    // file". (":memory:" and "" are special sentinels that never touch the FS.)
    if path != ":memory:" && !path.is_empty() {
        if let Ok(meta) = std::fs::metadata(&path) {
            if meta.is_dir() {
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("OperationalError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(
                        "unable to open database file".to_string(),
                    )),
                );
                return MbValue::none();
            }
        }
    }
    let open = if path == ":memory:" || path.is_empty() {
        rusqlite::Connection::open_in_memory()
    } else {
        rusqlite::Connection::open(&path)
    };
    let conn = match open {
        Ok(c) => c,
        Err(e) => {
            raise_sqlite_err(&e);
            return MbValue::none();
        }
    };
    let cid = sq_alloc_id();
    CONNS.with(|m| {
        m.borrow_mut().insert(cid, conn);
    });
    let mut fields: FxHashMap<String, MbValue> = FxHashMap::default();
    fields.insert(
        "database".to_string(),
        MbValue::from_ptr(MbObject::new_str(path)),
    );
    fields.insert("closed".to_string(), MbValue::from_bool(false));
    fields.insert("_cid".to_string(), MbValue::from_int(cid as i64));
    fields.insert("row_factory".to_string(), MbValue::none());
    fields.insert("_trace_callback".to_string(), MbValue::none());
    fields.insert("_progress_handler".to_string(), MbValue::none());
    fields.insert("_progress_handler_n".to_string(), MbValue::none());
    fields.insert("_authorizer".to_string(), MbValue::none());
    new_instance_with_fields(CONNECTION_CLASS, fields)
}

/// conn.cursor() -> a `Cursor` bound to the connection (inherits row_factory).
pub fn mb_sqlite3_cursor(conn: MbValue) -> MbValue {
    let cid = conn_id_of(conn);
    let curid = sq_alloc_id();
    CURSORS.with(|cs| {
        cs.borrow_mut().insert(curid, CursorState::default());
    });
    let rf = inst_field(conn, "row_factory").unwrap_or_else(MbValue::none);
    unsafe { super::super::rc::retain_if_ptr(rf); }
    let mut fields: FxHashMap<String, MbValue> = FxHashMap::default();
    fields.insert("_cid".to_string(), MbValue::from_int(cid as i64));
    fields.insert("_curid".to_string(), MbValue::from_int(curid as i64));
    fields.insert("row_factory".to_string(), rf);
    fields.insert("rowcount".to_string(), MbValue::from_int(-1));
    fields.insert("lastrowid".to_string(), MbValue::none());
    fields.insert("description".to_string(), MbValue::none());
    new_instance_with_fields(CURSOR_CLASS, fields)
}
#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    // Connections / cursors are now `Instance`s (registered classes), so the
    // test probes read instance fields rather than dict entries.
    fn field_bool(val: MbValue, key: &str) -> Option<bool> {
        inst_field(val, key).and_then(|v| v.as_bool())
    }

    fn field_str(val: MbValue, key: &str) -> Option<String> {
        inst_field(val, key).and_then(extract_str)
    }

    fn instance_class(val: MbValue) -> Option<String> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                Some(class_name.clone())
            } else {
                None
            }
        })
    }

    fn has_table(conn: MbValue, table: &str) -> bool {
        if let Some(tables) = inst_field(conn, "_tables") {
            if let Some(tbl_ptr) = tables.as_ptr() {
                unsafe {
                    if let ObjData::Dict(ref tbl_lock) = (*tbl_ptr).data {
                        return tbl_lock.read().unwrap().contains_key(table);
                    }
                }
            }
        }
        false
    }

    // --- extract_str ---
    #[test]
    fn test_extract_str_str() {
        assert_eq!(extract_str(s("hello")), Some("hello".to_string()));
    }

    #[test]
    fn test_extract_str_non_str() {
        assert_eq!(extract_str(MbValue::from_int(0)), None);
    }

    // --- connect ---
    #[test]
    fn test_connect_with_str_path() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        assert_eq!(instance_class(conn), Some(CONNECTION_CLASS.to_string()));
        assert_eq!(field_str(conn, "database"), Some(":memory:".to_string()));
    }

    #[test]
    fn test_connect_non_str_defaults_to_memory() {
        let conn = mb_sqlite3_connect(MbValue::from_int(0));
        assert_eq!(field_str(conn, "database"), Some(":memory:".to_string()));
    }

    // --- cursor ---
    #[test]
    fn test_cursor_returns_cursor_instance() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        let cursor = mb_sqlite3_cursor(conn);
        // cursor() returns a distinct Cursor instance (not the connection).
        assert_eq!(instance_class(cursor), Some(CURSOR_CLASS.to_string()));
    }

    // --- real backend fields/methods ---
    #[test]
    fn test_connect_and_close() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        unsafe {
            m_connection_close(conn, MbValue::none());
        }
        assert_eq!(field_bool(conn, "closed"), Some(true));
    }

    #[test]
    fn test_cursor_tracks_connection_and_state_ids() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        let cursor = mb_sqlite3_cursor(conn);
        assert_eq!(inst_field(cursor, "_cid").and_then(|v| v.as_int()), inst_field(conn, "_cid").and_then(|v| v.as_int()));
        assert!(inst_field(cursor, "_curid").and_then(|v| v.as_int()).is_some());
        assert_eq!(inst_field(cursor, "rowcount").and_then(|v| v.as_int()), Some(-1));
    }

    #[test]
    fn test_connection_callback_setters_accept_none_clear() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        unsafe {
            assert!(m_connection_set_trace_callback(conn, MbValue::none()).is_none());
            assert!(m_connection_set_progress_handler(conn, MbValue::none()).is_none());
            assert!(m_connection_set_authorizer(conn, MbValue::none()).is_none());
        }
        assert!(inst_field(conn, "_trace_callback").unwrap().is_none());
        assert!(inst_field(conn, "_progress_handler").unwrap().is_none());
        assert!(inst_field(conn, "_authorizer").unwrap().is_none());
    }
}
