import { apiFetch } from "./client";

export type AiTurn = {
  role: "user" | "model";
  content: string;
};

export const sendAiChat = async (messages: AiTurn[]) => {
  const res = await apiFetch("/api/ai/chat", {
    method: "POST",
    body: JSON.stringify({ messages }),
  });

  return res.json() as Promise<{ reply?: string }>;
};
