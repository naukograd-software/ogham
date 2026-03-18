// @generated
impl serde::Serialize for AnnotationArgument {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        if self.value.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.AnnotationArgument", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AnnotationArgument {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "value",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Value,
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
                            "name" => Ok(GeneratedField::Name),
                            "value" => Ok(GeneratedField::Value),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AnnotationArgument;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.AnnotationArgument")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AnnotationArgument, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut value__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map_.next_value()?;
                        }
                    }
                }
                Ok(AnnotationArgument {
                    name: name__.unwrap_or_default(),
                    value: value__,
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.AnnotationArgument", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AnnotationCall {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.library.is_empty() {
            len += 1;
        }
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.arguments.is_empty() {
            len += 1;
        }
        if self.definition.is_some() {
            len += 1;
        }
        if self.location.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.AnnotationCall", len)?;
        if !self.library.is_empty() {
            struct_ser.serialize_field("library", &self.library)?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.arguments.is_empty() {
            struct_ser.serialize_field("arguments", &self.arguments)?;
        }
        if let Some(v) = self.definition.as_ref() {
            struct_ser.serialize_field("definition", v)?;
        }
        if let Some(v) = self.location.as_ref() {
            struct_ser.serialize_field("location", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AnnotationCall {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "library",
            "name",
            "arguments",
            "definition",
            "location",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Library,
            Name,
            Arguments,
            Definition,
            Location,
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
                            "library" => Ok(GeneratedField::Library),
                            "name" => Ok(GeneratedField::Name),
                            "arguments" => Ok(GeneratedField::Arguments),
                            "definition" => Ok(GeneratedField::Definition),
                            "location" => Ok(GeneratedField::Location),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AnnotationCall;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.AnnotationCall")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AnnotationCall, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut library__ = None;
                let mut name__ = None;
                let mut arguments__ = None;
                let mut definition__ = None;
                let mut location__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Library => {
                            if library__.is_some() {
                                return Err(serde::de::Error::duplicate_field("library"));
                            }
                            library__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Arguments => {
                            if arguments__.is_some() {
                                return Err(serde::de::Error::duplicate_field("arguments"));
                            }
                            arguments__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Definition => {
                            if definition__.is_some() {
                                return Err(serde::de::Error::duplicate_field("definition"));
                            }
                            definition__ = map_.next_value()?;
                        }
                        GeneratedField::Location => {
                            if location__.is_some() {
                                return Err(serde::de::Error::duplicate_field("location"));
                            }
                            location__ = map_.next_value()?;
                        }
                    }
                }
                Ok(AnnotationCall {
                    library: library__.unwrap_or_default(),
                    name: name__.unwrap_or_default(),
                    arguments: arguments__.unwrap_or_default(),
                    definition: definition__,
                    location: location__,
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.AnnotationCall", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AnnotationCompositionRef {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.library.is_empty() {
            len += 1;
        }
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.arguments.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.AnnotationCompositionRef", len)?;
        if !self.library.is_empty() {
            struct_ser.serialize_field("library", &self.library)?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.arguments.is_empty() {
            struct_ser.serialize_field("arguments", &self.arguments)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AnnotationCompositionRef {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "library",
            "name",
            "arguments",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Library,
            Name,
            Arguments,
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
                            "library" => Ok(GeneratedField::Library),
                            "name" => Ok(GeneratedField::Name),
                            "arguments" => Ok(GeneratedField::Arguments),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AnnotationCompositionRef;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.AnnotationCompositionRef")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AnnotationCompositionRef, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut library__ = None;
                let mut name__ = None;
                let mut arguments__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Library => {
                            if library__.is_some() {
                                return Err(serde::de::Error::duplicate_field("library"));
                            }
                            library__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Arguments => {
                            if arguments__.is_some() {
                                return Err(serde::de::Error::duplicate_field("arguments"));
                            }
                            arguments__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(AnnotationCompositionRef {
                    library: library__.unwrap_or_default(),
                    name: name__.unwrap_or_default(),
                    arguments: arguments__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.AnnotationCompositionRef", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AnnotationDefinition {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.library.is_empty() {
            len += 1;
        }
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.targets.is_empty() {
            len += 1;
        }
        if !self.parameters.is_empty() {
            len += 1;
        }
        if !self.compositions.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.AnnotationDefinition", len)?;
        if !self.library.is_empty() {
            struct_ser.serialize_field("library", &self.library)?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.targets.is_empty() {
            struct_ser.serialize_field("targets", &self.targets)?;
        }
        if !self.parameters.is_empty() {
            struct_ser.serialize_field("parameters", &self.parameters)?;
        }
        if !self.compositions.is_empty() {
            struct_ser.serialize_field("compositions", &self.compositions)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AnnotationDefinition {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "library",
            "name",
            "targets",
            "parameters",
            "compositions",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Library,
            Name,
            Targets,
            Parameters,
            Compositions,
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
                            "library" => Ok(GeneratedField::Library),
                            "name" => Ok(GeneratedField::Name),
                            "targets" => Ok(GeneratedField::Targets),
                            "parameters" => Ok(GeneratedField::Parameters),
                            "compositions" => Ok(GeneratedField::Compositions),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AnnotationDefinition;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.AnnotationDefinition")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AnnotationDefinition, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut library__ = None;
                let mut name__ = None;
                let mut targets__ = None;
                let mut parameters__ = None;
                let mut compositions__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Library => {
                            if library__.is_some() {
                                return Err(serde::de::Error::duplicate_field("library"));
                            }
                            library__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Targets => {
                            if targets__.is_some() {
                                return Err(serde::de::Error::duplicate_field("targets"));
                            }
                            targets__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Parameters => {
                            if parameters__.is_some() {
                                return Err(serde::de::Error::duplicate_field("parameters"));
                            }
                            parameters__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Compositions => {
                            if compositions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("compositions"));
                            }
                            compositions__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(AnnotationDefinition {
                    library: library__.unwrap_or_default(),
                    name: name__.unwrap_or_default(),
                    targets: targets__.unwrap_or_default(),
                    parameters: parameters__.unwrap_or_default(),
                    compositions: compositions__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.AnnotationDefinition", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AnnotationList {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.values.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.AnnotationList", len)?;
        if !self.values.is_empty() {
            struct_ser.serialize_field("values", &self.values)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AnnotationList {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "values",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Values,
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
                            "values" => Ok(GeneratedField::Values),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AnnotationList;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.AnnotationList")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AnnotationList, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut values__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Values => {
                            if values__.is_some() {
                                return Err(serde::de::Error::duplicate_field("values"));
                            }
                            values__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(AnnotationList {
                    values: values__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.AnnotationList", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AnnotationLiteral {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.value.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.AnnotationLiteral", len)?;
        if let Some(v) = self.value.as_ref() {
            match v {
                annotation_literal::Value::StringValue(v) => {
                    struct_ser.serialize_field("stringValue", v)?;
                }
                annotation_literal::Value::IntValue(v) => {
                    #[allow(clippy::needless_borrow)]
                    #[allow(clippy::needless_borrows_for_generic_args)]
                    struct_ser.serialize_field("intValue", ToString::to_string(&v).as_str())?;
                }
                annotation_literal::Value::FloatValue(v) => {
                    struct_ser.serialize_field("floatValue", v)?;
                }
                annotation_literal::Value::BoolValue(v) => {
                    struct_ser.serialize_field("boolValue", v)?;
                }
                annotation_literal::Value::StructValue(v) => {
                    struct_ser.serialize_field("structValue", v)?;
                }
                annotation_literal::Value::ListValue(v) => {
                    struct_ser.serialize_field("listValue", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AnnotationLiteral {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "string_value",
            "stringValue",
            "int_value",
            "intValue",
            "float_value",
            "floatValue",
            "bool_value",
            "boolValue",
            "struct_value",
            "structValue",
            "list_value",
            "listValue",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StringValue,
            IntValue,
            FloatValue,
            BoolValue,
            StructValue,
            ListValue,
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
                            "stringValue" | "string_value" => Ok(GeneratedField::StringValue),
                            "intValue" | "int_value" => Ok(GeneratedField::IntValue),
                            "floatValue" | "float_value" => Ok(GeneratedField::FloatValue),
                            "boolValue" | "bool_value" => Ok(GeneratedField::BoolValue),
                            "structValue" | "struct_value" => Ok(GeneratedField::StructValue),
                            "listValue" | "list_value" => Ok(GeneratedField::ListValue),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AnnotationLiteral;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.AnnotationLiteral")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AnnotationLiteral, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::StringValue => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stringValue"));
                            }
                            value__ = map_.next_value::<::std::option::Option<_>>()?.map(annotation_literal::Value::StringValue);
                        }
                        GeneratedField::IntValue => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("intValue"));
                            }
                            value__ = map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| annotation_literal::Value::IntValue(x.0));
                        }
                        GeneratedField::FloatValue => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("floatValue"));
                            }
                            value__ = map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| annotation_literal::Value::FloatValue(x.0));
                        }
                        GeneratedField::BoolValue => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("boolValue"));
                            }
                            value__ = map_.next_value::<::std::option::Option<_>>()?.map(annotation_literal::Value::BoolValue);
                        }
                        GeneratedField::StructValue => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("structValue"));
                            }
                            value__ = map_.next_value::<::std::option::Option<_>>()?.map(annotation_literal::Value::StructValue)
;
                        }
                        GeneratedField::ListValue => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("listValue"));
                            }
                            value__ = map_.next_value::<::std::option::Option<_>>()?.map(annotation_literal::Value::ListValue)
;
                        }
                    }
                }
                Ok(AnnotationLiteral {
                    value: value__,
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.AnnotationLiteral", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AnnotationParameter {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        if self.r#type.is_some() {
            len += 1;
        }
        if self.is_optional {
            len += 1;
        }
        if self.default_value.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.AnnotationParameter", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if let Some(v) = self.r#type.as_ref() {
            struct_ser.serialize_field("type", v)?;
        }
        if self.is_optional {
            struct_ser.serialize_field("isOptional", &self.is_optional)?;
        }
        if let Some(v) = self.default_value.as_ref() {
            struct_ser.serialize_field("defaultValue", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AnnotationParameter {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "type",
            "is_optional",
            "isOptional",
            "default_value",
            "defaultValue",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Type,
            IsOptional,
            DefaultValue,
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
                            "name" => Ok(GeneratedField::Name),
                            "type" => Ok(GeneratedField::Type),
                            "isOptional" | "is_optional" => Ok(GeneratedField::IsOptional),
                            "defaultValue" | "default_value" => Ok(GeneratedField::DefaultValue),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AnnotationParameter;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.AnnotationParameter")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AnnotationParameter, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut r#type__ = None;
                let mut is_optional__ = None;
                let mut default_value__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = map_.next_value()?;
                        }
                        GeneratedField::IsOptional => {
                            if is_optional__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isOptional"));
                            }
                            is_optional__ = Some(map_.next_value()?);
                        }
                        GeneratedField::DefaultValue => {
                            if default_value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("defaultValue"));
                            }
                            default_value__ = map_.next_value()?;
                        }
                    }
                }
                Ok(AnnotationParameter {
                    name: name__.unwrap_or_default(),
                    r#type: r#type__,
                    is_optional: is_optional__.unwrap_or_default(),
                    default_value: default_value__,
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.AnnotationParameter", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AnnotationStruct {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.fields.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.AnnotationStruct", len)?;
        if !self.fields.is_empty() {
            struct_ser.serialize_field("fields", &self.fields)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AnnotationStruct {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fields",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Fields,
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
                            "fields" => Ok(GeneratedField::Fields),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AnnotationStruct;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.AnnotationStruct")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AnnotationStruct, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fields__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Fields => {
                            if fields__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fields"));
                            }
                            fields__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                    }
                }
                Ok(AnnotationStruct {
                    fields: fields__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.AnnotationStruct", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Enum {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.full_name.is_empty() {
            len += 1;
        }
        if !self.values.is_empty() {
            len += 1;
        }
        if !self.annotations.is_empty() {
            len += 1;
        }
        if self.location.is_some() {
            len += 1;
        }
        if self.module.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.Enum", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.full_name.is_empty() {
            struct_ser.serialize_field("fullName", &self.full_name)?;
        }
        if !self.values.is_empty() {
            struct_ser.serialize_field("values", &self.values)?;
        }
        if !self.annotations.is_empty() {
            struct_ser.serialize_field("annotations", &self.annotations)?;
        }
        if let Some(v) = self.location.as_ref() {
            struct_ser.serialize_field("location", v)?;
        }
        if let Some(v) = self.module.as_ref() {
            struct_ser.serialize_field("module", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Enum {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "full_name",
            "fullName",
            "values",
            "annotations",
            "location",
            "module",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            FullName,
            Values,
            Annotations,
            Location,
            Module,
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
                            "name" => Ok(GeneratedField::Name),
                            "fullName" | "full_name" => Ok(GeneratedField::FullName),
                            "values" => Ok(GeneratedField::Values),
                            "annotations" => Ok(GeneratedField::Annotations),
                            "location" => Ok(GeneratedField::Location),
                            "module" => Ok(GeneratedField::Module),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Enum;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.Enum")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Enum, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut full_name__ = None;
                let mut values__ = None;
                let mut annotations__ = None;
                let mut location__ = None;
                let mut module__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FullName => {
                            if full_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fullName"));
                            }
                            full_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Values => {
                            if values__.is_some() {
                                return Err(serde::de::Error::duplicate_field("values"));
                            }
                            values__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Annotations => {
                            if annotations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("annotations"));
                            }
                            annotations__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Location => {
                            if location__.is_some() {
                                return Err(serde::de::Error::duplicate_field("location"));
                            }
                            location__ = map_.next_value()?;
                        }
                        GeneratedField::Module => {
                            if module__.is_some() {
                                return Err(serde::de::Error::duplicate_field("module"));
                            }
                            module__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Enum {
                    name: name__.unwrap_or_default(),
                    full_name: full_name__.unwrap_or_default(),
                    values: values__.unwrap_or_default(),
                    annotations: annotations__.unwrap_or_default(),
                    location: location__,
                    module: module__,
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.Enum", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EnumType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.full_name.is_empty() {
            len += 1;
        }
        if !self.values.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.EnumType", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.full_name.is_empty() {
            struct_ser.serialize_field("fullName", &self.full_name)?;
        }
        if !self.values.is_empty() {
            struct_ser.serialize_field("values", &self.values)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EnumType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "full_name",
            "fullName",
            "values",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            FullName,
            Values,
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
                            "name" => Ok(GeneratedField::Name),
                            "fullName" | "full_name" => Ok(GeneratedField::FullName),
                            "values" => Ok(GeneratedField::Values),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EnumType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.EnumType")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EnumType, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut full_name__ = None;
                let mut values__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FullName => {
                            if full_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fullName"));
                            }
                            full_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Values => {
                            if values__.is_some() {
                                return Err(serde::de::Error::duplicate_field("values"));
                            }
                            values__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(EnumType {
                    name: name__.unwrap_or_default(),
                    full_name: full_name__.unwrap_or_default(),
                    values: values__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.EnumType", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EnumValue {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        if self.number != 0 {
            len += 1;
        }
        if self.is_removed {
            len += 1;
        }
        if !self.fallback.is_empty() {
            len += 1;
        }
        if !self.annotations.is_empty() {
            len += 1;
        }
        if self.location.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.EnumValue", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if self.number != 0 {
            struct_ser.serialize_field("number", &self.number)?;
        }
        if self.is_removed {
            struct_ser.serialize_field("isRemoved", &self.is_removed)?;
        }
        if !self.fallback.is_empty() {
            struct_ser.serialize_field("fallback", &self.fallback)?;
        }
        if !self.annotations.is_empty() {
            struct_ser.serialize_field("annotations", &self.annotations)?;
        }
        if let Some(v) = self.location.as_ref() {
            struct_ser.serialize_field("location", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EnumValue {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "number",
            "is_removed",
            "isRemoved",
            "fallback",
            "annotations",
            "location",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Number,
            IsRemoved,
            Fallback,
            Annotations,
            Location,
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
                            "name" => Ok(GeneratedField::Name),
                            "number" => Ok(GeneratedField::Number),
                            "isRemoved" | "is_removed" => Ok(GeneratedField::IsRemoved),
                            "fallback" => Ok(GeneratedField::Fallback),
                            "annotations" => Ok(GeneratedField::Annotations),
                            "location" => Ok(GeneratedField::Location),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EnumValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.EnumValue")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EnumValue, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut number__ = None;
                let mut is_removed__ = None;
                let mut fallback__ = None;
                let mut annotations__ = None;
                let mut location__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Number => {
                            if number__.is_some() {
                                return Err(serde::de::Error::duplicate_field("number"));
                            }
                            number__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::IsRemoved => {
                            if is_removed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isRemoved"));
                            }
                            is_removed__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Fallback => {
                            if fallback__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fallback"));
                            }
                            fallback__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Annotations => {
                            if annotations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("annotations"));
                            }
                            annotations__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Location => {
                            if location__.is_some() {
                                return Err(serde::de::Error::duplicate_field("location"));
                            }
                            location__ = map_.next_value()?;
                        }
                    }
                }
                Ok(EnumValue {
                    name: name__.unwrap_or_default(),
                    number: number__.unwrap_or_default(),
                    is_removed: is_removed__.unwrap_or_default(),
                    fallback: fallback__.unwrap_or_default(),
                    annotations: annotations__.unwrap_or_default(),
                    location: location__,
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.EnumValue", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Field {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        if self.number != 0 {
            len += 1;
        }
        if self.r#type.is_some() {
            len += 1;
        }
        if self.is_optional {
            len += 1;
        }
        if self.is_repeated {
            len += 1;
        }
        if !self.annotations.is_empty() {
            len += 1;
        }
        if self.mapping.is_some() {
            len += 1;
        }
        if self.trace.is_some() {
            len += 1;
        }
        if self.location.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.Field", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if self.number != 0 {
            struct_ser.serialize_field("number", &self.number)?;
        }
        if let Some(v) = self.r#type.as_ref() {
            struct_ser.serialize_field("type", v)?;
        }
        if self.is_optional {
            struct_ser.serialize_field("isOptional", &self.is_optional)?;
        }
        if self.is_repeated {
            struct_ser.serialize_field("isRepeated", &self.is_repeated)?;
        }
        if !self.annotations.is_empty() {
            struct_ser.serialize_field("annotations", &self.annotations)?;
        }
        if let Some(v) = self.mapping.as_ref() {
            struct_ser.serialize_field("mapping", v)?;
        }
        if let Some(v) = self.trace.as_ref() {
            struct_ser.serialize_field("trace", v)?;
        }
        if let Some(v) = self.location.as_ref() {
            struct_ser.serialize_field("location", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Field {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "number",
            "type",
            "is_optional",
            "isOptional",
            "is_repeated",
            "isRepeated",
            "annotations",
            "mapping",
            "trace",
            "location",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Number,
            Type,
            IsOptional,
            IsRepeated,
            Annotations,
            Mapping,
            Trace,
            Location,
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
                            "name" => Ok(GeneratedField::Name),
                            "number" => Ok(GeneratedField::Number),
                            "type" => Ok(GeneratedField::Type),
                            "isOptional" | "is_optional" => Ok(GeneratedField::IsOptional),
                            "isRepeated" | "is_repeated" => Ok(GeneratedField::IsRepeated),
                            "annotations" => Ok(GeneratedField::Annotations),
                            "mapping" => Ok(GeneratedField::Mapping),
                            "trace" => Ok(GeneratedField::Trace),
                            "location" => Ok(GeneratedField::Location),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Field;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.Field")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Field, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut number__ = None;
                let mut r#type__ = None;
                let mut is_optional__ = None;
                let mut is_repeated__ = None;
                let mut annotations__ = None;
                let mut mapping__ = None;
                let mut trace__ = None;
                let mut location__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Number => {
                            if number__.is_some() {
                                return Err(serde::de::Error::duplicate_field("number"));
                            }
                            number__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = map_.next_value()?;
                        }
                        GeneratedField::IsOptional => {
                            if is_optional__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isOptional"));
                            }
                            is_optional__ = Some(map_.next_value()?);
                        }
                        GeneratedField::IsRepeated => {
                            if is_repeated__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isRepeated"));
                            }
                            is_repeated__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Annotations => {
                            if annotations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("annotations"));
                            }
                            annotations__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Mapping => {
                            if mapping__.is_some() {
                                return Err(serde::de::Error::duplicate_field("mapping"));
                            }
                            mapping__ = map_.next_value()?;
                        }
                        GeneratedField::Trace => {
                            if trace__.is_some() {
                                return Err(serde::de::Error::duplicate_field("trace"));
                            }
                            trace__ = map_.next_value()?;
                        }
                        GeneratedField::Location => {
                            if location__.is_some() {
                                return Err(serde::de::Error::duplicate_field("location"));
                            }
                            location__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Field {
                    name: name__.unwrap_or_default(),
                    number: number__.unwrap_or_default(),
                    r#type: r#type__,
                    is_optional: is_optional__.unwrap_or_default(),
                    is_repeated: is_repeated__.unwrap_or_default(),
                    annotations: annotations__.unwrap_or_default(),
                    mapping: mapping__,
                    trace: trace__,
                    location: location__,
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.Field", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FieldMapping {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.chain.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.FieldMapping", len)?;
        if !self.chain.is_empty() {
            struct_ser.serialize_field("chain", &self.chain)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FieldMapping {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Chain,
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
                            "chain" => Ok(GeneratedField::Chain),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FieldMapping;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.FieldMapping")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FieldMapping, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Chain => {
                            if chain__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chain"));
                            }
                            chain__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(FieldMapping {
                    chain: chain__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.FieldMapping", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FieldTrace {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.shape.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.FieldTrace", len)?;
        if let Some(v) = self.shape.as_ref() {
            struct_ser.serialize_field("shape", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FieldTrace {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "shape",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Shape,
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
                            "shape" => Ok(GeneratedField::Shape),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FieldTrace;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.FieldTrace")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FieldTrace, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut shape__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Shape => {
                            if shape__.is_some() {
                                return Err(serde::de::Error::duplicate_field("shape"));
                            }
                            shape__ = map_.next_value()?;
                        }
                    }
                }
                Ok(FieldTrace {
                    shape: shape__,
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.FieldTrace", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GenericOrigin {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.source_name.is_empty() {
            len += 1;
        }
        if !self.type_arguments.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.GenericOrigin", len)?;
        if !self.source_name.is_empty() {
            struct_ser.serialize_field("sourceName", &self.source_name)?;
        }
        if !self.type_arguments.is_empty() {
            struct_ser.serialize_field("typeArguments", &self.type_arguments)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GenericOrigin {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "source_name",
            "sourceName",
            "type_arguments",
            "typeArguments",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SourceName,
            TypeArguments,
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
                            "sourceName" | "source_name" => Ok(GeneratedField::SourceName),
                            "typeArguments" | "type_arguments" => Ok(GeneratedField::TypeArguments),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GenericOrigin;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.GenericOrigin")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenericOrigin, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut source_name__ = None;
                let mut type_arguments__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SourceName => {
                            if source_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sourceName"));
                            }
                            source_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TypeArguments => {
                            if type_arguments__.is_some() {
                                return Err(serde::de::Error::duplicate_field("typeArguments"));
                            }
                            type_arguments__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(GenericOrigin {
                    source_name: source_name__.unwrap_or_default(),
                    type_arguments: type_arguments__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.GenericOrigin", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MapType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.key.is_some() {
            len += 1;
        }
        if self.value.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.MapType", len)?;
        if let Some(v) = self.key.as_ref() {
            struct_ser.serialize_field("key", v)?;
        }
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MapType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "key",
            "value",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Key,
            Value,
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
                            "key" => Ok(GeneratedField::Key),
                            "value" => Ok(GeneratedField::Value),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MapType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.MapType")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<MapType, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut key__ = None;
                let mut value__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Key => {
                            if key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("key"));
                            }
                            key__ = map_.next_value()?;
                        }
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map_.next_value()?;
                        }
                    }
                }
                Ok(MapType {
                    key: key__,
                    value: value__,
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.MapType", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MappingLink {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.source_type_name.is_empty() {
            len += 1;
        }
        if !self.source_type_full_name.is_empty() {
            len += 1;
        }
        if !self.source_field_name.is_empty() {
            len += 1;
        }
        if !self.path.is_empty() {
            len += 1;
        }
        if self.source_field_type.is_some() {
            len += 1;
        }
        if !self.source_field_annotations.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.MappingLink", len)?;
        if !self.source_type_name.is_empty() {
            struct_ser.serialize_field("sourceTypeName", &self.source_type_name)?;
        }
        if !self.source_type_full_name.is_empty() {
            struct_ser.serialize_field("sourceTypeFullName", &self.source_type_full_name)?;
        }
        if !self.source_field_name.is_empty() {
            struct_ser.serialize_field("sourceFieldName", &self.source_field_name)?;
        }
        if !self.path.is_empty() {
            struct_ser.serialize_field("path", &self.path)?;
        }
        if let Some(v) = self.source_field_type.as_ref() {
            struct_ser.serialize_field("sourceFieldType", v)?;
        }
        if !self.source_field_annotations.is_empty() {
            struct_ser.serialize_field("sourceFieldAnnotations", &self.source_field_annotations)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MappingLink {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "source_type_name",
            "sourceTypeName",
            "source_type_full_name",
            "sourceTypeFullName",
            "source_field_name",
            "sourceFieldName",
            "path",
            "source_field_type",
            "sourceFieldType",
            "source_field_annotations",
            "sourceFieldAnnotations",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SourceTypeName,
            SourceTypeFullName,
            SourceFieldName,
            Path,
            SourceFieldType,
            SourceFieldAnnotations,
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
                            "sourceTypeName" | "source_type_name" => Ok(GeneratedField::SourceTypeName),
                            "sourceTypeFullName" | "source_type_full_name" => Ok(GeneratedField::SourceTypeFullName),
                            "sourceFieldName" | "source_field_name" => Ok(GeneratedField::SourceFieldName),
                            "path" => Ok(GeneratedField::Path),
                            "sourceFieldType" | "source_field_type" => Ok(GeneratedField::SourceFieldType),
                            "sourceFieldAnnotations" | "source_field_annotations" => Ok(GeneratedField::SourceFieldAnnotations),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MappingLink;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.MappingLink")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<MappingLink, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut source_type_name__ = None;
                let mut source_type_full_name__ = None;
                let mut source_field_name__ = None;
                let mut path__ = None;
                let mut source_field_type__ = None;
                let mut source_field_annotations__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SourceTypeName => {
                            if source_type_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sourceTypeName"));
                            }
                            source_type_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SourceTypeFullName => {
                            if source_type_full_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sourceTypeFullName"));
                            }
                            source_type_full_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SourceFieldName => {
                            if source_field_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sourceFieldName"));
                            }
                            source_field_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SourceFieldType => {
                            if source_field_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sourceFieldType"));
                            }
                            source_field_type__ = map_.next_value()?;
                        }
                        GeneratedField::SourceFieldAnnotations => {
                            if source_field_annotations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sourceFieldAnnotations"));
                            }
                            source_field_annotations__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(MappingLink {
                    source_type_name: source_type_name__.unwrap_or_default(),
                    source_type_full_name: source_type_full_name__.unwrap_or_default(),
                    source_field_name: source_field_name__.unwrap_or_default(),
                    path: path__.unwrap_or_default(),
                    source_field_type: source_field_type__,
                    source_field_annotations: source_field_annotations__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.MappingLink", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MessageType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.full_name.is_empty() {
            len += 1;
        }
        if !self.fields.is_empty() {
            len += 1;
        }
        if !self.oneofs.is_empty() {
            len += 1;
        }
        if !self.nested_enums.is_empty() {
            len += 1;
        }
        if !self.annotations.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.MessageType", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.full_name.is_empty() {
            struct_ser.serialize_field("fullName", &self.full_name)?;
        }
        if !self.fields.is_empty() {
            struct_ser.serialize_field("fields", &self.fields)?;
        }
        if !self.oneofs.is_empty() {
            struct_ser.serialize_field("oneofs", &self.oneofs)?;
        }
        if !self.nested_enums.is_empty() {
            struct_ser.serialize_field("nestedEnums", &self.nested_enums)?;
        }
        if !self.annotations.is_empty() {
            struct_ser.serialize_field("annotations", &self.annotations)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MessageType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "full_name",
            "fullName",
            "fields",
            "oneofs",
            "nested_enums",
            "nestedEnums",
            "annotations",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            FullName,
            Fields,
            Oneofs,
            NestedEnums,
            Annotations,
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
                            "name" => Ok(GeneratedField::Name),
                            "fullName" | "full_name" => Ok(GeneratedField::FullName),
                            "fields" => Ok(GeneratedField::Fields),
                            "oneofs" => Ok(GeneratedField::Oneofs),
                            "nestedEnums" | "nested_enums" => Ok(GeneratedField::NestedEnums),
                            "annotations" => Ok(GeneratedField::Annotations),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MessageType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.MessageType")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<MessageType, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut full_name__ = None;
                let mut fields__ = None;
                let mut oneofs__ = None;
                let mut nested_enums__ = None;
                let mut annotations__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FullName => {
                            if full_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fullName"));
                            }
                            full_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Fields => {
                            if fields__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fields"));
                            }
                            fields__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Oneofs => {
                            if oneofs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("oneofs"));
                            }
                            oneofs__ = Some(map_.next_value()?);
                        }
                        GeneratedField::NestedEnums => {
                            if nested_enums__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nestedEnums"));
                            }
                            nested_enums__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Annotations => {
                            if annotations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("annotations"));
                            }
                            annotations__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(MessageType {
                    name: name__.unwrap_or_default(),
                    full_name: full_name__.unwrap_or_default(),
                    fields: fields__.unwrap_or_default(),
                    oneofs: oneofs__.unwrap_or_default(),
                    nested_enums: nested_enums__.unwrap_or_default(),
                    annotations: annotations__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.MessageType", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Module {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.package.is_empty() {
            len += 1;
        }
        if !self.types.is_empty() {
            len += 1;
        }
        if !self.enums.is_empty() {
            len += 1;
        }
        if !self.services.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.Module", len)?;
        if !self.package.is_empty() {
            struct_ser.serialize_field("package", &self.package)?;
        }
        if !self.types.is_empty() {
            struct_ser.serialize_field("types", &self.types)?;
        }
        if !self.enums.is_empty() {
            struct_ser.serialize_field("enums", &self.enums)?;
        }
        if !self.services.is_empty() {
            struct_ser.serialize_field("services", &self.services)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Module {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "package",
            "types",
            "enums",
            "services",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Package,
            Types,
            Enums,
            Services,
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
                            "package" => Ok(GeneratedField::Package),
                            "types" => Ok(GeneratedField::Types),
                            "enums" => Ok(GeneratedField::Enums),
                            "services" => Ok(GeneratedField::Services),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Module;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.Module")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Module, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut package__ = None;
                let mut types__ = None;
                let mut enums__ = None;
                let mut services__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Package => {
                            if package__.is_some() {
                                return Err(serde::de::Error::duplicate_field("package"));
                            }
                            package__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Types => {
                            if types__.is_some() {
                                return Err(serde::de::Error::duplicate_field("types"));
                            }
                            types__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Enums => {
                            if enums__.is_some() {
                                return Err(serde::de::Error::duplicate_field("enums"));
                            }
                            enums__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Services => {
                            if services__.is_some() {
                                return Err(serde::de::Error::duplicate_field("services"));
                            }
                            services__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Module {
                    package: package__.unwrap_or_default(),
                    types: types__.unwrap_or_default(),
                    enums: enums__.unwrap_or_default(),
                    services: services__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.Module", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ModuleInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.module_path.is_empty() {
            len += 1;
        }
        if !self.package.is_empty() {
            len += 1;
        }
        if !self.version.is_empty() {
            len += 1;
        }
        if self.generate {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.ModuleInfo", len)?;
        if !self.module_path.is_empty() {
            struct_ser.serialize_field("modulePath", &self.module_path)?;
        }
        if !self.package.is_empty() {
            struct_ser.serialize_field("package", &self.package)?;
        }
        if !self.version.is_empty() {
            struct_ser.serialize_field("version", &self.version)?;
        }
        if self.generate {
            struct_ser.serialize_field("generate", &self.generate)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ModuleInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "module_path",
            "modulePath",
            "package",
            "version",
            "generate",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ModulePath,
            Package,
            Version,
            Generate,
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
                            "modulePath" | "module_path" => Ok(GeneratedField::ModulePath),
                            "package" => Ok(GeneratedField::Package),
                            "version" => Ok(GeneratedField::Version),
                            "generate" => Ok(GeneratedField::Generate),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ModuleInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.ModuleInfo")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ModuleInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut module_path__ = None;
                let mut package__ = None;
                let mut version__ = None;
                let mut generate__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ModulePath => {
                            if module_path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("modulePath"));
                            }
                            module_path__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Package => {
                            if package__.is_some() {
                                return Err(serde::de::Error::duplicate_field("package"));
                            }
                            package__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Generate => {
                            if generate__.is_some() {
                                return Err(serde::de::Error::duplicate_field("generate"));
                            }
                            generate__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ModuleInfo {
                    module_path: module_path__.unwrap_or_default(),
                    package: package__.unwrap_or_default(),
                    version: version__.unwrap_or_default(),
                    generate: generate__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.ModuleInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for OneofField {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        if self.number != 0 {
            len += 1;
        }
        if self.r#type.is_some() {
            len += 1;
        }
        if !self.annotations.is_empty() {
            len += 1;
        }
        if self.mapping.is_some() {
            len += 1;
        }
        if self.location.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.OneofField", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if self.number != 0 {
            struct_ser.serialize_field("number", &self.number)?;
        }
        if let Some(v) = self.r#type.as_ref() {
            struct_ser.serialize_field("type", v)?;
        }
        if !self.annotations.is_empty() {
            struct_ser.serialize_field("annotations", &self.annotations)?;
        }
        if let Some(v) = self.mapping.as_ref() {
            struct_ser.serialize_field("mapping", v)?;
        }
        if let Some(v) = self.location.as_ref() {
            struct_ser.serialize_field("location", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OneofField {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "number",
            "type",
            "annotations",
            "mapping",
            "location",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Number,
            Type,
            Annotations,
            Mapping,
            Location,
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
                            "name" => Ok(GeneratedField::Name),
                            "number" => Ok(GeneratedField::Number),
                            "type" => Ok(GeneratedField::Type),
                            "annotations" => Ok(GeneratedField::Annotations),
                            "mapping" => Ok(GeneratedField::Mapping),
                            "location" => Ok(GeneratedField::Location),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OneofField;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.OneofField")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<OneofField, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut number__ = None;
                let mut r#type__ = None;
                let mut annotations__ = None;
                let mut mapping__ = None;
                let mut location__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Number => {
                            if number__.is_some() {
                                return Err(serde::de::Error::duplicate_field("number"));
                            }
                            number__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = map_.next_value()?;
                        }
                        GeneratedField::Annotations => {
                            if annotations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("annotations"));
                            }
                            annotations__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Mapping => {
                            if mapping__.is_some() {
                                return Err(serde::de::Error::duplicate_field("mapping"));
                            }
                            mapping__ = map_.next_value()?;
                        }
                        GeneratedField::Location => {
                            if location__.is_some() {
                                return Err(serde::de::Error::duplicate_field("location"));
                            }
                            location__ = map_.next_value()?;
                        }
                    }
                }
                Ok(OneofField {
                    name: name__.unwrap_or_default(),
                    number: number__.unwrap_or_default(),
                    r#type: r#type__,
                    annotations: annotations__.unwrap_or_default(),
                    mapping: mapping__,
                    location: location__,
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.OneofField", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for OneofGroup {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.fields.is_empty() {
            len += 1;
        }
        if !self.annotations.is_empty() {
            len += 1;
        }
        if self.location.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.OneofGroup", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.fields.is_empty() {
            struct_ser.serialize_field("fields", &self.fields)?;
        }
        if !self.annotations.is_empty() {
            struct_ser.serialize_field("annotations", &self.annotations)?;
        }
        if let Some(v) = self.location.as_ref() {
            struct_ser.serialize_field("location", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OneofGroup {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "fields",
            "annotations",
            "location",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Fields,
            Annotations,
            Location,
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
                            "name" => Ok(GeneratedField::Name),
                            "fields" => Ok(GeneratedField::Fields),
                            "annotations" => Ok(GeneratedField::Annotations),
                            "location" => Ok(GeneratedField::Location),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OneofGroup;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.OneofGroup")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<OneofGroup, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut fields__ = None;
                let mut annotations__ = None;
                let mut location__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Fields => {
                            if fields__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fields"));
                            }
                            fields__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Annotations => {
                            if annotations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("annotations"));
                            }
                            annotations__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Location => {
                            if location__.is_some() {
                                return Err(serde::de::Error::duplicate_field("location"));
                            }
                            location__ = map_.next_value()?;
                        }
                    }
                }
                Ok(OneofGroup {
                    name: name__.unwrap_or_default(),
                    fields: fields__.unwrap_or_default(),
                    annotations: annotations__.unwrap_or_default(),
                    location: location__,
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.OneofGroup", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PickOmitOrigin {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.kind.is_empty() {
            len += 1;
        }
        if !self.source_type_name.is_empty() {
            len += 1;
        }
        if !self.field_names.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.PickOmitOrigin", len)?;
        if !self.kind.is_empty() {
            struct_ser.serialize_field("kind", &self.kind)?;
        }
        if !self.source_type_name.is_empty() {
            struct_ser.serialize_field("sourceTypeName", &self.source_type_name)?;
        }
        if !self.field_names.is_empty() {
            struct_ser.serialize_field("fieldNames", &self.field_names)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PickOmitOrigin {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "kind",
            "source_type_name",
            "sourceTypeName",
            "field_names",
            "fieldNames",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Kind,
            SourceTypeName,
            FieldNames,
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
                            "kind" => Ok(GeneratedField::Kind),
                            "sourceTypeName" | "source_type_name" => Ok(GeneratedField::SourceTypeName),
                            "fieldNames" | "field_names" => Ok(GeneratedField::FieldNames),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PickOmitOrigin;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.PickOmitOrigin")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PickOmitOrigin, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut kind__ = None;
                let mut source_type_name__ = None;
                let mut field_names__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Kind => {
                            if kind__.is_some() {
                                return Err(serde::de::Error::duplicate_field("kind"));
                            }
                            kind__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SourceTypeName => {
                            if source_type_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sourceTypeName"));
                            }
                            source_type_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FieldNames => {
                            if field_names__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fieldNames"));
                            }
                            field_names__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(PickOmitOrigin {
                    kind: kind__.unwrap_or_default(),
                    source_type_name: source_type_name__.unwrap_or_default(),
                    field_names: field_names__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.PickOmitOrigin", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Rpc {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        if self.input.is_some() {
            len += 1;
        }
        if self.output.is_some() {
            len += 1;
        }
        if !self.annotations.is_empty() {
            len += 1;
        }
        if self.location.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.Rpc", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if let Some(v) = self.input.as_ref() {
            struct_ser.serialize_field("input", v)?;
        }
        if let Some(v) = self.output.as_ref() {
            struct_ser.serialize_field("output", v)?;
        }
        if !self.annotations.is_empty() {
            struct_ser.serialize_field("annotations", &self.annotations)?;
        }
        if let Some(v) = self.location.as_ref() {
            struct_ser.serialize_field("location", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Rpc {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "input",
            "output",
            "annotations",
            "location",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Input,
            Output,
            Annotations,
            Location,
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
                            "name" => Ok(GeneratedField::Name),
                            "input" => Ok(GeneratedField::Input),
                            "output" => Ok(GeneratedField::Output),
                            "annotations" => Ok(GeneratedField::Annotations),
                            "location" => Ok(GeneratedField::Location),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Rpc;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.Rpc")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Rpc, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut input__ = None;
                let mut output__ = None;
                let mut annotations__ = None;
                let mut location__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Input => {
                            if input__.is_some() {
                                return Err(serde::de::Error::duplicate_field("input"));
                            }
                            input__ = map_.next_value()?;
                        }
                        GeneratedField::Output => {
                            if output__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output"));
                            }
                            output__ = map_.next_value()?;
                        }
                        GeneratedField::Annotations => {
                            if annotations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("annotations"));
                            }
                            annotations__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Location => {
                            if location__.is_some() {
                                return Err(serde::de::Error::duplicate_field("location"));
                            }
                            location__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Rpc {
                    name: name__.unwrap_or_default(),
                    input: input__,
                    output: output__,
                    annotations: annotations__.unwrap_or_default(),
                    location: location__,
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.Rpc", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RpcParam {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.is_void {
            len += 1;
        }
        if self.is_stream {
            len += 1;
        }
        if self.r#type.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.RpcParam", len)?;
        if self.is_void {
            struct_ser.serialize_field("isVoid", &self.is_void)?;
        }
        if self.is_stream {
            struct_ser.serialize_field("isStream", &self.is_stream)?;
        }
        if let Some(v) = self.r#type.as_ref() {
            struct_ser.serialize_field("type", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RpcParam {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "is_void",
            "isVoid",
            "is_stream",
            "isStream",
            "type",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            IsVoid,
            IsStream,
            Type,
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
                            "isVoid" | "is_void" => Ok(GeneratedField::IsVoid),
                            "isStream" | "is_stream" => Ok(GeneratedField::IsStream),
                            "type" => Ok(GeneratedField::Type),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RpcParam;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.RpcParam")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<RpcParam, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut is_void__ = None;
                let mut is_stream__ = None;
                let mut r#type__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::IsVoid => {
                            if is_void__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isVoid"));
                            }
                            is_void__ = Some(map_.next_value()?);
                        }
                        GeneratedField::IsStream => {
                            if is_stream__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isStream"));
                            }
                            is_stream__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = map_.next_value()?;
                        }
                    }
                }
                Ok(RpcParam {
                    is_void: is_void__.unwrap_or_default(),
                    is_stream: is_stream__.unwrap_or_default(),
                    r#type: r#type__,
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.RpcParam", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ScalarKind {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::None => "SCALAR_KIND_NONE",
            Self::Bool => "BOOL",
            Self::String => "STRING",
            Self::Bytes => "BYTES",
            Self::Int8 => "INT8",
            Self::Int16 => "INT16",
            Self::Int32 => "INT32",
            Self::Int64 => "INT64",
            Self::Uint8 => "UINT8",
            Self::Uint16 => "UINT16",
            Self::Uint32 => "UINT32",
            Self::Uint64 => "UINT64",
            Self::Float => "FLOAT",
            Self::Double => "DOUBLE",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for ScalarKind {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "SCALAR_KIND_NONE",
            "BOOL",
            "STRING",
            "BYTES",
            "INT8",
            "INT16",
            "INT32",
            "INT64",
            "UINT8",
            "UINT16",
            "UINT32",
            "UINT64",
            "FLOAT",
            "DOUBLE",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ScalarKind;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "SCALAR_KIND_NONE" => Ok(ScalarKind::None),
                    "BOOL" => Ok(ScalarKind::Bool),
                    "STRING" => Ok(ScalarKind::String),
                    "BYTES" => Ok(ScalarKind::Bytes),
                    "INT8" => Ok(ScalarKind::Int8),
                    "INT16" => Ok(ScalarKind::Int16),
                    "INT32" => Ok(ScalarKind::Int32),
                    "INT64" => Ok(ScalarKind::Int64),
                    "UINT8" => Ok(ScalarKind::Uint8),
                    "UINT16" => Ok(ScalarKind::Uint16),
                    "UINT32" => Ok(ScalarKind::Uint32),
                    "UINT64" => Ok(ScalarKind::Uint64),
                    "FLOAT" => Ok(ScalarKind::Float),
                    "DOUBLE" => Ok(ScalarKind::Double),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for ScalarType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.scalar_kind != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.ScalarType", len)?;
        if self.scalar_kind != 0 {
            let v = ScalarKind::try_from(self.scalar_kind)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.scalar_kind)))?;
            struct_ser.serialize_field("scalarKind", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ScalarType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "scalar_kind",
            "scalarKind",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ScalarKind,
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
                            "scalarKind" | "scalar_kind" => Ok(GeneratedField::ScalarKind),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ScalarType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.ScalarType")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ScalarType, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut scalar_kind__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ScalarKind => {
                            if scalar_kind__.is_some() {
                                return Err(serde::de::Error::duplicate_field("scalarKind"));
                            }
                            scalar_kind__ = Some(map_.next_value::<ScalarKind>()? as i32);
                        }
                    }
                }
                Ok(ScalarType {
                    scalar_kind: scalar_kind__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.ScalarType", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Service {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.full_name.is_empty() {
            len += 1;
        }
        if !self.rpcs.is_empty() {
            len += 1;
        }
        if !self.annotations.is_empty() {
            len += 1;
        }
        if self.location.is_some() {
            len += 1;
        }
        if self.module.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.Service", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.full_name.is_empty() {
            struct_ser.serialize_field("fullName", &self.full_name)?;
        }
        if !self.rpcs.is_empty() {
            struct_ser.serialize_field("rpcs", &self.rpcs)?;
        }
        if !self.annotations.is_empty() {
            struct_ser.serialize_field("annotations", &self.annotations)?;
        }
        if let Some(v) = self.location.as_ref() {
            struct_ser.serialize_field("location", v)?;
        }
        if let Some(v) = self.module.as_ref() {
            struct_ser.serialize_field("module", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Service {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "full_name",
            "fullName",
            "rpcs",
            "annotations",
            "location",
            "module",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            FullName,
            Rpcs,
            Annotations,
            Location,
            Module,
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
                            "name" => Ok(GeneratedField::Name),
                            "fullName" | "full_name" => Ok(GeneratedField::FullName),
                            "rpcs" => Ok(GeneratedField::Rpcs),
                            "annotations" => Ok(GeneratedField::Annotations),
                            "location" => Ok(GeneratedField::Location),
                            "module" => Ok(GeneratedField::Module),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Service;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.Service")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Service, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut full_name__ = None;
                let mut rpcs__ = None;
                let mut annotations__ = None;
                let mut location__ = None;
                let mut module__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FullName => {
                            if full_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fullName"));
                            }
                            full_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Rpcs => {
                            if rpcs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rpcs"));
                            }
                            rpcs__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Annotations => {
                            if annotations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("annotations"));
                            }
                            annotations__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Location => {
                            if location__.is_some() {
                                return Err(serde::de::Error::duplicate_field("location"));
                            }
                            location__ = map_.next_value()?;
                        }
                        GeneratedField::Module => {
                            if module__.is_some() {
                                return Err(serde::de::Error::duplicate_field("module"));
                            }
                            module__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Service {
                    name: name__.unwrap_or_default(),
                    full_name: full_name__.unwrap_or_default(),
                    rpcs: rpcs__.unwrap_or_default(),
                    annotations: annotations__.unwrap_or_default(),
                    location: location__,
                    module: module__,
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.Service", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ShapeOrigin {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.shape_name.is_empty() {
            len += 1;
        }
        if !self.shape_full_name.is_empty() {
            len += 1;
        }
        if self.injection_range_start != 0 {
            len += 1;
        }
        if self.injection_range_end != 0 {
            len += 1;
        }
        if self.shape_location.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.ShapeOrigin", len)?;
        if !self.shape_name.is_empty() {
            struct_ser.serialize_field("shapeName", &self.shape_name)?;
        }
        if !self.shape_full_name.is_empty() {
            struct_ser.serialize_field("shapeFullName", &self.shape_full_name)?;
        }
        if self.injection_range_start != 0 {
            struct_ser.serialize_field("injectionRangeStart", &self.injection_range_start)?;
        }
        if self.injection_range_end != 0 {
            struct_ser.serialize_field("injectionRangeEnd", &self.injection_range_end)?;
        }
        if let Some(v) = self.shape_location.as_ref() {
            struct_ser.serialize_field("shapeLocation", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ShapeOrigin {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "shape_name",
            "shapeName",
            "shape_full_name",
            "shapeFullName",
            "injection_range_start",
            "injectionRangeStart",
            "injection_range_end",
            "injectionRangeEnd",
            "shape_location",
            "shapeLocation",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ShapeName,
            ShapeFullName,
            InjectionRangeStart,
            InjectionRangeEnd,
            ShapeLocation,
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
                            "shapeName" | "shape_name" => Ok(GeneratedField::ShapeName),
                            "shapeFullName" | "shape_full_name" => Ok(GeneratedField::ShapeFullName),
                            "injectionRangeStart" | "injection_range_start" => Ok(GeneratedField::InjectionRangeStart),
                            "injectionRangeEnd" | "injection_range_end" => Ok(GeneratedField::InjectionRangeEnd),
                            "shapeLocation" | "shape_location" => Ok(GeneratedField::ShapeLocation),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ShapeOrigin;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.ShapeOrigin")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ShapeOrigin, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut shape_name__ = None;
                let mut shape_full_name__ = None;
                let mut injection_range_start__ = None;
                let mut injection_range_end__ = None;
                let mut shape_location__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ShapeName => {
                            if shape_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("shapeName"));
                            }
                            shape_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ShapeFullName => {
                            if shape_full_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("shapeFullName"));
                            }
                            shape_full_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::InjectionRangeStart => {
                            if injection_range_start__.is_some() {
                                return Err(serde::de::Error::duplicate_field("injectionRangeStart"));
                            }
                            injection_range_start__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::InjectionRangeEnd => {
                            if injection_range_end__.is_some() {
                                return Err(serde::de::Error::duplicate_field("injectionRangeEnd"));
                            }
                            injection_range_end__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ShapeLocation => {
                            if shape_location__.is_some() {
                                return Err(serde::de::Error::duplicate_field("shapeLocation"));
                            }
                            shape_location__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ShapeOrigin {
                    shape_name: shape_name__.unwrap_or_default(),
                    shape_full_name: shape_full_name__.unwrap_or_default(),
                    injection_range_start: injection_range_start__.unwrap_or_default(),
                    injection_range_end: injection_range_end__.unwrap_or_default(),
                    shape_location: shape_location__,
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.ShapeOrigin", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Type {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.full_name.is_empty() {
            len += 1;
        }
        if !self.fields.is_empty() {
            len += 1;
        }
        if !self.oneofs.is_empty() {
            len += 1;
        }
        if !self.nested_types.is_empty() {
            len += 1;
        }
        if !self.nested_enums.is_empty() {
            len += 1;
        }
        if !self.annotations.is_empty() {
            len += 1;
        }
        if !self.back_references.is_empty() {
            len += 1;
        }
        if self.trace.is_some() {
            len += 1;
        }
        if self.location.is_some() {
            len += 1;
        }
        if self.module.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.Type", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.full_name.is_empty() {
            struct_ser.serialize_field("fullName", &self.full_name)?;
        }
        if !self.fields.is_empty() {
            struct_ser.serialize_field("fields", &self.fields)?;
        }
        if !self.oneofs.is_empty() {
            struct_ser.serialize_field("oneofs", &self.oneofs)?;
        }
        if !self.nested_types.is_empty() {
            struct_ser.serialize_field("nestedTypes", &self.nested_types)?;
        }
        if !self.nested_enums.is_empty() {
            struct_ser.serialize_field("nestedEnums", &self.nested_enums)?;
        }
        if !self.annotations.is_empty() {
            struct_ser.serialize_field("annotations", &self.annotations)?;
        }
        if !self.back_references.is_empty() {
            struct_ser.serialize_field("backReferences", &self.back_references)?;
        }
        if let Some(v) = self.trace.as_ref() {
            struct_ser.serialize_field("trace", v)?;
        }
        if let Some(v) = self.location.as_ref() {
            struct_ser.serialize_field("location", v)?;
        }
        if let Some(v) = self.module.as_ref() {
            struct_ser.serialize_field("module", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Type {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "full_name",
            "fullName",
            "fields",
            "oneofs",
            "nested_types",
            "nestedTypes",
            "nested_enums",
            "nestedEnums",
            "annotations",
            "back_references",
            "backReferences",
            "trace",
            "location",
            "module",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            FullName,
            Fields,
            Oneofs,
            NestedTypes,
            NestedEnums,
            Annotations,
            BackReferences,
            Trace,
            Location,
            Module,
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
                            "name" => Ok(GeneratedField::Name),
                            "fullName" | "full_name" => Ok(GeneratedField::FullName),
                            "fields" => Ok(GeneratedField::Fields),
                            "oneofs" => Ok(GeneratedField::Oneofs),
                            "nestedTypes" | "nested_types" => Ok(GeneratedField::NestedTypes),
                            "nestedEnums" | "nested_enums" => Ok(GeneratedField::NestedEnums),
                            "annotations" => Ok(GeneratedField::Annotations),
                            "backReferences" | "back_references" => Ok(GeneratedField::BackReferences),
                            "trace" => Ok(GeneratedField::Trace),
                            "location" => Ok(GeneratedField::Location),
                            "module" => Ok(GeneratedField::Module),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Type;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.Type")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Type, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut full_name__ = None;
                let mut fields__ = None;
                let mut oneofs__ = None;
                let mut nested_types__ = None;
                let mut nested_enums__ = None;
                let mut annotations__ = None;
                let mut back_references__ = None;
                let mut trace__ = None;
                let mut location__ = None;
                let mut module__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FullName => {
                            if full_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fullName"));
                            }
                            full_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Fields => {
                            if fields__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fields"));
                            }
                            fields__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Oneofs => {
                            if oneofs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("oneofs"));
                            }
                            oneofs__ = Some(map_.next_value()?);
                        }
                        GeneratedField::NestedTypes => {
                            if nested_types__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nestedTypes"));
                            }
                            nested_types__ = Some(map_.next_value()?);
                        }
                        GeneratedField::NestedEnums => {
                            if nested_enums__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nestedEnums"));
                            }
                            nested_enums__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Annotations => {
                            if annotations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("annotations"));
                            }
                            annotations__ = Some(map_.next_value()?);
                        }
                        GeneratedField::BackReferences => {
                            if back_references__.is_some() {
                                return Err(serde::de::Error::duplicate_field("backReferences"));
                            }
                            back_references__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Trace => {
                            if trace__.is_some() {
                                return Err(serde::de::Error::duplicate_field("trace"));
                            }
                            trace__ = map_.next_value()?;
                        }
                        GeneratedField::Location => {
                            if location__.is_some() {
                                return Err(serde::de::Error::duplicate_field("location"));
                            }
                            location__ = map_.next_value()?;
                        }
                        GeneratedField::Module => {
                            if module__.is_some() {
                                return Err(serde::de::Error::duplicate_field("module"));
                            }
                            module__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Type {
                    name: name__.unwrap_or_default(),
                    full_name: full_name__.unwrap_or_default(),
                    fields: fields__.unwrap_or_default(),
                    oneofs: oneofs__.unwrap_or_default(),
                    nested_types: nested_types__.unwrap_or_default(),
                    nested_enums: nested_enums__.unwrap_or_default(),
                    annotations: annotations__.unwrap_or_default(),
                    back_references: back_references__.unwrap_or_default(),
                    trace: trace__,
                    location: location__,
                    module: module__,
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.Type", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TypeBackRef {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.referencing_type_name.is_empty() {
            len += 1;
        }
        if !self.referencing_type_full_name.is_empty() {
            len += 1;
        }
        if !self.field_name.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.TypeBackRef", len)?;
        if !self.referencing_type_name.is_empty() {
            struct_ser.serialize_field("referencingTypeName", &self.referencing_type_name)?;
        }
        if !self.referencing_type_full_name.is_empty() {
            struct_ser.serialize_field("referencingTypeFullName", &self.referencing_type_full_name)?;
        }
        if !self.field_name.is_empty() {
            struct_ser.serialize_field("fieldName", &self.field_name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TypeBackRef {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "referencing_type_name",
            "referencingTypeName",
            "referencing_type_full_name",
            "referencingTypeFullName",
            "field_name",
            "fieldName",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ReferencingTypeName,
            ReferencingTypeFullName,
            FieldName,
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
                            "referencingTypeName" | "referencing_type_name" => Ok(GeneratedField::ReferencingTypeName),
                            "referencingTypeFullName" | "referencing_type_full_name" => Ok(GeneratedField::ReferencingTypeFullName),
                            "fieldName" | "field_name" => Ok(GeneratedField::FieldName),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TypeBackRef;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.TypeBackRef")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TypeBackRef, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut referencing_type_name__ = None;
                let mut referencing_type_full_name__ = None;
                let mut field_name__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ReferencingTypeName => {
                            if referencing_type_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("referencingTypeName"));
                            }
                            referencing_type_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ReferencingTypeFullName => {
                            if referencing_type_full_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("referencingTypeFullName"));
                            }
                            referencing_type_full_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FieldName => {
                            if field_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fieldName"));
                            }
                            field_name__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(TypeBackRef {
                    referencing_type_name: referencing_type_name__.unwrap_or_default(),
                    referencing_type_full_name: referencing_type_full_name__.unwrap_or_default(),
                    field_name: field_name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.TypeBackRef", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TypeReference {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.kind.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.TypeReference", len)?;
        if let Some(v) = self.kind.as_ref() {
            match v {
                type_reference::Kind::Scalar(v) => {
                    struct_ser.serialize_field("scalar", v)?;
                }
                type_reference::Kind::MessageType(v) => {
                    struct_ser.serialize_field("messageType", v)?;
                }
                type_reference::Kind::EnumType(v) => {
                    struct_ser.serialize_field("enumType", v)?;
                }
                type_reference::Kind::Map(v) => {
                    struct_ser.serialize_field("map", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TypeReference {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "scalar",
            "message_type",
            "messageType",
            "enum_type",
            "enumType",
            "map",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Scalar,
            MessageType,
            EnumType,
            Map,
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
                            "scalar" => Ok(GeneratedField::Scalar),
                            "messageType" | "message_type" => Ok(GeneratedField::MessageType),
                            "enumType" | "enum_type" => Ok(GeneratedField::EnumType),
                            "map" => Ok(GeneratedField::Map),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TypeReference;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.TypeReference")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TypeReference, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut kind__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Scalar => {
                            if kind__.is_some() {
                                return Err(serde::de::Error::duplicate_field("scalar"));
                            }
                            kind__ = map_.next_value::<::std::option::Option<_>>()?.map(type_reference::Kind::Scalar)
;
                        }
                        GeneratedField::MessageType => {
                            if kind__.is_some() {
                                return Err(serde::de::Error::duplicate_field("messageType"));
                            }
                            kind__ = map_.next_value::<::std::option::Option<_>>()?.map(type_reference::Kind::MessageType)
;
                        }
                        GeneratedField::EnumType => {
                            if kind__.is_some() {
                                return Err(serde::de::Error::duplicate_field("enumType"));
                            }
                            kind__ = map_.next_value::<::std::option::Option<_>>()?.map(type_reference::Kind::EnumType)
;
                        }
                        GeneratedField::Map => {
                            if kind__.is_some() {
                                return Err(serde::de::Error::duplicate_field("map"));
                            }
                            kind__ = map_.next_value::<::std::option::Option<_>>()?.map(type_reference::Kind::Map)
;
                        }
                    }
                }
                Ok(TypeReference {
                    kind: kind__,
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.TypeReference", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TypeTrace {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.origin.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.ir.TypeTrace", len)?;
        if let Some(v) = self.origin.as_ref() {
            match v {
                type_trace::Origin::Generic(v) => {
                    struct_ser.serialize_field("generic", v)?;
                }
                type_trace::Origin::PickOmit(v) => {
                    struct_ser.serialize_field("pickOmit", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TypeTrace {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "generic",
            "pick_omit",
            "pickOmit",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Generic,
            PickOmit,
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
                            "generic" => Ok(GeneratedField::Generic),
                            "pickOmit" | "pick_omit" => Ok(GeneratedField::PickOmit),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TypeTrace;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.ir.TypeTrace")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TypeTrace, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut origin__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Generic => {
                            if origin__.is_some() {
                                return Err(serde::de::Error::duplicate_field("generic"));
                            }
                            origin__ = map_.next_value::<::std::option::Option<_>>()?.map(type_trace::Origin::Generic)
;
                        }
                        GeneratedField::PickOmit => {
                            if origin__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pickOmit"));
                            }
                            origin__ = map_.next_value::<::std::option::Option<_>>()?.map(type_trace::Origin::PickOmit)
;
                        }
                    }
                }
                Ok(TypeTrace {
                    origin: origin__,
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.ir.TypeTrace", FIELDS, GeneratedVisitor)
    }
}
