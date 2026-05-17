import React from "react";
import { EmailAccount } from "./types";

interface EmailSidebarProps {
  accounts: EmailAccount[];
  activeAccount: number | null;
  setActiveAccount: (id: number | null) => void;
  activeFolder: "inbox" | "sent";
  setActiveFolder: (folder: "inbox" | "sent") => void;
  viewMode: "email" | "files";
  onOpenFiles: () => void;
  onAddGmail: () => void;
  onAddOutlook: () => void;
  onCompose: () => void;
  composeDisabled: boolean;
}

export const EmailSidebar: React.FC<EmailSidebarProps> = ({
  accounts,
  activeAccount,
  setActiveAccount,
  activeFolder,
  setActiveFolder,
  viewMode,
  onOpenFiles,
  onAddGmail,
  onAddOutlook,
  onCompose,
  composeDisabled,
}) => {
  return (
    <div className="sidebar">
      <button
        className="compose-btn"
        onClick={onCompose}
        disabled={composeDisabled}
        title={composeDisabled ? "Add an account first" : "Compose"}
      >
        Compose
      </button>

      <div className="mail-section-title">Accounts</div>

      <button
        className={`filter-btn ${activeAccount === null ? "active" : ""}`}
        onClick={() => setActiveAccount(null)}
      >
        🌐 All Accounts
      </button>

      {accounts.map((acc) => (
        <button
          key={acc.id}
          className={`filter-btn ${activeAccount === acc.id ? "active" : ""}`}
          onClick={() => setActiveAccount(acc.id)}
        >
          {acc.email}
        </button>
      ))}

      <button className="add-email-btn" onClick={onAddGmail}>➕ Add Gmail</button>
      <button className="add-email-btn" onClick={onAddOutlook}>➕ Add Outlook</button>

      <div className="mail-section-title">Folders</div>

      <div className="mail-filters">
        <button className={`filter-btn ${activeFolder === "inbox" && viewMode === "email" ? "active" : ""}`} onClick={() => setActiveFolder("inbox")}>📥 Inbox</button>
        <button className={`filter-btn ${activeFolder === "sent" && viewMode === "email" ? "active" : ""}`} onClick={() => setActiveFolder("sent")}>📤 Sent</button>
        <button
          className={`filter-btn ${viewMode === "files" ? "active" : ""}`}
          onClick={onOpenFiles}
        >
          📎 Files
        </button>
      </div>
    </div>
  );
};