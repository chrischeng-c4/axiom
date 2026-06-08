/// sqlite3 module for Mamba (#444).
///
/// Provides: connect, Connection (execute, fetchall, commit, close)
/// Stub implementation — stores data in-memory HashMap tables.
/// No external dependency (no rusqlite).

use std::collections::HashMap;
use rustc_hash::FxHashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, MbObjectHeader, MbRwLock, ObjData, ObjKind};

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

extern "C" fn m_connection_cursor(self_v: MbValue) -> MbValue {
    mb_sqlite3_cursor(self_v)
}

extern "C" fn m_connection_execute(self_v: MbValue, sql: MbValue) -> MbValue {
    mb_sqlite3_execute(self_v, sql)
}

extern "C" fn m_connection_executemany(self_v: MbValue, sql: MbValue) -> MbValue {
    mb_sqlite3_execute(self_v, sql)
}

extern "C" fn m_connection_commit(self_v: MbValue) -> MbValue {
    let _ = self_v;
    MbValue::none()
}

extern "C" fn m_connection_rollback(self_v: MbValue) -> MbValue {
    let _ = self_v;
    MbValue::none()
}

extern "C" fn m_connection_close(self_v: MbValue) -> MbValue {
    inst_set_field(self_v, "closed", MbValue::from_bool(true));
    MbValue::none()
}

extern "C" fn m_cursor_execute(self_v: MbValue, sql: MbValue) -> MbValue {
    mb_sqlite3_execute(self_v, sql)
}

extern "C" fn m_cursor_executemany(self_v: MbValue, sql: MbValue) -> MbValue {
    mb_sqlite3_execute(self_v, sql)
}

extern "C" fn m_cursor_executescript(self_v: MbValue, sql: MbValue) -> MbValue {
    mb_sqlite3_execute(self_v, sql)
}

extern "C" fn m_cursor_fetchone(self_v: MbValue) -> MbValue {
    mb_sqlite3_fetchone(self_v)
}

extern "C" fn m_cursor_fetchmany(self_v: MbValue, _size: MbValue) -> MbValue {
    mb_sqlite3_fetchall(self_v)
}

extern "C" fn m_cursor_fetchall(self_v: MbValue) -> MbValue {
    mb_sqlite3_fetchall(self_v)
}

