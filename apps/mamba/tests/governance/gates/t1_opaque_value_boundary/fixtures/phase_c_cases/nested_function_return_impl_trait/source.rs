fn nested_outer(value: MbValue) -> impl ResultTrait {
    fn nested_conversion(value: MbValue) -> impl Into<MbValue> {
        let raw = 7;
        MbValue::from_int(raw as i64)
    }
    nested_conversion(value)
}
