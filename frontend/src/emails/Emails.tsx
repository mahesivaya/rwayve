import { useEffect, useState } from "react";
import "./emails.css";
import SendEmail from "../emails/send_email";

type Email = {
  id: number;
  sender: string;
  receiver: string;
  subject: string;
  body: string;
  created_at: string;
};

type Account = {
  id: number;
  email: string;
};

export default function Emails() {
  const BASE_URL = "http://localhost:8080";

  const [emails, setEmails] = useState<Email[]>([]);
  const [selected, setSelected] = useState<Email | null>(null);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [activeAccount, setActiveAccount] = useState<number | null>(null);
  const [showCompose, setShowCompose] = useState(false);

  const [lastTimestamp, setLastTimestamp] = useState<string | null>(null);
  const [lastId, setLastId] = useState<number | null>(null);
  const [loadingMore, setLoadingMore] = useState(false);
  const [loading, setLoading] = useState(false);

  // ================= CONNECT GMAIL =================
  const connectGmail = () => {
    const token = localStorage.getItem("token");

    if (!token) {
      alert("User not logged in");
      return;
    }

    window.location.href = `${BASE_URL}/gmail/login?token=${token}`;
  };

  // ================= ACCOUNTS =================
  useEffect(() => {
    const token = localStorage.getItem("token");
    if (!token) return;

    fetch(`${BASE_URL}/api/accounts`, {
      headers: {
        Authorization: `Bearer ${token}`,
      },
    })
      .then(async res => {
        if (!res.ok) {
          const text = await res.text();
          console.error("Accounts error:", text);
          throw new Error("Unauthorized");
        }
        return res.json();
      })
      .then(setAccounts)
      .catch(err => {
        console.error("Accounts fetch failed:", err);
        setAccounts([]);
      });
  }, []);

  // ================= LOAD EMAILS =================
  useEffect(() => {
    const token = localStorage.getItem("token");
    if (!token) return;

    // ❌ Don't fetch emails if no accounts
    if (accounts.length === 0) {
      setEmails([]);
      setSelected(null);
      return;
    }

    setLoading(true);

    const url =
      activeAccount !== null
        ? `/api/emails?account_id=${activeAccount}`
        : `/api/emails`;

    fetch(`${BASE_URL}${url}`, {
      headers: {
        Authorization: `Bearer ${token}`,
      },
    })
      .then(async res => {
        if (!res.ok) {
          const text = await res.text();
          console.error("Emails error:", text);
          throw new Error("Unauthorized");
        }
        return res.json();
      })
      .then((data: Email[]) => {
        setEmails(data);

        if (data.length > 0) {
          setSelected(data[0]);

          const last = data[data.length - 1];
          setLastTimestamp(last.created_at);
          setLastId(last.id);
        } else {
          setSelected(null);
        }
      })
      .catch(err => {
        console.error("Email fetch failed:", err);
        setEmails([]);
      })
      .finally(() => setLoading(false));
  }, [activeAccount, accounts]);

  // ================= LOAD MORE =================
  const loadMore = async () => {
    if (!lastTimestamp || !lastId || loadingMore) return;

    setLoadingMore(true);

    try {
      const token = localStorage.getItem("token");
      if (!token) return;

      const url =
        activeAccount !== null
          ? `/api/emails?account_id=${activeAccount}&before=${lastTimestamp}&before_id=${lastId}`
          : `/api/emails?before=${lastTimestamp}&before_id=${lastId}`;

      const res = await fetch(`${BASE_URL}${url}`, {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });

      if (!res.ok) {
        const text = await res.text();
        console.error("Load more error:", text);
        return;
      }

      const data: Email[] = await res.json();
      if (!data.length) return;

      setEmails(prev => {
        const ids = new Set(prev.map(e => e.id));
        return [...prev, ...data.filter(e => !ids.has(e.id))];
      });

      const last = data[data.length - 1];
      setLastTimestamp(last.created_at);
      setLastId(last.id);

    } catch (err) {
      console.error("Load more failed:", err);
    } finally {
      setLoadingMore(false);
    }
  };

  // ================= UI =================
  return (
    <div className="main">

      {/* SIDEBAR */}
      <div className="sidebar">

        <button className="compose-btn" onClick={() => setShowCompose(true)}>
          + Compose
        </button>

        <button onClick={connectGmail} className="add-email-btn">
          ➕ Add Account
        </button>

        <div className="mail-filters">
          <button
            className={`filter-btn ${activeAccount === null ? "active" : ""}`}
            onClick={() => setActiveAccount(null)}
          >
            All
          </button>

          {accounts.map(acc => (
            <button
              key={acc.id}
              className={`filter-btn ${activeAccount === acc.id ? "active" : ""}`}
              onClick={() => setActiveAccount(acc.id)}
            >
              {acc.email}
            </button>
          ))}
        </div>

        {showCompose && (
          <div className="compose-modal">
            <div className="compose-box">
              <SendEmail />
              <button onClick={() => setShowCompose(false)}>Close</button>
            </div>
          </div>
        )}
      </div>

      {/* EMAIL LIST */}
      <div className="email-list">

        {accounts.length === 0 ? (
          <div className="empty-state">
            <p>📭 No accounts added</p>
            <button onClick={connectGmail}>Connect Gmail</button>
          </div>
        ) : loading ? (
          <p>Loading emails...</p>
        ) : emails.length === 0 ? (
          <p>Loading emails</p>
        ) : (
          emails.map(email => (
            <div
              key={email.id}
              className={`email-item ${selected?.id === email.id ? "active" : ""}`}
              onClick={() => setSelected(email)}
            >
              <div className="email-top">
                <span className="email-sender">{email.sender}</span>
                <span className="email-receiver">{email.receiver}</span>

                <span className="email-time">
                  {new Date(email.created_at).toLocaleTimeString([], {
                    hour: "2-digit",
                    minute: "2-digit",
                  })}
                </span>
              </div>

              <div className="email-subject">{email.subject}</div>
            </div>
          ))
        )}

        {accounts.length > 0 && emails.length > 0 && (
          <div className="load-more-container">
            <button onClick={loadMore} disabled={loadingMore}>
              {loadingMore ? "Loading..." : "Load More"}
            </button>
          </div>
        )}

      </div>

      {/* EMAIL DETAIL */}
      <div className="email-detail">
        {accounts.length === 0 ? (
          <p>No accounts connected</p>
        ) : selected ? (
          <>
            <h2>{selected.subject}</h2>
            <p><b>From:</b> {selected.sender}</p>
            <p><b>To:</b> {selected.receiver}</p>
            <hr />

            <div className="email-body">
              {selected.body ? (
                selected.body.includes("<") ? (
                  <div dangerouslySetInnerHTML={{ __html: selected.body }} />
                ) : (
                  <pre>{selected.body}</pre>
                )
              ) : (
                <p>No content</p>
              )}
            </div>
          </>
        ) : (
          <p>No email selected</p>
        )}
      </div>

    </div>
  );
}