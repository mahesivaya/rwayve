// src/pages/Emails.jsx
import { useEffect, useState } from "react";
import "./emails.css";

export default function Emails() {
  const [emails, setEmails] = useState([]);
  const [selected, setSelected] = useState(null);

  useEffect(() => {
    fetch("/api/emails")
      .then(res => res.json())
      .then(data => {
        setEmails(data);
        if (data.length > 0) setSelected(data[0]);
      });
  }, []);

  return (
    <div className="email-container">
      
      {/* Sidebar */}
      <div className="sidebar">
        <h3>Mailbox</h3>
        <button>Inbox</button>
        <button>Sent</button>
        <button>Drafts</button>
      </div>

      {/* Email List */}
      <div className="email-list">
        {emails.map(email => (
          <div
            key={email.id}
            className={`email-item ${selected?.id === email.id ? "active" : ""}`}
            onClick={() => setSelected(email)}
          >
            <h4>{email.subject}</h4>
            <p>{email.sender}</p>
          </div>
        ))}
      </div>

      {/* Email Detail */}
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