mod inline_scope {
    fn inline_conversion(value: MbValue) -> impl Into<MbValue> {
        let raw = 7;
        MbValue::from_int(raw as i64)
    }
}
