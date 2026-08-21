use super::super::rc::ObjData;
use super::MbValue;

pub(crate) fn mb_str_value(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

pub(crate) fn mb_first_index_value(val: MbValue) -> Option<i64> {
    let ptr = val.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Tuple(items) => items.first().and_then(|v| super::resolve_index_value(*v)),
            ObjData::List(lock) => lock
                .read()
                .unwrap()
                .first()
                .and_then(|v| super::resolve_index_value(*v)),
            _ => None,
        }
    }
}
