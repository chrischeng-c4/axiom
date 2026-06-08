use super::rc::MbObject;
use super::value::MbValue;
/// File I/O runtime support (#379).
///
/// Implements Python-compatible file operations: open, read, write, close.
/// Files are stored as a thread-local handle table (not heap objects) to avoid
/// complicating ObjData with non-Send types.
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};

/// File handle state.
#[allow(dead_code)]
struct MbFile {
    reader: Option<BufReader<fs::File>>,
    writer: Option<fs::File>,
    mode: String,
    path: String,
    closed: bool,
}

thread_local! {
    static FILES: std::cell::RefCell<HashMap<u64, MbFile>> =
        std::cell::RefCell::new(HashMap::new());
    static NEXT_FILE_ID: std::cell::Cell<u64> = std::cell::Cell::new(1);
}

fn alloc_file_id() -> u64 {
    NEXT_FILE_ID.with(|cell| {
        let id = cell.get();
        cell.set(id + 1);
        id
    })
}

/// Check if a given ID corresponds to an open file handle.
pub fn is_file_handle(id: u64) -> bool {
    FILES.with(|files| files.borrow().contains_key(&id))
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            super::rc::ObjData::Str(s) => Some(s.clone()),
            super::rc::ObjData::Bytes(bytes) => Some(String::from_utf8_lossy(bytes).into_owned()),
            super::rc::ObjData::ByteArray(lock) => {
                Some(String::from_utf8_lossy(&lock.read().unwrap()).into_owned())
            }
            _ => None,
        }
    })
}

fn extract_bytes(val: MbValue) -> Option<Vec<u8>> {
    val.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            super::rc::ObjData::Str(s) => Some(s.as_bytes().to_vec()),
            super::rc::ObjData::Bytes(bytes) => Some(bytes.clone()),
            super::rc::ObjData::ByteArray(lock) => Some(lock.read().unwrap().clone()),
            _ => None,
        }
    })
}

pub fn mb_file_name(handle: MbValue) -> MbValue {
    if let Some(id) = handle.as_int() {
        FILES.with(|files| {
            files
                .borrow()
                .get(&(id as u64))
                .map(|file| MbValue::from_ptr(MbObject::new_str(file.path.clone())))
                .unwrap_or_else(MbValue::none)
        })
    } else {
        MbValue::none()
    }
}

/// open(path, mode) → file handle (as MbValue int)
pub fn mb_open(path: MbValue, mode: MbValue) -> MbValue {
    let file_path = match extract_str(path) {
        Some(p) => p,
        None => {
            raise_type_error("open() argument must be a string");
            return MbValue::none();
        }
    };
    let mode_str = extract_str(mode).unwrap_or_else(|| "r".to_string());

    match mode_str.as_str() {
        "r" => match fs::File::open(&file_path) {
            Ok(f) => {
                let id = alloc_file_id();
                let mf = MbFile {
                    reader: Some(BufReader::new(f)),
                    writer: None,
                    mode: mode_str,
                    path: file_path,
                    closed: false,
                };
                FILES.with(|files| files.borrow_mut().insert(id, mf));
                MbValue::from_int(id as i64)
            }
            Err(_) => {
                raise_file_not_found(&file_path);
                MbValue::none()
            }
        },
        "w" => match fs::File::create(&file_path) {
            Ok(f) => {
                let id = alloc_file_id();
                let mf = MbFile {
                    reader: None,
                    writer: Some(f),
                    mode: mode_str,
                    path: file_path,
                    closed: false,
                };
                FILES.with(|files| files.borrow_mut().insert(id, mf));
                MbValue::from_int(id as i64)
            }
            Err(_) => {
                raise_os_error(&format!("Cannot open '{file_path}' for writing"));
                MbValue::none()
            }
        },
        "a" => {
            match fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&file_path)
            {
                Ok(f) => {
                    let id = alloc_file_id();
                    let mf = MbFile {
                        reader: None,
                        writer: Some(f),
                        mode: mode_str,
                        path: file_path,
                        closed: false,
                    };
                    FILES.with(|files| files.borrow_mut().insert(id, mf));
                    MbValue::from_int(id as i64)
                }
                Err(_) => {
                    raise_os_error(&format!("Cannot open '{file_path}' for appending"));
                    MbValue::none()
                }
            }
        }
        _ => {
            raise_value_error(&format!("invalid mode: '{mode_str}'"));
            MbValue::none()
        }
    }
}

