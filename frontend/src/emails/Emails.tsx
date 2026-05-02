import { useEffect, useRef, useState } from "react";
import "./emails.css";
import "./loadMore.css";
import SendEmail from "./SendEmail";

const API_BASE = import.meta.env.VITE_API_URL;

export default function Emails() {
  const [accounts, setAccounts] = useState<any[]>([]);
  const [emails, setEmails] = useState<any[]>([]);
  const [selectedEmail, setSelectedEmail] = useState<any | null>(null);

  const [activeAccount, setActiveAccount] = useState<number | null>(null);
  const [activeFolder, setActiveFolder] = useState<"inbox" | "sent">("inbox");

  const [hasMore, setHasMore] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);

  const [composeOpen, setComposeOpen] = useState(false);

  const emailCache = useRef<{ [key: number]: any }>({});

  // ================= FETCH ACCOUNTS =================
  const fetchAccounts = async () => {
    const token = localStorage.getItem("token");

    const res = await fetch(`${API_BASE}/api/accounts`, {
      headers: { Authorization: `Bearer ${token}` },
    });

    const data = await res.json();
    setAccounts(data);
  };

  useEffect(() => {
    fetchAccounts();
  }, []);

  // ================= HANDLE OAUTH RETURN =================
  // After /oauth/callback redirects back with ?connected=true, refresh the
  // account list so the newly linked account shows up immediately. The 30s
  // sync worker will import its emails on the next tick.
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    if (params.get("connected") === "true") {
      fetchAccounts();
      window.history.replaceState({}, "", "/emails");
    }
  }, []);

  // ================= ADD ACCOUNT =================
  const addAccount = () => {
    const token = localStorage.getItem("token");
    if (!token) return;
    window.location.href = `${API_BASE}/gmail/login?token=${encodeURIComponent(token)}`;
  };

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
        <button
          className="compose-btn"
          onClick={() => setComposeOpen(true)}
          disabled={accounts.length === 0}
          title={accounts.length === 0 ? "Add an account first" : "Compose"}
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

        <button className="add-email-btn" onClick={addAccount}>
          ➕ Add Account
        </button>

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
          <div className="load-more-wrap">
            <button
              className="load-more-btn"
              onClick={loadMore}
              disabled={loadingMore}
            >
              {loadingMore ? "Loading..." : "Load More"}
            </button>
          </div>
        )}
      </div>

      {/* ================= COMPOSE MODAL ================= */}
      {composeOpen && accounts.length > 0 && (
        <div
          onClick={() => setComposeOpen(false)}
          style={{
            position: "fixed",
            inset: 0,
            background: "rgba(0,0,0,0.4)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 1000,
          }}
        >
          <div
            onClick={(e) => e.stopPropagation()}
            style={{
              background: "#fff",
              padding: 20,
              borderRadius: 8,
              width: 480,
              maxWidth: "90vw",
              boxShadow: "0 10px 30px rgba(0,0,0,0.2)",
            }}
          >
            <SendEmail
              accountId={activeAccount ?? accounts[0].id}
              onClose={() => setComposeOpen(false)}
            />
          </div>
        </div>
      )}

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