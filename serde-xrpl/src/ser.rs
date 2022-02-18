use crate::definitions::is_signing_field;
use crate::hash_prefixes;

use super::definitions::{get_field_code_and_type_code, get_transaction_type, is_serialized_field};
use super::error::{Error, Result};
use super::types::{Amount, Blob, Hash256, Value, Vector256};
use super::utils::{
    decode_base58, encode_field_id, encode_issued_currency_amount, encode_variable_length,
    StringSerializer,
};
use serde::{ser, Serialize};

#[derive(Default)]
pub struct SerializerOptions {
    pub prefix: Option<Vec<u8>>,
    pub suffix: Option<Vec<u8>>,
    pub signing_fields_only: bool,
}

#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub struct FieldHeader {
    type_code: u8,
    field_code: u8,
    sub_type: Option<SubType>,
}

#[derive(PartialEq, PartialOrd, Clone, Debug)]
enum SubType {
    IssuedCurrency {
        current_key: String,
        value: Option<String>,
        currency: Option<String>,
        issuer: Option<String>,
    },
}

impl FieldHeader {
    fn to_bytes(&self) -> Vec<u8> {
        encode_field_id(self.type_code, self.field_code)
    }
}

#[derive(Default)]
pub struct Serializer {
    options: SerializerOptions,
    sequence: usize,
    field: Option<(FieldHeader, Value)>,
    fields: Vec<(FieldHeader, Value)>,
    output: Vec<u8>,
}

pub fn to_bytes_with_opts<T>(value: &T, opts: Option<SerializerOptions>) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut serializer = Serializer::default();
    serializer.options = opts.unwrap_or_default();
    if let Some(prefix) = &serializer.options.prefix {
        serializer.output.append(&mut prefix.clone());
    }
    value.serialize(&mut serializer)?;
    serializer
        .fields
        .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    for (header, value) in &mut serializer.fields {
        serializer
            .output
            .append(&mut [header.to_bytes(), value.to_bytes()?.clone()].concat());
    }
    if let Some(suffix) = &serializer.options.suffix {
        serializer.output.append(&mut suffix.clone());
    }
    Ok(serializer.output)
}

pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    to_bytes_with_opts(value, None)
}

pub fn to_bytes_for_signing<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    to_bytes_with_opts(
        value,
        Some(SerializerOptions {
            prefix: Some(hash_prefixes::TRANSACTION_SIG.to_vec()),
            signing_fields_only: true,
            suffix: None,
        }),
    )
}

pub fn to_bytes_for_claim<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    to_bytes_with_opts(
        value,
        Some(SerializerOptions {
            prefix: Some(hash_prefixes::PAYMENT_CHANNEL_CLAIM.to_vec()),
            signing_fields_only: true,
            suffix: None,
        }),
    )
}

