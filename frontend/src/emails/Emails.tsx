  import { useEffect, useState } from "react";
  import "./emails.css";
  import SendEmail from "../emails/send_email";

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
    const [lastId, setLastId] = useState<number | null>(null);
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

    // ================= LOAD =================
    useEffect(() => {
      const url =
        activeAccount !== null
          ? `/api/emails?account_id=${activeAccount}`
          : `/api/emails`;

      fetch(url)
        .then(res => res.json())
        .then((data: Email[]) => {
          setEmails(data);
          setSelected(data[0]);

          if (data.length) {
            const last = data[data.length - 1];
            setLastTimestamp(last.created_at);
            setLastId(last.id);
          }
        });
    }, [activeAccount]);

    // ================= LOAD MORE =================
    const loadMore = async () => {
      if (!lastTimestamp || !lastId || loadingMore) return;

      setLoadingMore(true);

      const url =
        activeAccount !== null
          ? `/api/emails?account_id=${activeAccount}&before=${lastTimestamp}&before_id=${lastId}`
          : `/api/emails?before=${lastTimestamp}&before_id=${lastId}`;

      const res = await fetch(url);
      const data: Email[] = await res.json();

      setEmails(prev => [...prev, ...data]);

      if (data.length) {
        const last = data[data.length - 1];
        setLastTimestamp(last.created_at);
        setLastId(last.id);
      }

      setLoadingMore(false);
    };

    return (
      <div className="main">

        {/* SIDEBAR */}
        {/* SIDEBAR */}
<div className="sidebar">

<button className="compose-btn" onClick={() => setShowCompose(true)}>
  + Compose
</button>

<button onClick={connectGmail} className="add-email-btn">
  ➕ Add Account
</button>

{/* 🔥 FILTER BUTTONS */}
<div className="mail-filters">

  {/* ✅ DEFAULT: ALL */}
  <button
    className={`filter-btn ${activeAccount === null ? "active" : ""}`}
    onClick={() => setActiveAccount(null)}
  >
    All
  </button>

  {/* ✅ ACCOUNTS */}
  {accounts.map(acc => (
    <button
      key={acc.id}
      className={`filter-btn ${
        activeAccount === acc.id ? "active" : ""
      }`}
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
        <div className="email-top">
          <span className="email-sender">{email.sender}</span>

          <span className="email-time">
            {new Date(email.created_at).toLocaleTimeString([], {
              hour: "2-digit",
              minute: "2-digit",
            })}
          </span>
        </div>

        <div className="email-subject">{email.subject}</div>
      </div>
    ))}

    <div className="load-more-container">
      <button onClick={loadMore} disabled={loadingMore}>
        {loadingMore ? "Loading..." : "Load More"}
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
                  <p>No content</p>
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