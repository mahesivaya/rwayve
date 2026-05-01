import { useEffect, useRef, useState } from "react";
import "./emails.css";

const API_BASE = import.meta.env.VITE_API_URL;

export default function Emails() {
  const [accounts, setAccounts] = useState<any[]>([]);
  const [emails, setEmails] = useState<any[]>([]);
  const [selectedEmail, setSelectedEmail] = useState<any | null>(null);

  const [activeAccount, setActiveAccount] = useState<number | null>(null);
  const [activeFolder, setActiveFolder] = useState<"inbox" | "sent">("inbox");

  const [hasMore, setHasMore] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);

  const emailCache = useRef<{ [key: number]: any }>({});

  // ================= FETCH ACCOUNTS =================
  useEffect(() => {
    const fetchAccounts = async () => {
      const token = localStorage.getItem("token");

      const res = await fetch(`${API_BASE}/api/accounts`, {
        headers: { Authorization: `Bearer ${token}` },
      });

      const data = await res.json();
      setAccounts(data);
    };

    fetchAccounts();
  }, []);

  // ================= FETCH EMAILS =================
  useEffect(() => {
    const fetchEmails = async () => {
      const token = localStorage.getItem("token");

      let url = `${API_BASE}/api/emails?folder=${activeFolder}`;

      if (activeAccount !== null) {
        url += `&account_id=${activeAccount}`;
      }

      const res = await fetch(url, {
        headers: { Authorization: `Bearer ${token}` },
      });

      const data = await res.json();

      setEmails(data);
      setHasMore(data.length === 50);
      setSelectedEmail(null);
    };

    fetchEmails();
  }, [activeAccount, activeFolder]);

  // ================= LOAD MORE =================
  const loadMore = async () => {
    if (!hasMore || emails.length === 0) return;

    setLoadingMore(true);

    const token = localStorage.getItem("token");
    const last = emails[emails.length - 1];

    const before = Math.floor(new Date(last.created_at).getTime() / 1000);
    const before_id = last.id;

    let url = `${API_BASE}/api/emails?folder=${activeFolder}&before=${before}&before_id=${before_id}`;

    if (activeAccount !== null) {
      url += `&account_id=${activeAccount}`;
    }

    const res = await fetch(url, {
      headers: { Authorization: `Bearer ${token}` },
    });

    const data = await res.json();

    setEmails((prev) => [...prev, ...data]);
    setHasMore(data.length === 50);
    setLoadingMore(false);
  };

  // ================= OPEN EMAIL =================
  const openEmail = async (email: any) => {
    if (emailCache.current[email.id]) {
      setSelectedEmail(emailCache.current[email.id]);
      return;
    }

    const token = localStorage.getItem("token");

    const res = await fetch(`${API_BASE}/api/emails/${email.id}`, {
      headers: { Authorization: `Bearer ${token}` },
    });

    const data = await res.json();

    emailCache.current[email.id] = data;
    setSelectedEmail(data);
  };

  // ================= UI =================
  return (
    <div className="main">

      {/* ================= SIDEBAR ================= */}
      <div className="sidebar">
        <button className="compose-btn">Compose</button>

        <button
          className="add-email-btn"
          onClick={() => setActiveAccount(null)}
        >
          🌐 All
        </button>

        <div className="mail-section-title">Accounts</div>

        {accounts.map((acc) => (
          <button
            key={acc.id}
            className={`filter-btn ${activeAccount === acc.id ? "active" : ""}`}
            onClick={() => setActiveAccount(acc.id)}
          >
            {acc.email}
          </button>
        ))}

        <div className="mail-section-title">Folders</div>

        <div className="mail-filters">
          <button
            className={`filter-btn ${activeFolder === "inbox" ? "active" : ""}`}
            onClick={() => setActiveFolder("inbox")}
          >
            📥 Inbox
          </button>

          <button
            className={`filter-btn ${activeFolder === "sent" ? "active" : ""}`}
            onClick={() => setActiveFolder("sent")}
          >
            📤 Sent
          </button>
        </div>
      </div>

      {/* ================= EMAIL LIST ================= */}
      <div className="email-list">
        {emails.map((email) => (
          <div
            key={email.id}
            className={`email-item ${
              selectedEmail?.id === email.id ? "active" : ""
            }`}
            onClick={() => openEmail(email)}
          >
            <div className="email-top">
              <span className="email-sender">{email.sender}</span>
              <span className="email-time">
                {new Date(email.created_at).toLocaleTimeString()}
              </span>
            </div>

            <div className="email-subject">{email.subject}</div>

            <div className="email-preview">
              {email.preview || ""}
            </div>
          </div>
        ))}

        {hasMore && (
          <button className="add-email-btn" onClick={loadMore}>
            {loadingMore ? "Loading..." : "Load More"}
          </button>
        )}
      </div>

      {/* ================= EMAIL DETAIL ================= */}
      <div className="email-detail">
        {!selectedEmail ? (
          <p>Select an email</p>
        ) : (
          <>
            <h2>{selectedEmail.subject}</h2>

            <p><b>From:</b> {selectedEmail.sender}</p>
            <p><b>To:</b> {selectedEmail.receiver}</p>

            <div className="email-body">
              {selectedEmail.body}
            </div>
          </>
        )}
      </div>

    </div>
  );
}