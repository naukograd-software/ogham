//! Pass 12: IR inflation — converts HIR (arena-based) into proto IR (inline).
//!
//! This is the final pass. Every type reference becomes a fully inline
//! `TypeReference` proto message carrying all fields, annotations, etc.

use crate::hir::*;
use ogham_proto::oghamproto::{compiler, ir};

/// Maximum recursion depth for inline type expansion.
const MAX_DEPTH: usize = 8;

/// Convert the resolved HIR into a proto `ir::Module`.
///
/// `module_info` is set on every top-level Type/Enum/Service so plugins know
/// which module each declaration originates from.
pub fn inflate(
    interner: &Interner,
    arenas: &Arenas,
    symbols: &SymbolTable,
    package: &str,
    module_info: Option<ir::ModuleInfo>,
) -> ir::Module {
    let mut ctx = Ctx {
        interner,
        arenas,
        depth: 0,
        module_info,
    };

    let types = symbols
        .types
        .values()
        .map(|&id| ctx.inflate_type(id))
        .collect();

    let enums = symbols
        .enums
        .values()
        .map(|&id| ctx.inflate_enum(id))
        .collect();

    let services = symbols
        .services
        .values()
        .map(|&id| ctx.inflate_service(id))
        .collect();

    ir::Module {
        package: package.to_string(),
        types,
        enums,
        services,
    }
}

struct Ctx<'a> {
    interner: &'a Interner,
    arenas: &'a Arenas,
    depth: usize,
    module_info: Option<ir::ModuleInfo>,
}

impl<'a> Ctx<'a> {
    fn sym(&self, s: Sym) -> String {
        self.interner.resolve(s).to_string()
    }

    fn inflate_annotations(&self, annotations: &[crate::hir::AnnotationCall]) -> Vec<ir::AnnotationCall> {
        annotations
            .iter()
            .map(|ann| ir::AnnotationCall {
                library: self.sym(ann.library),
                name: self.sym(ann.name),
                arguments: ann
                    .arguments
                    .iter()
                    .map(|arg| ir::AnnotationArgument {
                        name: self.sym(arg.name),
                        value: Some(self.inflate_literal(&arg.value)),
                    })
                    .collect(),
                definition: None,
                location: None,
            })
            .collect()
    }

    fn inflate_literal(&self, value: &LiteralValue) -> ir::AnnotationLiteral {
        use ir::annotation_literal::Value;
        let v = match value {
            LiteralValue::String(s) => Value::StringValue(self.sym(*s)),
            LiteralValue::Int(i) => Value::IntValue(*i),
            LiteralValue::Float(f) => Value::FloatValue(*f),
            LiteralValue::Bool(b) => Value::BoolValue(*b),
            LiteralValue::Ident(s) => Value::StringValue(self.sym(*s)),
            LiteralValue::Struct(fields) => {
                let map = fields
                    .iter()
                    .map(|(k, v)| (self.sym(*k), self.inflate_literal(v)))
                    .collect();
                Value::StructValue(ir::AnnotationStruct { fields: map })
            }
            LiteralValue::List(items) => {
                Value::ListValue(ir::AnnotationList {
                    values: items.iter().map(|v| self.inflate_literal(v)).collect(),
                })
            }
        };
        ir::AnnotationLiteral { value: Some(v) }
    }

    fn inflate_type(&mut self, id: TypeId) -> ir::Type {
        let ty = &self.arenas.types[id];
        ir::Type {
            name: self.sym(ty.name),
            full_name: self.sym(ty.full_name),
            fields: ty.fields.iter().map(|f| self.inflate_field(f)).collect(),
            oneofs: ty.oneofs.iter().map(|o| self.inflate_oneof(o)).collect(),
            nested_types: ty
                .nested_types
                .iter()
                .map(|&id| self.inflate_type(id))
                .collect(),
            nested_enums: ty
                .nested_enums
                .iter()
                .map(|&id| self.inflate_enum(id))
                .collect(),
            annotations: self.inflate_annotations(&ty.annotations),
            back_references: ty
                .back_references
                .iter()
                .map(|br| ir::TypeBackRef {
                    referencing_type_name: {
                        let t = &self.arenas.types[br.referencing_type];
                        self.sym(t.name)
                    },
                    referencing_type_full_name: {
                        let t = &self.arenas.types[br.referencing_type];
                        self.sym(t.full_name)
                    },
                    field_name: self.sym(br.field_name),
                })
                .collect(),
            trace: ty.trace.as_ref().map(|t| self.inflate_type_trace(t)),
            location: None,
            module: self.module_info.clone(),
        }
    }

