use super::super::stdlib::array_mod::is_array_handle;
use super::super::value::MbValue;

pub(crate) fn is_array_handle_value(v: MbValue) -> bool {
    v.as_int().is_some_and(|id| is_array_handle(id as u64))
}
