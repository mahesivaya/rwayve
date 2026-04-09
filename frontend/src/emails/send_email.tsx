import { useState, useEffect } from "react";

export default function SendEmail() {
  const [accountId, setAccountId] = useState(1);
  const [to, setTo] = useState("");
  const [subject, setSubject] = useState("");
  const [body, setBody] = useState("");

  const [status, setStatus] = useState("");
  const [loading, setLoading] = useState(false);

  // auto-hide status
  useEffect(() => {
    if (!status) return;
    const timer = setTimeout(() => setStatus(""), 3000);
    return () => clearTimeout(timer);
  }, [status]);

  const sendEmail = async () => {
    console.log("Sending email to: ", to);
    console.log("Subject: ", subject);
    console.log("Body: ", body);
    if (!to || !subject || !body) {
      setStatus("Please fill all fields ⚠️");
      return;
    }

    const token = localStorage.getItem("token");

    if (!token) {
      setStatus("You must login first ❌");
      return;
    }

    setLoading(true);
    setStatus("");

    try {
      const res = await fetch("http://localhost:8080/api/send", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${token}`, // 🔥 IMPORTANT
        },
        body: JSON.stringify({
          account_id: 1,
          to,
          subject,
          body,
        }),
      });

      const text = await res.text();

      if (!res.ok) {
        throw new Error(text || "Failed to send");
      }

      setStatus("Email sent successfully ✅");

      setTo("");
      setSubject("");
      setBody("");

    } catch (err: any) {
      console.error(err);
      setStatus(err.message || "Failed to send email ❌");
    }

    setLoading(false);
  };

  return (
    <div className="send-container">
      <h3>Compose Email</h3>

      <input
        placeholder="To"
        value={to}
        onChange={e => setTo(e.target.value)}
      />

      <input
        placeholder="Subject"
        value={subject}
        onChange={e => setSubject(e.target.value)}
      />

      <textarea
        placeholder="Message"
        value={body}
        onChange={e => setBody(e.target.value)}
      />

      <button
        onClick={() => {
          console.log("🔥 BUTTON CLICKED");
          sendEmail();
        }}
        disabled={loading}
      >
        {loading ? "Sending..." : "Send"}
      </button>

      {status && (
        <div className={`status ${status.includes("success") ? "success" : "error"}`}>
          {status}
        </div>
      )}
    </div>
  );
}