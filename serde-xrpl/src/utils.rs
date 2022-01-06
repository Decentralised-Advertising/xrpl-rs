use std::str::FromStr;

use super::error::{Error, Result};
use bs58::Alphabet;
use rust_decimal::{prelude::ToPrimitive, Decimal, MathematicalOps};
use serde::{ser, Serialize};

pub fn encode_variable_length(length: usize) -> Vec<u8> {
    let mut len_bytes = [0u8; 3];
    if length <= 192 {
        return vec![length as u8];
    } else if length <= 12480 {
        let length_a = length - 193;
        len_bytes[0] = 193 + (length_a >> 8usize) as u8;
        len_bytes[1] = (length_a & 0xff) as u8;
        return vec![len_bytes[0], len_bytes[1]];
    } else if length <= 918744 {
        let length_a = length - 12481;
        len_bytes[0] = 241 + (length_a >> 16usize) as u8;
        len_bytes[1] = ((length_a >> 8isize) & 0xff) as u8;
        len_bytes[2] = (length_a & 0xff) as u8;
        return vec![len_bytes[0], len_bytes[1], len_bytes[2]];
    }
    panic!("overflow error");
}

pub fn encode_field_id(type_code: u8, field_code: u8) -> Vec<u8> {
    if type_code < 16 && field_code < 16 {
        let field_id = type_code << 4 | field_code;
        return vec![field_id as u8];
    } else if type_code < 16 && field_code >= 16 {
        let field_id = type_code << 4;
        return vec![field_id as u8, field_code as u8];
    } else if type_code >= 16 && field_code < 16 {
        return vec![field_code as u8, type_code as u8];
    }
    vec![0u8, type_code, field_code]
}

pub const XRPL_ALPHABET: Alphabet = *bs58::Alphabet::RIPPLE;

pub fn decode_base58(b58_string: &str, prefix: &[u8]) -> Result<Vec<u8>> {
    let prefix_len = prefix.len();
    let decoded = bs58::decode(b58_string)
        .with_alphabet(&XRPL_ALPHABET)
        .with_check(None)
        .into_vec()
        .map_err(|e| Error::InvalidAddress)?;
    if &decoded[..prefix_len] != prefix {
        Err(Error::InvalidAddress)
    } else {
        Ok(decoded[prefix_len..].to_vec())
    }
}

pub fn encode_currency_code(currency_code: &str) -> Vec<u8> {
    if currency_code.as_bytes().len() == 3 {
        return [
            [0u8; 12].to_vec(),
            currency_code.as_bytes().to_vec(),
            [0u8; 5].to_vec(),
        ]
        .concat();
    }
    if currency_code.as_bytes().len() == 20 {
        return currency_code.as_bytes().to_vec();
    }
    panic!("invalid currency code")
}

pub fn encode_issued_currency_amount(
    amount: &str,
    currency: &str,
    issuer: &str,
) -> Result<Vec<u8>> {
    let encoded_address = decode_base58(issuer, &[0x00])?;

    let mut decimal_amount = Decimal::from_str(amount)
        .map_err(|e| Error::InvalidIssuedCurrencyAmount(format!("{:?}", e)))?;

    let mut encoded_amount;

    if decimal_amount.is_zero() {
        encoded_amount = [0u8; 8];
        encoded_amount[0] |= 0x80;
    } else {
        // Rescale decimal to normalise the mantisssa between 10e15 (1000000000000000) to 10e16-1 (9999999999999999) inclusive.
        let e = decimal_amount.log10().floor().to_u32().unwrap();
        decimal_amount.rescale(15 - e);
        encoded_amount = decimal_amount.mantissa().to_u64().unwrap().to_be_bytes();
        encoded_amount[0] |= 0x80;
        if decimal_amount.is_sign_positive() {
            encoded_amount[0] |= 0x40;
        }
        let exponent = e as i32 - 15;
        let exponent_bytes = (97 + exponent).to_u8().unwrap();
        encoded_amount[0] |= exponent_bytes >> 2u8;
        encoded_amount[1] |= (exponent_bytes & 0x03) << 6u8;
    }

    let encoded_currency = encode_currency_code(currency);

    Ok([encoded_amount.to_vec(), encoded_currency, encoded_address]
        .concat()
        .to_vec())
}

#[derive(Default)]
pub struct StringSerializer {
    pub value: Option<String>,
}

impl<'a> ser::Serializer for &'a mut StringSerializer {
    // The output type produced by this `Serializer` during successful
    // serialization. Most serializers that produce text or binary output should
    // set `Ok = ()` and serialize into an `io::Write` or buffer contained
    // within the `Serializer` instance, as happens here. Serializers that build
    // in-memory data structures may be simplified by using `Ok` to propagate
    // the data structure around.
    type Ok = ();

    // The error type when some error occurs during serialization.
    type Error = Error;

    // Associated types for keeping track of additional state while serializing
    // compound data structures like sequences and maps. In this case no
    // additional state is required beyond what is already stored in the
    // Serializer struct.
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    // Here we go with the simple methods. The following 12 methods receive one
    // of the primitive types of the data model and map it to JSON by appending
    // into the output string.
    fn serialize_bool(self, v: bool) -> Result<()> {
        unimplemented!()
    }

    // JSON does not distinguish between different sizes of integers, so all
    // signed integers will be serialized the same and all unsigned integers
    // will be serialized the same. Other formats, especially compact binary
    // formats, may need independent logic for the different sizes.
    fn serialize_i8(self, v: i8) -> Result<()> {
        unimplemented!()
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        unimplemented!()
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        unimplemented!()
    }

