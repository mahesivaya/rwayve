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
            <span className="email-sender">{email.sender}</span>
            <span className="email-row-meta">
              {email.has_attachments && <span className="email-attachment-pin" title="Has attachments">📎</span>}
              <span className="email-time">
                {new Date(email.created_at).toLocaleTimeString()}
              </span>
            </span>
          </div>
          <div className="email-subject">{email.subject}</div>
          <div className="email-preview">{email.preview || ""}</div>
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