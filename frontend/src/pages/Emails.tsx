import { useEffect, useState } from "react";

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
      .then(res => {
        console.log("STATUS:", res.status);
        return res.json();
      })
      .then(data => {
        console.log("DATA:", data);
        setEmails(data);
        if (data.length > 0) setSelected(data[0]);
      })
      .catch(err => console.error("FETCH ERROR:", err));
  }, []);

  return (
    <div style={{ display: "flex", height: "100vh" }}>
      
      {/* Sidebar */}
      <div style={{ width: 200, borderRight: "1px solid #ccc", padding: 10 }}>
        <h3>Mailbox</h3>
        <button>Inbox</button>
      </div>

      {/* Email List */}
      <div style={{ width: 300, borderRight: "1px solid #ccc", padding: 10 }}>
        <h2>Email List</h2>

        {emails.length === 0 ? (
          <p>No emails found</p>
        ) : (
          emails.map(email => (
            <div
              key={email.id}
              style={{
                padding: 10,
                cursor: "pointer",
                background:
                  selected?.id === email.id ? "#e8f0fe" : "white"
              }}
              onClick={() => setSelected(email)}
            >
              <h4>{email.subject}</h4>
              <p>{email.sender}</p>
            </div>
          ))
        )}
      </div>

      {/* Email Detail */}
      <div style={{ flex: 1, padding: 20 }}>
        <h2>Email Detail</h2>

        {selected ? (
          <>
            <h3>{selected.subject}</h3>
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