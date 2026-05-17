import React from "react";
import { EmailItem } from "./types";

interface EmailListProps {
  emails: EmailItem[];
  selectedEmailId: number | null;
  onOpenEmail: (email: EmailItem) => void;
  hasMore: boolean;
  loadMore: () => void;
  loadingMore: boolean;
}

function senderDisplayName(sender?: string | null) {
  const value = sender?.trim() || "Unknown";
  const nameMatch = value.match(/^"?([^"<]+?)"?\s*</);
  const rawName = (nameMatch?.[1] || value.split("<")[0] || value).trim();
  const withoutEmail = rawName.includes("@") ? rawName.split("@")[0] : rawName;
  const parts = withoutEmail
    .replace(/[._-]+/g, " ")
    .split(/\s+/)
    .filter(Boolean);

  if (parts.length === 0) return "Unknown";
  return parts.slice(0, 2).join(" ");
}

export const EmailList: React.FC<EmailListProps> = ({
  emails,
  selectedEmailId,
  onOpenEmail,
  hasMore,
  loadMore,
  loadingMore,
}) => {
  return (
    <div className="email-list">
      {emails.map((email) => (
        <div
          key={email.id}
          className={`email-item ${selectedEmailId === email.id ? "active" : ""}`}
          onClick={() => onOpenEmail(email)}
        >
          <div className="email-top">
            <span className="email-primary">
              <span className="email-sender-name">{senderDisplayName(email.sender)}</span>
              <span className="email-list-subject">{email.subject || "(No Subject)"}</span>
            </span>
            <span className="email-row-meta">
              {email.has_attachments && <span className="email-attachment-pin" title="Has attachments">📎</span>}
              <span className="email-time">
                {new Date(email.created_at).toLocaleTimeString()}
              </span>
            </span>
          </div>
        </div>
      ))}

      {hasMore && (
        <div className="load-more-wrap">
          <button className="load-more-btn" onClick={loadMore} disabled={loadingMore}>{loadingMore ? "Loading..." : "Show more emails"}</button>
        </div>
      )}
    </div>
  );
};
