//! Ogham Language Server — full IDE experience.

mod state;

use dashmap::DashMap;
use ogham_compiler::ast::{self, AstNode};
use ogham_compiler::parser;
use ogham_compiler::syntax_kind::SyntaxKind;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use state::{DocumentState, SymbolDef, WorkspaceIndex};

struct Backend {
    client: Client,
    documents: DashMap<Url, DocumentState>,
    index: WorkspaceIndex,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        // Index embedded std library
        self.index.index_std();

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".into(), "@".into(), ":".into(), "<".into()]),
                    ..Default::default()
                }),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::KEYWORD,
                                    SemanticTokenType::TYPE,
                                    SemanticTokenType::ENUM,
                                    SemanticTokenType::ENUM_MEMBER,
                                    SemanticTokenType::PROPERTY,
                                    SemanticTokenType::METHOD,
                                    SemanticTokenType::NAMESPACE,
                                    SemanticTokenType::STRING,
                                    SemanticTokenType::NUMBER,
                                    SemanticTokenType::COMMENT,
                                    SemanticTokenType::DECORATOR,
                                ],
                                token_modifiers: vec![
                                    SemanticTokenModifier::DEFINITION,
                                    SemanticTokenModifier::DECLARATION,
                                ],
                            },
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: None,
                            ..Default::default()
                        },
                    ),
                ),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "ogham-lsp initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    // ── Document sync ──────────────────────────────────────────────────

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.on_change(params.text_document.uri, params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().next() {
            self.on_change(params.text_document.uri, change.text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri);
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    // ── Hover ──────────────────────────────────────────────────────────

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return Ok(None),
        };

        let offset = position_to_offset(&doc.source, pos);
        let root_node = doc.parse.syntax();

        let token = match root_node.token_at_offset(rowan::TextSize::from(offset as u32)) {
            rowan::TokenAtOffset::Single(t) => t,
            rowan::TokenAtOffset::Between(_, right) => right,
            rowan::TokenAtOffset::None => return Ok(None),
        };

        // First try local AST hover
        if let Some(text) = find_hover_info(&token) {
            return Ok(Some(make_hover(text)));
        }

        // Then try workspace index hover
        let name = token.text().to_string();
        if let Some(def) = self.index.find_definition(&name) {
            return Ok(Some(make_hover(format!(
                "**{}** `{}`\n\n{}",
                symbol_kind_label(def.kind),
                def.name,
                def.detail
            ))));
        }

        Ok(None)
    }

    // ── Go-to-definition (cross-file + std) ────────────────────────────

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return Ok(None),
        };

        let offset = position_to_offset(&doc.source, pos);
        let root_node = doc.parse.syntax();

        let token = match root_node.token_at_offset(rowan::TextSize::from(offset as u32)) {
            rowan::TokenAtOffset::Single(t) => t,
            rowan::TokenAtOffset::Between(_, right) => right,
            rowan::TokenAtOffset::None => return Ok(None),
        };

        let name = token.text().to_string();

        // Search workspace index (includes std + all open files)
        if let Some(def) = self.index.find_definition(&name) {
            let range = byte_range_to_lsp_range_from_uri(&def.uri, &def.range, &self.documents);
            return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                uri: def.uri,
                range,
            })));
        }

        Ok(None)
    }

    // ── Find references ────────────────────────────────────────────────

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = &params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;

        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return Ok(None),
        };

        let offset = position_to_offset(&doc.source, pos);
        let root_node = doc.parse.syntax();

        let token = match root_node.token_at_offset(rowan::TextSize::from(offset as u32)) {
            rowan::TokenAtOffset::Single(t) => t,
            rowan::TokenAtOffset::Between(_, right) => right,
            rowan::TokenAtOffset::None => return Ok(None),
        };

        let name = token.text().to_string();

        // Find all usages across all documents
        let mut locations = Vec::new();
        for entry in self.documents.iter() {
            let doc_uri = entry.key();
            let doc = entry.value();
            find_name_usages(&doc.source, &doc.parse, doc_uri, &name, &mut locations);
        }

        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(locations))
        }
    }

    // ── Document symbols (outline) ─────────────────────────────────────

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;
        let defs = self.index.document_symbols(uri);

        let symbols: Vec<DocumentSymbol> = defs.iter().map(def_to_document_symbol).collect();

        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    // ── Workspace symbols ──────────────────────────────────────────────

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let results = self.index.search(&params.query);

        let symbols: Vec<SymbolInformation> = results
            .iter()
            .filter(|d| !d.uri.as_str().starts_with("ogham-std://"))
            .map(|d| {
                #[allow(deprecated)]
                SymbolInformation {
                    name: d.name.clone(),
                    kind: d.kind,
                    tags: None,
                    deprecated: None,
                    location: Location {
                        uri: d.uri.clone(),
                        range: Range::default(),
                    },
                    container_name: Some(d.detail.clone()),
                }
            })
            .collect();

        Ok(Some(symbols))
    }

    // ── Completion (context-aware) ─────────────────────────────────────

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;

        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return Ok(None),
        };

        let offset = position_to_offset(&doc.source, pos);
        let root_node = doc.parse.syntax();
        let mut items = Vec::new();

        // Detect context
        let context = detect_completion_context(&root_node, offset);

        match context {
            CompletionContext::Annotation => {
                // Offer annotation names from index
                for entry in self.index.symbols.iter() {
                    for def in entry.value().iter() {
                        if def.kind == SymbolKind::PROPERTY {
                            items.push(CompletionItem {
                                label: def.name.clone(),
                                kind: Some(CompletionItemKind::PROPERTY),
                                detail: Some(def.detail.clone()),
                                ..Default::default()
                            });
                        }
                    }
                }
            }
            CompletionContext::TypePosition => {
                // Types, enums, shapes from index
                for entry in self.index.symbols.iter() {
                    for def in entry.value().iter() {
                        if matches!(def.kind, SymbolKind::STRUCT | SymbolKind::ENUM | SymbolKind::INTERFACE) {
                            items.push(CompletionItem {
                                label: def.name.clone(),
                                kind: Some(def.kind.into_completion_kind()),
                                detail: Some(def.detail.clone()),
                                ..Default::default()
                            });
                        }
                    }
                }
                // Built-in types
                for builtin in &[
                    "string", "bytes", "bool", "int32", "int64", "uint32", "uint64",
                    "float", "double", "int", "uint",
                ] {
                    items.push(CompletionItem {
                        label: builtin.to_string(),
                        kind: Some(CompletionItemKind::TYPE_PARAMETER),
                        detail: Some("builtin".into()),
                        ..Default::default()
                    });
                }
            }
            CompletionContext::TopLevel | CompletionContext::Unknown => {
                // All symbols from index
                for entry in self.index.symbols.iter() {
                    for def in entry.value().iter() {
                        items.push(CompletionItem {
                            label: def.name.clone(),
                            kind: Some(def.kind.into_completion_kind()),
                            detail: Some(def.detail.clone()),
                            ..Default::default()
                        });
                    }
                }
                // Keywords
                for kw in &[
                    "type", "enum", "shape", "service", "annotation", "import", "package",
                    "oneof", "rpc", "void", "stream", "map", "Pick", "Omit",
                ] {
                    items.push(CompletionItem {
                        label: kw.to_string(),
                        kind: Some(CompletionItemKind::KEYWORD),
                        ..Default::default()
                    });
                }
            }
        }

        // Deduplicate by label
        items.sort_by(|a, b| a.label.cmp(&b.label));
        items.dedup_by(|a, b| a.label == b.label);

        Ok(Some(CompletionResponse::Array(items)))
    }

    // ── Semantic tokens ────────────────────────────────────────────────

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = &params.text_document.uri;

        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return Ok(None),
        };

        let tokens = compute_semantic_tokens(&doc.source, &doc.parse);

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: tokens,
        })))
    }
}

