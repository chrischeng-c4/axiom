//! Limit diagnostics made while pricing untrusted WAL tokens. Successful
//! values are forwarded without a copy. JSON type mismatches reach the
//! bounded visitor through deserialize_any; CBOR keeps its native methods.
use serde::de::{
    self, DeserializeSeed, EnumAccess, Error as _, MapAccess, SeqAccess, VariantAccess, Visitor,
};
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::marker::PhantomData;

#[derive(Debug)]
struct QuietError;
impl fmt::Display for QuietError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid WAL wire value")
    }
}
impl std::error::Error for QuietError {}
impl de::Error for QuietError {
    fn custom<T: fmt::Display>(_: T) -> Self {
        Self
    }
}
fn quiet<E: de::Error, T>(result: Result<T, impl fmt::Display>) -> Result<T, E> {
    result.map_err(|_| E::custom(QuietError))
}

pub(super) struct Quiet<T>(pub T);
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Quiet<T> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let json = d.is_human_readable();
        T::deserialize(QuietDeserializer(d, json)).map(Self)
    }
}
struct QuietDeserializer<D>(D, bool);
struct QuietVisitor<V>(V, bool);
struct QuietSeed<S>(S, bool);
impl<'de, S: DeserializeSeed<'de>> DeserializeSeed<'de> for QuietSeed<S> {
    type Value = S::Value;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        self.0.deserialize(QuietDeserializer(d, self.1))
    }
}
impl<'de, V: Visitor<'de>> Visitor<'de> for QuietVisitor<V> {
    type Value = V::Value;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.expecting(f)
    }
    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
        quiet(self.0.visit_bool::<QuietError>(value))
    }
    fn visit_i8<E: de::Error>(self, value: i8) -> Result<Self::Value, E> {
        quiet(self.0.visit_i8::<QuietError>(value))
    }
    fn visit_i16<E: de::Error>(self, value: i16) -> Result<Self::Value, E> {
        quiet(self.0.visit_i16::<QuietError>(value))
    }
    fn visit_i32<E: de::Error>(self, value: i32) -> Result<Self::Value, E> {
        quiet(self.0.visit_i32::<QuietError>(value))
    }
    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
        quiet(self.0.visit_i64::<QuietError>(value))
    }
    fn visit_i128<E: de::Error>(self, value: i128) -> Result<Self::Value, E> {
        quiet(self.0.visit_i128::<QuietError>(value))
    }
    fn visit_u8<E: de::Error>(self, value: u8) -> Result<Self::Value, E> {
        quiet(self.0.visit_u8::<QuietError>(value))
    }
    fn visit_u16<E: de::Error>(self, value: u16) -> Result<Self::Value, E> {
        quiet(self.0.visit_u16::<QuietError>(value))
    }
    fn visit_u32<E: de::Error>(self, value: u32) -> Result<Self::Value, E> {
        quiet(self.0.visit_u32::<QuietError>(value))
    }
    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
        quiet(self.0.visit_u64::<QuietError>(value))
    }
    fn visit_u128<E: de::Error>(self, value: u128) -> Result<Self::Value, E> {
        quiet(self.0.visit_u128::<QuietError>(value))
    }
    fn visit_f32<E: de::Error>(self, value: f32) -> Result<Self::Value, E> {
        quiet(self.0.visit_f32::<QuietError>(value))
    }
    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
        quiet(self.0.visit_f64::<QuietError>(value))
    }
    fn visit_char<E: de::Error>(self, value: char) -> Result<Self::Value, E> {
        quiet(self.0.visit_char::<QuietError>(value))
    }
    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        quiet(self.0.visit_str::<QuietError>(value))
    }
    fn visit_borrowed_str<E: de::Error>(self, value: &'de str) -> Result<Self::Value, E> {
        quiet(self.0.visit_borrowed_str::<QuietError>(value))
    }
    fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
        quiet(self.0.visit_string::<QuietError>(value))
    }
    fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<Self::Value, E> {
        quiet(self.0.visit_bytes::<QuietError>(value))
    }
    fn visit_borrowed_bytes<E: de::Error>(self, value: &'de [u8]) -> Result<Self::Value, E> {
        quiet(self.0.visit_borrowed_bytes::<QuietError>(value))
    }
    fn visit_byte_buf<E: de::Error>(self, value: Vec<u8>) -> Result<Self::Value, E> {
        quiet(self.0.visit_byte_buf::<QuietError>(value))
    }
    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
        quiet(self.0.visit_none::<QuietError>())
    }
    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        quiet(self.0.visit_unit::<QuietError>())
    }
    fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        quiet(self.0.visit_some(QuietDeserializer(d, self.1)))
    }
    fn visit_newtype_struct<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        quiet(self.0.visit_newtype_struct(QuietDeserializer(d, self.1)))
    }
    fn visit_seq<A: SeqAccess<'de>>(self, a: A) -> Result<Self::Value, A::Error> {
        quiet(self.0.visit_seq(QuietSeq(a, self.1)))
    }
    fn visit_map<A: MapAccess<'de>>(self, a: A) -> Result<Self::Value, A::Error> {
        quiet(self.0.visit_map(QuietMap(a, self.1)))
    }
    fn visit_enum<A: EnumAccess<'de>>(self, a: A) -> Result<Self::Value, A::Error> {
        quiet(self.0.visit_enum(QuietEnum(a, self.1)))
    }
}
struct QuietSeq<A>(A, bool);
impl<'de, A: SeqAccess<'de>> SeqAccess<'de> for QuietSeq<A> {
    type Error = A::Error;
    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, A::Error> {
        quiet(self.0.next_element_seed(QuietSeed(seed, self.1)))
    }
    fn size_hint(&self) -> Option<usize> {
        self.0.size_hint()
    }
}
struct QuietMap<A>(A, bool);
impl<'de, A: MapAccess<'de>> MapAccess<'de> for QuietMap<A> {
    type Error = A::Error;
    fn next_key_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, A::Error> {
        quiet(self.0.next_key_seed(QuietSeed(seed, self.1)))
    }
    fn next_value_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<T::Value, A::Error> {
        quiet(self.0.next_value_seed(QuietSeed(seed, self.1)))
    }
    fn size_hint(&self) -> Option<usize> {
        self.0.size_hint()
    }
}
struct QuietEnum<A>(A, bool);
impl<'de, A: EnumAccess<'de>> EnumAccess<'de> for QuietEnum<A> {
    type Error = A::Error;
    type Variant = QuietVariant<A::Variant>;
    fn variant_seed<T: DeserializeSeed<'de>>(
        self,
        seed: T,
    ) -> Result<(T::Value, Self::Variant), A::Error> {
        let (value, variant) = quiet::<A::Error, _>(self.0.variant_seed(QuietSeed(seed, self.1)))?;
        Ok((value, QuietVariant(variant, self.1)))
    }
}
struct QuietVariant<A>(A, bool);
impl<'de, A: VariantAccess<'de>> VariantAccess<'de> for QuietVariant<A> {
    type Error = A::Error;
    fn unit_variant(self) -> Result<(), A::Error> {
        quiet(self.0.unit_variant())
    }
    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, A::Error> {
        quiet(self.0.newtype_variant_seed(QuietSeed(seed, self.1)))
    }
    fn tuple_variant<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value, A::Error> {
        quiet(self.0.tuple_variant(len, QuietVisitor(visitor, self.1)))
    }
    fn struct_variant<V: Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, A::Error> {
        quiet(self.0.struct_variant(fields, QuietVisitor(visitor, self.1)))
    }
}
// JSON enum decoding also goes through a visitor. The backend's normal
// VariantAccess would format an entire wrong-typed unit/tuple/struct payload.
struct JsonEnumVisitor<V>(V);
impl<'de, V: Visitor<'de>> Visitor<'de> for JsonEnumVisitor<V> {
    type Value = V::Value;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("an enum")
    }
    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        quiet(
            self.0
                .visit_enum(de::value::StrDeserializer::<QuietError>::new(value)),
        )
    }
    fn visit_borrowed_str<E: de::Error>(self, value: &'de str) -> Result<Self::Value, E> {
        quiet(
            self.0
                .visit_enum(de::value::BorrowedStrDeserializer::<QuietError>::new(value)),
        )
    }
    fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
        quiet(
            self.0
                .visit_enum(de::value::StringDeserializer::<QuietError>::new(value)),
        )
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let value = quiet::<A::Error, _>(self.0.visit_enum(JsonEnumMap(&mut map)))?;
        if map.next_key::<de::IgnoredAny>()?.is_some() {
            return Err(A::Error::custom(QuietError));
        }
        Ok(value)
    }
}
struct JsonEnumMap<'a, A>(&'a mut A);
struct JsonVariant<'a, A>(&'a mut A);
impl<'de, 'a, A: MapAccess<'de>> EnumAccess<'de> for JsonEnumMap<'a, A> {
    type Error = A::Error;
    type Variant = JsonVariant<'a, A>;
    fn variant_seed<T: DeserializeSeed<'de>>(
        self,
        seed: T,
    ) -> Result<(T::Value, Self::Variant), A::Error> {
        let key = self
            .0
            .next_key_seed(QuietSeed(seed, true))?
            .ok_or_else(|| A::Error::custom(QuietError))?;
        Ok((key, JsonVariant(self.0)))
    }
}
struct JsonVisitorSeed<V>(V);
impl<'de, V: Visitor<'de>> DeserializeSeed<'de> for JsonVisitorSeed<V> {
    type Value = V::Value;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        d.deserialize_any(QuietVisitor(self.0, true))
    }
}
impl<'de, A: MapAccess<'de>> VariantAccess<'de> for JsonVariant<'_, A> {
    type Error = A::Error;
    fn unit_variant(self) -> Result<(), A::Error> {
        self.0.next_value_seed(QuietSeed(PhantomData::<()>, true))
    }
    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, A::Error> {
        self.0.next_value_seed(QuietSeed(seed, true))
    }
    fn tuple_variant<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value, A::Error> {
        self.0.next_value_seed(JsonVisitorSeed(visitor))
    }
    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, A::Error> {
        self.0.next_value_seed(JsonVisitorSeed(visitor))
    }
}
impl<'de, D: Deserializer<'de>> Deserializer<'de> for QuietDeserializer<D> {
    type Error = D::Error;
    fn is_human_readable(&self) -> bool {
        self.1
    }
    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_bool(QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_i8(QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_i16(QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_i32(QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_i64(QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_i128(QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_u8(QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_u16(QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_u32(QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_u64(QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_u128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_u128(QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_f32(QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_f64(QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_char(QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_str(QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_string(QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_bytes(QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_byte_buf(QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_unit(QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_seq(QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_map(QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_identifier(QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(
                self.0
                    .deserialize_unit_struct(name, QuietVisitor(visitor, false)),
            )
        }
    }
    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(self.0.deserialize_tuple(len, QuietVisitor(visitor, false)))
        }
    }
    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(
                self.0
                    .deserialize_tuple_struct(name, len, QuietVisitor(visitor, false)),
            )
        }
    }
    fn deserialize_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(QuietVisitor(visitor, true)))
        } else {
            quiet(
                self.0
                    .deserialize_struct(name, fields, QuietVisitor(visitor, false)),
            )
        }
    }
    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, D::Error> {
        if self.1 {
            quiet(self.0.deserialize_any(JsonEnumVisitor(visitor)))
        } else {
            quiet(
                self.0
                    .deserialize_enum(name, variants, QuietVisitor(visitor, false)),
            )
        }
    }
    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        quiet(self.0.deserialize_option(QuietVisitor(visitor, self.1)))
    }
    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, D::Error> {
        quiet(
            self.0
                .deserialize_newtype_struct(name, QuietVisitor(visitor, self.1)),
        )
    }
    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        quiet(
            self.0
                .deserialize_ignored_any(QuietVisitor(visitor, self.1)),
        )
    }
}
