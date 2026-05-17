import React, { useState } from "react";
import { downloadEmailAttachment, sendEmail } from "../api/email";
import { formatFileSize, renderEmailBody } from "./renderUtils";
import { EmailItem, EmailAttachment } from "./types";

interface EmailDetailProps {
  selectedEmail: EmailItem | null;
  viewMode: "email" | "files";
  isNarrow: boolean;
  onBack: () => void;
  onDeleteEmail: (emailId: number) => Promise<void>;
  files: EmailAttachment[];
  filesLoading: boolean;
  filesError: string | null;
  normalizedSearchQuery: string;
}

export const EmailDetail: React.FC<EmailDetailProps> = ({
  selectedEmail,
  viewMode,
  isNarrow,
  onBack,
  onDeleteEmail,
  files,
  filesLoading,
  filesError,
  normalizedSearchQuery,
}) => {
  const [deleting, setDeleting] = useState(false);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [replyOpen, setReplyOpen] = useState(false);
  const [replyBody, setReplyBody] = useState("");
  const [replySending, setReplySending] = useState(false);
  const [replyError, setReplyError] = useState<string | null>(null);

  const visibleFiles = normalizedSearchQuery
    ? files.filter((file) =>
        [file.filename, file.mime_type ?? "", file.subject ?? "", file.sender ?? "", file.receiver ?? ""]
          .join(" ")
          .toLowerCase()
          .includes(normalizedSearchQuery)
      )
    : files;

  if (viewMode === "files") {
    return (
      <div className="email-detail">
        {isNarrow && (
          <div className="email-detail-actions">
            <button className="email-detail-back" onClick={onBack} title="Close" aria-label="Close">✕</button>
          </div>
        )}
        <div className="email-files-inline">
          <div className="email-files-header">
            <h2>Files</h2>
            <button onClick={onBack}>Back to email</button>
          </div>
          {filesLoading ? (
            <div className="email-files-empty">Loading files…</div>
          ) : filesError ? (
            <div className="email-files-error">{filesError}</div>
          ) : visibleFiles.length === 0 ? (
            <div className="email-files-empty">
              {normalizedSearchQuery ? "No files match your search" : "No attached files found"}
            </div>
          ) : (
            <div className="email-files-list">
              {visibleFiles.map((file) => (
                <button key={file.id} className="email-files-row" onClick={() => downloadEmailAttachment(file)}>
                  <span className="email-files-icon">📎</span>
                  <span className="email-files-main">
                    <span className="email-files-name">{file.filename}</span>
                    <span className="email-files-meta">{file.subject || "No subject"} · {file.sender || "Unknown sender"}</span>
                  </span>
                  <span className="email-files-size">{formatFileSize(file.size)}</span>
                </button>
              ))}
            </div>
          )}
        </div>
      </div>
    );
  }

  if (!selectedEmail) {
    return <div className="email-detail"><p>Select an email</p></div>;
  }

  const handleDelete = async () => {
    const ok = window.confirm("Delete this email permanently from Wayve and your mail provider?");
    if (!ok) return;

    setDeleting(true);
    setDeleteError(null);
    try {
      await onDeleteEmail(selectedEmail.id);
    } catch (err) {
      setDeleteError(err instanceof Error ? err.message : "Failed to delete email");
    } finally {
      setDeleting(false);
    }
  };

  const replyTo = emailAddress(selectedEmail.sender);
  const handleReply = async () => {
    const body = replyBody.trim();

    if (!selectedEmail.account_id) {
      setReplyError("Missing sender account for this email.");
      return;
    }

    if (!replyTo) {
      setReplyError("Missing recipient for reply.");
      return;
    }

    if (!body) {
      setReplyError("Enter a reply before sending.");
      return;
    }

    setReplySending(true);
    setReplyError(null);
    try {
      const subject = selectedEmail.subject?.trim() || "(No Subject)";
      await sendEmail({
        account_id: selectedEmail.account_id,
        to: replyTo,
        subject: subject.toLowerCase().startsWith("re:") ? subject : `Re: ${subject}`,
        body,
      });
      setReplyBody("");
      setReplyOpen(false);
    } catch (err) {
      setReplyError(err instanceof Error ? err.message : "Failed to send reply");
    } finally {
      setReplySending(false);
    }
  };

  return (
    <div className="email-detail">
      {isNarrow && (
        <div className="email-detail-actions">
          <button
            className="email-detail-reply"
            onClick={() => {
              setReplyOpen((open) => !open);
              setReplyError(null);
            }}
            title="Reply"
            aria-label="Reply"
          >
            <svg className="email-detail-reply-icon" viewBox="0 0 24 24" aria-hidden="true">
              <path d="M10 8 5 13l5 5" />
              <path d="M5 13h9a5 5 0 0 1 5 5v1" />
            </svg>
          </button>
          <button
            className="email-detail-delete"
            onClick={() => void handleDelete()}
            disabled={deleting}
            title="Delete email"
            aria-label="Delete email"
          >
            {deleting ? (
              "…"
            ) : (
              <svg className="email-detail-delete-icon" viewBox="0 0 24 24" aria-hidden="true">
                <path d="M4 7h16" />
                <path d="M10 11v6" />
                <path d="M14 11v6" />
                <path d="M6 7l1 14h10l1-14" />
                <path d="M9 7V4h6v3" />
              </svg>
            )}
          </button>
          <button className="email-detail-back" onClick={onBack} title="Close" aria-label="Close">✕</button>
        </div>
      )}
      <h2>{selectedEmail.subject}</h2>
      {deleteError && <p className="email-body-error">{deleteError}</p>}
      <p><b>From:</b> {selectedEmail.sender}</p>
      <p><b>To:</b> {selectedEmail.receiver}</p>

      {replyOpen && (
        <div className="email-reply-box">
          <textarea
            value={replyBody}
            onChange={(e) => setReplyBody(e.target.value)}
            placeholder={`Reply to ${replyTo || "sender"}`}
            aria-label="Reply body"
          />
          {replyError && <p className="email-body-error">{replyError}</p>}
          <div className="email-reply-actions">
            <button
              type="button"
              className="email-reply-send"
              onClick={() => void handleReply()}
              disabled={replySending}
            >
              {replySending ? "Sending..." : "Send"}
            </button>
            <button
              type="button"
              className="email-reply-cancel"
              onClick={() => {
                setReplyOpen(false);
                setReplyError(null);
              }}
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      <div className="email-body">
        {selectedEmail._bodyLoading ? (
          <div className="email-body-loading">
            <span className="spinner" aria-hidden="true" />
            <span>Loading email …</span>
          </div>
        ) : selectedEmail._bodyError ? (
          <p className="email-body-error">
            {typeof selectedEmail._bodyError === "string"
              ? selectedEmail._bodyError
              : "Failed to load email body. Try again."}
          </p>
        ) : (
          renderEmailBody(selectedEmail.body || "")
        )}
      </div>

      {(selectedEmail.attachments?.length ?? 0) > 0 && (
        <div className="email-attachments">
          <div className="email-attachments-title">Attachments</div>
          {selectedEmail.attachments?.map((attachment) => (
            <button
              key={attachment.id}
              className="email-attachment"
              onClick={() => downloadEmailAttachment(attachment)}
            >
              <span className="email-attachment-icon">📎</span>
              <span className="email-attachment-name">{attachment.filename}</span>
              <span className="email-attachment-size">{formatFileSize(attachment.size)}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
};

function emailAddress(value?: string | null) {
  const text = value?.trim() || "";
  const match = text.match(/<([^>]+)>/);
  return (match?.[1] || text).trim();
}
