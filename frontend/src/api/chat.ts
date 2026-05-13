import { apiFetch } from "./client";

export type ChatUser = {
  id: number;
  email: string;
};

export type ChatMessage = {
  sender_id: number;
  receiver_id: number;
  content: string;
  status: "sent" | "delivered" | "read";
  created_at: string;
};

export const getChatUsers = async () => {
  const res = await apiFetch("/api/users/all");
  return res.json() as Promise<ChatUser[]>;
};

export const getChatMessages = async (userId: number, otherUserId: number) => {
  const params = new URLSearchParams({
    user1: String(userId),
    user2: String(otherUserId),
  });

  const res = await apiFetch(`/api/messages?${params.toString()}`);
  return res.json() as Promise<ChatMessage[]>;
};