    fn inflate_field(&mut self, f: &FieldDef) -> ir::Field {
        ir::Field {
            name: self.sym(f.name),
            number: f.number,
            r#type: Some(self.inflate_resolved_type(&f.ty)),
            is_optional: f.is_optional,
            is_repeated: f.is_repeated,
            annotations: self.inflate_annotations(&f.annotations),
            mapping: f.mapping.as_ref().map(|m| self.inflate_mapping(m)),
            trace: f.trace.as_ref().map(|t| self.inflate_field_trace(t)),
            location: None,
        }
    }

    fn inflate_oneof(&mut self, o: &OneofDef) -> ir::OneofGroup {
        ir::OneofGroup {
            name: self.sym(o.name),
            fields: o
                .fields
                .iter()
                .map(|f| ir::OneofField {
                    name: self.sym(f.name),
                    number: f.number,
                    r#type: Some(self.inflate_resolved_type(&f.ty)),
                    annotations: self.inflate_annotations(&f.annotations),
                    mapping: f.mapping.as_ref().map(|m| self.inflate_mapping(m)),
                    location: None,
                })
                .collect(),
            annotations: self.inflate_annotations(&o.annotations),
            location: None,
        }
    }

    fn inflate_enum(&self, id: EnumId) -> ir::Enum {
        let e = &self.arenas.enums[id];
        ir::Enum {
            name: self.sym(e.name),
            full_name: self.sym(e.full_name),
            values: e
                .values
                .iter()
                .map(|v| ir::EnumValue {
                    name: self.sym(v.name),
                    number: v.number,
                    is_removed: false,
                    fallback: String::new(),
                    annotations: self.inflate_annotations(&v.annotations),
                    location: None,
                })
                .collect(),
            annotations: self.inflate_annotations(&e.annotations),
            location: None,
            module: self.module_info.clone(),
        }
    }

    fn inflate_service(&mut self, id: ServiceId) -> ir::Service {
        let svc = &self.arenas.services[id];
        ir::Service {
            name: self.sym(svc.name),
            full_name: self.sym(svc.full_name),
            rpcs: svc
                .rpcs
                .iter()
                .map(|r| ir::Rpc {
                    name: self.sym(r.name),
                    input: Some(self.inflate_rpc_param(&r.input)),
                    output: Some(self.inflate_rpc_param(&r.output)),
                    annotations: self.inflate_annotations(&r.annotations),
                    location: None,
                })
                .collect(),
            annotations: self.inflate_annotations(&svc.annotations),
            location: None,
            module: self.module_info.clone(),
        }
    }