/// file.read() → string (entire file contents)
pub fn mb_file_read(handle: MbValue) -> MbValue {
    if let Some(id) = handle.as_int() {
        FILES.with(|files| {
            let mut files = files.borrow_mut();
            if let Some(mf) = files.get_mut(&(id as u64)) {
                if mf.closed {
                    raise_value_error("I/O operation on closed file");
                    return MbValue::none();
                }
                if let Some(ref mut reader) = mf.reader {
                    let mut contents = String::new();
                    let _ = reader.read_to_string(&mut contents);
                    return MbValue::from_ptr(MbObject::new_str(contents));
                }
            }
            MbValue::none()
        })
    } else {
        MbValue::none()
    }
}

/// file.readline() → string (one line)
pub fn mb_file_readline(handle: MbValue) -> MbValue {
    if let Some(id) = handle.as_int() {
        FILES.with(|files| {
            let mut files = files.borrow_mut();
            if let Some(mf) = files.get_mut(&(id as u64)) {
                if mf.closed {
                    raise_value_error("I/O operation on closed file");
                    return MbValue::none();
                }
                if let Some(ref mut reader) = mf.reader {
                    let mut line = String::new();
                    match reader.read_line(&mut line) {
                        Ok(0) => return MbValue::from_ptr(MbObject::new_str(String::new())),
                        Ok(_) => return MbValue::from_ptr(MbObject::new_str(line)),
                        Err(_) => return MbValue::none(),
                    }
                }
            }
            MbValue::none()
        })
    } else {
        MbValue::none()
    }
}

/// file.readlines() → list of strings
pub fn mb_file_readlines(handle: MbValue) -> MbValue {
    if let Some(id) = handle.as_int() {
        FILES.with(|files| {
            let mut files = files.borrow_mut();
            if let Some(mf) = files.get_mut(&(id as u64)) {
                if mf.closed {
                    raise_value_error("I/O operation on closed file");
                    return MbValue::none();
                }
                if let Some(ref mut reader) = mf.reader {
                    let mut lines = Vec::new();
                    loop {
                        let mut line = String::new();
                        match reader.read_line(&mut line) {
                            Ok(0) => break,
                            Ok(_) => lines.push(MbValue::from_ptr(MbObject::new_str(line))),
                            Err(_) => break,
                        }
                    }
                    return MbValue::from_ptr(MbObject::new_list(lines));
                }
            }
            MbValue::none()
        })
    } else {
        MbValue::none()
    }
}

/// file.write(text) → number of characters written
pub fn mb_file_write(handle: MbValue, text: MbValue) -> MbValue {
    let content = extract_bytes(text).unwrap_or_default();
    if let Some(id) = handle.as_int() {
        FILES.with(|files| {
            let mut files = files.borrow_mut();
            if let Some(mf) = files.get_mut(&(id as u64)) {
                if mf.closed {
                    raise_value_error("I/O operation on closed file");
                    return MbValue::none();
                }
                if let Some(ref mut writer) = mf.writer {
                    match writer.write_all(&content) {
                        Ok(()) => {
                            let _ = writer.flush();
                            return MbValue::from_int(content.len() as i64);
                        }
                        Err(_) => return MbValue::none(),
                    }
                }
            }
            MbValue::none()
        })
    } else {
        MbValue::none()
    }
}

/// file.writelines(lines) → None
/// Writes each element of the iterable to the file (no separator added).
pub fn mb_file_writelines(handle: MbValue, lines: MbValue) -> MbValue {
    if let Some(items_ptr) = lines.as_ptr() {
        let items: Vec<MbValue> = unsafe {
            if let super::rc::ObjData::List(ref lock) = (*items_ptr).data {
                lock.read().unwrap().to_vec()
            } else {
                Vec::new()
            }
        };
        for item in items {
            mb_file_write(handle, item);
        }
    }
    MbValue::none()
}

