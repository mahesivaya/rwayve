import React from "react";
import { downloadEmailAttachment } from "../api/email";
import { formatFileSize, renderEmailBody } from "./renderUtils";
import { EmailItem, EmailAttachment } from "./types";

interface EmailDetailProps {
  selectedEmail: EmailItem | null;
  viewMode: "email" | "files";
  isNarrow: boolean;
  onBack: () => void;
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
  files,
  filesLoading,
  filesError,
  normalizedSearchQuery,
}) => {
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
        {isNarrow && <button className="email-detail-back" onClick={onBack}>✕ Close</button>}
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

  return (
    <div className="email-detail">
      {isNarrow && <button className="email-detail-back" onClick={onBack}>✕ Close</button>}
      <h2>{selectedEmail.subject}</h2>
      <p><b>From:</b> {selectedEmail.sender}</p>
      <p><b>To:</b> {selectedEmail.receiver}</p>

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