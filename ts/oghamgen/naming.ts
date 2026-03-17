/**
 * Name conversion utilities.
 */

/** Convert snake_case to PascalCase. */
export function toPascalCase(s: string): string {
  return s
    .split("_")
    .map((part) => (part.length > 0 ? part[0].toUpperCase() + part.slice(1) : ""))
    .join("");
}

/** Convert PascalCase/camelCase to snake_case. */
export function toSnakeCase(s: string): string {
  return s.replace(/([A-Z])/g, (_, c, i) => (i > 0 ? "_" : "") + c.toLowerCase());
}

/** Convert snake_case to camelCase. */
export function toCamelCase(s: string): string {
  const pascal = toPascalCase(s);
  return pascal.length > 0 ? pascal[0].toLowerCase() + pascal.slice(1) : "";
}

/** Convert to SCREAMING_SNAKE_CASE. */
export function toScreamingSnakeCase(s: string): string {
  return toSnakeCase(s).toUpperCase();
}