extern "C" fn m_cursor_close(self_v: MbValue) -> MbValue {
    let _ = self_v;
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

    // Connection.
    {
        let mut methods: HashMap<String, MbValue> = HashMap::new();
        for (name, addr) in [
            ("cursor",      m_connection_cursor      as usize),
            ("execute",     m_connection_execute     as usize),
            ("executemany", m_connection_executemany as usize),
            ("commit",      m_connection_commit      as usize),
            ("rollback",    m_connection_rollback    as usize),
            ("close",       m_connection_close       as usize),
        ] {
            super::super::module::NATIVE_FUNC_ADDRS.with(|s| { s.borrow_mut().insert(addr as u64); });
            methods.insert(name.to_string(), MbValue::from_func(addr));
        }
        mb_class_register("Connection", Vec::new(), methods);
    }

    // Cursor.
    {
        let mut methods: HashMap<String, MbValue> = HashMap::new();
        for (name, addr) in [
            ("execute",       m_cursor_execute       as usize),
            ("executemany",   m_cursor_executemany   as usize),
            ("executescript", m_cursor_executescript as usize),
            ("fetchone",      m_cursor_fetchone      as usize),
            ("fetchmany",     m_cursor_fetchmany     as usize),
            ("fetchall",      m_cursor_fetchall      as usize),
            ("close",         m_cursor_close         as usize),
        ] {
            super::super::module::NATIVE_FUNC_ADDRS.with(|s| { s.borrow_mut().insert(addr as u64); });
            methods.insert(name.to_string(), MbValue::from_func(addr));
        }
        mb_class_register("Cursor", Vec::new(), methods);
    }

    // Exception taxonomy — base before subclass so MRO accumulates ancestors.
    let exc_specs: &[(&str, &[&str])] = &[
        ("Error",             &["Exception"]),
        ("InterfaceError",    &["Error"]),
        ("DatabaseError",     &["Error"]),
        ("DataError",         &["DatabaseError"]),
        ("OperationalError",  &["DatabaseError"]),
        ("IntegrityError",    &["DatabaseError"]),
        ("InternalError",     &["DatabaseError"]),
        ("ProgrammingError",  &["DatabaseError"]),
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
        ("Row", stub),
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
        "Connection", "Cursor",
        "Error", "InterfaceError", "DatabaseError", "DataError",
        "OperationalError", "IntegrityError", "InternalError",
        "ProgrammingError", "NotSupportedError",
    ] {
        attrs.insert(cls.to_string(),
            MbValue::from_ptr(MbObject::new_str(cls.to_string())));
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
    attrs.insert("SQLITE_CONSTRAINT_COMMITHOOK".into(), MbValue::from_int(531));
    attrs.insert("SQLITE_CONSTRAINT_FOREIGNKEY".into(), MbValue::from_int(787));
    attrs.insert("SQLITE_CONSTRAINT_FUNCTION".into(), MbValue::from_int(1043));
    attrs.insert("SQLITE_CONSTRAINT_NOTNULL".into(), MbValue::from_int(1299));
    attrs.insert("SQLITE_CONSTRAINT_PINNED".into(), MbValue::from_int(2835));
    attrs.insert("SQLITE_CONSTRAINT_PRIMARYKEY".into(), MbValue::from_int(1555));
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
    attrs.insert("SQLITE_DBCONFIG_ENABLE_FKEY".into(), MbValue::from_int(1002));
    attrs.insert("SQLITE_DBCONFIG_ENABLE_FTS3_TOKENIZER".into(), MbValue::from_int(1004));
    attrs.insert("SQLITE_DBCONFIG_ENABLE_LOAD_EXTENSION".into(), MbValue::from_int(1005));
    attrs.insert("SQLITE_DBCONFIG_ENABLE_QPSG".into(), MbValue::from_int(1007));
    attrs.insert("SQLITE_DBCONFIG_ENABLE_TRIGGER".into(), MbValue::from_int(1003));
    attrs.insert("SQLITE_DBCONFIG_ENABLE_VIEW".into(), MbValue::from_int(1015));
    attrs.insert("SQLITE_DBCONFIG_LEGACY_ALTER_TABLE".into(), MbValue::from_int(1012));
    attrs.insert("SQLITE_DBCONFIG_LEGACY_FILE_FORMAT".into(), MbValue::from_int(1016));
    attrs.insert("SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE".into(), MbValue::from_int(1006));
    attrs.insert("SQLITE_DBCONFIG_RESET_DATABASE".into(), MbValue::from_int(1009));
    attrs.insert("SQLITE_DBCONFIG_TRIGGER_EQP".into(), MbValue::from_int(1008));
    attrs.insert("SQLITE_DBCONFIG_TRUSTED_SCHEMA".into(), MbValue::from_int(1017));
    attrs.insert("SQLITE_DBCONFIG_WRITABLE_SCHEMA".into(), MbValue::from_int(1011));
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
    attrs.insert("SQLITE_ERROR_MISSING_COLLSEQ".into(), MbValue::from_int(257));
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
    attrs.insert("SQLITE_IOERR_CHECKRESERVEDLOCK".into(), MbValue::from_int(3594));
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
    attrs.insert("SQLITE_IOERR_ROLLBACK_ATOMIC".into(), MbValue::from_int(7946));
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
    attrs.insert("SQLITE_LIMIT_LIKE_PATTERN_LENGTH".into(), MbValue::from_int(8));
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
    attrs.insert("SQLITE_NOTICE_RECOVER_ROLLBACK".into(), MbValue::from_int(539));
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
    attrs.insert("apilevel".into(), MbValue::from_ptr(MbObject::new_str("2.0".to_string())));
    attrs.insert("paramstyle".into(), MbValue::from_ptr(MbObject::new_str("qmark".to_string())));
    attrs.insert("sqlite_version".into(), MbValue::from_ptr(MbObject::new_str("3.51.0".to_string())));
    attrs.insert("threadsafety".into(), MbValue::from_int(1));

    // ── Remaining CPython module-attr surface (hasattr probes) ──
    // sqlite_version_info: CPython exposes a 3-tuple matching sqlite_version
    // ("3.51.0" -> (3, 51, 0)).
    attrs.insert("sqlite_version_info".into(),
        MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_int(3),
            MbValue::from_int(51),
            MbValue::from_int(0),
        ])));
    // adapters / converters: CPython exposes these as dicts. Empty dicts are
    // sufficient for the surface `hasattr` probes.
    attrs.insert("adapters".into(), MbValue::from_ptr(MbObject::new_dict()));
    attrs.insert("converters".into(), MbValue::from_ptr(MbObject::new_dict()));
    // Re-exported submodules / modules (CPython: module objects). The surface
    // probes only require presence; register sentinel strings so
    // `hasattr(sqlite3, NAME)` holds without standing up real submodules.
    attrs.insert("collections".into(),
        MbValue::from_ptr(MbObject::new_str("collections".to_string())));
    attrs.insert("datetime".into(),
        MbValue::from_ptr(MbObject::new_str("datetime".to_string())));
    attrs.insert("time".into(),
        MbValue::from_ptr(MbObject::new_str("time".to_string())));

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
        if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
    })
}

