import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";

const server = new Server(
  {
    name: "jarvis-os-bridge",
    version: "0.1.0",
  },
  {
    capabilities: {
      tools: {},
    },
  }
);

server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      {
        name: "dispatch_instruction",
        description: "Dispatch an external instruction to the JARVIS OS MAG",
        inputSchema: {
          type: "object",
          properties: {
            instruction_type: { type: "number" },
            payload: { type: "number" },
          },
          required: ["instruction_type"],
        },
      },
      {
        name: "read_telemetry",
        description: "Read the latest telemetry data",
        inputSchema: { type: "object", properties: {} },
      },
    ],
  };
});

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  switch (request.params.name) {
    case "dispatch_instruction": {
      return {
        content: [{ type: "text", text: "Instruction ACK: 0x01" }],
      };
    }
    case "read_telemetry": {
      return {
        content: [{ type: "text", text: "Telemetry: Stable." }],
      };
    }
    default:
      throw new Error("Unknown tool");
  }
});

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("JARVIS OS MCP Bridge running");
}

main().catch((error) => {
  console.error("Server error:", error);
  process.exit(1);
});
