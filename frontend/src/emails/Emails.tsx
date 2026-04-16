  import { useEffect, useState } from "react";
  import "./emails.css";
  import SendEmail from "../emails/send_email";

  type Email = {
    id: number;
    sender: string;
    receiver: string,
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

    const connectGmail = () => {
      const token = localStorage.getItem("token");
    
      if (!token) {
        alert("User not logged in");
        return;
      }
    
      window.location.href = `http://localhost:8080/gmail/login?token=${token}`;
    };

    // ================= ACCOUNTS =================
    useEffect(() => {
      const token = localStorage.getItem("token");
    
      fetch(`${BASE_URL}/api/accounts`, {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      })
        .then(res => {
          if (!res.ok) {
            throw new Error("Unauthorized");
          }
          return res.json();
        })
        .then(setAccounts)
        .catch(() => setAccounts([]));
    }, []);

    // ================= LOAD =================
    useEffect(() => {
  const token = localStorage.getItem("token");

  const url =
    activeAccount !== null
      ? `/api/emails?account_id=${activeAccount}`
      : `/api/emails`;

  fetch(`${BASE_URL}${url}`, {
    headers: {
      Authorization: `Bearer ${token}`,
    },
  })
    .then(res => {
      if (!res.ok) throw new Error("Unauthorized");
      return res.json();
    })
    .then((data: Email[]) => {
      setEmails(data);
      setSelected(data[0]);

      if (data.length) {
        const last = data[data.length - 1];
        setLastTimestamp(last.created_at);
        setLastId(last.id);
      }
    })
    .catch(err => {
      console.error(err);
      setEmails([]);
    });
}, [activeAccount]);

    // ================= LOAD MORE =================
    
    const loadMore = async () => {
      if (!lastTimestamp || !lastId || loadingMore) return;
    
      setLoadingMore(true);
    
      try {
        const token = localStorage.getItem("token");
    
        if (!token) {
          console.error("❌ No token found");
          setLoadingMore(false);
          return;
        }
    
        const url =
          activeAccount !== null
            ? `/api/emails?account_id=${activeAccount}&before=${lastTimestamp}&before_id=${lastId}`
            : `/api/emails?before=${lastTimestamp}&before_id=${lastId}`;
    
        const res = await fetch(`${BASE_URL}${url}`, {
          headers: {
            Authorization: `Bearer ${token}`, // 🔥 FIX 1
          },
        });
    
        // 🔥 FIX 2 — prevent JSON crash
        if (!res.ok) {
          const text = await res.text();
          console.error("❌ Load more API error:", text);
          setLoadingMore(false);
          return;
        }
    
        const data: Email[] = await res.json();
    
        // 🔥 FIX 3 — prevent duplicates (important)
        setEmails(prev => {
          const existingIds = new Set(prev.map(e => e.id));
          const newItems = data.filter(e => !existingIds.has(e.id));
          return [...prev, ...newItems];
        });
    
        // 🔥 FIX 4 — update cursor safely
        if (data.length > 0) {
          const last = data[data.length - 1];
          setLastTimestamp(last.created_at);
          setLastId(last.id);
        }
    
      } catch (err) {
        console.error("❌ Load more failed:", err);
      } finally {
        setLoadingMore(false);
      }
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
            <p>Select an email</p>
          )}
        </div>
      </div>
    );
  }