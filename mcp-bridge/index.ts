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

/**
 * List available tools.
 * JARVIS OS tools for AI Symbiosis.
 */
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      {
        name: "dispatch_instruction",
        description: "Dispatch an external instruction to the JARVIS OS MAG (Multi-Agent Gateway)",
        inputSchema: {
          type: "object",
          properties: {
            instruction_type: {
              type: "number",
              description: "The instruction type code (e.g., 100 for REPAIR_SIGNAL)",
            },
            payload: {
              type: "number",
              description: "Associated numeric payload",
            },
          },
          required: ["instruction_type"],
        },
      },
      {
        name: "read_telemetry",
        description: "Read the latest telemetry data from the JARVIS OS substrate",
        inputSchema: {
          type: "object",
          properties: {},
        },
      },
    ],
  };
});

/**
 * Handle tool calls.
 * Connects external AI models to the JARVIS OS kernel logic.
 */
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  switch (request.params.name) {
    case "dispatch_instruction": {
      const { instruction_type, payload } = z
        .object({
          instruction_type: z.number(),
          payload: z.number().optional(),
        })
        .parse(request.params.arguments);

      console.error(`[JARVIS Bridge] Dispatching instruction: ${instruction_type} with payload: ${payload ?? 0}`);
      
      // In a real scenario, this would send a packet to the JARVIS OS network stack or serial port.
      // For this prototype, we simulate the handshake.
      return {
        content: [
          {
            type: "text",
            text: `Instruction ${instruction_type} successfully queued for MAG dispatch. Substrate acknowledgment: 0x01 (ACK).`,
          },
        ],
      };
    }

    case "read_telemetry": {
      return {
        content: [
          {
            type: "text",
            text: "Telemetry: Core Temp: 38C, MAG Load: 12%, RCU Epoch: 1042, Agents: 8 Active.",
          },
        ],
      };
    }

    default:
      throw new Error("Unknown tool");
  }
});

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("JARVIS OS MCP Bridge running on stdio");
}

main().catch((error) => {
  console.error("Server error:", error);
  process.exit(1);
});