// ── Core ───────────────────────────────────────────────────────────────

impl Backend {
    async fn on_change(&self, uri: Url, text: String) {
        let parse = parser::parse(&text);

        // Parse diagnostics
        let mut lsp_diags: Vec<Diagnostic> = parse
            .errors
            .iter()
            .map(|err| Diagnostic {
                range: byte_range_to_lsp_range(&text, &err.range),
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("ogham".into()),
                message: err.message.clone(),
                ..Default::default()
            })
            .collect();

        // Semantic diagnostics — run compile pipeline for single file
        let semantic = ogham_compiler::pipeline::compile(&[
            ogham_compiler::pipeline::SourceFile {
                name: uri.path().to_string(),
                content: text.clone(),
            },
        ]);
        for diag in semantic.diagnostics.all() {
            let severity = match diag.severity {
                ogham_compiler::diagnostics::Severity::Error => DiagnosticSeverity::ERROR,
                ogham_compiler::diagnostics::Severity::Warning => DiagnosticSeverity::WARNING,
                ogham_compiler::diagnostics::Severity::Info => DiagnosticSeverity::INFORMATION,
                ogham_compiler::diagnostics::Severity::Note => DiagnosticSeverity::HINT,
            };
            lsp_diags.push(Diagnostic {
                range: byte_range_to_lsp_range(&text, &diag.span()),
                severity: Some(severity),
                source: Some("ogham".into()),
                message: diag.message.clone(),
                ..Default::default()
            });
        }

        // Update index
        self.index.index_document(&uri, &text, &parse);

        self.client
            .publish_diagnostics(uri.clone(), lsp_diags, None)
            .await;

        self.documents
            .insert(uri, DocumentState { source: text, parse });
    }
}