impl<'a> ser::Serializer for &'a mut Serializer {
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
        Ok(())
    }

    // JSON does not distinguish between different sizes of integers, so all
    // signed integers will be serialized the same and all unsigned integers
    // will be serialized the same. Other formats, especially compact binary
    // formats, may need independent logic for the different sizes.
    fn serialize_i8(self, v: i8) -> Result<()> {
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        Ok(())
    }

    // Not particularly efficient but this is example code anyway. A more
    // performant approach would be to use the `itoa` crate.
    fn serialize_i64(self, v: i64) -> Result<()> {
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        if let Some((field, value)) = &mut self.field {
            match field.type_code {
                16 => {
                    *value = Value::UInt8(v as u8);
                }
                1 => {
                    *value = Value::UInt16(v as u16);
                }
                2 => {
                    *value = Value::UInt32(v);
                }
                4 => {
                    *value = Value::UInt64(v as u64);
                }
                _ => unimplemented!(),
            };
            self.fields.push((field.clone(), value.clone()));
            self.field = None;
        }
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        if let Some((field, value)) = &mut self.field {
            match field.type_code {
                16 => {
                    *value = Value::UInt8(v as u8);
                }
                1 => {
                    *value = Value::UInt16(v as u16);
                }
                2 => {
                    *value = Value::UInt32(v as u32);
                }
                4 => {
                    *value = Value::UInt64(v);
                }
                _ => unimplemented!(),
            };
            self.fields.push((field.clone(), value.clone()));
            self.field = None;
        }
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        Ok(())
    }

    // Serialize a char as a single-character string. Other formats may
    // represent this differently.
    fn serialize_char(self, v: char) -> Result<()> {
        Ok(())
    }

    // This only works for strings that don't require escape sequences but you
    // get the idea. For example it would emit invalid JSON if the input string
    // contains a '"' character.
    fn serialize_str(self, v: &str) -> Result<()> {
        if let Some((field, data)) = &mut self.field {
            match field.type_code {
                19 => {
                    match data {
                        Value::Vector256(vec) => {
                            vec.0.push(Hash256(v.to_owned()));
                            self.sequence -= 1;
                        }
                        _ => {
                            *data = Value::Vector256(Vector256(vec![Hash256(v.to_owned())]));
                            self.sequence -= 1;
                        }
                    };
                    if self.sequence != 0 {
                        return Ok(());
                    }
                }
                8 => {
                    *data = Value::AccountID(v.to_owned());
                }
                7 => *data = Value::Blob(Blob(v.to_owned())),
                6 => match &mut field.sub_type {
                    None => {
                        let mut i: u64 = v
                            .parse()
                            .map_err(|e| Error::InvalidAmount(e, v.to_owned()))?;
                        *data = Value::Amount(Amount::XRP(i))
                    }
                    Some(SubType::IssuedCurrency {
                        value,
                        currency,
                        issuer,
                        current_key,
                    }) => {
                        match current_key.as_str() {
                            "value" => {
                                *value = Some(v.to_owned());
                            }
                            "currency" => {
                                *currency = Some(v.to_owned());
                            }
                            "issuer" => {
                                *issuer = Some(v.to_owned());
                            }
                            _ => {}
                        };
                        if value.is_some() && currency.is_some() && issuer.is_some() {
                            *data = Value::Amount(Amount::IssuedCurrency {
                                value: value.as_ref().unwrap().to_owned(),
                                currency: currency.as_ref().unwrap().to_owned(),
                                issuer: issuer.as_ref().unwrap().to_owned(),
                            })
                        } else {
                            return Ok(());
                        }
                    }
                },
                5 => *data = Value::Hash256(Hash256(v.to_owned())),
                1 => {
                    let i = get_transaction_type(v)?;
                    *data = Value::Transaction(i as u16)
                }
                3 => {
                    let i: u64 = v
                        .parse()
                        .map_err(|e| Error::InvalidAmount(e, v.to_owned()))?;
                    *data = Value::UInt64(i);
                }
                _ => unimplemented!("header: {:?}, value: {:?}", field, v),
            };
            self.fields.push((field.clone(), data.clone()));
            self.field = None;
        }
        Ok(())
    }

    // Serialize a byte array as an array of bytes. Could also use a base64
    // string here. Binary formats will typically represent byte arrays more
    // compactly.
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        // use serde::ser::SerializeSeq;
        // let mut seq = self.serialize_seq(Some(v.len()))?;
        // for byte in v {
        //     seq.serialize_element(byte)?;
        // }
        // seq.end()
        Ok(())
    }

    // An absent optional is represented as the JSON `null`.
    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
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
        value.serialize(self)
    }

    // In Serde, unit means an anonymous value containing no data. Map this to
    // JSON as `null`.
    fn serialize_unit(self) -> Result<()> {
        // self.output += "null";
        Ok(())
    }

    // Unit struct means a named value containing no data. Again, since there is
    // no data, map this to JSON as `null`. There is no need to serialize the
    // name in most formats.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
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
        // self.serialize_str(variant)
        Ok(())
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain.
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
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
        // self.output.append(&mut "{".as_bytes().to_vec());
        // variant.serialize(&mut *self)?;
        // self.output.append(&mut ":".as_bytes().to_vec());
        // value.serialize(&mut *self)?;
        // self.output.append(&mut "}".as_bytes().to_vec());
        Ok(())
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
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.sequence = len.unwrap_or_default();
        Ok(self)
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently by omitting the length, since tuple
    // means that the corresponding `Deserialize implementation will know the
    // length without needing to look at the serialized data.
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    // Tuple structs look just like sequences in JSON.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
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
        // self.output += "{";
        // variant.serialize(&mut *self)?;
        // self.output += ":[";
        Ok(self)
    }

    // Maps are represented in JSON as `{ K: V, K: V, ... }`.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(self)
    }

    // Structs look just like maps in JSON. In particular, JSON requires that we
    // serialize the field names of the struct. Other formats may be able to
    // omit the field names when serializing structs because the corresponding
    // Deserialize implementation is required to know what the keys are without
    // looking at the serialized data.
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
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
        // self.output += "{";
        // variant.serialize(&mut *self)?;
        // self.output += ":{";
        Ok(self)
    }
}

