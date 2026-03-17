package oghamgen

import (
	"testing"
)

func TestCodeWriter(t *testing.T) {
	w := NewCodeWriter()
	w.Line("package main")
	w.Newline()
	w.Open("func main() {")
	w.Line("fmt.Println(\"hello\")")
	w.Close("}")

	got := w.String()
	want := "package main\n\nfunc main() {\n\tfmt.Println(\"hello\")\n}\n"
	if got != want {
		t.Errorf("got:\n%s\nwant:\n%s", got, want)
	}
}

func TestCodeWriterSpaces(t *testing.T) {
	w := NewCodeWriterSpaces(4)
	w.Open("if true {")
	w.Line("do()")
	w.Close("}")

	got := w.String()
	want := "if true {\n    do()\n}\n"
	if got != want {
		t.Errorf("got:\n%s\nwant:\n%s", got, want)
	}
}

func TestCodeWriterImports(t *testing.T) {
	w := NewCodeWriter()
	w.AddImport("fmt")
	w.AddImport("os")
	w.AddImport("fmt") // duplicate

	if len(w.Imports()) != 2 {
		t.Errorf("expected 2 imports, got %d", len(w.Imports()))
	}
}

func TestCodeWriterToFile(t *testing.T) {
	w := NewCodeWriter()
	w.Line("hello")
	f := w.ToFile("test.go")

	if f.Name != "test.go" {
		t.Errorf("expected name test.go, got %s", f.Name)
	}
	if string(f.Content) != "hello\n" {
		t.Errorf("expected content 'hello\\n', got %q", string(f.Content))
	}
}

func TestToPascalCase(t *testing.T) {
	tests := []struct{ in, want string }{
		{"user_name", "UserName"},
		{"id", "Id"},
		{"created_at", "CreatedAt"},
	}
	for _, tt := range tests {
		if got := ToPascalCase(tt.in); got != tt.want {
			t.Errorf("ToPascalCase(%q) = %q, want %q", tt.in, got, tt.want)
		}
	}
}

func TestToSnakeCase(t *testing.T) {
	tests := []struct{ in, want string }{
		{"UserName", "user_name"},
		{"createdAt", "created_at"},
	}
	for _, tt := range tests {
		if got := ToSnakeCase(tt.in); got != tt.want {
			t.Errorf("ToSnakeCase(%q) = %q, want %q", tt.in, got, tt.want)
		}
	}
}

func TestToCamelCase(t *testing.T) {
	tests := []struct{ in, want string }{
		{"user_name", "userName"},
		{"created_at", "createdAt"},
	}
	for _, tt := range tests {
		if got := ToCamelCase(tt.in); got != tt.want {
			t.Errorf("ToCamelCase(%q) = %q, want %q", tt.in, got, tt.want)
		}
	}
}

func TestToScreamingSnakeCase(t *testing.T) {
	if got := ToScreamingSnakeCase("OrderStatus"); got != "ORDER_STATUS" {
		t.Errorf("got %q, want ORDER_STATUS", got)
	}
}
