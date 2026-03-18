//! Breaking change detection between two compiled IR modules.
//!
//! Compares old vs new `ir::Module` and reports violations at three severity levels:
//! - ERROR: wire-breaking (field number changed, type changed, field removed)
//! - WARNING: JSON/codegen-breaking (field renamed, optional changed, rpc renamed)
//! - INFO: safe changes (new fields, new types, annotation changes)

use ogham_proto::oghamproto::ir;
use std::collections::HashMap;

// ── Violation types ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Level {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct Violation {
    pub level: Level,
    pub code: &'static str,
    pub message: String,
    pub context: String, // "User.email", "OrderStatus.Refunded", "UserAPI.GetUser"
}

// ── Public API ─────────────────────────────────────────────────────────

/// Compare two modules and return all violations.
pub fn compare(old: &ir::Module, new: &ir::Module) -> Vec<Violation> {
    let mut violations = Vec::new();

    compare_types(old, new, &mut violations);
    compare_enums(old, new, &mut violations);
    compare_services(old, new, &mut violations);

    violations
}

// ── Type comparison ────────────────────────────────────────────────────

fn compare_types(old: &ir::Module, new: &ir::Module, out: &mut Vec<Violation>) {
    let old_map: HashMap<&str, &ir::Type> = old.types.iter().map(|t| (t.full_name.as_str(), t)).collect();
    let new_map: HashMap<&str, &ir::Type> = new.types.iter().map(|t| (t.full_name.as_str(), t)).collect();

    // Removed types
    for name in old_map.keys() {
        if !new_map.contains_key(name) {
            out.push(Violation {
                level: Level::Error,
                code: "B001",
                message: format!("type removed: {}", name),
                context: name.to_string(),
            });
        }
    }

    // Added types
    for name in new_map.keys() {
        if !old_map.contains_key(name) {
            out.push(Violation {
                level: Level::Info,
                code: "B002",
                message: format!("type added: {}", name),
                context: name.to_string(),
            });
        }
    }

    // Changed types
    for (name, old_type) in &old_map {
        if let Some(new_type) = new_map.get(name) {
            compare_type_fields(name, old_type, new_type, out);
            compare_type_oneofs(name, old_type, new_type, out);
        }
    }
}

