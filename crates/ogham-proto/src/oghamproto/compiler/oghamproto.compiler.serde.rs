// @generated
impl serde::Serialize for CompileError {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.message.is_empty() {
            len += 1;
        }
        if self.severity != 0 {
            len += 1;
        }
        if !self.source_type.is_empty() {
            len += 1;
        }
        if !self.source_field.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.compiler.CompileError", len)?;
        if !self.message.is_empty() {
            struct_ser.serialize_field("message", &self.message)?;
        }
        if self.severity != 0 {
            let v = Severity::try_from(self.severity)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.severity)))?;
            struct_ser.serialize_field("severity", &v)?;
        }
        if !self.source_type.is_empty() {
            struct_ser.serialize_field("sourceType", &self.source_type)?;
        }
        if !self.source_field.is_empty() {
            struct_ser.serialize_field("sourceField", &self.source_field)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CompileError {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "message",
            "severity",
            "source_type",
            "sourceType",
            "source_field",
            "sourceField",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Message,
            Severity,
            SourceType,
            SourceField,
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
                            "message" => Ok(GeneratedField::Message),
                            "severity" => Ok(GeneratedField::Severity),
                            "sourceType" | "source_type" => Ok(GeneratedField::SourceType),
                            "sourceField" | "source_field" => Ok(GeneratedField::SourceField),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CompileError;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.compiler.CompileError")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CompileError, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut message__ = None;
                let mut severity__ = None;
                let mut source_type__ = None;
                let mut source_field__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Message => {
                            if message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("message"));
                            }
                            message__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Severity => {
                            if severity__.is_some() {
                                return Err(serde::de::Error::duplicate_field("severity"));
                            }
                            severity__ = Some(map_.next_value::<Severity>()? as i32);
                        }
                        GeneratedField::SourceType => {
                            if source_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sourceType"));
                            }
                            source_type__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SourceField => {
                            if source_field__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sourceField"));
                            }
                            source_field__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CompileError {
                    message: message__.unwrap_or_default(),
                    severity: severity__.unwrap_or_default(),
                    source_type: source_type__.unwrap_or_default(),
                    source_field: source_field__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.compiler.CompileError", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GeneratedFile {
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
        if !self.content.is_empty() {
            len += 1;
        }
        if self.append {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.compiler.GeneratedFile", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.content.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("content", pbjson::private::base64::encode(&self.content).as_str())?;
        }
        if self.append {
            struct_ser.serialize_field("append", &self.append)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GeneratedFile {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "content",
            "append",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Content,
            Append,
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
                            "content" => Ok(GeneratedField::Content),
                            "append" => Ok(GeneratedField::Append),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GeneratedFile;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.compiler.GeneratedFile")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GeneratedFile, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut content__ = None;
                let mut append__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Content => {
                            if content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("content"));
                            }
                            content__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Append => {
                            if append__.is_some() {
                                return Err(serde::de::Error::duplicate_field("append"));
                            }
                            append__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(GeneratedFile {
                    name: name__.unwrap_or_default(),
                    content: content__.unwrap_or_default(),
                    append: append__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.compiler.GeneratedFile", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for OghamCompileRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.compiler_version.is_empty() {
            len += 1;
        }
        if self.module.is_some() {
            len += 1;
        }
        if !self.options.is_empty() {
            len += 1;
        }
        if !self.output_dir.is_empty() {
            len += 1;
        }
        if !self.module_path.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.compiler.OghamCompileRequest", len)?;
        if !self.compiler_version.is_empty() {
            struct_ser.serialize_field("compilerVersion", &self.compiler_version)?;
        }
        if let Some(v) = self.module.as_ref() {
            struct_ser.serialize_field("module", v)?;
        }
        if !self.options.is_empty() {
            struct_ser.serialize_field("options", &self.options)?;
        }
        if !self.output_dir.is_empty() {
            struct_ser.serialize_field("outputDir", &self.output_dir)?;
        }
        if !self.module_path.is_empty() {
            struct_ser.serialize_field("modulePath", &self.module_path)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OghamCompileRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "compiler_version",
            "compilerVersion",
            "module",
            "options",
            "output_dir",
            "outputDir",
            "module_path",
            "modulePath",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CompilerVersion,
            Module,
            Options,
            OutputDir,
            ModulePath,
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
                            "compilerVersion" | "compiler_version" => Ok(GeneratedField::CompilerVersion),
                            "module" => Ok(GeneratedField::Module),
                            "options" => Ok(GeneratedField::Options),
                            "outputDir" | "output_dir" => Ok(GeneratedField::OutputDir),
                            "modulePath" | "module_path" => Ok(GeneratedField::ModulePath),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OghamCompileRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.compiler.OghamCompileRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<OghamCompileRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut compiler_version__ = None;
                let mut module__ = None;
                let mut options__ = None;
                let mut output_dir__ = None;
                let mut module_path__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CompilerVersion => {
                            if compiler_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("compilerVersion"));
                            }
                            compiler_version__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Module => {
                            if module__.is_some() {
                                return Err(serde::de::Error::duplicate_field("module"));
                            }
                            module__ = map_.next_value()?;
                        }
                        GeneratedField::Options => {
                            if options__.is_some() {
                                return Err(serde::de::Error::duplicate_field("options"));
                            }
                            options__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::OutputDir => {
                            if output_dir__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outputDir"));
                            }
                            output_dir__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ModulePath => {
                            if module_path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("modulePath"));
                            }
                            module_path__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(OghamCompileRequest {
                    compiler_version: compiler_version__.unwrap_or_default(),
                    module: module__,
                    options: options__.unwrap_or_default(),
                    output_dir: output_dir__.unwrap_or_default(),
                    module_path: module_path__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.compiler.OghamCompileRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for OghamCompileResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.files.is_empty() {
            len += 1;
        }
        if !self.errors.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("oghamproto.compiler.OghamCompileResponse", len)?;
        if !self.files.is_empty() {
            struct_ser.serialize_field("files", &self.files)?;
        }
        if !self.errors.is_empty() {
            struct_ser.serialize_field("errors", &self.errors)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OghamCompileResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "files",
            "errors",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Files,
            Errors,
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
                            "files" => Ok(GeneratedField::Files),
                            "errors" => Ok(GeneratedField::Errors),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OghamCompileResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct oghamproto.compiler.OghamCompileResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<OghamCompileResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut files__ = None;
                let mut errors__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Files => {
                            if files__.is_some() {
                                return Err(serde::de::Error::duplicate_field("files"));
                            }
                            files__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Errors => {
                            if errors__.is_some() {
                                return Err(serde::de::Error::duplicate_field("errors"));
                            }
                            errors__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(OghamCompileResponse {
                    files: files__.unwrap_or_default(),
                    errors: errors__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("oghamproto.compiler.OghamCompileResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Severity {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::None => "SEVERITY_NONE",
            Self::Error => "ERROR",
            Self::Warning => "WARNING",
            Self::Info => "INFO",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Severity {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "SEVERITY_NONE",
            "ERROR",
            "WARNING",
            "INFO",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Severity;

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
                    "SEVERITY_NONE" => Ok(Severity::None),
                    "ERROR" => Ok(Severity::Error),
                    "WARNING" => Ok(Severity::Warning),
                    "INFO" => Ok(Severity::Info),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
