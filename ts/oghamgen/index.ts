/**
 * Ogham Plugin SDK for TypeScript.
 *
 * Build code generation plugins for the Ogham schema language.
 *
 * @example
 * ```ts
 * import { run } from "@ogham/sdk/oghamgen";
 * import { OghamCompileRequestSchema } from "@ogham/sdk/oghamproto/compiler/request_pb.ts";
 *
 * run((req) => {
 *   const files = [];
 *   for (const type of req.module?.types ?? []) {
 *     files.push({
 *       name: `${type.name}.ts`,
 *       content: new TextEncoder().encode(`export interface ${type.name} {}\n`),
 *     });
 *   }
 *   return { files, errors: [] };
 * });
 * ```
 *
 * @module
 */

export { run } from "./run.ts";
export { CodeWriter } from "./codegen.ts";
export {
  toPascalCase,
  toSnakeCase,
  toCamelCase,
  toScreamingSnakeCase,
} from "./naming.ts";

// Re-export proto types for convenience
export type {
  OghamCompileRequest,
  OghamCompileResponse,
  GeneratedFile,
  CompileError,
} from "../oghamproto/compiler/request_pb.ts";
