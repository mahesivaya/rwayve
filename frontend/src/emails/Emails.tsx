import { logger } from "../utils/logger";
const API_BASE = import.meta.env.VITE_API_URL;

import { useEffect, useRef, useState } from "react";
import SendEmail from "./SendEmail";
import { decryptMessage } from "../crypto/crypto";
import { loadPrivateKey } from "../crypto/keyStore";
import { apiFetch } from "../api";


export default function Emails() {
  const [emails, setEmails] = useState<any[]>([]);
  const [loadingMore, setLoadingMore] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  const [accounts, setAccounts] = useState<any[]>([]);
  const [activeAccount, setActiveAccount] = useState<number | null>(null);

  const [selected, setSelected] = useState<any>(null);
  const [privateKey, setPrivateKey] = useState<CryptoKey | null>(null);

  const [showCompose, setShowCompose] = useState(false);

  const clickTimerRef = useRef<number | null>(null);
  

  // 🔐 Load private key
useEffect(() => {
  const initKey = async () => {
    try {
      const key = await loadPrivateKey();
      if (key) setPrivateKey(key);
    } catch (err) {
      logger.error("❌ Failed to load private key:", err);
    }
  };

  initKey();
}, []);



  // 📧 Fetch accounts (production-safe)
  const fetchAccounts = async () => {
    try {
      const res = await apiFetch("/api/accounts");
      const data = await res.json();
      setAccounts(data);
    } catch (err) {
      logger.error(err);
    }
  };


  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
  
    if (params.get("connected") === "true") {
      logger.log("🔄 Refreshing accounts after OAuth");
  
      fetchAccounts();
  
      // clean URL
      window.history.replaceState({}, document.title, "/emails");
    }
  }, []);
  
  useEffect(() => {
    fetchAccounts();
  }, []);


  // 📥 Fetch emails
  useEffect(() => {
    const fetchEmails = async () => {
      const token = localStorage.getItem("token");
  
      let url = `${API_BASE}/api/emails`;
  
      if (activeAccount !== null) {
        url += `?account_id=${activeAccount}`;
      }
  
      const res = await fetch(url, {
        headers: { Authorization: `Bearer ${token}` },
      });
  
      const data = await res.json();
  
      setEmails(data);
      setHasMore(data.length === 50); // pagination check
    };
  
    fetchEmails();
  }, [activeAccount]);


  const loadMore = async () => {
    if (emails.length === 0 || !hasMore) return;
  
    setLoadingMore(true);
  
    const token = localStorage.getItem("token");
  
    const last = emails[emails.length - 1];
  
    const before = Math.floor(new Date(last.created_at).getTime() / 1000);
    const before_id = last.id;
  
    let url = `${API_BASE}/api/emails?before=${before}&before_id=${before_id}`;
  
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



// Connect to gmail
const connectGmail = () => {
  const token = localStorage.getItem("token");
  if (!token) {
    alert("Login required ❌");
    return;
  }

  window.location.href =
    `${API_BASE}/gmail/login?token=${token}`;
};


  // 🔓 Open email
  const openEmail = async (email: any) => {
    let bodyText = email.body;

    try {
      if (privateKey && email.body?.startsWith("WAYVE_SECURE_V1")) {
        let raw = email.body.replace("WAYVE_SECURE_V1", "").trim();
        const payload = JSON.parse(raw);

        const decrypted = await decryptMessage(
          new Uint8Array(payload.data),
          new Uint8Array(payload.key),
          new Uint8Array(payload.iv),
          privateKey
        );

        bodyText = decrypted;
      }
    } catch (err) {
      logger.error("Decrypt failed", err);
      bodyText = "❌ Unable to decrypt";
    }

    setSelected({ ...email, body: bodyText });
  };

  return (
    <div style={{ display: "flex", height: "100%" }}>

      {/* LEFT PANEL */}
      <div style={{ width: "35%", borderRight: "1px solid #ddd" }}>

        {/* 🔥 TOP ACTIONS */}
        <div style={{ padding: 10 }}>

  {/* PRIMARY */}
  <button
    onClick={() => setShowCompose(true)}
    style={{
      width: "100%",
      background: "#007bff",
      color: "white",
      padding: "10px",
      borderRadius: 6,
      border: "none",
      marginBottom: 10
    }}
  >
    + Compose
  </button>

  {/* SECONDARY */}
  <button
    onClick={connectGmail}
    style={{
      width: "100%",
      background: "#f5f5f5",
      padding: "10px",
      borderRadius: 6,
      border: "1px solid #ddd"
    }}
  >
    ➕ Add Account
  </button>

</div>

<div style={{ padding: 10, display: "flex", flexDirection: "column" }}>
  
  {/* ALL */}
  <button
    onClick={() => {
      if (activeAccount === null) return;
      setActiveAccount(null);
      setEmails([]);
      setHasMore(true);
    }}
    onDoubleClick={(e) => {
      e.preventDefault();
      e.stopPropagation();
    }}
    style={{
      marginBottom: 5,                 // 🔥 vertical spacing
      textAlign: "left",
      background: activeAccount === null ? "#ddd" : "white",
      userSelect: "none"
    }}
  >
    All
  </button>

  {/* ACCOUNTS */}
  {accounts.map((acc) => (
    <button
      key={acc.id}
      onClick={() => {
        if (activeAccount === acc.id) return;
        setActiveAccount(acc.id);
        setEmails([]);
        setHasMore(true);
      }}
      onDoubleClick={(e) => {
        e.preventDefault();
        e.stopPropagation();
      }}
      style={{
        marginBottom: 5,               // 🔥 vertical spacing
        textAlign: "left",
        background: activeAccount === acc.id ? "#ddd" : "white",
        userSelect: "none"
      }}
    >
      {acc.email}
    </button>
  ))}

</div>

        {/* 🔥 EMAIL LIST */}
        <div style={{ overflowY: "auto", height: "80%" }}>
          {emails.map((email) => (
            <div
            key={`${email.account_id}-${email.gmail_id || email.id}-${email.created_at}`}
              style={{ padding: 10, cursor: "pointer", userSelect: "none" }}
              onClick={(e) => {
                e.preventDefault();
                if (clickTimerRef.current !== null) {
                  window.clearTimeout(clickTimerRef.current);
                  clickTimerRef.current = null;
                }
                clickTimerRef.current = window.setTimeout(() => {
                  clickTimerRef.current = null;
                  openEmail(email);
                }, 220);
              }}
              onDoubleClick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                if (clickTimerRef.current !== null) {
                  window.clearTimeout(clickTimerRef.current);
                  clickTimerRef.current = null;
                }
              }}
            >
              <strong>{email.sender}</strong>
              <div>{email.subject}</div>

              {email.body?.startsWith("WAYVE_SECURE_V1") && (
                <span>🔐</span>
              )}
            </div>
          ))}
          {hasMore && (
          <button onClick={loadMore} disabled={loadingMore}>
            {loadingMore ? "Loading..." : "Load More"}
          </button>
        )}
        </div>
      </div>

      {/* RIGHT PANEL */}
      <div style={{ flex: 1, padding: 20 }}>
        {selected ? (
          <>
            <h2>{selected.subject}</h2>

            {selected.body?.startsWith("WAYVE_SECURE_V1") ? (
              <p>{selected.body}</p>
            ) : (
              <div dangerouslySetInnerHTML={{ __html: selected.body }} />
            )}
          </>
        ) : (
          <p>Select an email</p>
        )}
      </div>



      {/* 🔥 COMPOSE MODAL */}
      {showCompose && (
  <div style={{
    position: "fixed",
    bottom: 20,
    right: 20,
    width: 400,
    height: 500,
    background: "white",
    boxShadow: "0 4px 20px rgba(0,0,0,0.2)",
    borderRadius: 8,
    display: "flex",
    flexDirection: "column",
    zIndex: 1000
  }}>

    {/* HEADER */}
    <div style={{
      background: "#007bff",
      color: "white",
      padding: "10px",
      borderTopLeftRadius: 8,
      borderTopRightRadius: 8,
      display: "flex",
      justifyContent: "space-between",
      alignItems: "center"
    }}>
      <span>New Message</span>
      <button
        onClick={() => setShowCompose(false)}
        style={{
          background: "transparent",
          border: "none",
          color: "white",
          fontSize: 16,
          cursor: "pointer"
        }}
      >
        ✕
      </button>
    </div>

    {/* BODY */}
    <div style={{
      flex: 1,
      overflow: "auto",
      padding: 10
    }}>
      <SendEmail />
    </div>

  </div>
)}

      

    </div>
  );
}