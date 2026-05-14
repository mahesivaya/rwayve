import { apiFetch } from "./client";

export type ChatUser = {
  id: number;
  email: string;
};

export type ChatMessage = {
  message_id?: number;
  sender_id: number;
  receiver_id?: number;
  channel_id?: number;
  content: string;
  status: "sent" | "delivered" | "read";
  created_at: string;
};

export type ChatChannel = {
  id: number;
  name: string;
  visibility: "public" | "private";
  created_by: number;
  created_at: string;
  current_user_role?: "admin" | "user";
  is_member: boolean;
  join_status?: "pending";
  member_ids: number[];
  member_emails: string[];
  admin_emails?: string[];
  user_emails?: string[];
  invite_emails?: string[];
  invite_role?: "admin" | "user";
  admin_invite_emails?: string[];
  user_invite_emails?: string[];
  pending_join_requests?: Array<{
    user_id: number;
    email: string;
  }>;
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

export const getChatChannels = async () => {
  const res = await apiFetch("/api/chat/channels");
  return res.json() as Promise<ChatChannel[]>;
};

export const createChatChannel = async (
  name: string,
  inviteRole: "admin" | "user",
  inviteEmails: string[],
) => {
  const res = await apiFetch("/api/chat/channels", {
    method: "POST",
    body: JSON.stringify({
      name,
      invite_role: inviteRole,
      invite_emails: inviteEmails,
    }),
  });

  if (!res.ok) {
    const data = await res.json().catch(() => null);
    throw new Error(data?.error ?? "Failed to create channel");
  }

  return res.json() as Promise<ChatChannel>;
};

export const updateChatChannelSubject = async (
  channelId: number,
  name: string,
) => {
  const res = await apiFetch(`/api/chat/channels/${channelId}`, {
    method: "PATCH",
    body: JSON.stringify({ name }),
  });

  if (!res.ok) {
    const data = await res.json().catch(() => null);
    throw new Error(data?.error ?? "Failed to update channel subject");
  }
};

export const updateChatChannelVisibility = async (
  channelId: number,
  visibility: "public" | "private",
) => {
  const res = await apiFetch(`/api/chat/channels/${channelId}/visibility`, {
    method: "PATCH",
    body: JSON.stringify({ visibility }),
  });

  if (!res.ok) {
    const data = await res.json().catch(() => null);
    throw new Error(data?.error ?? "Failed to update channel visibility");
  }
};

export const joinChatChannel = async (channelId: number) => {
  const res = await apiFetch(`/api/chat/channels/${channelId}/join`, {
    method: "POST",
  });

  if (!res.ok) {
    const data = await res.json().catch(() => null);
    throw new Error(data?.error ?? "Failed to join channel");
  }

  return res.json() as Promise<{ status: "joined" | "pending" }>;
};

export const approveChatChannelJoinRequest = async (
  channelId: number,
  userId: number,
) => {
  const res = await apiFetch(`/api/chat/channels/${channelId}/join-requests/approve`, {
    method: "POST",
    body: JSON.stringify({ user_id: userId }),
  });

  if (!res.ok) {
    const data = await res.json().catch(() => null);
    throw new Error(data?.error ?? "Failed to approve join request");
  }
};

export const addChatChannelUsers = async (
  channelId: number,
  inviteRole: "admin" | "user",
  inviteEmails: string[],
) => {
  const res = await apiFetch(`/api/chat/channels/${channelId}/members`, {
    method: "POST",
    body: JSON.stringify({
      invite_role: inviteRole,
      invite_emails: inviteEmails,
    }),
  });

  if (!res.ok) {
    const data = await res.json().catch(() => null);
    throw new Error(data?.error ?? "Failed to add channel users");
  }
};

export const removeChatChannelUser = async (
  channelId: number,
  email: string,
) => {
  const res = await apiFetch(`/api/chat/channels/${channelId}/members`, {
    method: "DELETE",
    body: JSON.stringify({ email }),
  });

  if (!res.ok) {
    const data = await res.json().catch(() => null);
    throw new Error(data?.error ?? "Failed to delete channel user");
  }
};

export const getChannelMessages = async (channelId: number) => {
  const params = new URLSearchParams({
    channel_id: String(channelId),
  });

  const res = await apiFetch(`/api/chat/channel-messages?${params.toString()}`);
  return res.json() as Promise<ChatMessage[]>;
};
