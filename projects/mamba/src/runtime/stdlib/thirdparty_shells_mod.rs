use super::super::rc::MbObject;
use super::super::value::MbValue;
/// Third-party package probe shells for Mamba (#1261).
///
/// Zero-machinery marker modules for the most-frequently-imported
/// third-party Python packages. The goal is purely import-time
/// survival — legacy probe code often does `try: import X` to
/// feature-detect, and crashing inside the import is worse than
/// surfacing an AttributeError at the first real call site.
///
/// Each registered module is a dict with one `__name__` entry. Any
/// attribute lookup beyond `__name__` will fail normally — which is
/// the correct behaviour since the package's real functionality is
/// not available in Mamba.
use std::collections::HashMap;

fn register_marker(name: &str) {
    let mut attrs = HashMap::new();
    attrs.insert(
        "__name__".into(),
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
    );
    super::register_module(name, attrs);
}

pub fn register() {
    let names: &[&str] = &[
        // CLI / TUI
        "click",
        "colorama",
        "tabulate",
        "tqdm",
        "rich",
        "pygments",
        "rich_click",
        "textual",
        "prompt_toolkit",
        "blessed",
        "humanize",
        // Graph / network
        "networkx",
        // i18n / utils
        "babel",
        "blinker",
        "markupsafe",
        "itsdangerous",
        // File watching / async io
        "watchfiles",
        "watchdog",
        "uvloop",
        "anyio",
        "sniffio",
        // HTTP low-level
        "h11",
        "h2",
        "httpcore",
        "httptools",
        "wsproto",
        "tornado",
        "twisted",
        // Crypto utilities
        "asn1crypto",
        "cffi",
        "rsa",
        "ecdsa",
        "pynacl",
        "argon2",
        "bcrypt",
        "passlib",
        "jwt",
        "authlib",
        "oauthlib",
        "python_jose",
        "jose",
        "ldap3",
        // Process / system
        "greenlet",
        "setproctitle",
        "psutil",
        // Docs / parsing
        "docutils",
        "markdown",
        "beautifulsoup4",
        "bs4",
        "lxml",
        "lxml_html_clean",
        "requests_html",
        // Office / spreadsheet
        "openpyxl",
        "xlrd",
        "xlwt",
        // Date / time
        "python_dateutil",
        "dateutil",
        "pytz",
        "tzdata",
        "tomli",
        "pendulum",
        "arrow",
        // Text / matching
        "regex",
        "ujson",
        "simplejson",
        "cchardet",
        "chardet",
        "more_itertools",
        // File systems / cloud storage
        "smart_open",
        "gcsfs",
        "fsspec",
        "aiocache",
        "aiosignal",
        "frozenlist",
        "yarl",
        "multidict",
        "async_timeout",
        // Async DB
        "aiosqlite",
        "asyncpg",
        // Data libs
        "pandera",
        "polars",
        "duckdb",
        "pyarrow",
        "fastparquet",
        // DB clients
        "elasticsearch",
        "pymongo",
        "motor",
        "pymysql",
        "mysql_connector",
        // LLM ecosystem
        "tiktoken",
        "openai",
        "anthropic",
        "langchain",
        "langchain_core",
        "instructor",
        // Config
        "dotenv",
        "python_dotenv",
        "python_decouple",
        "environs",
        // Browser automation
        "playwright",
        "selenium",
        // Modern serialization / typing
        "msgspec",
        "cattrs",
        "typeguard",
        // Other commonly-imported
        "attrs",
    ];
    for n in names {
        register_marker(n);
    }
}
