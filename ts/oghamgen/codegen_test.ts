import { CodeWriter } from "./codegen.ts";
import assert from "node:assert";
import { describe, it } from "node:test";

describe("CodeWriter", () => {
  it("basic lines", () => {
    const w = new CodeWriter();
    w.line("hello");
    w.line("world");
    assert.strictEqual(w.toString(), "hello\nworld\n");
  });

  it("indentation with spaces", () => {
    const w = CodeWriter.withSpaces(2);
    w.line("func main() {");
    w.indent();
    w.line('fmt.Println("hello")');
    w.dedent();
    w.line("}");
    assert.strictEqual(w.toString(), 'func main() {\n  fmt.Println("hello")\n}\n');
  });

  it("open/close", () => {
    const w = CodeWriter.withSpaces(4);
    w.open("if true {");
    w.line("do_thing()");
    w.close("}");
    assert.strictEqual(w.toString(), "if true {\n    do_thing()\n}\n");
  });

  it("imports deduplicated", () => {
    const w = new CodeWriter();
    w.addImport("fmt");
    w.addImport("os");
    w.addImport("fmt");
    assert.strictEqual(w.imports.length, 2);
  });

  it("toFile", () => {
    const w = new CodeWriter();
    w.line("hello");
    const f = w.toFile("test.go");
    assert.strictEqual(f.name, "test.go");
    assert.strictEqual(new TextDecoder().decode(f.content), "hello\n");
  });
});
