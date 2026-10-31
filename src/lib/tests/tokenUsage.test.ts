import { describe, expect, it } from "vitest";
import type { ChatMessage } from "../../api/bindings/ChatMessage";
import { compactCount, conversationTokens, latestModel } from "../tokenUsage";

const question = (content: string): ChatMessage => ({
  role: "User",
  content,
  turn_context: { game_id: null, ply: 0 },
  sent_at: "",
  usage: null,
  tool_calls: [],
});

const answer = (model: string, input: number, output: number): ChatMessage => ({
  role: "Assistant",
  content: "",
  turn_context: { game_id: null, ply: 0 },
  sent_at: "",
  usage: {
    model,
    input_tokens: input,
    cached_input_tokens: input / 2,
    cache_write_tokens: 0,
    output_tokens: output,
    reasoning_tokens: output / 2,
  },
  tool_calls: [],
});

describe("conversationTokens", () => {
  it("adds up the answers and skips the questions", () => {
    const messages = [
      question("a"),
      answer("m", 100, 20),
      question("b"),
      answer("m", 300, 40),
    ];
    expect(conversationTokens(messages)).toEqual({
      input: 400,
      cachedInput: 200,
      output: 60,
      reasoning: 30,
    });
  });

  it("is all zeros before the first answer", () => {
    expect(conversationTokens([question("a")])).toEqual({
      input: 0,
      cachedInput: 0,
      output: 0,
      reasoning: 0,
    });
  });
});

describe("latestModel", () => {
  it("names the model of the newest answer", () => {
    const messages = [
      answer("old", 1, 1),
      question("a"),
      answer("new", 1, 1),
      question("b"),
    ];
    expect(latestModel(messages)).toBe("new");
  });

  it("is null before the first answer", () => {
    expect(latestModel([question("a")])).toBeNull();
  });
});

describe("compactCount", () => {
  it("keeps small counts whole and shortens large ones", () => {
    expect(compactCount(950)).toBe("950");
    expect(compactCount(12_400)).toBe("12.4K");
  });
});
