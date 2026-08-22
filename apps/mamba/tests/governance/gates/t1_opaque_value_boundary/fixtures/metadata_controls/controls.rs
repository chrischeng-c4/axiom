fn metadata_uid(record: &Metadata) -> MbValue {
    MbValue::from_int(record.uid as i64)
}

fn ordinary_int(uid: i64) -> MbValue {
    MbValue::from_int(uid)
}

fn metadata_index(record: &Metadata) -> usize {
    record.index
}
