#[test]
fn hidden_iterator_handle(id: u64) -> MbValue {
    MbValue::from_int(id as i64)
}

fn visible_iterator_handle(id: u64) -> MbValue {
    MbValue::from_int(id as i64)
}
