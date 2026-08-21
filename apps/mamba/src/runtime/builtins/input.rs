use crate::runtime::{
    rc::{MbObject, ObjData},
    value::MbValue,
};

/// input(prompt) — read a line from stdin.
pub fn mb_input(prompt: MbValue) -> MbValue {
    // Print prompt without newline
    if let Some(ptr) = prompt.as_ptr() {
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                eprint!("{s}");
            }
        }
    }
    let mut line = String::new();
    match std::io::stdin().read_line(&mut line) {
        Ok(_) => {
            // Strip trailing newline
            if line.ends_with('\n') {
                line.pop();
            }
            if line.ends_with('\r') {
                line.pop();
            }
            MbValue::from_ptr(MbObject::new_str(line))
        }
        Err(_) => MbValue::from_ptr(MbObject::new_str(String::new())),
    }
}
