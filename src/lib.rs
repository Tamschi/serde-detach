use serde::de;
use std::marker::PhantomData;
use wyz::Pipe as _;

pub fn detach<T, E>(detach: Result<Detach<T>, E>) -> Result<T, E> {
    detach.map(|detach| detach.0)
}

#[derive(Debug)]
pub struct Detach<T>(T);

impl<'de, T: de::Deserialize<'static>> de::Deserialize<'de> for Detach<T> {
    fn deserialize<D>(
        deserializer: D,
    ) -> std::result::Result<Self, <D as serde::de::Deserializer<'de>>::Error>
    where
        D: de::Deserializer<'de>,
    {
        T::deserialize(Deserializer(deserializer, PhantomData)).map(Detach)
    }
}

struct Deserializer<'de, D: de::Deserializer<'de>>(D, PhantomData<&'de ()>);
impl<'de, D: de::Deserializer<'de>> Deserializer<'de, D> {
    fn new(deserializer: D) -> Self {
        Self(deserializer, PhantomData)
    }
}

macro_rules! deserialize {
    ($($deserialize_:ident$(($($param:ident: $param_type:ty),*$(,)?))?),*$(,)?) => {
        $(
            fn $deserialize_<V>(self, $($($param: $param_type, )*)?visitor: V) -> Result<V::Value, Self::Error>
            where
                V: de::Visitor<'static>
            {
                self.0 .$deserialize_($($($param, )?)*Visitor(visitor))
            }
        )*
    };
}

impl<'de, D: de::Deserializer<'de>> de::Deserializer<'static> for Deserializer<'de, D> {
    type Error = D::Error;

