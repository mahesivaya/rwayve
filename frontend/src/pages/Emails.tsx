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

  const [lastTimestamp, setLastTimestamp] = useState<string | null>(null);
  const [lastId, setLastId] = useState<number | null>(null); // 🔥 FIXED
  const [loadingMore, setLoadingMore] = useState(false);

  const connectGmail = () => {
    window.location.href = "http://localhost:8080/gmail/login";
  };

  // ================= ACCOUNTS =================
  useEffect(() => {
    fetch("/api/accounts")
      .then(res => res.json())
      .then(setAccounts)
      .catch(() => setAccounts([]));
  }, []);

  // ================= INITIAL LOAD =================
  useEffect(() => {
    if (activeAccount === undefined) return;
  
    setEmails([]);
    setSelected(null);
    setLastTimestamp(null);
    setLastId(null);
  
    const url =
      activeAccount !== null
        ? `/api/emails?account_id=${activeAccount}`
        : `/api/emails`;
  
    console.log("Initial Fetch:", url);
  
    fetch(url)
      .then(async (res) => {
        if (!res.ok) {
          console.error(await res.text());
          return [];
        }
        return res.json();
      })
      .then((data: Email[]) => {
        if (!data.length) return;
  
        setEmails(data);
        setSelected(data[0]);
  
        const last = data[data.length - 1];
  
        // 🔥 FIX: USE FULL TIMESTAMP STRING
        setLastTimestamp(last.created_at);
        setLastId(last.id);
      })
      .catch(console.error);
  
  }, [activeAccount]);

  // ================= LOAD MORE =================
  const loadMore = async () => {
    if (
      lastTimestamp === null ||
      lastId === null ||
      emails.length === 0 ||
      loadingMore
    ) return;
  
    setLoadingMore(true);
  
    const url =
      activeAccount !== null
        ? `/api/emails?account_id=${activeAccount}&before=${encodeURIComponent(lastTimestamp)}&before_id=${lastId}`
        : `/api/emails?before=${encodeURIComponent(lastTimestamp)}&before_id=${lastId}`;
  
    console.log("LoadMore URL:", url);
  
    try {
      const res = await fetch(url);
  
      if (!res.ok) {
        console.error(await res.text());
        return;
      }
  
      const data: Email[] = await res.json();
  
      if (!data.length) return;
  
      // 🔥 DEDUPE + MERGE
      setEmails((prev) => {
        const map = new Map(prev.map((e) => [e.id, e]));
        data.forEach((e) => map.set(e.id, e));
        return Array.from(map.values());
      });
  
      const newLast = data[data.length - 1];
  
      // 🔥 FIX: KEEP STRING TIMESTAMP
      setLastTimestamp(newLast.created_at);
      setLastId(newLast.id);
  
    } catch (err) {
      console.error("LoadMore error:", err);
    } finally {
      setLoadingMore(false);
    }
  };

  // ================= UI =================
  return (
    <div className="main">

      {/* SIDEBAR */}
      <div className="sidebar">
        <button onClick={() => setShowCompose(true)}>+ Compose</button>

        <button onClick={connectGmail} className="add-email-btn">
          ➕ Add Email
        </button>

        <div
          className={`account-item ${activeAccount === null ? "active" : ""}`}
          onClick={() => setActiveAccount(null)}
        >
          📥 ALL
        </div>

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
              <button onClick={() => setShowCompose(false)}>Close</button>
            </div>
          </div>
        )}
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

        <div className="load-more-container">
          <button onClick={loadMore} disabled={loadingMore}>
            {loadingMore ? "Loading..." : "Load More Emails"}
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
              {selected.body ? (
                selected.body.includes("<") ? (
                  <div dangerouslySetInnerHTML={{ __html: selected.body }} />
                ) : (
                  <pre>{selected.body}</pre>
                )
              ) : (
                <p>No content available</p>
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