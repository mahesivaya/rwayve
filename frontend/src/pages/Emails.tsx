import { useEffect, useState } from "react";
import "./emails.css";

type Email = {
  id: number;
  sender: string;
  subject: string;
  body: string;
};


export default function Emails() {
  const [emails, setEmails] = useState<Email[]>([]);
  const [selected, setSelected] = useState<Email | null>(null);

  useEffect(() => {
    fetch("/api/emails")
      .then(res => res.json())
      .then((data: Email[]) => {
        setEmails(data);
        if (data.length > 0) setSelected(data[0]);
      })
      .catch(err => console.error(err));
  }, []);

  return (
    <div className="main">   {/* ❗ ONLY main content */}

      {/* SIDEBAR */}
      <div className="sidebar">
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