    fn inflate_rpc_param(&mut self, p: &RpcParamDef) -> ir::RpcParam {
        ir::RpcParam {
            is_void: p.is_void,
            is_stream: p.is_stream,
            r#type: if p.is_void {
                None
            } else {
                Some(self.inflate_resolved_type(&p.ty))
            },
        }
    }

    fn inflate_resolved_type(&mut self, ty: &ResolvedType) -> ir::TypeReference {
        use ir::type_reference::Kind;

        let kind = match ty {
            ResolvedType::Scalar(sk) => Kind::Scalar(ir::ScalarType {
                scalar_kind: scalar_to_proto(*sk) as i32,
            }),
            ResolvedType::Message(id) => {
                if self.depth >= MAX_DEPTH {
                    let t = &self.arenas.types[*id];
                    let anns = self.inflate_annotations(&t.annotations);
                    Kind::MessageType(ir::MessageType {
                        name: self.sym(t.name),
                        full_name: self.sym(t.full_name),
                        fields: Vec::new(),
                        oneofs: Vec::new(),
                        nested_enums: Vec::new(),
                        annotations: anns,
                    })
                } else {
                    self.depth += 1;
                    let t = &self.arenas.types[*id];
                    let anns = self.inflate_annotations(&t.annotations);
                    let fields: Vec<_> = t.fields.iter().map(|f| self.inflate_field(f)).collect();
                    let oneofs: Vec<_> = t.oneofs.iter().map(|o| self.inflate_oneof(o)).collect();
                    let nested_enums: Vec<_> = t.nested_enums.iter().map(|&eid| self.inflate_enum(eid)).collect();
                    let msg = ir::MessageType {
                        name: self.sym(t.name),
                        full_name: self.sym(t.full_name),
                        fields,
                        oneofs,
                        nested_enums,
                        annotations: anns,
                    };
                    self.depth -= 1;
                    Kind::MessageType(msg)
                }
            }
            ResolvedType::Enum(id) => {
                let e = &self.arenas.enums[*id];
                Kind::EnumType(ir::EnumType {
                    name: self.sym(e.name),
                    full_name: self.sym(e.full_name),
                    values: e
                        .values
                        .iter()
                        .map(|v| ir::EnumValue {
                            name: self.sym(v.name),
                            number: v.number,
                            is_removed: false,
                            fallback: String::new(),
                            annotations: self.inflate_annotations(&v.annotations),
                            location: None,
                        })
                        .collect(),
                })
            }
            ResolvedType::Map { key, value } => Kind::Map(Box::new(ir::MapType {
                key: Some(Box::new(self.inflate_resolved_type(key))),
                value: Some(Box::new(self.inflate_resolved_type(value))),
            })),
            ResolvedType::Array(inner) => {
                // Array is represented as the inner type with is_repeated=true on the field
                return self.inflate_resolved_type(inner);
            }
            ResolvedType::Unresolved(_) | ResolvedType::Error => {
                Kind::Scalar(ir::ScalarType {
                    scalar_kind: ir::ScalarKind::None as i32,
                })
            }
        };

        ir::TypeReference { kind: Some(kind) }
    }

    fn inflate_mapping(&self, m: &FieldMapping) -> ir::FieldMapping {
        ir::FieldMapping {
            chain: m
                .chain
                .iter()
                .map(|link| ir::MappingLink {
                    source_type_name: String::new(),
                    source_type_full_name: String::new(),
                    source_field_name: self.sym(link.source_field_name),
                    path: link.path.iter().map(|&s| self.sym(s)).collect(),
                    source_field_type: None,
                    source_field_annotations: Vec::new(),
                })
                .collect(),
        }
    }

    fn inflate_type_trace(&self, t: &TypeTrace) -> ir::TypeTrace {
        ir::TypeTrace {
            origin: Some(match t {
                TypeTrace::Generic {
                    source_name,
                    type_arguments,
                } => ir::type_trace::Origin::Generic(ir::GenericOrigin {
                    source_name: self.sym(*source_name),
                    type_arguments: type_arguments.iter().map(|&s| self.sym(s)).collect(),
                }),
                TypeTrace::PickOmit {
                    kind,
                    source_type: _,
                    field_names,
                } => ir::type_trace::Origin::PickOmit(ir::PickOmitOrigin {
                    kind: self.sym(*kind),
                    source_type_name: String::new(),
                    field_names: field_names.iter().map(|&s| self.sym(s)).collect(),
                }),
            }),
        }
    }

    fn inflate_field_trace(&self, t: &FieldTrace) -> ir::FieldTrace {
        ir::FieldTrace {
            shape: t.shape.as_ref().map(|s| ir::ShapeOrigin {
                shape_name: self.sym(s.shape_name),
                shape_full_name: String::new(),
                injection_range_start: s.range_start,
                injection_range_end: s.range_end,
                shape_location: None,
            }),
        }
    }
}

fn scalar_to_proto(sk: ScalarKind) -> ir::ScalarKind {
    match sk {
        ScalarKind::Bool => ir::ScalarKind::Bool,
        ScalarKind::String => ir::ScalarKind::String,
        ScalarKind::Bytes => ir::ScalarKind::Bytes,
        ScalarKind::Int8 => ir::ScalarKind::Int8,
        ScalarKind::Int16 => ir::ScalarKind::Int16,
        ScalarKind::Int32 => ir::ScalarKind::Int32,
        ScalarKind::Int64 => ir::ScalarKind::Int64,
        ScalarKind::Uint8 => ir::ScalarKind::Uint8,
        ScalarKind::Uint16 => ir::ScalarKind::Uint16,
        ScalarKind::Uint32 => ir::ScalarKind::Uint32,
        ScalarKind::Uint64 => ir::ScalarKind::Uint64,
        ScalarKind::Float => ir::ScalarKind::Float,
        ScalarKind::Double => ir::ScalarKind::Double,
    }
}

