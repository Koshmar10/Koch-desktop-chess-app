import { invoke } from "@tauri-apps/api/core";
import type { ChatMessage } from "./bindings/ChatMessage";

export const sendAssistantRequest = (
  message: string,
  ply: number,
): Promise<ChatMessage> =>
  invoke<ChatMessage>("send_assistent_request", { message, ply });