fn compare_type_fields(
    type_name: &str,
    old_type: &ir::Type,
    new_type: &ir::Type,
    out: &mut Vec<Violation>,
) {
    let old_by_number: HashMap<u32, &ir::Field> = old_type.fields.iter().map(|f| (f.number, f)).collect();
    let new_by_number: HashMap<u32, &ir::Field> = new_type.fields.iter().map(|f| (f.number, f)).collect();
    let old_by_name: HashMap<&str, &ir::Field> = old_type.fields.iter().map(|f| (f.name.as_str(), f)).collect();
    let new_by_name: HashMap<&str, &ir::Field> = new_type.fields.iter().map(|f| (f.name.as_str(), f)).collect();

    // Fields removed by number
    for (num, old_field) in &old_by_number {
        if !new_by_number.contains_key(num) {
            // Check if it was just renamed (same number exists with different name)
            out.push(Violation {
                level: Level::Error,
                code: "B010",
                message: format!("field removed: {}.{} (= {})", type_name, old_field.name, num),
                context: format!("{}.{}", type_name, old_field.name),
            });
        }
    }

    // Fields added
    for (num, new_field) in &new_by_number {
        if !old_by_number.contains_key(num) {
            out.push(Violation {
                level: Level::Info,
                code: "B011",
                message: format!("field added: {}.{} (= {})", type_name, new_field.name, num),
                context: format!("{}.{}", type_name, new_field.name),
            });
        }
    }

    // Fields with same number — check for breaking changes
    for (num, old_field) in &old_by_number {
        if let Some(new_field) = new_by_number.get(num) {
            let ctx = format!("{}.{}", type_name, new_field.name);

            // Name changed (same number)
            if old_field.name != new_field.name {
                out.push(Violation {
                    level: Level::Warning,
                    code: "B012",
                    message: format!(
                        "field renamed: {}.{} → {} (= {})",
                        type_name, old_field.name, new_field.name, num
                    ),
                    context: ctx.clone(),
                });
            }

            // Wire type changed
            let old_wire = wire_type_of(old_field.r#type.as_ref());
            let new_wire = wire_type_of(new_field.r#type.as_ref());
            if old_wire != new_wire {
                out.push(Violation {
                    level: Level::Error,
                    code: "B013",
                    message: format!(
                        "field type changed: {} (= {}): {} → {}",
                        ctx, num, old_wire, new_wire
                    ),
                    context: ctx.clone(),
                });
            }

            // Repeated changed
            if old_field.is_repeated != new_field.is_repeated {
                out.push(Violation {
                    level: Level::Error,
                    code: "B014",
                    message: format!(
                        "field repeated changed: {} (= {}): {} → {}",
                        ctx, num, old_field.is_repeated, new_field.is_repeated
                    ),
                    context: ctx.clone(),
                });
            }

            // Optional changed
            if old_field.is_optional != new_field.is_optional {
                out.push(Violation {
                    level: Level::Warning,
                    code: "B015",
                    message: format!(
                        "field optionality changed: {} (= {}): optional={} → optional={}",
                        ctx, num, old_field.is_optional, new_field.is_optional
                    ),
                    context: ctx.clone(),
                });
            }
        }
    }

    // Check for number reuse (old name→number mapping vs new)
    for (name, old_field) in &old_by_name {
        if let Some(new_field) = new_by_name.get(name) {
            if old_field.number != new_field.number {
                out.push(Violation {
                    level: Level::Error,
                    code: "B016",
                    message: format!(
                        "field number changed: {}.{}: {} → {}",
                        type_name, name, old_field.number, new_field.number
                    ),
                    context: format!("{}.{}", type_name, name),
                });
            }
        }
    }
}

fn compare_type_oneofs(
    type_name: &str,
    old_type: &ir::Type,
    new_type: &ir::Type,
    out: &mut Vec<Violation>,
) {
    let old_map: HashMap<&str, &ir::OneofGroup> = old_type.oneofs.iter().map(|o| (o.name.as_str(), o)).collect();
    let new_map: HashMap<&str, &ir::OneofGroup> = new_type.oneofs.iter().map(|o| (o.name.as_str(), o)).collect();

    for name in old_map.keys() {
        if !new_map.contains_key(name) {
            out.push(Violation {
                level: Level::Error,
                code: "B020",
                message: format!("oneof removed: {}.{}", type_name, name),
                context: format!("{}.{}", type_name, name),
            });
        }
    }

    for (name, old_oneof) in &old_map {
        if let Some(new_oneof) = new_map.get(name) {
            let old_fields: HashMap<u32, &ir::OneofField> =
                old_oneof.fields.iter().map(|f| (f.number, f)).collect();
            let new_fields: HashMap<u32, &ir::OneofField> =
                new_oneof.fields.iter().map(|f| (f.number, f)).collect();

            for (num, old_f) in &old_fields {
                if !new_fields.contains_key(num) {
                    out.push(Violation {
                        level: Level::Error,
                        code: "B021",
                        message: format!(
                            "oneof field removed: {}.{}.{} (= {})",
                            type_name, name, old_f.name, num
                        ),
                        context: format!("{}.{}.{}", type_name, name, old_f.name),
                    });
                }
            }
        }
    }
}

// ── Enum comparison ────────────────────────────────────────────────────

fn compare_enums(old: &ir::Module, new: &ir::Module, out: &mut Vec<Violation>) {
    let old_map: HashMap<&str, &ir::Enum> = old.enums.iter().map(|e| (e.full_name.as_str(), e)).collect();
    let new_map: HashMap<&str, &ir::Enum> = new.enums.iter().map(|e| (e.full_name.as_str(), e)).collect();

    for name in old_map.keys() {
        if !new_map.contains_key(name) {
            out.push(Violation {
                level: Level::Error,
                code: "B030",
                message: format!("enum removed: {}", name),
                context: name.to_string(),
            });
        }
    }

    for name in new_map.keys() {
        if !old_map.contains_key(name) {
            out.push(Violation {
                level: Level::Info,
                code: "B031",
                message: format!("enum added: {}", name),
                context: name.to_string(),
            });
        }
    }

    for (name, old_enum) in &old_map {
        if let Some(new_enum) = new_map.get(name) {
            let old_vals: HashMap<i32, &ir::EnumValue> =
                old_enum.values.iter().map(|v| (v.number, v)).collect();
            let new_vals: HashMap<i32, &ir::EnumValue> =
                new_enum.values.iter().map(|v| (v.number, v)).collect();

            // Removed values
            for (num, old_val) in &old_vals {
                if !new_vals.contains_key(num) {
                    out.push(Violation {
                        level: Level::Error,
                        code: "B032",
                        message: format!("enum value removed: {}.{} (= {})", name, old_val.name, num),
                        context: format!("{}.{}", name, old_val.name),
                    });
                }
            }

            // Added values
            for (num, new_val) in &new_vals {
                if !old_vals.contains_key(num) {
                    out.push(Violation {
                        level: Level::Info,
                        code: "B033",
                        message: format!("enum value added: {}.{} (= {})", name, new_val.name, num),
                        context: format!("{}.{}", name, new_val.name),
                    });
                }
            }

            // Renamed values
            for (num, old_val) in &old_vals {
                if let Some(new_val) = new_vals.get(num) {
                    if old_val.name != new_val.name {
                        out.push(Violation {
                            level: Level::Warning,
                            code: "B034",
                            message: format!(
                                "enum value renamed: {}: {} → {} (= {})",
                                name, old_val.name, new_val.name, num
                            ),
                            context: format!("{}.{}", name, new_val.name),
                        });
                    }
                }
            }
        }
    }
}

// ── Service comparison ─────────────────────────────────────────────────

fn compare_services(old: &ir::Module, new: &ir::Module, out: &mut Vec<Violation>) {
    let old_map: HashMap<&str, &ir::Service> = old.services.iter().map(|s| (s.full_name.as_str(), s)).collect();
    let new_map: HashMap<&str, &ir::Service> = new.services.iter().map(|s| (s.full_name.as_str(), s)).collect();

    for name in old_map.keys() {
        if !new_map.contains_key(name) {
            out.push(Violation {
                level: Level::Error,
                code: "B040",
                message: format!("service removed: {}", name),
                context: name.to_string(),
            });
        }
    }

    for (name, old_svc) in &old_map {
        if let Some(new_svc) = new_map.get(name) {
            let old_rpcs: HashMap<&str, &ir::Rpc> =
                old_svc.rpcs.iter().map(|r| (r.name.as_str(), r)).collect();
            let new_rpcs: HashMap<&str, &ir::Rpc> =
                new_svc.rpcs.iter().map(|r| (r.name.as_str(), r)).collect();

            // Removed RPCs
            for rpc_name in old_rpcs.keys() {
                if !new_rpcs.contains_key(rpc_name) {
                    out.push(Violation {
                        level: Level::Warning,
                        code: "B041",
                        message: format!("rpc removed: {}.{}", name, rpc_name),
                        context: format!("{}.{}", name, rpc_name),
                    });
                }
            }

            // Added RPCs
            for rpc_name in new_rpcs.keys() {
                if !old_rpcs.contains_key(rpc_name) {
                    out.push(Violation {
                        level: Level::Info,
                        code: "B042",
                        message: format!("rpc added: {}.{}", name, rpc_name),
                        context: format!("{}.{}", name, rpc_name),
                    });
                }
            }

            // Changed RPCs
            for (rpc_name, old_rpc) in &old_rpcs {
                if let Some(new_rpc) = new_rpcs.get(rpc_name) {
                    let ctx = format!("{}.{}", name, rpc_name);

                    // Input type changed
                    let old_in = rpc_param_sig(old_rpc.input.as_ref());
                    let new_in = rpc_param_sig(new_rpc.input.as_ref());
                    if old_in != new_in {
                        out.push(Violation {
                            level: Level::Error,
                            code: "B043",
                            message: format!("rpc input changed: {}: {} → {}", ctx, old_in, new_in),
                            context: ctx.clone(),
                        });
                    }

                    // Output type changed
                    let old_out = rpc_param_sig(old_rpc.output.as_ref());
                    let new_out = rpc_param_sig(new_rpc.output.as_ref());
                    if old_out != new_out {
                        out.push(Violation {
                            level: Level::Error,
                            code: "B044",
                            message: format!("rpc output changed: {}: {} → {}", ctx, old_out, new_out),
                            context: ctx.clone(),
                        });
                    }

                    // Stream modifier changed
                    let old_in_stream = old_rpc.input.as_ref().is_some_and(|p| p.is_stream);
                    let new_in_stream = new_rpc.input.as_ref().is_some_and(|p| p.is_stream);
                    let old_out_stream = old_rpc.output.as_ref().is_some_and(|p| p.is_stream);
                    let new_out_stream = new_rpc.output.as_ref().is_some_and(|p| p.is_stream);

                    if old_in_stream != new_in_stream || old_out_stream != new_out_stream {
                        out.push(Violation {
                            level: Level::Error,
                            code: "B045",
                            message: format!("rpc streaming changed: {}", ctx),
                            context: ctx.clone(),
                        });
                    }
                }
            }
        }
    }
}

// ── Helpers ────────────────────────────────────────────────────────────

/// Simplified wire type string for comparison.
fn wire_type_of(tr: Option<&ir::TypeReference>) -> String {
    let tr = match tr {
        Some(t) => t,
        None => return "none".to_string(),
    };
    match &tr.kind {
        Some(ir::type_reference::Kind::Scalar(s)) => format!("scalar({})", s.scalar_kind),
        Some(ir::type_reference::Kind::MessageType(m)) => format!("message({})", m.full_name),
        Some(ir::type_reference::Kind::EnumType(e)) => format!("enum({})", e.full_name),
        Some(ir::type_reference::Kind::Map(m)) => {
            format!(
                "map({},{})",
                wire_type_of(m.key.as_deref()),
                wire_type_of(m.value.as_deref())
            )
        }
        None => "none".to_string(),
    }
}

fn rpc_param_sig(param: Option<&ir::RpcParam>) -> String {
    match param {
        None => "none".to_string(),
        Some(p) if p.is_void => "void".to_string(),
        Some(p) => {
            let prefix = if p.is_stream { "stream " } else { "" };
            format!("{}{}", prefix, wire_type_of(p.r#type.as_ref()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_module(types: Vec<ir::Type>, enums: Vec<ir::Enum>, services: Vec<ir::Service>) -> ir::Module {
        ir::Module {
            package: "test".to_string(),
            types,
            enums,
            services,
        }
    }

    fn make_field(name: &str, number: u32, scalar_kind: i32) -> ir::Field {
        ir::Field {
            name: name.to_string(),
            number,
            r#type: Some(ir::TypeReference {
                kind: Some(ir::type_reference::Kind::Scalar(ir::ScalarType { scalar_kind })),
            }),
            is_optional: false,
            is_repeated: false,
            annotations: Vec::new(),
            mapping: None,
            trace: None,
            location: None,
        }
    }

    fn make_type(name: &str, fields: Vec<ir::Field>) -> ir::Type {
        ir::Type {
            name: name.to_string(),
            full_name: format!("test.{}", name),
            fields,
            oneofs: Vec::new(),
            nested_types: Vec::new(),
            nested_enums: Vec::new(),
            annotations: Vec::new(),
            back_references: Vec::new(),
            trace: None,
            location: None,
            module: None,
        }
    }

    fn make_enum_val(name: &str, number: i32) -> ir::EnumValue {
        ir::EnumValue {
            name: name.to_string(),
            number,
            is_removed: false,
            fallback: String::new(),
            annotations: Vec::new(),
            location: None,
        }
    }

    #[test]
    fn no_changes() {
        let old = make_module(
            vec![make_type("User", vec![make_field("email", 1, 2)])],
            Vec::new(),
            Vec::new(),
        );
        let new = old.clone();
        let violations = compare(&old, &new);
        assert!(violations.is_empty());
    }

    #[test]
    fn added_field_is_info() {
        let old = make_module(
            vec![make_type("User", vec![make_field("email", 1, 2)])],
            Vec::new(),
            Vec::new(),
        );
        let new = make_module(
            vec![make_type("User", vec![
                make_field("email", 1, 2),
                make_field("name", 2, 2),
            ])],
            Vec::new(),
            Vec::new(),
        );
        let violations = compare(&old, &new);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].level, Level::Info);
        assert_eq!(violations[0].code, "B011");
    }

    #[test]
    fn removed_field_is_error() {
        let old = make_module(
            vec![make_type("User", vec![
                make_field("email", 1, 2),
                make_field("name", 2, 2),
            ])],
            Vec::new(),
            Vec::new(),
        );
        let new = make_module(
            vec![make_type("User", vec![make_field("email", 1, 2)])],
            Vec::new(),
            Vec::new(),
        );
        let violations = compare(&old, &new);
        assert!(violations.iter().any(|v| v.level == Level::Error && v.code == "B010"));
    }

    #[test]
    fn field_number_changed_is_error() {
        let old = make_module(
            vec![make_type("User", vec![make_field("email", 1, 2)])],
            Vec::new(),
            Vec::new(),
        );
        let new = make_module(
            vec![make_type("User", vec![make_field("email", 5, 2)])],
            Vec::new(),
            Vec::new(),
        );
        let violations = compare(&old, &new);
        assert!(violations.iter().any(|v| v.level == Level::Error && v.code == "B016"));
    }

    #[test]
    fn field_type_changed_is_error() {
        let old = make_module(
            vec![make_type("User", vec![make_field("age", 1, 2)])], // string
            Vec::new(),
            Vec::new(),
        );
        let new = make_module(
            vec![make_type("User", vec![make_field("age", 1, 6)])], // int32
            Vec::new(),
            Vec::new(),
        );
        let violations = compare(&old, &new);
        assert!(violations.iter().any(|v| v.level == Level::Error && v.code == "B013"));
    }

    #[test]
    fn field_renamed_is_warning() {
        let old = make_module(
            vec![make_type("User", vec![make_field("name", 1, 2)])],
            Vec::new(),
            Vec::new(),
        );
        let new = make_module(
            vec![make_type("User", vec![make_field("full_name", 1, 2)])],
            Vec::new(),
            Vec::new(),
        );
        let violations = compare(&old, &new);
        assert!(violations.iter().any(|v| v.level == Level::Warning && v.code == "B012"));
    }

    #[test]
    fn type_removed_is_error() {
        let old = make_module(
            vec![make_type("User", vec![make_field("email", 1, 2)])],
            Vec::new(),
            Vec::new(),
        );
        let new = make_module(Vec::new(), Vec::new(), Vec::new());
        let violations = compare(&old, &new);
        assert!(violations.iter().any(|v| v.level == Level::Error && v.code == "B001"));
    }

    #[test]
    fn type_added_is_info() {
        let old = make_module(Vec::new(), Vec::new(), Vec::new());
        let new = make_module(
            vec![make_type("User", vec![make_field("email", 1, 2)])],
            Vec::new(),
            Vec::new(),
        );
        let violations = compare(&old, &new);
        assert!(violations.iter().any(|v| v.level == Level::Info && v.code == "B002"));
    }

    #[test]
    fn enum_value_removed_is_error() {
        let old = make_module(
            Vec::new(),
            vec![ir::Enum {
                name: "Status".to_string(),
                full_name: "test.Status".to_string(),
                values: vec![make_enum_val("Unspecified", 0), make_enum_val("Active", 1), make_enum_val("Deleted", 2)],
                annotations: Vec::new(),
                location: None,
                module: None,
            }],
            Vec::new(),
        );
        let new = make_module(
            Vec::new(),
            vec![ir::Enum {
                name: "Status".to_string(),
                full_name: "test.Status".to_string(),
                values: vec![make_enum_val("Unspecified", 0), make_enum_val("Active", 1)],
                annotations: Vec::new(),
                location: None,
                module: None,
            }],
            Vec::new(),
        );
        let violations = compare(&old, &new);
        assert!(violations.iter().any(|v| v.level == Level::Error && v.code == "B032"));
    }

    #[test]
    fn enum_value_renamed_is_warning() {
        let old = make_module(
            Vec::new(),
            vec![ir::Enum {
                name: "Status".to_string(),
                full_name: "test.Status".to_string(),
                values: vec![make_enum_val("Unspecified", 0), make_enum_val("Active", 1)],
                annotations: Vec::new(),
                location: None,
                module: None,
            }],
            Vec::new(),
        );
        let new = make_module(
            Vec::new(),
            vec![ir::Enum {
                name: "Status".to_string(),
                full_name: "test.Status".to_string(),
                values: vec![make_enum_val("Unspecified", 0), make_enum_val("Enabled", 1)],
                annotations: Vec::new(),
                location: None,
                module: None,
            }],
            Vec::new(),
        );
        let violations = compare(&old, &new);
        assert!(violations.iter().any(|v| v.level == Level::Warning && v.code == "B034"));
    }
}
