// @generated
impl serde::Serialize for Test {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.test_field.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("ogham.Test", len)?;
        if !self.test_field.is_empty() {
            struct_ser.serialize_field("testField", &self.test_field)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Test {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "test_field",
            "testField",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TestField,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "testField" | "test_field" => Ok(GeneratedField::TestField),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Test;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ogham.Test")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Test, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut test_field__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TestField => {
                            if test_field__.is_some() {
                                return Err(serde::de::Error::duplicate_field("testField"));
                            }
                            test_field__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Test {
                    test_field: test_field__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ogham.Test", FIELDS, GeneratedVisitor)
    }
}
