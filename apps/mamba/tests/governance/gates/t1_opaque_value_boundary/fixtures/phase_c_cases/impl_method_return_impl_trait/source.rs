struct ImplHolder;

impl ImplHolder {
    fn impl_conversion(value: MbValue) -> impl Into<MbValue> {
        let raw = 7;
        MbValue::from_int(raw as i64)
    }
}
