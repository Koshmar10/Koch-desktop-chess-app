import { invoke } from "@tauri-apps/api/core";
import type { ChatMessage } from "./bindings/ChatMessage";
import type { ChatSummary } from "./bindings/ChatSummary";

// `gameId` is null for the sandbox, whose chat isn't saved. The backend
// keeps one chat open at a time.

export const sendAssistantRequest = (
  gameId: number | null,
  message: string,
  ply: number,
): Promise<ChatMessage> =>
  invoke<ChatMessage>("send_assistent_request", { gameId, message, ply });

// The open chat about this game: its latest saved one if another game's
// chat was open.
export const getAssistantConversation = (
  gameId: number | null,
): Promise<ChatMessage[]> =>
  invoke<ChatMessage[]>("get_assistent_conversation", { gameId });

export const listAssistantChats = (gameId: number): Promise<ChatSummary[]> =>
  invoke<ChatSummary[]>("list_assistent_chats", { gameId });

export const openAssistantChat = (
  gameId: number,
  chatId: number,
): Promise<ChatMessage[]> =>
  invoke<ChatMessage[]>("open_assistent_chat", { gameId, chatId });

export const startNewAssistantChat = (): Promise<void> =>
  invoke("start_new_assistent_chat");