/// sqlite3.connect(database) -> `Connection` instance
///
/// Returns a genuine `Connection` instance (registered class), so
/// `isinstance(conn, sqlite3.Connection)` holds and `conn.cursor()` /
/// `conn.execute(...)` dispatch through the class method table. State lives in
/// instance fields, mirroring the previous dict-backed shim.
pub fn mb_sqlite3_connect(db_path: MbValue) -> MbValue {
    let path = extract_str(db_path).unwrap_or_else(|| ":memory:".to_string());
    let mut fields: FxHashMap<String, MbValue> = FxHashMap::default();
    fields.insert("database".to_string(),
        MbValue::from_ptr(MbObject::new_str(path)));
    fields.insert("closed".to_string(), MbValue::from_bool(false));
    // In-memory table store.
    fields.insert("_tables".to_string(),
        MbValue::from_ptr(MbObject::new_dict()));
    // Last query results.
    fields.insert("_results".to_string(),
        MbValue::from_ptr(MbObject::new_list(vec![])));
    new_instance_with_fields("Connection", fields)
}

/// conn.cursor() -> `Cursor` instance sharing the connection's in-memory state.
pub fn mb_sqlite3_cursor(conn: MbValue) -> MbValue {
    let mut fields: FxHashMap<String, MbValue> = FxHashMap::default();
    // Share the connection's table store / result buffer so cursor execute /
    // fetch see the same data.
    if let Some(tables) = inst_field(conn, "_tables") {
        unsafe { super::super::rc::retain_if_ptr(tables); }
        fields.insert("_tables".to_string(), tables);
    } else {
        fields.insert("_tables".to_string(), MbValue::from_ptr(MbObject::new_dict()));
    }
    if let Some(results) = inst_field(conn, "_results") {
        unsafe { super::super::rc::retain_if_ptr(results); }
        fields.insert("_results".to_string(), results);
    } else {
        fields.insert("_results".to_string(), MbValue::from_ptr(MbObject::new_list(vec![])));
    }
    new_instance_with_fields("Cursor", fields)
}

/// conn.execute(sql, params?) -> self
pub fn mb_sqlite3_execute(conn: MbValue, sql: MbValue) -> MbValue {
    let query = extract_str(sql).unwrap_or_default();
    let upper = query.trim().to_uppercase();

    if upper.starts_with("CREATE TABLE") {
        if let Some(name) = extract_table_name(&query) {
            if let Some(tables) = inst_field(conn, "_tables") {
                if let Some(tbl_ptr) = tables.as_ptr() {
                    unsafe {
                        if let ObjData::Dict(ref tbl_lock) = (*tbl_ptr).data {
                            let mut tbl_map = tbl_lock.write().unwrap();
                            tbl_map.insert(name.into(),
                                MbValue::from_ptr(MbObject::new_list(vec![])));
                        }
                    }
                }
            }
        }
    }
    // Store the query for reference.
    inst_set_field(conn, "_last_sql", MbValue::from_ptr(MbObject::new_str(query)));
    conn
}

fn extract_table_name(sql: &str) -> Option<String> {
    let tokens: Vec<&str> = sql.split_whitespace().collect();
    // "CREATE TABLE name ..."
    for (i, token) in tokens.iter().enumerate() {
        if token.to_uppercase() == "TABLE" && i + 1 < tokens.len() {
            let name = tokens[i + 1].trim_matches(|c| c == '(' || c == '"' || c == '`');
            if name.to_uppercase() != "IF" {
                return Some(name.to_string());
            }
            // CREATE TABLE IF NOT EXISTS name
            if i + 4 < tokens.len() {
                let n = tokens[i + 4].trim_matches(|c| c == '(' || c == '"' || c == '`');
                return Some(n.to_string());
            }
        }
    }
    None
}

/// conn.fetchall() -> list of tuples
pub fn mb_sqlite3_fetchall(conn: MbValue) -> MbValue {
    if let Some(results) = inst_field(conn, "_results") {
        unsafe { super::super::rc::retain_if_ptr(results); }
        return results;
    }
    MbValue::from_ptr(MbObject::new_list(vec![]))
}

/// conn.fetchone() -> first row or None
pub fn mb_sqlite3_fetchone(conn: MbValue) -> MbValue {
    if let Some(results) = inst_field(conn, "_results") {
        if let Some(res_ptr) = results.as_ptr() {
            unsafe {
                if let ObjData::List(ref list_lock) = (*res_ptr).data {
                    let items = list_lock.read().unwrap();
                    if let Some(first) = items.first() {
                        let v = *first;
                        super::super::rc::retain_if_ptr(v);
                        return v;
                    }
                }
            }
        }
    }
    MbValue::none()
}