    // Not particularly efficient but this is example code anyway. A more
    // performant approach would be to use the `itoa` crate.
    fn serialize_i64(self, v: i64) -> Result<()> {
        unimplemented!()
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        unimplemented!()
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        unimplemented!()
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        unimplemented!()
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        unimplemented!()
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        unimplemented!()
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        unimplemented!()
    }

    // Serialize a char as a single-character string. Other formats may
    // represent this differently.
    fn serialize_char(self, v: char) -> Result<()> {
        unimplemented!()
    }

    // This only works for strings that don't require escape sequences but you
    // get the idea. For example it would emit invalid JSON if the input string
    // contains a '"' character.
    fn serialize_str(self, v: &str) -> Result<()> {
        self.value = Some(v.to_owned());
        Ok(())
    }

    // Serialize a byte array as an array of bytes. Could also use a base64
    // string here. Binary formats will typically represent byte arrays more
    // compactly.
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        unimplemented!()
    }

    // An absent optional is represented as the JSON `null`.
    fn serialize_none(self) -> Result<()> {
        unimplemented!()
    }

    // A present optional is represented as just the contained value. Note that
    // this is a lossy representation. For example the values `Some(())` and
    // `None` both serialize as just `null`. Unfortunately this is typically
    // what people expect when working with JSON. Other formats are encouraged
    // to behave more intelligently if possible.
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    // In Serde, unit means an anonymous value containing no data. Map this to
    // JSON as `null`.
    fn serialize_unit(self) -> Result<()> {
        unimplemented!()
    }

    // Unit struct means a named value containing no data. Again, since there is
    // no data, map this to JSON as `null`. There is no need to serialize the
    // name in most formats.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        unimplemented!()
    }

    // When serializing a unit variant (or any other kind of variant), formats
    // can choose whether to keep track of it by index or by name. Binary
    // formats typically use the index of the variant and human-readable formats
    // typically use the name.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        unimplemented!()
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain.
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    // Note that newtype variant (and all of the other variant serialization
    // methods) refer exclusively to the "externally tagged" enum
    // representation.
    //
    // Serialize this to JSON in externally tagged form as `{ NAME: VALUE }`.
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    // Now we get to the serialization of compound types.
    //
    // The start of the sequence, each value, and the end are three separate
    // method calls. This one is responsible only for serializing the start,
    // which in JSON is `[`.
    //
    // The length of the sequence may or may not be known ahead of time. This
    // doesn't make a difference in JSON because the length is not represented
    // explicitly in the serialized form. Some serializers may only be able to
    // support sequences for which the length is known up front.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        unimplemented!()
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently by omitting the length, since tuple
    // means that the corresponding `Deserialize implementation will know the
    // length without needing to look at the serialized data.
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        unimplemented!()
    }

    // Tuple structs look just like sequences in JSON.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        unimplemented!()
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }`. Again
    // this method is only responsible for the externally tagged representation.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        unimplemented!()
    }

    // Maps are represented in JSON as `{ K: V, K: V, ... }`.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        unimplemented!()
    }

    // Structs look just like maps in JSON. In particular, JSON requires that we
    // serialize the field names of the struct. Other formats may be able to
    // omit the field names when serializing structs because the corresponding
    // Deserialize implementation is required to know what the keys are without
    // looking at the serialized data.
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        unimplemented!()
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }`.
    // This is the externally tagged representation.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        unimplemented!()
    }
}

// The following 7 impls deal with the serialization of compound types like
// sequences and maps. Serialization of such types is begun by a Serializer
// method and followed by zero or more calls to serialize individual elements of
// the compound type and one call to end the compound type.
//
// This impl is SerializeSeq so these methods are called after `serialize_seq`
// is called on the Serializer.
impl<'a> ser::SerializeSeq for &'a mut StringSerializer {
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    // Close the sequence.
    fn end(self) -> Result<()> {
        unimplemented!()
    }
}

// Same thing but for tuples.
impl<'a> ser::SerializeTuple for &'a mut StringSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<()> {
        unimplemented!()
    }
}

// Same thing but for tuple structs.
impl<'a> ser::SerializeTupleStruct for &'a mut StringSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<()> {
        unimplemented!()
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut StringSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<()> {
        unimplemented!()
    }
}

// Some `Serialize` types are not able to hold a key and value in memory at the
// same time so `SerializeMap` implementations are required to support
// `serialize_key` and `serialize_value` individually.
//
// There is a third optional method on the `SerializeMap` trait. The
// `serialize_entry` method allows serializers to optimize for the case where
// key and value are both available simultaneously. In JSON it doesn't make a
// difference so the default behavior for `serialize_entry` is fine.
impl<'a> ser::SerializeMap for &'a mut StringSerializer {
    type Ok = ();
    type Error = Error;

    // The Serde data model allows map keys to be any serializable type. JSON
    // only allows string keys so the implementation below will produce invalid
    // JSON if the key serializes as something other than a string.
    //
    // A real JSON serializer would need to validate that map keys are strings.
    // This can be done by using a different Serializer to serialize the key
    // (instead of `&mut **self`) and having that other serializer only
    // implement `serialize_str` and return an error on any other data type.
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<()> {
        unimplemented!()
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'a> ser::SerializeStruct for &'a mut StringSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<()> {
        unimplemented!()
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a> ser::SerializeStructVariant for &'a mut StringSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<()> {
        unimplemented!()
    }
}
