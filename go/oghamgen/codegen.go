package oghamgen

import (
	"fmt"
	"strings"
	"unicode"

	"github.com/oghamlang/go/oghamproto/compiler"
)

// CodeWriter builds generated code with automatic indentation.
type CodeWriter struct {
	buf     strings.Builder
	indent  int
	tab     string
	imports []string
}

// NewCodeWriter creates a writer with tab indentation.
func NewCodeWriter() *CodeWriter {
	return &CodeWriter{tab: "\t"}
}

// NewCodeWriterSpaces creates a writer with N-space indentation.
func NewCodeWriterSpaces(n int) *CodeWriter {
	return &CodeWriter{tab: strings.Repeat(" ", n)}
}

// Line writes an indented line.
func (w *CodeWriter) Line(format string, args ...any) {
	for i := 0; i < w.indent; i++ {
		w.buf.WriteString(w.tab)
	}
	fmt.Fprintf(&w.buf, format, args...)
	w.buf.WriteByte('\n')
}

// Raw writes a line without indentation.
func (w *CodeWriter) Raw(s string) {
	w.buf.WriteString(s)
	w.buf.WriteByte('\n')
}

// Newline writes an empty line.
func (w *CodeWriter) Newline() {
	w.buf.WriteByte('\n')
}

// Indent increases indentation.
func (w *CodeWriter) Indent() { w.indent++ }

// Dedent decreases indentation.
func (w *CodeWriter) Dedent() {
	if w.indent > 0 {
		w.indent--
	}
}

// Open writes a line and indents (e.g., "func main() {").
func (w *CodeWriter) Open(format string, args ...any) {
	w.Line(format, args...)
	w.Indent()
}

// Close dedents and writes a line (e.g., "}").
func (w *CodeWriter) Close(s string) {
	w.Dedent()
	w.Line("%s", s)
}

// Comment writes a comment line.
func (w *CodeWriter) Comment(format string, args ...any) {
	w.Line("// "+format, args...)
}

// AddImport tracks an import path.
func (w *CodeWriter) AddImport(path string) {
	for _, imp := range w.imports {
		if imp == path {
			return
		}
	}
	w.imports = append(w.imports, path)
}

// Imports returns tracked import paths.
func (w *CodeWriter) Imports() []string {
	return w.imports
}

// String returns the generated code.
func (w *CodeWriter) String() string {
	return w.buf.String()
}

// Bytes returns the generated code as bytes.
func (w *CodeWriter) Bytes() []byte {
	return []byte(w.buf.String())
}

// ToFile converts to a GeneratedFile.
func (w *CodeWriter) ToFile(name string) *compiler.GeneratedFile {
	return &compiler.GeneratedFile{
		Name:    name,
		Content: w.Bytes(),
	}
}

// ── Name conversion utilities ──────────────────────────────────────────

// ToPascalCase converts snake_case to PascalCase.
func ToPascalCase(s string) string {
	var result strings.Builder
	upper := true
	for _, r := range s {
		if r == '_' {
			upper = true
			continue
		}
		if upper {
			result.WriteRune(unicode.ToUpper(r))
			upper = false
		} else {
			result.WriteRune(r)
		}
	}
	return result.String()
}

// ToSnakeCase converts PascalCase/camelCase to snake_case.
func ToSnakeCase(s string) string {
	var result strings.Builder
	for i, r := range s {
		if unicode.IsUpper(r) {
			if i > 0 {
				result.WriteByte('_')
			}
			result.WriteRune(unicode.ToLower(r))
		} else {
			result.WriteRune(r)
		}
	}
	return result.String()
}

// ToCamelCase converts snake_case to camelCase.
func ToCamelCase(s string) string {
	p := ToPascalCase(s)
	if len(p) == 0 {
		return p
	}
	return strings.ToLower(p[:1]) + p[1:]
}

// ToScreamingSnakeCase converts to SCREAMING_SNAKE_CASE.
func ToScreamingSnakeCase(s string) string {
	return strings.ToUpper(ToSnakeCase(s))
}