// The following 7 impls deal with the serialization of compound types like
// sequences and maps. Serialization of such types is begun by a Serializer
// method and followed by zero or more calls to serialize individual elements of
// the compound type and one call to end the compound type.
//
// This impl is SerializeSeq so these methods are called after `serialize_seq`
// is called on the Serializer.
impl<'a> ser::SerializeSeq for &'a mut Serializer {
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        // if !self.output.ends_with('[') {
        //     self.output += ",";
        // }
        value.serialize(&mut **self)
    }

    // Close the sequence.
    fn end(self) -> Result<()> {
        // self.output += "]";
        Ok(())
    }
}

// Same thing but for tuples.
impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        // if !self.output.ends_with('[') {
        //     self.output += ",";
        // }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        // self.output += "]";
        Ok(())
    }
}

// Same thing but for tuple structs.
impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        // self.output += "]";
        Ok(())
    }
}

// Tuple variants are a little different. Refer back to the
// `serialize_tuple_variant` method above:
//
//    self.output += "{";
//    variant.serialize(&mut *self)?;
//    self.output += ":[";
//
// So the `end` method in this impl is responsible for closing both the `]` and
// the `}`.
impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        // if !self.output.ends_with('[') {
        //     self.output += ",";
        // }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        // self.output += "]}";
        Ok(())
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
impl<'a> ser::SerializeMap for &'a mut Serializer {
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
        // Extract the string key for the map entry.
        let mut str_serializer = StringSerializer::default();
        key.serialize(&mut str_serializer)?;
        let key_str = str_serializer.value.unwrap();
        if let Some((header, _)) = &mut self.field {
            match header.type_code {
                6 => {
                    // Reentering the amount means we are handling an issued currency.
                    match &mut header.sub_type {
                        None => {
                            header.sub_type = Some(SubType::IssuedCurrency {
                                current_key: key_str.to_owned(),
                                value: None,
                                currency: None,
                                issuer: None,
                            });
                        }
                        Some(SubType::IssuedCurrency { current_key, .. }) => {
                            *current_key = key_str.to_owned();
                        }
                        _ => {}
                    }
                    return Ok(());
                }
                _ => {}
            }
        }
        if is_serialized_field(&key_str).unwrap_or_default() {
            if self.options.signing_fields_only && !is_signing_field(&key_str).unwrap_or_default() {
                return Ok(());
            }
            let (field_code, type_code) = get_field_code_and_type_code(&key_str)?;
            self.field = Some((
                FieldHeader {
                    type_code,
                    field_code,
                    sub_type: None,
                },
                Value::NotPresent,
            ));
        } else {
            self.field = None;
        }
        Ok(())
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if is_serialized_field(key).unwrap_or_default() {
            let (field_code, type_code) = get_field_code_and_type_code(key)?;
            self.field = Some((
                FieldHeader {
                    type_code,
                    field_code,
                    sub_type: None,
                },
                Value::NotPresent,
            ));
            println!("{:?}", self.field);
            return value.serialize(&mut **self);
        }
        Ok(())
    }

    fn end(self) -> Result<()> {
        if let Some((header, bytes)) = &self.field {
            self.fields.push((header.clone(), bytes.clone()));
        }
        Ok(())
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        // if !self.output.ends_with('{') {
        //     self.output += ",";
        // }
        // key.serialize(&mut **self)?;
        // self.output += ":";
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        // self.output += "}}";
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_example() {
    // let example_transaction = serde_json::json!({
    //   "Account": "rMBzp8CgpE441cp5PVyA9rpVV7oT8hP3ys",
    //   "Expiration": 595640108,
    //   "Fee": "10",
    //   "Flags": 524288,
    //   "OfferSequence": 1752791,
    //   "Sequence": 1752792,
    //   "SigningPubKey": "03EE83BB432547885C219634A1BC407A9DB0474145D69737D09CCDC63E1DEE7FE3",
    //   "TakerGets": "15000000000",
    //   "TakerPays": {
    //     "currency": "USD",
    //     "issuer": "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
    //     "value": "7072.8"
    //   },
    //   "TransactionType": "OfferCreate",
    //   "TxnSignature": "30440220143759437C04F7B61F012563AFE90D8DAFC46E86035E1D965A9CED282C97D4CE02204CFD241E86F17E011298FC1A39B63386C74306A5DE047E213B0F29EFA4571C2C",
    //   "hash": "73734B611DDA23D3F5F62E20A173B78AB8406AC5015094DA53F53D39B9EDB06C"
    // });
    let example_transaction = serde_json::json!({
        "Channel": "CB21BE824D6CF3DC085E7BDD2006ECB2D6B4D80BD6667B2CBE85B0544C49E5A3",
        "Amount": "15000000000",
      });
    let expected = hex_literal::hex!("120007220008000024001ABED82A2380BF2C2019001ABED764D55920AC9391400000000000000000000000000055534400000000000A20B3C85F482532A9578DBB3950B85CA06594D165400000037E11D60068400000000000000A732103EE83BB432547885C219634A1BC407A9DB0474145D69737D09CCDC63E1DEE7FE3744630440220143759437C04F7B61F012563AFE90D8DAFC46E86035E1D965A9CED282C97D4CE02204CFD241E86F17E011298FC1A39B63386C74306A5DE047E213B0F29EFA4571C2C8114DD76483FACDEE26E60D8A586BB58D09F27045C46");
    let output = to_bytes(&example_transaction).unwrap();
    println!("{}", hex::encode(output.clone()));
    assert_eq!(output, expected);
}

#[cfg(test)]
mod tests {
    use crate::ser::to_bytes;
    use serde::Deserialize;
    use serde_json::Value;
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct CodecFixtures {
        account_state: Vec<AccountState>,
    }
    #[derive(Deserialize)]
    struct AccountState {
        binary: String,
        json: Value,
    }
    // #[test]
    // fn test_codec_fixtures() {
    //     let codec_fixtures_bytes = include_bytes!("../test/fixtures/codec-fixtures.json");
    //     let codec_fixtures: CodecFixtures = serde_json::from_slice(codec_fixtures_bytes).unwrap();
    //     for fixture in codec_fixtures.account_state {
    //         let binary = to_bytes(&fixture.json).unwrap();
    //         assert_eq!(
    //             fixture.binary.to_lowercase(),
    //             hex::encode(binary).to_lowercase()
    //         );
    //     }
    // }
}