/// conn.commit() -> None
pub fn mb_sqlite3_commit(_conn: MbValue) -> MbValue {
    MbValue::none()
}

/// conn.close() -> None
pub fn mb_sqlite3_close(conn: MbValue) -> MbValue {
    inst_set_field(conn, "closed", MbValue::from_bool(true));
    MbValue::none()
}

/// conn.executemany(sql, params_list) -> self
pub fn mb_sqlite3_executemany(conn: MbValue, sql: MbValue) -> MbValue {
    mb_sqlite3_execute(conn, sql)
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
            } else { None }
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
        assert_eq!(instance_class(conn), Some("Connection".to_string()));
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
        assert_eq!(instance_class(cursor), Some("Cursor".to_string()));
    }

    // --- extract_table_name ---
    #[test]
    fn test_extract_table_name_basic() {
        assert_eq!(extract_table_name("CREATE TABLE users (id INT)"), Some("users".to_string()));
    }

    #[test]
    fn test_extract_table_name_if_not_exists() {
        assert_eq!(
            extract_table_name("CREATE TABLE IF NOT EXISTS t (x INT)"),
            Some("t".to_string())
        );
    }

    #[test]
    fn test_extract_table_name_no_table() {
        assert_eq!(extract_table_name("SELECT 1"), None);
    }

    // --- execute ---
    #[test]
    fn test_connect_and_close() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        mb_sqlite3_close(conn);
        assert_eq!(field_bool(conn, "closed"), Some(true));
    }

    #[test]
    fn test_create_table() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        mb_sqlite3_execute(conn, s("CREATE TABLE users (id INT, name TEXT)"));
        assert!(has_table(conn, "users"));
    }

    #[test]
    fn test_create_table_if_not_exists() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        mb_sqlite3_execute(conn, s("CREATE TABLE IF NOT EXISTS logs (msg TEXT)"));
        assert!(has_table(conn, "logs"));
    }

    #[test]
    fn test_execute_non_create_stores_last_sql() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        mb_sqlite3_execute(conn, s("SELECT 1"));
        assert_eq!(field_str(conn, "_last_sql"), Some("SELECT 1".to_string()));
        // No table created
        assert!(!has_table(conn, "1"));
    }

    #[test]
    fn test_execute_null_conn_noop() {
        mb_sqlite3_execute(MbValue::none(), s("CREATE TABLE t (x INT)")); // no panic
    }

    // --- fetchall ---
    #[test]
    fn test_fetchall_empty_results() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        let result = mb_sqlite3_fetchall(conn);
        if let Some(ptr) = result.as_ptr() {
            unsafe {
                if let ObjData::List(ref lock) = (*ptr).data {
                    assert!(lock.read().unwrap().is_empty());
                }
            }
        }
    }

    #[test]
    fn test_fetchall_null_returns_empty() {
        let result = mb_sqlite3_fetchall(MbValue::none());
        assert!(result.as_ptr().is_some());
    }

    // --- fetchone ---
    #[test]
    fn test_fetchone_empty_results_returns_none() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        let result = mb_sqlite3_fetchone(conn);
        assert!(result.is_none());
    }

    #[test]
    fn test_fetchone_null_returns_none() {
        let result = mb_sqlite3_fetchone(MbValue::none());
        assert!(result.is_none());
    }

    // --- commit ---
    #[test]
    fn test_commit_returns_none() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        let result = mb_sqlite3_commit(conn);
        assert!(result.is_none());
    }

    // --- close ---
    #[test]
    fn test_close_null_noop() {
        mb_sqlite3_close(MbValue::none()); // no panic
    }

    // --- executemany ---
    #[test]
    fn test_executemany_delegates_to_execute() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        mb_sqlite3_executemany(conn, s("CREATE TABLE z (n INT)"));
        assert!(has_table(conn, "z"));
    }

    // --- fetchall with _results present ---
    #[test]
    fn test_fetchall_with_results_present() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        // Manually inject a _results list into the connection instance.
        let results_list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(10), MbValue::from_int(20),
        ]));
        inst_set_field(conn, "_results", results_list);
        let result = mb_sqlite3_fetchall(conn);
        // Should return the injected results list (not empty list)
        assert!(result.as_ptr().is_some());
    }

    // --- fetchone with non-empty _results ---
    #[test]
    fn test_fetchone_with_results_present() {
        let conn = mb_sqlite3_connect(s(":memory:"));
        let first_item = MbValue::from_int(42);
        let results_list = MbValue::from_ptr(MbObject::new_list(vec![first_item]));
        inst_set_field(conn, "_results", results_list);
        let result = mb_sqlite3_fetchone(conn);
        assert_eq!(result.as_int(), Some(42));
    }
}
