import { toPascalCase, toSnakeCase, toCamelCase, toScreamingSnakeCase } from "./naming.ts";
import assert from "node:assert";
import { describe, it } from "node:test";

describe("toPascalCase", () => {
  it("user_name → UserName", () => assert.strictEqual(toPascalCase("user_name"), "UserName"));
  it("id → Id", () => assert.strictEqual(toPascalCase("id"), "Id"));
  it("created_at → CreatedAt", () => assert.strictEqual(toPascalCase("created_at"), "CreatedAt"));
});

describe("toSnakeCase", () => {
  it("UserName → user_name", () => assert.strictEqual(toSnakeCase("UserName"), "user_name"));
  it("createdAt → created_at", () => assert.strictEqual(toSnakeCase("createdAt"), "created_at"));
});

describe("toCamelCase", () => {
  it("user_name → userName", () => assert.strictEqual(toCamelCase("user_name"), "userName"));
  it("created_at → createdAt", () => assert.strictEqual(toCamelCase("created_at"), "createdAt"));
});

describe("toScreamingSnakeCase", () => {
  it("OrderStatus → ORDER_STATUS", () => assert.strictEqual(toScreamingSnakeCase("OrderStatus"), "ORDER_STATUS"));
});