// ── Hover helpers ──────────────────────────────────────────────────────

fn find_hover_info(token: &ogham_compiler::syntax_kind::SyntaxToken) -> Option<String> {
    let mut node = token.parent();
    loop {
        match node {
            Some(ref n) => {
                if let Some(ty) = ast::TypeDecl::cast(n.clone()) {
                    let name = ty.name()?.text().to_string();
                    let params = ty.type_params().map(|tp| {
                        let p: Vec<_> = tp.params().iter().map(|t| t.text().to_string()).collect();
                        format!("<{}>", p.join(", "))
                    }).unwrap_or_default();
                    if ty.alias().is_some() {
                        return Some(format!("**type alias** `{}{}`", name, params));
                    }
                    let fields = ty.body().map(|b| b.fields().len()).unwrap_or(0);
                    return Some(format!("**type** `{}{}` ({} fields)", name, params, fields));
                }
                if let Some(f) = ast::FieldDecl::cast(n.clone()) {
                    let name = f.name()?.text().to_string();
                    let num = f.field_number().unwrap_or(0);
                    return Some(format!("**field** `{}` = {}", name, num));
                }
                if let Some(en) = ast::EnumDecl::cast(n.clone()) {
                    let name = en.name()?.text().to_string();
                    return Some(format!("**enum** `{}` ({} values)", name, en.values().len()));
                }
                if let Some(sh) = ast::ShapeDecl::cast(n.clone()) {
                    let name = sh.name()?.text().to_string();
                    return Some(format!("**shape** `{}`", name));
                }
                if let Some(svc) = ast::ServiceDecl::cast(n.clone()) {
                    let name = svc.name()?.text().to_string();
                    return Some(format!("**service** `{}` ({} rpcs)", name, svc.rpcs().len()));
                }
                if let Some(rpc) = ast::RpcDecl::cast(n.clone()) {
                    let name = rpc.name()?.text().to_string();
                    return Some(format!("**rpc** `{}`", name));
                }
                if let Some(ann) = ast::AnnotationDecl::cast(n.clone()) {
                    let name = ann.name()?.text().to_string();
                    let targets = ann.targets().map(|t| t.targets().join("|")).unwrap_or_default();
                    return Some(format!("**annotation** `{}` for {}", name, targets));
                }
                node = n.parent();
            }
            None => return None,
        }
    }
}

fn make_hover(text: String) -> Hover {
    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: text,
        }),
        range: None,
    }
}

// ── Find references ────────────────────────────────────────────────────

fn find_name_usages(
    source: &str,
    parse: &parser::Parse,
    uri: &Url,
    name: &str,
    locations: &mut Vec<Location>,
) {
    let root = parse.syntax();
    for event in root.preorder_with_tokens() {
        if let rowan::WalkEvent::Enter(rowan::NodeOrToken::Token(tok)) = event {
            if tok.text() == name {
                let range = tok.text_range();
                let span = usize::from(range.start())..usize::from(range.end());
                locations.push(Location {
                    uri: uri.clone(),
                    range: byte_range_to_lsp_range(source, &span),
                });
            }
        }
    }
}

// ── Document symbols ───────────────────────────────────────────────────

#[allow(deprecated)]
fn def_to_document_symbol(def: &SymbolDef) -> DocumentSymbol {
    let range = Range::default(); // simplified — would need source to compute
    let children = if def.children.is_empty() {
        None
    } else {
        Some(def.children.iter().map(def_to_document_symbol).collect())
    };

    DocumentSymbol {
        name: def.name.clone(),
        detail: Some(def.detail.clone()),
        kind: def.kind,
        tags: None,
        deprecated: None,
        range,
        selection_range: range,
        children,
    }
}

// ── Completion context ─────────────────────────────────────────────────

enum CompletionContext {
    Annotation,
    TypePosition,
    TopLevel,
    Unknown,
}

fn detect_completion_context(
    root: &ogham_compiler::syntax_kind::SyntaxNode,
    offset: usize,
) -> CompletionContext {
    let token = match root.token_at_offset(rowan::TextSize::from(offset as u32)) {
        rowan::TokenAtOffset::Single(t) => t,
        rowan::TokenAtOffset::Between(_, right) => right,
        rowan::TokenAtOffset::None => return CompletionContext::Unknown,
    };

    // Check previous tokens for context
    let mut prev = token.prev_token();
    for _ in 0..5 {
        match prev {
            Some(ref t) => {
                let kind = t.kind();
                if kind == SyntaxKind::At || kind == SyntaxKind::ColonColon {
                    return CompletionContext::Annotation;
                }
                if kind == SyntaxKind::LBrace || kind == SyntaxKind::Semicolon {
                    // Inside a body — could be type position
                    return CompletionContext::TypePosition;
                }
                if kind == SyntaxKind::Whitespace || kind == SyntaxKind::LineComment {
                    prev = t.prev_token();
                    continue;
                }
                break;
            }
            None => break,
        }
    }

    CompletionContext::TopLevel
}

