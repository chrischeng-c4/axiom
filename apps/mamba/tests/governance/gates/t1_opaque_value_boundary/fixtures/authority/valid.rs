struct MbValue(i64);

impl MbValue {
    fn from_int(value: i64) -> Self { Self(value) }
    fn as_int(self) -> Option<i64> { Some(self.0) }
}

fn produce() -> MbValue {
    MbValue::from_int(7)
}

fn consume(value: MbValue) -> i64 {
    value.as_int().unwrap()
}
