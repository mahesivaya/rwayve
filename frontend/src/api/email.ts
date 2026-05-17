import { apiFetch } from "./client";

export type EmailFolder = "inbox" | "sent";

export type EmailListParams = {
  folder: EmailFolder;
  accountId?: number | null;
  query?: string;
  before?: number;
  beforeId?: number;
};

export type EmailListResult<T = unknown> = {
  emails: T[];
  hasMore: boolean;
};

export type SendEmailPayload = {
  account_id: number;
  to: string;
  subject: string;
  body: string;
};

export type EmailAttachment = {
  id: number;
  email_id: number;
  filename: string;
  mime_type?: string | null;
  size?: number | null;
  created_at?: string | null;
  subject?: string | null;
  sender?: string | null;
  receiver?: string | null;
};

const emailListPath = ({
  folder,
  accountId,
  query,
  before,
  beforeId,
}: EmailListParams) => {
  const params = new URLSearchParams({ folder });

  if (accountId !== null && accountId !== undefined) {
    params.set("account_id", String(accountId));
  }

  if (query) {
    params.set("q", query);
  }

  if (before !== undefined) {
    params.set("before", String(before));
  }

  if (beforeId !== undefined) {
    params.set("before_id", String(beforeId));
  }

  return `/api/emails?${params.toString()}`;
};

export const getAccounts = async <T = unknown>() => {
  const res = await apiFetch("/api/accounts");
  return res.json() as Promise<T[]>;
};

export const deleteAccount = async (id: number) => {
  await apiFetch(`/api/accounts/${id}`, {
    method: "DELETE",
  });
};

export const updateAccountDisplayName = async (
  id: number,
  displayName: string | null
) => {
  await apiFetch(`/api/accounts/${id}/display-name`, {
    method: "PUT",
    body: JSON.stringify({ display_name: displayName }),
  });
};

export const getGmailConnectUrl = async () => {
  const res = await apiFetch("/api/gmail/connect-url", {
    method: "POST",
  });
  const data = (await res.json()) as { url: string };
  return data.url;
};

export const getOutlookConnectUrl = async () => {
  const res = await apiFetch("/api/outlook/connect-url", {
    method: "POST",
  });
  const data = (await res.json()) as { url: string };
  return data.url;
};

export const getEmails = async <T = unknown>(
  params: EmailListParams
): Promise<EmailListResult<T>> => {
  const res = await apiFetch(emailListPath(params));
  const emails = (await res.json()) as T[];

  return {
    emails,
    hasMore: res.headers.get("x-has-more") === "true",
  };
};

export const getEmail = async <T = unknown>(id: number) => {
  const res = await apiFetch(`/api/emails/${id}`);
  return res.json() as Promise<T>;
};

export const getEmailBody = async (id: number) => {
  const res = await apiFetch(`/api/emails/${id}/body`);
  return res.json() as Promise<{ body?: string }>;
};

export const getEmailAttachments = async (emailId: number) => {
  const res = await apiFetch(`/api/emails/${emailId}/attachments`);
  return res.json() as Promise<EmailAttachment[]>;
};

export const getAllEmailAttachments = async () => {
  const res = await apiFetch("/api/emails/attachments");
  return res.json() as Promise<EmailAttachment[]>;
};

export const downloadEmailAttachment = async (attachment: EmailAttachment) => {
  const res = await apiFetch(`/api/email-attachments/${attachment.id}/download`);
  const blob = await res.blob();
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");

  link.href = url;
  link.download = attachment.filename || "attachment";
  document.body.appendChild(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
};

export const sendEmail = async (payload: SendEmailPayload) => {
  const res = await apiFetch("/api/send", {
    method: "POST",
    body: JSON.stringify(payload),
  });

  return res.text();
};

export const getWayveRecipientByEmail = async <T = unknown>(
  email: string,
  token?: string
) => {
  const res = await apiFetch(`/api/users?email=${encodeURIComponent(email)}`, {
    headers: token
      ? {
          Authorization: `Bearer ${token}`,
        }
      : undefined,
  });

  return res.json() as Promise<T[] | T>;
};
