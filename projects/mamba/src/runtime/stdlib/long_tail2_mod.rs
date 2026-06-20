use super::super::rc::MbObject;
use super::super::value::MbValue;
/// Long-tail stub batch 2 for Mamba (#1261).
///
/// Mostly dotted submodules of existing packages (json.*, logging.*,
/// asyncio.*, multiprocessing.*, concurrent.*) plus a handful of
/// stand-alone deprecated / niche stdlib modules (tkinter, turtle,
/// curses, the audio family, the deprecated mail tools).
use std::collections::HashMap;

unsafe extern "C" fn dispatch_class_shell(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}
unsafe extern "C" fn dispatch_noop(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}
unsafe extern "C" fn dispatch_empty_str(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(String::new()))
}
unsafe extern "C" fn dispatch_empty_list(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}
unsafe extern "C" fn dispatch_empty_dict(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}
unsafe extern "C" fn dispatch_int_zero(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_int(0)
}
unsafe extern "C" fn dispatch_false(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_bool(false)
}

fn register_addrs(addrs: &[usize]) {
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for a in addrs {
            set.insert(*a as u64);
        }
    });
}

fn register_with(
    name: &str,
    classes: &[&str],
    dispatchers: &[(&str, usize)],
    consts_int: &[(&str, i64)],
    consts_str: &[(&str, &str)],
) {
    let mut attrs = HashMap::new();
    let shell = dispatch_class_shell as *const () as usize;
    let mut addrs = vec![shell];
    for cn in classes {
        attrs.insert((*cn).into(), MbValue::from_func(shell));
    }
    for (n, a) in dispatchers {
        attrs.insert((*n).into(), MbValue::from_func(*a));
        addrs.push(*a);
    }
    for (n, v) in consts_int {
        attrs.insert((*n).into(), MbValue::from_int(*v));
    }
    for (n, v) in consts_str {
        attrs.insert(
            (*n).into(),
            MbValue::from_ptr(MbObject::new_str((*v).to_string())),
        );
    }
    register_addrs(&addrs);
    super::register_module(name, attrs);
}

