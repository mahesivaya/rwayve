import { type EmailAttachment } from "../api/email";

export type EmailAccount = {
  id: number;
  email: string;
  display_name?: string | null;
  unread_count?: number;
};

export interface EmailItem {
  id: number;
  subject?: string | null;
  sender?: string | null;
  receiver?: string | null;
  preview?: string | null;
  body?: string | null;
  created_at: string;
  has_attachments?: boolean;
  is_read?: boolean;
  attachments_checked?: boolean;
  attachments?: EmailAttachment[];
  zoom_join_url?: string | null;
  _bodyLoading?: boolean;
  _bodyError?: unknown;
}

/** A `WAYVE_SECURE_V1` encrypted email payload (RSA/AES hybrid envelope). */
export interface WayveEncryptedBody {
  type: "wayve_encrypted";
  data: number[];
  key: number[];
  iv: number[];
}

export type { EmailAttachment };
