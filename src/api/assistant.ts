import { invoke } from "@tauri-apps/api/core";
import type { ChatSummary } from "./bindings/ChatSummary";
import type { RenderedMessage } from "./bindings/RenderedMessage";

// `gameId` is null for the sandbox, whose chat isn't saved. The backend
// keeps one chat open at a time.

export const sendAssistantRequest = (
  gameId: number | null,
  message: string,
  ply: number,
): Promise<RenderedMessage> =>
  invoke<RenderedMessage>("send_assistent_request", { gameId, message, ply });

// The open chat about this game: its latest saved one if another game's
// chat was open.
export const getAssistantConversation = (
  gameId: number | null,
): Promise<RenderedMessage[]> =>
  invoke<RenderedMessage[]>("get_assistent_conversation", { gameId });

export const listAssistantChats = (gameId: number): Promise<ChatSummary[]> =>
  invoke<ChatSummary[]>("list_assistent_chats", { gameId });

export const openAssistantChat = (
  gameId: number,
  chatId: number,
): Promise<RenderedMessage[]> =>
  invoke<RenderedMessage[]>("open_assistent_chat", { gameId, chatId });

export const startNewAssistantChat = (): Promise<void> =>
  invoke("start_new_assistent_chat");