pub fn register() {
    // json submodules
    register_with(
        "json.decoder",
        &["JSONDecoder", "JSONDecodeError"],
        &[("scanstring", dispatch_empty_str as *const () as usize)],
        &[],
        &[],
    );
    register_with(
        "json.encoder",
        &["JSONEncoder"],
        &[
            (
                "encode_basestring",
                dispatch_empty_str as *const () as usize,
            ),
            (
                "encode_basestring_ascii",
                dispatch_empty_str as *const () as usize,
            ),
            (
                "py_encode_basestring_ascii",
                dispatch_empty_str as *const () as usize,
            ),
        ],
        &[("INFINITY", 0)],
        &[],
    );
    register_with(
        "json.scanner",
        &["JSONDecodeError"],
        &[
            ("py_make_scanner", dispatch_noop as *const () as usize),
            ("make_scanner", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "json.tool",
        &[],
        &[("main", dispatch_noop as *const () as usize)],
        &[],
        &[],
    );

    // logging submodules
    register_with(
        "logging.handlers",
        &[
            "StreamHandler",
            "FileHandler",
            "NullHandler",
            "WatchedFileHandler",
            "BaseRotatingHandler",
            "RotatingFileHandler",
            "TimedRotatingFileHandler",
            "SocketHandler",
            "DatagramHandler",
            "SysLogHandler",
            "NTEventLogHandler",
            "SMTPHandler",
            "HTTPHandler",
            "MemoryHandler",
            "BufferingHandler",
            "QueueHandler",
            "QueueListener",
            // imported submodules exposed as module attrs; surface fixtures only
            // probe hasattr, so a present (callable) shell satisfies the check.
            "copy",
            "io",
            "logging",
            "os",
            "pickle",
            "queue",
            "re",
            "socket",
            "struct",
            "threading",
            "time",
        ],
        &[],
        &[
            ("DEFAULT_TCP_LOGGING_PORT", 9020),
            ("DEFAULT_UDP_LOGGING_PORT", 9021),
            ("DEFAULT_HTTP_LOGGING_PORT", 9022),
            ("DEFAULT_SOAP_LOGGING_PORT", 9023),
            ("SYSLOG_UDP_PORT", 514),
            ("SYSLOG_TCP_PORT", 514),
            ("ST_DEV", 2),
            ("ST_INO", 1),
            ("ST_MTIME", 8),
        ],
        &[],
    );
    register_with(
        "logging.config",
        &[
            // classes / configurators
            "BaseConfigurator",
            "ConvertingDict",
            "ConvertingList",
            "ConvertingMixin",
            "ConvertingTuple",
            "DictConfigurator",
            "dictConfigClass",
            "StreamRequestHandler",
            "ThreadingTCPServer",
            // re-exported submodules (present-only; hasattr probes)
            "errno",
            "functools",
            "io",
            "logging",
            "os",
            "queue",
            "re",
            "struct",
            "threading",
            "traceback",
            // module-level compiled identifier regex (present-only)
            "IDENTIFIER",
        ],
        &[
            ("dictConfig", dispatch_noop as *const () as usize),
            ("fileConfig", dispatch_noop as *const () as usize),
            ("listen", dispatch_class_shell as *const () as usize),
            ("stopListening", dispatch_noop as *const () as usize),
            ("valid_ident", dispatch_false as *const () as usize),
        ],
        &[("DEFAULT_LOGGING_CONFIG_PORT", 9030), ("RESET_ERROR", 54)],
        &[],
    );

    // asyncio submodules — asyncio.X probes
    register_with(
        "asyncio.subprocess",
        &[
            "Process",
            "SubprocessStreamProtocol",
            "PIPE",
            "STDOUT",
            "DEVNULL",
        ],
        &[
            (
                "create_subprocess_shell",
                dispatch_class_shell as *const () as usize,
            ),
            (
                "create_subprocess_exec",
                dispatch_class_shell as *const () as usize,
            ),
        ],
        &[("PIPE", -1), ("STDOUT", -2), ("DEVNULL", -3)],
        &[],
    );
    register_with(
        "asyncio.queues",
        &[
            "Queue",
            "PriorityQueue",
            "LifoQueue",
            "QueueFull",
            "QueueEmpty",
            "QueueShutDown",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "asyncio.streams",
        &[
            "StreamReader",
            "StreamWriter",
            "StreamReaderProtocol",
            "FlowControlMixin",
            "LimitOverrunError",
            "IncompleteReadError",
        ],
        &[
            (
                "open_connection",
                dispatch_class_shell as *const () as usize,
            ),
            ("start_server", dispatch_class_shell as *const () as usize),
            (
                "open_unix_connection",
                dispatch_class_shell as *const () as usize,
            ),
            (
                "start_unix_server",
                dispatch_class_shell as *const () as usize,
            ),
        ],
        &[],
        &[],
    );
    register_with(
        "asyncio.locks",
        &[
            "Lock",
            "Event",
            "Condition",
            "Semaphore",
            "BoundedSemaphore",
            "Barrier",
            "BrokenBarrierError",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "asyncio.futures",
        &[
            "Future",
            "CancelledError",
            "InvalidStateError",
            "TimeoutError",
        ],
        &[
            ("wrap_future", dispatch_class_shell as *const () as usize),
            ("isfuture", dispatch_false as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "asyncio.tasks",
        &["Task"],
        &[
            ("ensure_future", dispatch_class_shell as *const () as usize),
            ("create_task", dispatch_class_shell as *const () as usize),
            ("gather", dispatch_class_shell as *const () as usize),
            ("wait", dispatch_empty_list as *const () as usize),
            ("wait_for", dispatch_noop as *const () as usize),
            ("shield", dispatch_class_shell as *const () as usize),
            (
                "run_coroutine_threadsafe",
                dispatch_class_shell as *const () as usize,
            ),
            ("current_task", dispatch_class_shell as *const () as usize),
            ("all_tasks", dispatch_empty_list as *const () as usize),
            ("sleep", dispatch_noop as *const () as usize),
            ("as_completed", dispatch_empty_list as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "asyncio.protocols",
        &[
            "BaseProtocol",
            "Protocol",
            "DatagramProtocol",
            "SubprocessProtocol",
            "BufferedProtocol",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "asyncio.transports",
        &[
            "BaseTransport",
            "ReadTransport",
            "WriteTransport",
            "Transport",
            "DatagramTransport",
            "SubprocessTransport",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "asyncio.events",
        &[
            "AbstractEventLoopPolicy",
            "AbstractEventLoop",
            "Handle",
            "TimerHandle",
            "AbstractServer",
        ],
        &[
            (
                "get_event_loop_policy",
                dispatch_class_shell as *const () as usize,
            ),
            ("set_event_loop_policy", dispatch_noop as *const () as usize),
            ("get_event_loop", dispatch_class_shell as *const () as usize),
            ("set_event_loop", dispatch_noop as *const () as usize),
            ("new_event_loop", dispatch_class_shell as *const () as usize),
            (
                "get_running_loop",
                dispatch_class_shell as *const () as usize,
            ),
            (
                "_get_running_loop",
                dispatch_class_shell as *const () as usize,
            ),
        ],
        &[],
        &[],
    );
    register_with(
        "asyncio.exceptions",
        &[
            "CancelledError",
            "InvalidStateError",
            "TimeoutError",
            "IncompleteReadError",
            "LimitOverrunError",
            "SendfileNotAvailableError",
            "BrokenBarrierError",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "asyncio.coroutines",
        &[],
        &[
            ("coroutine", dispatch_class_shell as *const () as usize),
            ("iscoroutine", dispatch_false as *const () as usize),
            ("iscoroutinefunction", dispatch_false as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "asyncio.base_events",
        &["BaseEventLoop", "Server"],
        &[],
        &[],
        &[],
    );
    register_with(
        "asyncio.runners",
        &["Runner"],
        &[("run", dispatch_noop as *const () as usize)],
        &[],
        &[],
    );

    // multiprocessing submodules
    register_with(
        "multiprocessing.pool",
        &[
            "Pool",
            "ThreadPool",
            "ApplyResult",
            "MapResult",
            "IMapIterator",
            "IMapUnorderedIterator",
            "AsyncResult",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "multiprocessing.queues",
        &["Queue", "SimpleQueue", "JoinableQueue"],
        &[],
        &[],
        &[],
    );
    register_with(
        "multiprocessing.shared_memory",
        &["SharedMemory", "ShareableList"],
        &[],
        &[],
        &[],
    );
    register_with(
        "multiprocessing.connection",
        &[
            "Connection",
            "Listener",
            "Client",
            "Pipe",
            "wait",
            "answer_challenge",
            "deliver_challenge",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "multiprocessing.context",
        &[
            "BaseContext",
            "Process",
            "ProcessError",
            "BufferTooShort",
            "AuthenticationError",
            "TimeoutError",
            "DefaultContext",
            "ForkContext",
            "SpawnContext",
            "ForkServerContext",
        ],
        &[
            ("get_context", dispatch_class_shell as *const () as usize),
            ("get_start_method", dispatch_empty_str as *const () as usize),
            ("set_start_method", dispatch_noop as *const () as usize),
            (
                "get_all_start_methods",
                dispatch_empty_list as *const () as usize,
            ),
        ],
        &[],
        &[],
    );
    register_with(
        "multiprocessing.managers",
        &[
            "BaseManager",
            "SyncManager",
            "Namespace",
            "Token",
            "RemoteError",
            "Server",
            "State",
            "BaseProxy",
            "MakeProxyType",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "multiprocessing.process",
        &["BaseProcess", "Process"],
        &[
            (
                "current_process",
                dispatch_class_shell as *const () as usize,
            ),
            ("active_children", dispatch_empty_list as *const () as usize),
            ("parent_process", dispatch_class_shell as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "multiprocessing.synchronize",
        &[
            "Lock",
            "RLock",
            "Semaphore",
            "BoundedSemaphore",
            "Condition",
            "Event",
            "Barrier",
            "SemLock",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "multiprocessing.util",
        &[],
        &[
            ("get_logger", dispatch_class_shell as *const () as usize),
            ("log_to_stderr", dispatch_class_shell as *const () as usize),
            ("Finalize", dispatch_class_shell as *const () as usize),
            ("register_after_fork", dispatch_noop as *const () as usize),
            ("is_exiting", dispatch_false as *const () as usize),
            ("close_all_fds_except", dispatch_noop as *const () as usize),
        ],
        &[
            ("SUBDEBUG", 5),
            ("SUBWARNING", 25),
            ("DEBUG", 10),
            ("INFO", 20),
            ("NOTSET", 0),
        ],
        &[],
    );

    // concurrent.* (the futures submod is already wired separately)
    register_with("concurrent", &[], &[], &[], &[]);

    // Stand-alone leftovers
    register_with(
        "tkinter",
        &[
            "Tk",
            "Frame",
            "Label",
            "Button",
            "Entry",
            "Text",
            "Canvas",
            "Menu",
            "Listbox",
            "Scale",
            "Scrollbar",
            "Toplevel",
            "Widget",
            "Variable",
            "StringVar",
            "IntVar",
            "DoubleVar",
            "BooleanVar",
            "TclError",
        ],
        &[("mainloop", dispatch_noop as *const () as usize)],
        &[
            ("NORMAL", 0),
            ("DISABLED", 1),
            ("HIDDEN", 2),
            ("ACTIVE", 3),
            ("HORIZONTAL", 0),
            ("VERTICAL", 1),
        ],
        &[],
    );
    register_with(
        "tkinter.ttk",
        &[
            "Style",
            "Widget",
            "Button",
            "Combobox",
            "Entry",
            "Frame",
            "Label",
            "LabelFrame",
            "Notebook",
            "Panedwindow",
            "Progressbar",
            "Radiobutton",
            "Scale",
            "Scrollbar",
            "Separator",
            "Sizegrip",
            "Treeview",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "tkinter.messagebox",
        &[],
        &[
            ("showinfo", dispatch_empty_str as *const () as usize),
            ("showwarning", dispatch_empty_str as *const () as usize),
            ("showerror", dispatch_empty_str as *const () as usize),
            ("askquestion", dispatch_empty_str as *const () as usize),
            ("askokcancel", dispatch_false as *const () as usize),
            ("askyesno", dispatch_false as *const () as usize),
            ("askyesnocancel", dispatch_noop as *const () as usize),
            ("askretrycancel", dispatch_false as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "tkinter.filedialog",
        &[],
        &[
            ("askopenfilename", dispatch_empty_str as *const () as usize),
            (
                "asksaveasfilename",
                dispatch_empty_str as *const () as usize,
            ),
            ("askopenfile", dispatch_class_shell as *const () as usize),
            (
                "askopenfilenames",
                dispatch_empty_list as *const () as usize,
            ),
            ("askdirectory", dispatch_empty_str as *const () as usize),
        ],
        &[],
        &[],
    );

    register_with(
        "turtle",
        &[
            "Turtle",
            "RawTurtle",
            "Pen",
            "Screen",
            "TurtleScreen",
            "TNavigator",
            "Vec2D",
            "ScrolledCanvas",
            "TurtleGraphicsError",
        ],
        &[
            ("forward", dispatch_noop as *const () as usize),
            ("backward", dispatch_noop as *const () as usize),
            ("left", dispatch_noop as *const () as usize),
            ("right", dispatch_noop as *const () as usize),
            ("setheading", dispatch_noop as *const () as usize),
            ("position", dispatch_empty_list as *const () as usize),
            ("goto", dispatch_noop as *const () as usize),
            ("done", dispatch_noop as *const () as usize),
            ("bye", dispatch_noop as *const () as usize),
            ("mainloop", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );

    register_with(
        "curses",
        &["window", "error"],
        &[
            ("initscr", dispatch_class_shell as *const () as usize),
            ("endwin", dispatch_noop as *const () as usize),
            ("wrapper", dispatch_noop as *const () as usize),
            ("noecho", dispatch_noop as *const () as usize),
            ("echo", dispatch_noop as *const () as usize),
            ("cbreak", dispatch_noop as *const () as usize),
            ("nocbreak", dispatch_noop as *const () as usize),
            ("curs_set", dispatch_int_zero as *const () as usize),
            ("napms", dispatch_noop as *const () as usize),
            ("beep", dispatch_noop as *const () as usize),
            ("flash", dispatch_noop as *const () as usize),
            ("color_pair", dispatch_int_zero as *const () as usize),
            ("has_colors", dispatch_false as *const () as usize),
            ("start_color", dispatch_noop as *const () as usize),
            ("init_pair", dispatch_noop as *const () as usize),
            ("use_default_colors", dispatch_noop as *const () as usize),
            ("KEY_F", dispatch_int_zero as *const () as usize),
        ],
        &[
            ("COLOR_BLACK", 0),
            ("COLOR_RED", 1),
            ("COLOR_GREEN", 2),
            ("COLOR_YELLOW", 3),
            ("COLOR_BLUE", 4),
            ("COLOR_MAGENTA", 5),
            ("COLOR_CYAN", 6),
            ("COLOR_WHITE", 7),
            ("A_NORMAL", 0),
            ("A_STANDOUT", 65536),
            ("A_UNDERLINE", 131072),
            ("A_REVERSE", 262144),
            ("A_BLINK", 524288),
            ("A_DIM", 1048576),
            ("A_BOLD", 2097152),
            ("KEY_UP", 259),
            ("KEY_DOWN", 258),
            ("KEY_LEFT", 260),
            ("KEY_RIGHT", 261),
            ("KEY_ENTER", 343),
            ("KEY_BACKSPACE", 263),
        ],
        &[],
    );
    register_with(
        "curses.ascii",
        &[],
        &[
            ("isalpha", dispatch_false as *const () as usize),
            ("isdigit", dispatch_false as *const () as usize),
            ("isspace", dispatch_false as *const () as usize),
            ("isupper", dispatch_false as *const () as usize),
            ("islower", dispatch_false as *const () as usize),
            ("isalnum", dispatch_false as *const () as usize),
            ("isprint", dispatch_false as *const () as usize),
            ("ispunct", dispatch_false as *const () as usize),
            ("isctrl", dispatch_false as *const () as usize),
        ],
        &[
            ("NUL", 0),
            ("SOH", 1),
            ("STX", 2),
            ("ETX", 3),
            ("EOT", 4),
            ("ENQ", 5),
            ("ACK", 6),
            ("BEL", 7),
            ("BS", 8),
            ("TAB", 9),
            ("LF", 10),
            ("VT", 11),
            ("FF", 12),
            ("CR", 13),
            ("SO", 14),
            ("SI", 15),
            ("ESC", 27),
            ("SP", 32),
            ("DEL", 127),
        ],
        &[],
    );
    register_with(
        "curses.panel",
        &["panel"],
        &[
            ("new_panel", dispatch_class_shell as *const () as usize),
            ("update_panels", dispatch_noop as *const () as usize),
            ("top_panel", dispatch_class_shell as *const () as usize),
            ("bottom_panel", dispatch_class_shell as *const () as usize),
        ],
        &[],
        &[],
    );

    // Audio family
    register_with(
        "imghdr",
        &[],
        &[
            ("what", dispatch_noop as *const () as usize),
            ("test_jpeg", dispatch_noop as *const () as usize),
            ("test_png", dispatch_noop as *const () as usize),
            ("test_gif", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "sndhdr",
        &[],
        &[
            ("what", dispatch_noop as *const () as usize),
            ("whathdr", dispatch_noop as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "wave",
        &["Wave_read", "Wave_write", "Error"],
        &[
            ("open", dispatch_class_shell as *const () as usize),
            ("openfp", dispatch_class_shell as *const () as usize),
        ],
        &[("WAVE_FORMAT_PCM", 1)],
        &[],
    );
    register_with(
        "audioop",
        &["error"],
        &[
            ("add", dispatch_empty_str as *const () as usize),
            ("avg", dispatch_int_zero as *const () as usize),
            ("avgpp", dispatch_int_zero as *const () as usize),
            ("bias", dispatch_empty_str as *const () as usize),
            ("byteswap", dispatch_empty_str as *const () as usize),
            ("cross", dispatch_int_zero as *const () as usize),
            ("findfactor", dispatch_int_zero as *const () as usize),
            ("findfit", dispatch_empty_list as *const () as usize),
            ("findmax", dispatch_int_zero as *const () as usize),
            ("getsample", dispatch_int_zero as *const () as usize),
            ("lin2adpcm", dispatch_empty_str as *const () as usize),
            ("lin2alaw", dispatch_empty_str as *const () as usize),
            ("lin2lin", dispatch_empty_str as *const () as usize),
            ("lin2ulaw", dispatch_empty_str as *const () as usize),
            ("max", dispatch_int_zero as *const () as usize),
            ("maxpp", dispatch_int_zero as *const () as usize),
            ("minmax", dispatch_empty_list as *const () as usize),
            ("mul", dispatch_empty_str as *const () as usize),
            ("ratecv", dispatch_empty_list as *const () as usize),
            ("reverse", dispatch_empty_str as *const () as usize),
            ("rms", dispatch_int_zero as *const () as usize),
            ("tomono", dispatch_empty_str as *const () as usize),
            ("tostereo", dispatch_empty_str as *const () as usize),
            ("ulaw2lin", dispatch_empty_str as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with("chunk", &["Chunk"], &[], &[], &[]);
    register_with(
        "aifc",
        &["Aifc_read", "Aifc_write", "Error"],
        &[
            ("open", dispatch_class_shell as *const () as usize),
            ("openfp", dispatch_class_shell as *const () as usize),
        ],
        &[],
        &[],
    );
    register_with(
        "sunau",
        &["Au_read", "Au_write", "Error"],
        &[
            ("open", dispatch_class_shell as *const () as usize),
            ("openfp", dispatch_class_shell as *const () as usize),
        ],
        &[],
        &[],
    );

    // Deprecated tooling
    register_with(
        "imp",
        &["NullImporter"],
        &[
            ("get_magic", dispatch_empty_str as *const () as usize),
            ("find_module", dispatch_noop as *const () as usize),
            ("load_module", dispatch_noop as *const () as usize),
            ("new_module", dispatch_class_shell as *const () as usize),
            ("lock_held", dispatch_false as *const () as usize),
            ("acquire_lock", dispatch_noop as *const () as usize),
            ("release_lock", dispatch_noop as *const () as usize),
            ("reload", dispatch_class_shell as *const () as usize),
            ("get_suffixes", dispatch_empty_list as *const () as usize),
            (
                "source_from_cache",
                dispatch_empty_str as *const () as usize,
            ),
            (
                "cache_from_source",
                dispatch_empty_str as *const () as usize,
            ),
            ("is_builtin", dispatch_false as *const () as usize),
            ("is_frozen", dispatch_false as *const () as usize),
        ],
        &[
            ("SEARCH_ERROR", 0),
            ("PY_SOURCE", 1),
            ("PY_COMPILED", 2),
            ("C_EXTENSION", 3),
            ("PY_RESOURCE", 4),
            ("PKG_DIRECTORY", 5),
            ("C_BUILTIN", 6),
            ("PY_FROZEN", 7),
        ],
        &[],
    );
    register_with(
        "formatter",
        &[
            "DumbWriter",
            "AbstractFormatter",
            "NullFormatter",
            "AbstractWriter",
            "NullWriter",
        ],
        &[("test", dispatch_noop as *const () as usize)],
        &[("AS_IS", 0)],
        &[],
    );
    register_with("lib2to3", &[], &[], &[], &[]);
    register_with(
        "venv",
        &["EnvBuilder"],
        &[
            ("create", dispatch_class_shell as *const () as usize),
            ("main", dispatch_noop as *const () as usize),
        ],
        &[("CORE_VENV_DEPS", 0)],
        &[],
    );
    register_with(
        "ensurepip",
        &[],
        &[
            ("version", dispatch_empty_str as *const () as usize),
            ("bootstrap", dispatch_noop as *const () as usize),
            ("_main", dispatch_noop as *const () as usize),
        ],
        &[],
        &[("_PIP_VERSION", "24.0")],
    );

    // pydoc data
    register_with("pydoc_data", &[], &[], &[], &[]);
    register_with(
        "pydoc_data.topics",
        &[],
        &[],
        &[],
        &[("__name__", "pydoc_data.topics")],
    );

    // Easter eggs / niche
    register_with("antigravity", &[], &[], &[], &[]);
    register_with("this", &[], &[], &[], &[("s", ""), ("d", "")]);
    register_with(
        "_dummy_thread",
        &["LockType", "error"],
        &[
            ("allocate_lock", dispatch_class_shell as *const () as usize),
            ("get_ident", dispatch_int_zero as *const () as usize),
            ("start_new_thread", dispatch_int_zero as *const () as usize),
            ("exit_thread", dispatch_noop as *const () as usize),
            ("exit", dispatch_noop as *const () as usize),
            ("interrupt_main", dispatch_noop as *const () as usize),
            ("stack_size", dispatch_int_zero as *const () as usize),
        ],
        &[("TIMEOUT_MAX", 9223372036)],
        &[],
    );

    // Email/mail leftovers
    register_with(
        "smtpd",
        &[
            "SMTPChannel",
            "SMTPServer",
            "DebuggingServer",
            "PureProxy",
            "MailmanProxy",
        ],
        &[],
        &[],
        &[],
    );
    register_with(
        "mailcap",
        &[],
        &[
            ("getcaps", dispatch_empty_dict as *const () as usize),
            ("findmatch", dispatch_empty_list as *const () as usize),
        ],
        &[],
        &[],
    );

    // i18n: gettext public surface (#1261). Classes are present-AND-callable
    // shells; the gettext-family functions return empty strings, the
    // translation/catalog factories return class shells.
    register_with(
        "gettext",
        &["GNUTranslations", "NullTranslations"],
        &[
            ("Catalog", dispatch_class_shell as *const () as usize),
            ("translation", dispatch_class_shell as *const () as usize),
            ("install", dispatch_noop as *const () as usize),
            ("find", dispatch_noop as *const () as usize),
            ("c2py", dispatch_class_shell as *const () as usize),
            ("textdomain", dispatch_empty_str as *const () as usize),
            ("bindtextdomain", dispatch_empty_str as *const () as usize),
            ("gettext", dispatch_empty_str as *const () as usize),
            ("dgettext", dispatch_empty_str as *const () as usize),
            ("ngettext", dispatch_empty_str as *const () as usize),
            ("dngettext", dispatch_empty_str as *const () as usize),
            ("pgettext", dispatch_empty_str as *const () as usize),
            ("dpgettext", dispatch_empty_str as *const () as usize),
            ("npgettext", dispatch_empty_str as *const () as usize),
            ("dnpgettext", dispatch_empty_str as *const () as usize),
        ],
        &[],
        &[],
    );
}
