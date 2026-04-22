import { useEffect, useState } from "react";
import SendEmail from "./SendEmail";
import { decryptMessage } from "../crypto/crypto";
import { loadPrivateKey } from "../crypto/keyStore";

export default function Emails() {
  const [emails, setEmails] = useState<any[]>([]);
  const [accounts, setAccounts] = useState<any[]>([]);
  const [activeAccount, setActiveAccount] = useState<number | null>(null);

  const [selected, setSelected] = useState<any>(null);
  const [privateKey, setPrivateKey] = useState<CryptoKey | null>(null);

  const [showCompose, setShowCompose] = useState(false);

  // 🔐 Load private key
  useEffect(() => {
    (async () => {
      const key = await loadPrivateKey();
      if (key) setPrivateKey(key);
    })();
  }, []);

  const fetchAccounts = async () => {
    const token = localStorage.getItem("token");
  
    const res = await fetch("http://localhost:8080/api/accounts", {
      headers: { Authorization: `Bearer ${token}` },
    });
  
    const data = await res.json();
    setAccounts(data);
  };
  
  useEffect(() => {
    fetchAccounts();
  }, []);


  useEffect(() => {
    const urlParams = new URLSearchParams(window.location.search);
  
    if (urlParams.get("connected") === "true") {
      window.history.replaceState({}, document.title, "/emails");
      fetchAccounts(); // 🔥 refresh
    }
  }, []);


  // 📥 Fetch emails
  useEffect(() => {
    const fetchEmails = async () => {
      const token = localStorage.getItem("token");
  
      let url = "http://localhost:8080/api/emails";
  
      if (activeAccount !== null) {
        url += `?account_id=${activeAccount}`;
      }
  
      console.log("Fetching:", url);
  
      const res = await fetch(url, {
        headers: { Authorization: `Bearer ${token}` },
      });
  
      const data = await res.json();
      console.log("EMAILS:", data);
  
      setEmails(data);
    };
  
    fetchEmails();
  }, [activeAccount]);

// Connect to gmail
const connectGmail = () => {
  const token = localStorage.getItem("token");
  if (!token) {
    alert("Login required ❌");
    return;
  }

  window.location.href =
    `http://localhost:8080/gmail/login?token=${token}`;
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
      console.error("Decrypt failed", err);
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
          <button onClick={() => setShowCompose(true)}>+ Compose</button>
          <button style={{ marginLeft: 10 }}onClick={connectGmail}>➕ Add Account</button>
        </div>

        {/* 🔥 ACCOUNT FILTER */}
        <div style={{ padding: 10 }}>
          {/* ALL */}
          <button
            onClick={() => setActiveAccount(null)}
            style={{
              marginRight: 5,
              background: activeAccount === null ? "#ddd" : "white"
            }}
          >
            All
          </button>

          {/* ACCOUNTS */}
          {accounts.map((acc) => (
            <button
              key={acc.id}
              onClick={() => setActiveAccount(acc.id)}
              style={{
                marginRight: 5,
                background: activeAccount === acc.id ? "#ddd" : "white"
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
              key={email.id}
              style={{ padding: 10, cursor: "pointer" }}
              onClick={() => openEmail(email)}
            >
              <strong>{email.sender}</strong>
              <div>{email.subject}</div>

              {email.body?.startsWith("WAYVE_SECURE_V1") && (
                <span>🔐</span>
              )}
            </div>
          ))}
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
          top: 0,
          left: 0,
          width: "100%",
          height: "100%",
          background: "rgba(0,0,0,0.5)"
        }}>
          <div style={{
            background: "white",
            width: 500,
            margin: "100px auto",
            padding: 20
          }}>
            <SendEmail />
            <button onClick={() => setShowCompose(false)}>Close</button>
          </div>
        </div>
      )}

    </div>
  );
}