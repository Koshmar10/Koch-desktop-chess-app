import type { ChatMessage } from "../api/bindings/ChatMessage";

export interface TokenTotals {
  input: number;
  // The part of `input` billed at the cached rate.
  cachedInput: number;
  output: number;
  // The part of `output` the model spent thinking.
  reasoning: number;
}

const NO_TOKENS: TokenTotals = { input: 0, cachedInput: 0, output: 0, reasoning: 0 };

/** Every answer's tokens, added up. Questions carry no usage of their own. */
export const conversationTokens = (messages: ChatMessage[]): TokenTotals =>
  messages.reduce(
    (totals, { usage }) =>
      usage === null
        ? totals
        : {
            input: totals.input + usage.input_tokens,
            cachedInput: totals.cachedInput + usage.cached_input_tokens,
            output: totals.output + usage.output_tokens,
            reasoning: totals.reasoning + usage.reasoning_tokens,
          },
    NO_TOKENS,
  );

/** The model that wrote the latest answer, or null before the first one. */
export const latestModel = (messages: ChatMessage[]): string | null =>
  [...messages].reverse().find((message) => message.usage !== null)?.usage?.model ?? null;

const COMPACT_COUNT_FORMAT = new Intl.NumberFormat("en", {
  notation: "compact",
  maximumFractionDigits: 1,
});

/** 950 → "950", 12 400 → "12.4K". */
export const compactCount = (count: number): string => COMPACT_COUNT_FORMAT.format(count);
