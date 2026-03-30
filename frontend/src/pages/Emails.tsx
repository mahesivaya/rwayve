import { useEffect, useState } from "react";
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

  const connectGmail = () => {
    window.location.href = "http://localhost:8080/gmail/login";
  };

  useEffect(() => {
    fetch("/api/accounts")
      .then(res => res.json())
      .then((data: Account[]) => {
        setAccounts(data);

        // Auto select first account
        if (data.length > 0) {
          setActiveAccount(data[0].id);
        }
      })
      .catch(err => console.error(err));
  }, []);

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
            <p>{selected.body}</p>
          </>
        ) : (
          <p>Select an email</p>
        )}
      </div>

    </div>
  );
}