import { useEffect, useState } from "react";
import SendEmail from "../emails/send_emails";
import "./emails.css";

type Email = {
  id: number;
  sender: string;
  subject: string;
  body: string;
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
  const [sentMessage, setSentMessage] = useState("");
  const connectGmail = () => {
    window.location.href = "http://localhost:8080/gmail/login";
  };

  useEffect(() => {
    fetch("/api/accounts")
    .then(res => {
      if (!res.ok) throw new Error("API failed");
      return res.json();
    })
    .then(data => setAccounts(data))
    .catch(err => {
      console.error("Accounts error:", err);
      setAccounts([]);
    });
  },[]);

  useEffect(() => {
    fetch("/api/emails")
      .then(res => res.json())
      .then((data: Email[]) => {
        setEmails(data);
        if (data.length > 0) setSelected(data[0]);
      })
      .catch(err => console.error(err));
  }, []);

  useEffect(() => {
    if (!activeAccount) return;

    fetch(`/api/emails?account=${activeAccount}`)
      .then(res => res.json())
      .then((data: Email[]) => {
        setEmails(data);
        setSelected(data.length > 0 ? data[0] : null);
      })
      .catch(err => console.error(err));
  }, [activeAccount]);


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
          {accounts.length === 0 && (
            <div className="no-accounts">No accounts added</div>
          )}

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
        </div>
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
            <h4>{email.subject}</h4>
            <p>{email.sender}</p>
          </div>
        ))}
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
          <div dangerouslySetInnerHTML={{ __html: selected.body }} />
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