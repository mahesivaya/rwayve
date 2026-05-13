import { API_BASE } from "../config/env";
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

export const getGmailLoginUrl = (token: string) =>
  `${API_BASE}/gmail/login?token=${encodeURIComponent(token)}`;

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