    deserialize! {
        deserialize_any,
        deserialize_bool,

        deserialize_i8,
        deserialize_i16,
        deserialize_i32,
        deserialize_i64,
        deserialize_i128,

        deserialize_u8,
        deserialize_u16,
        deserialize_u32,
        deserialize_u64,
        deserialize_u128,

        deserialize_f32,
        deserialize_f64,

        deserialize_char,

        deserialize_str,
        deserialize_string,

        deserialize_bytes,
        deserialize_byte_buf,

        deserialize_option,
        deserialize_unit,
        deserialize_unit_struct(name: &'static str),
        deserialize_newtype_struct(name: &'static str),
        deserialize_seq,
        deserialize_tuple(len: usize),
        deserialize_tuple_struct(name: &'static str, len: usize),
        deserialize_map,
        deserialize_struct(name: &'static str, fields: &'static [&'static str]),
        deserialize_enum(name: &'static str, variants: &'static [&'static str]),
        deserialize_identifier,
        deserialize_ignored_any,
    }

    fn is_human_readable(&self) -> bool {
        self.0.is_human_readable()
    }
}

struct Visitor<V: de::Visitor<'static>>(V);

macro_rules! visit {
    ($($visit_:ident(
        $($ty:ty
            $( | $($expr:expr);+$(;)?)?
        )?
        ) $(/ ::$Error:ident where T: $t_path:path)?),*$(,)?) => {
        $(
            fn $visit_<T>(self$(, v: $ty)?) -> Result<Self::Value, T$(::$Error)?>
            where
                T: $($t_path, T::Error: )?de::Error,
            {
                self.0 .$visit_($({let _: $ty; v$($(.pipe($expr))+)?})?)
            }
        )*
    };
}

impl<'de, V: de::Visitor<'static>> de::Visitor<'de> for Visitor<V> {
    type Value = V::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.expecting(formatter)
    }

    visit! {
        visit_bool(bool),

        visit_i8(i8),
        visit_i16(i16),
        visit_i32(i32),
        visit_i64(i64),
        visit_i128(i128),

        visit_u8(u8),
        visit_u16(u16),
        visit_u32(u32),
        visit_u64(u64),
        visit_u128(u128),

        visit_f32(f32),
        visit_f64(f64),

        visit_char(char),

        visit_str(&str),
        // visit_borrowed_str not implemented! Default implementation forwards to visit_str ✨
        visit_string(String),

        visit_bytes(&[u8]),
        // visit_borrowed_bytes's default implementation forwards to visit_bytes.
        visit_byte_buf(Vec<u8>),

        visit_none(),
        visit_some(T | Deserializer::new) / ::Error where T: de::Deserializer<'de>,

        visit_unit(),
        visit_newtype_struct(T | Deserializer::new) / ::Error where T: de::Deserializer<'de>,
        visit_seq(T | SeqAccess::new) / ::Error where T: de::SeqAccess<'de>,
        visit_map(T | MapAccess::new) / ::Error where T: de::MapAccess<'de>,
        visit_enum(T | EnumAccess::new) / ::Error where T: de::EnumAccess<'de>,
    }
}

struct SeqAccess<'de, A: de::SeqAccess<'de>>(A, PhantomData<&'de ()>);
impl<'de, A: de::SeqAccess<'de>> SeqAccess<'de, A> {
    fn new(access: A) -> Self {
        Self(access, PhantomData)
    }
}
impl<'de, A: de::SeqAccess<'de>> de::SeqAccess<'static> for SeqAccess<'de, A> {
    type Error = A::Error;
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'static>,
    {
        self.0.next_element_seed(Seed(seed))
    }
    fn size_hint(&self) -> Option<usize> {
        self.0.size_hint()
    }
}

struct MapAccess<'de, A: de::MapAccess<'de>>(A, PhantomData<&'de ()>);
impl<'de, A: de::MapAccess<'de>> MapAccess<'de, A> {
    fn new(access: A) -> Self {
        Self(access, PhantomData)
    }
}
impl<'de, A: de::MapAccess<'de>> de::MapAccess<'static> for MapAccess<'de, A> {
    type Error = A::Error;
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'static>,
    {
        self.0.next_key_seed(Seed(seed))
    }
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'static>,
    {
        self.0.next_value_seed(Seed(seed))
    }
    #[allow(clippy::type_complexity)]
    fn next_entry_seed<K, V>(
        &mut self,
        kseed: K,
        vseed: V,
    ) -> Result<Option<(K::Value, V::Value)>, Self::Error>
    where
        K: de::DeserializeSeed<'static>,
        V: de::DeserializeSeed<'static>,
    {
        self.0.next_entry_seed(Seed(kseed), Seed(vseed))
    }
    fn size_hint(&self) -> Option<usize> {
        self.0.size_hint()
    }
}

struct EnumAccess<'de, A: de::EnumAccess<'de>>(A, PhantomData<&'de ()>);
impl<'de, A: de::EnumAccess<'de>> EnumAccess<'de, A> {
    fn new(access: A) -> Self {
        Self(access, PhantomData)
    }
}
impl<'de, A: de::EnumAccess<'de>> de::EnumAccess<'static> for EnumAccess<'de, A> {
    type Error = A::Error;
    type Variant = VariantAccess<'de, A::Variant>;
    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'static>,
    {
        self.0
            .variant_seed(Seed(seed))
            .map(|(value, variant)| (value, VariantAccess::new(variant)))
    }
}

struct VariantAccess<'de, A: de::VariantAccess<'de>>(A, PhantomData<&'de ()>);
impl<'de, A: de::VariantAccess<'de>> VariantAccess<'de, A> {
    fn new(access: A) -> Self {
        Self(access, PhantomData)
    }
}
impl<'de, A: de::VariantAccess<'de>> de::VariantAccess<'static> for VariantAccess<'de, A> {
    type Error = A::Error;
    fn unit_variant(self) -> Result<(), Self::Error> {
        self.0.unit_variant()
    }
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'static>,
    {
        self.0.newtype_variant_seed(Seed(seed))
    }
    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        self.0.tuple_variant(len, Visitor(visitor))
    }
    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        self.0.struct_variant(fields, Visitor(visitor))
    }
}

struct Seed<S: de::DeserializeSeed<'static>>(S);
impl<'de, S: de::DeserializeSeed<'static>> de::DeserializeSeed<'de> for Seed<S> {
    type Value = S::Value;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.0.deserialize(Deserializer::new(deserializer))
    }
}