/// Build an `OghamCompileRequest` proto from the compiled module.
pub fn build_request(
    module: ir::Module,
    compiler_version: &str,
    options: std::collections::HashMap<String, String>,
    output_dir: &str,
    module_path: &str,
) -> compiler::OghamCompileRequest {
    compiler::OghamCompileRequest {
        compiler_version: compiler_version.to_string(),
        module: Some(module),
        options,
        output_dir: output_dir.to_string(),
        module_path: module_path.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline;

    fn compile_and_inflate(source: &str) -> ir::Module {
        let result = pipeline::compile(&[pipeline::SourceFile {
            name: "test.ogham".to_string(),
            content: source.to_string(),
        }]);
        assert!(
            !result.diagnostics.has_errors(),
            "errors: {:?}",
            result.diagnostics.all()
        );
        inflate(&result.interner, &result.arenas, &result.symbols, "example", None)
    }

    #[test]
    fn inflate_simple_type() {
        let module = compile_and_inflate(
            "package example;\ntype User { string email = 1; int64 age = 2; }",
        );
        assert_eq!(module.types.len(), 1);
        assert_eq!(module.types[0].name, "User");
        assert_eq!(module.types[0].full_name, "example.User");
        assert_eq!(module.types[0].fields.len(), 2);
        assert_eq!(module.types[0].fields[0].name, "email");
        assert_eq!(module.types[0].fields[0].number, 1);
    }

    #[test]
    fn inflate_enum() {
        let module = compile_and_inflate(
            "package example;\nenum Status { Active = 1; Inactive = 2; }",
        );
        assert_eq!(module.enums.len(), 1);
        assert_eq!(module.enums[0].name, "Status");
        // Unspecified=0 + Active=1 + Inactive=2
        assert_eq!(module.enums[0].values.len(), 3);
        assert_eq!(module.enums[0].values[0].name, "Unspecified");
        assert_eq!(module.enums[0].values[0].number, 0);
    }

    #[test]
    fn inflate_service() {
        let module = compile_and_inflate(
            "package example;\ntype User { string name = 1; }\nservice UserAPI { rpc Get(void) -> User; }",
        );
        assert_eq!(module.services.len(), 1);
        assert_eq!(module.services[0].rpcs.len(), 1);
        assert_eq!(module.services[0].rpcs[0].name, "Get");
        let input = module.services[0].rpcs[0].input.as_ref().unwrap();
        assert!(input.is_void);
        let output = module.services[0].rpcs[0].output.as_ref().unwrap();
        assert!(!output.is_void);
        // Output type should be inline User
        assert!(output.r#type.is_some());
    }

    #[test]
    fn inflate_inline_message_type() {
        let module = compile_and_inflate(
            r#"package example;
type Address { string city = 1; string zip = 2; }
type User { Address home = 1; string name = 2; }
"#,
        );
        // User.home field should have an inline MessageType with Address fields
        let user = module.types.iter().find(|t| t.name == "User").unwrap();
        let home_field = &user.fields[0];
        let type_ref = home_field.r#type.as_ref().unwrap();
        match &type_ref.kind {
            Some(ir::type_reference::Kind::MessageType(msg)) => {
                assert_eq!(msg.name, "Address");
                assert_eq!(msg.fields.len(), 2);
            }
            other => panic!("expected MessageType, got {:?}", other),
        }
    }

    #[test]
    fn inflate_back_references() {
        let module = compile_and_inflate(
            r#"package example;
type Address { string city = 1; }
type User { Address home = 1; }
type Order { Address billing = 1; }
"#,
        );
        let addr = module.types.iter().find(|t| t.name == "Address").unwrap();
        assert_eq!(addr.back_references.len(), 2);
    }

    #[test]
    fn inflate_map_field() {
        let module = compile_and_inflate(
            "package example;\ntype User { map<string, int64> scores = 1; }",
        );
        let field = &module.types[0].fields[0];
        match &field.r#type.as_ref().unwrap().kind {
            Some(ir::type_reference::Kind::Map(m)) => {
                assert!(m.key.is_some());
                assert!(m.value.is_some());
            }
            other => panic!("expected Map, got {:?}", other),
        }
    }

    #[test]
    fn inflate_compile_request() {
        let module = compile_and_inflate(
            "package example;\ntype User { string name = 1; }",
        );
        let req = build_request(module, "0.1.0", Default::default(), "gen/", "github.com/test/example");
        assert_eq!(req.compiler_version, "0.1.0");
        assert_eq!(req.output_dir, "gen/");
        assert!(req.module.is_some());
    }
}