/// file.close()
pub fn mb_file_close(handle: MbValue) {
    if let Some(id) = handle.as_int() {
        FILES.with(|files| {
            let mut files = files.borrow_mut();
            if let Some(mf) = files.get_mut(&(id as u64)) {
                mf.closed = true;
                mf.reader = None;
                mf.writer = None;
            }
        });
    }
}

fn raise_type_error(msg: &str) {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
}

fn raise_value_error(msg: &str) {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
}

fn raise_file_not_found(path: &str) {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("FileNotFoundError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!(
            "No such file or directory: '{path}'"
        ))),
    );
}

fn raise_os_error(msg: &str) {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("OSError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
}

// ── Cleanup ──

/// Reset all file I/O thread_local state to defaults.
/// Drains the FILES HashMap, dropping MbFile handles to close fds.
/// Called as part of centralized runtime cleanup between test executions.
pub(crate) fn cleanup_all_files() {
    let _ = FILES.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = NEXT_FILE_ID.with(|c| c.set(1));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_and_read() {
        let tmp = std::env::temp_dir().join("mamba_test_file_io.txt");
        let path_str = tmp.to_string_lossy().to_string();
        let path = MbValue::from_ptr(MbObject::new_str(path_str.clone()));
        let mode_w = MbValue::from_ptr(MbObject::new_str("w".to_string()));

        // Write
        let fh = mb_open(path, mode_w);
        assert!(fh.as_int().is_some());
        let text = MbValue::from_ptr(MbObject::new_str("hello\nworld\n".to_string()));
        let written = mb_file_write(fh, text);
        assert_eq!(written.as_int(), Some(12));
        mb_file_close(fh);

        // Read
        let path2 = MbValue::from_ptr(MbObject::new_str(path_str.clone()));
        let mode_r = MbValue::from_ptr(MbObject::new_str("r".to_string()));
        let fh2 = mb_open(path2, mode_r);
        let content = mb_file_read(fh2);
        assert!(content.is_ptr());
        mb_file_close(fh2);

        // Cleanup
        let _ = std::fs::remove_file(&path_str);
    }

    // ── Cleanup tests (R1, S3: per-module cleanup for files) ──

    #[test]
    fn test_cleanup_all_files_closes_handles() {
        let tmp = std::env::temp_dir().join("mamba_cleanup_file_test.txt");
        let path_str = tmp.to_string_lossy().to_string();

        // Open a file for writing
        let path = MbValue::from_ptr(MbObject::new_str(path_str.clone()));
        let mode = MbValue::from_ptr(MbObject::new_str("w".to_string()));
        let fh = mb_open(path, mode);
        assert!(fh.as_int().is_some(), "should get a valid file handle");

        let fh_id = fh.as_int().unwrap() as u64;
        assert!(
            is_file_handle(fh_id),
            "file should be in FILES before cleanup"
        );

        cleanup_all_files();

        assert!(
            !is_file_handle(fh_id),
            "FILES should be empty after cleanup — file handles dropped"
        );

        // Cleanup temp file
        let _ = std::fs::remove_file(&path_str);
    }

    #[test]
    fn test_cleanup_all_files_resets_id_counter() {
        let tmp = std::env::temp_dir().join("mamba_cleanup_id_test.txt");
        let path_str = tmp.to_string_lossy().to_string();

        let path = MbValue::from_ptr(MbObject::new_str(path_str.clone()));
        let mode = MbValue::from_ptr(MbObject::new_str("w".to_string()));
        let fh1 = mb_open(path, mode);
        mb_file_close(fh1);

        cleanup_all_files();

        // After cleanup, next file should get ID 1 again
        let path2 = MbValue::from_ptr(MbObject::new_str(path_str.clone()));
        let mode2 = MbValue::from_ptr(MbObject::new_str("w".to_string()));
        let fh2 = mb_open(path2, mode2);
        assert_eq!(
            fh1.as_int(),
            fh2.as_int(),
            "file ID counter should reset after cleanup"
        );
        mb_file_close(fh2);

        let _ = std::fs::remove_file(&path_str);
    }

    #[test]
    fn test_cleanup_all_files_on_empty() {
        cleanup_all_files();
        // No panic = success
    }

    #[test]
    fn test_read_after_close_raises_value_error() {
        super::super::exception::mb_clear_exception();
        let tmp = std::env::temp_dir().join("mamba_test_read_after_close.txt");
        let path_str = tmp.to_string_lossy().to_string();
        let path = MbValue::from_ptr(MbObject::new_str(path_str.clone()));
        let mode = MbValue::from_ptr(MbObject::new_str("w".to_string()));
        let fh = mb_open(path, mode);
        let text = MbValue::from_ptr(MbObject::new_str("data".to_string()));
        mb_file_write(fh, text);
        mb_file_close(fh);

        let path2 = MbValue::from_ptr(MbObject::new_str(path_str.clone()));
        let mode_r = MbValue::from_ptr(MbObject::new_str("r".to_string()));
        let fh2 = mb_open(path2, mode_r);
        mb_file_close(fh2);
        let _result = mb_file_read(fh2);
        assert_eq!(
            super::super::exception::mb_has_exception().as_bool(),
            Some(true),
            "ValueError must be pending after read on closed file",
        );
        let exc = super::super::exception::mb_get_exception();
        let exc_type = super::super::exception::get_exception_type_pub(exc);
        assert_eq!(exc_type.as_deref(), Some("ValueError"));
        super::super::exception::mb_clear_exception();
        let _ = std::fs::remove_file(&path_str);
    }

    #[test]
    fn test_readline_and_readlines() {
        let tmp = std::env::temp_dir().join("mamba_test_readline.txt");
        let path_str = tmp.to_string_lossy().to_string();

        // Write two lines
        let p = MbValue::from_ptr(MbObject::new_str(path_str.clone()));
        let fh = mb_open(p, MbValue::from_ptr(MbObject::new_str("w".into())));
        mb_file_write(
            fh,
            MbValue::from_ptr(MbObject::new_str("aaa\nbbb\n".into())),
        );
        mb_file_close(fh);

        // readline returns first line
        let p2 = MbValue::from_ptr(MbObject::new_str(path_str.clone()));
        let fh2 = mb_open(p2, MbValue::from_ptr(MbObject::new_str("r".into())));
        let line = mb_file_readline(fh2);
        assert!(line.is_ptr(), "readline should return a string");
        mb_file_close(fh2);

        // readlines returns a list
        let p3 = MbValue::from_ptr(MbObject::new_str(path_str.clone()));
        let fh3 = mb_open(p3, MbValue::from_ptr(MbObject::new_str("r".into())));
        let lines = mb_file_readlines(fh3);
        assert!(lines.is_ptr(), "readlines should return a list");
        mb_file_close(fh3);

        let _ = std::fs::remove_file(&path_str);
    }

    // ── Method dispatch tests (REQ: file handle method dispatch on integer IDs) ──

    /// Verify that mb_call_method dispatches write/read/close via the file
    /// handle path (the bug fix: integers that are file handles must not fall
    /// through to the generic "int has no attribute" error).
    #[test]
    fn test_method_dispatch_write_and_read() {
        let tmp = std::env::temp_dir().join("mamba_dispatch_write_read.txt");
        let path_str = tmp.to_string_lossy().to_string();

        // --- write via mb_call_method ---
        let fh_w = mb_open(
            MbValue::from_ptr(MbObject::new_str(path_str.clone())),
            MbValue::from_ptr(MbObject::new_str("w".into())),
        );
        assert!(
            fh_w.as_int().is_some(),
            "open('w') should return an int handle"
        );

        let method_write = MbValue::from_ptr(MbObject::new_str("write".into()));
        let text_val = MbValue::from_ptr(MbObject::new_str("dispatch test\n".into()));
        let args_list = MbValue::from_ptr(MbObject::new_list(vec![text_val]));
        let written = super::super::class::mb_call_method(fh_w, method_write, args_list);
        assert!(
            written.as_int().is_some(),
            "write via dispatch should return byte count"
        );

        let method_close = MbValue::from_ptr(MbObject::new_str("close".into()));
        super::super::class::mb_call_method(fh_w, method_close, MbValue::none());

        // --- read via mb_call_method ---
        let fh_r = mb_open(
            MbValue::from_ptr(MbObject::new_str(path_str.clone())),
            MbValue::from_ptr(MbObject::new_str("r".into())),
        );
        let method_read = MbValue::from_ptr(MbObject::new_str("read".into()));
        let content = super::super::class::mb_call_method(fh_r, method_read, MbValue::none());
        assert!(content.is_ptr(), "read via dispatch should return a string");

        let method_close2 = MbValue::from_ptr(MbObject::new_str("close".into()));
        super::super::class::mb_call_method(fh_r, method_close2, MbValue::none());

        let _ = std::fs::remove_file(&path_str);
    }

    /// Verify that writelines dispatches correctly through mb_call_method.
    #[test]
    fn test_method_dispatch_writelines() {
        let tmp = std::env::temp_dir().join("mamba_dispatch_writelines.txt");
        let path_str = tmp.to_string_lossy().to_string();

        let fh = mb_open(
            MbValue::from_ptr(MbObject::new_str(path_str.clone())),
            MbValue::from_ptr(MbObject::new_str("w".into())),
        );

        let line1 = MbValue::from_ptr(MbObject::new_str("line one\n".into()));
        let line2 = MbValue::from_ptr(MbObject::new_str("line two\n".into()));
        let lines_list = MbValue::from_ptr(MbObject::new_list(vec![line1, line2]));
        let method_wl = MbValue::from_ptr(MbObject::new_str("writelines".into()));
        let args_list = MbValue::from_ptr(MbObject::new_list(vec![lines_list]));
        super::super::class::mb_call_method(fh, method_wl, args_list);
        mb_file_close(fh);

        // Verify the file contents
        let contents = std::fs::read_to_string(&path_str).unwrap_or_default();
        assert!(
            contents.contains("line one"),
            "writelines should write first line"
        );
        assert!(
            contents.contains("line two"),
            "writelines should write second line"
        );

        let _ = std::fs::remove_file(&path_str);
    }

    /// Verify that readline dispatches correctly through mb_call_method.
    #[test]
    fn test_method_dispatch_readline_readlines() {
        let tmp = std::env::temp_dir().join("mamba_dispatch_readline.txt");
        let path_str = tmp.to_string_lossy().to_string();

        // Prepare file
        let fh_w = mb_open(
            MbValue::from_ptr(MbObject::new_str(path_str.clone())),
            MbValue::from_ptr(MbObject::new_str("w".into())),
        );
        mb_file_write(fh_w, MbValue::from_ptr(MbObject::new_str("a\nb\n".into())));
        mb_file_close(fh_w);

        // readline via dispatch
        let fh_r = mb_open(
            MbValue::from_ptr(MbObject::new_str(path_str.clone())),
            MbValue::from_ptr(MbObject::new_str("r".into())),
        );
        let method_rl = MbValue::from_ptr(MbObject::new_str("readline".into()));
        let line = super::super::class::mb_call_method(fh_r, method_rl, MbValue::none());
        assert!(
            line.is_ptr(),
            "readline via dispatch should return a string"
        );
        mb_file_close(fh_r);

        // readlines via dispatch
        let fh_r2 = mb_open(
            MbValue::from_ptr(MbObject::new_str(path_str.clone())),
            MbValue::from_ptr(MbObject::new_str("r".into())),
        );
        let method_rls = MbValue::from_ptr(MbObject::new_str("readlines".into()));
        let lines = super::super::class::mb_call_method(fh_r2, method_rls, MbValue::none());
        assert!(
            lines.is_ptr(),
            "readlines via dispatch should return a list"
        );
        mb_file_close(fh_r2);

        let _ = std::fs::remove_file(&path_str);
    }
}