// ── Semantic tokens ────────────────────────────────────────────────────

fn compute_semantic_tokens(
    source: &str,
    parse: &parser::Parse,
) -> Vec<SemanticToken> {
    let root = parse.syntax();
    let mut tokens = Vec::new();
    let mut prev_line = 0u32;
    let mut prev_col = 0u32;

    for event in root.preorder_with_tokens() {
        if let rowan::WalkEvent::Enter(rowan::NodeOrToken::Token(tok)) = event {
            let kind = tok.kind();
            let token_type = match kind {
                SyntaxKind::KwType | SyntaxKind::KwEnum | SyntaxKind::KwShape
                | SyntaxKind::KwService | SyntaxKind::KwRpc | SyntaxKind::KwAnnotation
                | SyntaxKind::KwImport | SyntaxKind::KwPackage | SyntaxKind::KwOneof
                | SyntaxKind::KwVoid | SyntaxKind::KwStream | SyntaxKind::KwMap
                | SyntaxKind::KwPick | SyntaxKind::KwOmit | SyntaxKind::KwFor
                | SyntaxKind::KwAs => 0, // KEYWORD

                SyntaxKind::StringLiteral => 7, // STRING
                SyntaxKind::IntLiteral | SyntaxKind::FloatLiteral => 8, // NUMBER
                SyntaxKind::LineComment | SyntaxKind::BlockComment => 9, // COMMENT
                SyntaxKind::At => 10, // DECORATOR

                _ => continue,
            };

            let range = tok.text_range();
            let start_offset = usize::from(range.start());
            let pos = offset_to_position(source, start_offset);
            let length = tok.text().len() as u32;

            let delta_line = pos.line - prev_line;
            let delta_start = if delta_line == 0 {
                pos.character - prev_col
            } else {
                pos.character
            };

            tokens.push(SemanticToken {
                delta_line,
                delta_start,
                length,
                token_type,
                token_modifiers_bitset: 0,
            });

            prev_line = pos.line;
            prev_col = pos.character;
        }
    }

    tokens
}

// ── Position helpers ───────────────────────────────────────────────────

fn byte_range_to_lsp_range(source: &str, range: &std::ops::Range<usize>) -> Range {
    Range {
        start: offset_to_position(source, range.start),
        end: offset_to_position(source, range.end),
    }
}

fn byte_range_to_lsp_range_from_uri(
    uri: &Url,
    range: &std::ops::Range<usize>,
    documents: &DashMap<Url, DocumentState>,
) -> Range {
    if let Some(doc) = documents.get(uri) {
        byte_range_to_lsp_range(&doc.source, range)
    } else {
        Range::default()
    }
}

fn offset_to_position(source: &str, offset: usize) -> Position {
    let offset = offset.min(source.len());
    let prefix = &source[..offset];
    let line = prefix.matches('\n').count() as u32;
    let line_start = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let character = (offset - line_start) as u32;
    Position { line, character }
}

fn position_to_offset(source: &str, pos: Position) -> usize {
    let mut offset = 0;
    for (i, line) in source.lines().enumerate() {
        if i == pos.line as usize {
            return offset + (pos.character as usize).min(line.len());
        }
        offset += line.len() + 1;
    }
    source.len()
}

fn symbol_kind_label(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::STRUCT => "type",
        SymbolKind::ENUM => "enum",
        SymbolKind::INTERFACE => "shape",
        SymbolKind::MODULE => "service",
        SymbolKind::PROPERTY => "annotation",
        SymbolKind::FIELD => "field",
        SymbolKind::METHOD => "rpc",
        _ => "symbol",
    }
}

// ── Extension trait ────────────────────────────────────────────────────

trait SymbolKindExt {
    fn into_completion_kind(self) -> CompletionItemKind;
}

impl SymbolKindExt for SymbolKind {
    fn into_completion_kind(self) -> CompletionItemKind {
        match self {
            SymbolKind::STRUCT => CompletionItemKind::STRUCT,
            SymbolKind::ENUM => CompletionItemKind::ENUM,
            SymbolKind::INTERFACE => CompletionItemKind::INTERFACE,
            SymbolKind::MODULE => CompletionItemKind::MODULE,
            SymbolKind::PROPERTY => CompletionItemKind::PROPERTY,
            SymbolKind::FIELD => CompletionItemKind::FIELD,
            SymbolKind::METHOD => CompletionItemKind::METHOD,
            _ => CompletionItemKind::TEXT,
        }
    }
}

// ── Entry point ────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        documents: DashMap::new(),
        index: WorkspaceIndex::new(),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
