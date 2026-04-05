import { useEffect, useState } from "react";
import SendEmail from "../emails/send_emails";
import "./emails.css";

type Email = {
  id: number;
  sender: string;
  subject: string;
  body: string;
  created_at: string;
};

type Account = {
  id: number;
  email: string;
};

export default function Emails() {
  const [emails, setEmails] = useState<Email[]>([]);
  const [selected, setSelected] = useState<Email | null>(null);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [activeAccount, setActiveAccount] = useState<number | null>(null);
  const [showCompose, setShowCompose] = useState(false);

  // ✅ FIXED: declare globally
  const [lastTimestamp, setLastTimestamp] = useState<number | null>(null);
  const [loadingMore, setLoadingMore] = useState(false);

  const connectGmail = () => {
    window.location.href = "http://localhost:8080/gmail/login";
  };

  // ================= ACCOUNTS =================
  useEffect(() => {
    fetch("/api/accounts")
      .then(res => res.json())
      .then((data: Account[]) => {
        setAccounts(data);
  
        // ✅ AUTO SELECT FIRST ACCOUNT
        if (data.length > 0) {
          setActiveAccount(data[0].id);
        }
      })
      .catch(() => setAccounts([]));
  }, []);

  // ================= INITIAL LOAD =================
  useEffect(() => {
    if (!activeAccount) return;
  
    fetch(`/api/emails?account_id=${activeAccount}`)
      .then(res => res.json())
      .then((data: Email[]) => {
        setEmails(data);
  
        if (data.length > 0) {
          setSelected(data[0]);
  
          const last = data[data.length - 1];
          setLastTimestamp(
            Math.floor(new Date(last.created_at).getTime() / 1000)
          );
        }
      })
      .catch(console.error);
  }, [activeAccount]);

  // ================= LOAD MORE =================
  const loadMore = async () => {
    if (!activeAccount || !lastTimestamp || loadingMore) return;

    setLoadingMore(true);

    try {
      const res = await fetch(
        `/api/emails?account_id=${activeAccount}&before=${lastTimestamp}`
      );

      const data: Email[] = await res.json();

      if (data.length === 0) return;

      setEmails(prev => [...prev, ...data]);

      const last = data[data.length - 1];
      setLastTimestamp(
        Math.floor(new Date(last.created_at).getTime() / 1000)
      );
    } catch (err) {
      console.error(err);
    } finally {
      setLoadingMore(false);
    }
  };

  // ================= UI =================
  return (
    <div className="main">

      {/* SIDEBAR */}
      <div className="sidebar">

        <button onClick={() => setShowCompose(true)}>
          + Compose
        </button>

        <button onClick={connectGmail} className="add-email-btn">
          ➕ Add Email
        </button>

        <div className="account-list">
          {accounts.map(acc => (
            <div
              key={acc.id}
              className={`account-item ${
                activeAccount === acc.id ? "active" : ""
              }`}
              onClick={() => setActiveAccount(acc.id)}
            >
              📧 {acc.email}
            </div>
          ))}
        </div>

        {showCompose && (
          <div className="compose-modal">
            <div className="compose-box">
              <SendEmail />
              <button onClick={() => setShowCompose(false)}>
                Close
              </button>
            </div>
          </div>
        )}

        <h3>Mailbox</h3>
        <button>Inbox</button>
        <button>Sent</button>
        <button>Drafts</button>
      </div>

      {/* EMAIL LIST */}
      <div className="email-list">
        {emails.map(email => (
          <div
            key={email.id}
            className={`email-item ${
              selected?.id === email.id ? "active" : ""
            }`}
            onClick={() => setSelected(email)}
          >
            <h3>{email.subject}</h3>
          </div>
        ))}

        {/* ✅ Pagination button */}
        <div className="load-more-container">
        <button
          className="load-more-btn"
          onClick={loadMore}
          disabled={loadingMore}
        >
          {loadingMore ? (
            <>
              <span className="spinner"></span>
              Loading...
            </>
          ) : (
            "Load More Emails"
          )}
        </button>
      </div>
      </div>

      {/* EMAIL DETAIL */}
      <div className="email-detail">
        {selected ? (
          <>
            <h2>{selected.subject}</h2>
            <p><b>From:</b> {selected.sender}</p>
            <hr />

            <div className="email-body">
        {selected?.body ? (
          selected.body.includes("<") ? (
            <div
              className="email-html"
              dangerouslySetInnerHTML={{ __html: selected.body }}
            />
          ) : (
            <pre className="email-text">{selected.body}</pre>
          )
        ) : (
          <p className="empty">No content available</p>
        )}
      </div>
          </>
        ) : (
          <p>Select an email</p>
        )}
      </div>
    </div>
  );
}