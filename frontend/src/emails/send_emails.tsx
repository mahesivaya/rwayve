import { useState } from "react";

export default function SendEmail() {
  const [to, setTo] = useState("");
  const [subject, setSubject] = useState("");
  const [body, setBody] = useState("");

  const [status, setStatus] = useState(""); // success/error message
  const [loading, setLoading] = useState(false);

  const sendEmail = async () => {
    // ✅ simple validation
    if (!to || !subject || !body) {
      setStatus("Please fill all fields ⚠️");
      return;
    }

    setLoading(true);
    setStatus("");

    try {
      const res = await fetch("/api/send", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          to,
          subject,
          body,
        }),
      });

      if (!res.ok) throw new Error("Failed");

      // ✅ success
      setStatus("Email sent successfully ✅");

      // reset fields
      setTo("");
      setSubject("");
      setBody("");

    } catch (err) {
      console.error(err);
      setStatus("Failed to send email ❌");
    }

    setLoading(false);

    // auto-hide after 3 sec
    setTimeout(() => setStatus(""), 3000);
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

      <button onClick={sendEmail} disabled={loading}>
        {loading ? "Sending..." : "Send"}
      </button>

      {/* ✅ status message */}
      {status && (
        <div className={`status ${status.includes("successfully") ? "success" : "error"}`}>
          {status}
        </div>
      )}

    </div>
  );
}