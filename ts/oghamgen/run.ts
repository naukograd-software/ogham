/**
 * Plugin runner — reads OghamCompileRequest from stdin,
 * calls the handler, writes OghamCompileResponse to stdout.
 */

import {
  OghamCompileRequestSchema,
  OghamCompileResponseSchema,
  type OghamCompileRequest,
  type OghamCompileResponse,
} from "../oghamproto/compiler/request_pb.ts";
import { fromBinary, toBinary } from "@bufbuild/protobuf";

/**
 * Handler function type.
 */
export type Handler = (
  req: OghamCompileRequest
) => OghamCompileResponse | Promise<OghamCompileResponse>;

/**
 * Run a plugin handler.
 *
 * Reads `OghamCompileRequest` from stdin (protobuf),
 * calls `handler`, writes `OghamCompileResponse` to stdout.
 */
export async function run(handler: Handler): Promise<void> {
  try {
    const input = await readStdin();
    const req = fromBinary(OghamCompileRequestSchema, input);

    let resp: OghamCompileResponse;
    try {
      resp = await handler(req);
    } catch (err) {
      resp = {
        $typeName: "oghamproto.compiler.OghamCompileResponse",
        files: [],
        errors: [
          {
            $typeName: "oghamproto.compiler.CompileError",
            message: String(err),
            severity: 1,
            sourceType: "",
            sourceField: "",
          },
        ],
      };
    }

    const output = toBinary(OghamCompileResponseSchema, resp);
    process.stdout.write(output);
  } catch (err) {
    process.stderr.write(`oghamgen: ${err}\n`);
    process.exit(1);
  }
}

function readStdin(): Promise<Uint8Array> {
  return new Promise((resolve, reject) => {
    const chunks: Buffer[] = [];
    process.stdin.on("data", (chunk) => chunks.push(chunk));
    process.stdin.on("end", () => resolve(Buffer.concat(chunks)));
    process.stdin.on("error", reject);
  });
}
