/**
 * Code generation utilities — string builder with automatic indentation.
 */

import type { GeneratedFile } from "../oghamproto/compiler/request_pb.ts";

/**
 * A code writer with automatic indentation.
 *
 * @example
 * ```ts
 * const w = new CodeWriter();
 * w.line("package main");
 * w.newline();
 * w.open("func main() {");
 * w.line('fmt.Println("hello")');
 * w.close("}");
 * ```
 */
export class CodeWriter {
  private buf = "";
  private indentLevel = 0;
  private indentStr: string;
  private _imports: string[] = [];

  constructor(indent = "\t") {
    this.indentStr = indent;
  }

  /** Create a writer with N-space indentation. */
  static withSpaces(n: number): CodeWriter {
    return new CodeWriter(" ".repeat(n));
  }

  /** Write an indented line. */
  line(text: string): this {
    this.buf += this.indentStr.repeat(this.indentLevel) + text + "\n";
    return this;
  }

  /** Write a line without indentation. */
  raw(text: string): this {
    this.buf += text + "\n";
    return this;
  }

  /** Write an empty line. */
  newline(): this {
    this.buf += "\n";
    return this;
  }

  /** Increase indentation. */
  indent(): this {
    this.indentLevel++;
    return this;
  }

  /** Decrease indentation. */
  dedent(): this {
    if (this.indentLevel > 0) this.indentLevel--;
    return this;
  }

  /** Write a line and indent. */
  open(text: string): this {
    this.line(text);
    this.indent();
    return this;
  }

  /** Dedent and write a line. */
  close(text: string): this {
    this.dedent();
    this.line(text);
    return this;
  }

  /** Write a comment. */
  comment(text: string, prefix = "//"): this {
    this.line(`${prefix} ${text}`);
    return this;
  }

  /** Track an import path. */
  addImport(path: string): this {
    if (!this._imports.includes(path)) {
      this._imports.push(path);
    }
    return this;
  }

  /** Get tracked imports. */
  get imports(): string[] {
    return [...this._imports];
  }

  /** Get generated code as string. */
  toString(): string {
    return this.buf;
  }

  /** Get generated code as bytes. */
  toBytes(): Uint8Array {
    return new TextEncoder().encode(this.buf);
  }

  /** Convert to a GeneratedFile. */
  toFile(name: string): GeneratedFile {
    return {
      $typeName: "oghamproto.compiler.GeneratedFile",
      name,
      content: this.toBytes(),
      append: false,
    };
  }
}
