trait ConversionTrait {
    fn trait_conversion(value: MbValue) -> impl Into<MbValue> {
        let raw = 7;
        MbValue::from_int(raw as i64)
    }
}
