import { logger } from "../utils/logger";
import { sendEmail as sendEmailApi } from "../api/email";
import { getAuthToken } from "../auth/token";

import { useState, useEffect } from "react";
import {buildEncryptedBody, type EmailEncryptionMode} from "./encryptEmail";

type SendEmailProps = {
  accountId: number;
  onClose?: () => void;
  onSent?: () => void;
};

export default function SendEmail({ accountId, onClose, onSent }: SendEmailProps) {
  const [to, setTo] = useState("");
  const [subject, setSubject] = useState("");
  const [body, setBody] = useState("");
  const [encryptionMode, setEncryptionMode] =
    useState<EmailEncryptionMode>("fully_encrypted");

  const [status, setStatus] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!status) return;
    const timer = setTimeout(() => setStatus(""), 3000);
    return () => clearTimeout(timer);
  }, [status]);

  const sendEmail = async () => {
    if (!to || !subject || !body) {
      setStatus("Please fill all fields ⚠️");
      return;
    }

    const token = getAuthToken();
    if (!token) {
      setStatus("You must login first ❌");
      return;
    }

    setLoading(true);
    setStatus("");

    try {
      logger.warn("📨 BEFORE ENCRYPT:",body);
    
      const finalBody =
        await buildEncryptedBody(
          to,
          body,
          token,
          encryptionMode
        );
    
      logger.warn("🔐 AFTER ENCRYPT:",finalBody);

      await sendEmailApi({
        account_id: accountId,
        to,
        subject,
        body: finalBody,
      });

      setStatus("Email sent successfully ✅");
      setTo("");
      setSubject("");
      setBody("");
      setEncryptionMode("fully_encrypted");
      onSent?.();
      setTimeout(() => onClose?.(), 800);
    } catch (err: any) {
      logger.error(err);
      setStatus(err.message || "Failed to send email ❌");
    }finally{
    setLoading(false);
    }
  };

  return (
    <div style={{
      display: "flex",
      flexDirection: "column",
      gap: "10px"
    }}>
  
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h3 style={{ margin: 0 }}>Compose Email</h3>
        {onClose && (
          <button
            onClick={onClose}
            style={{
              background: "transparent",
              border: "none",
              fontSize: 18,
              cursor: "pointer",
              color: "#6b7280"
            }}
            aria-label="Close"
          >
            ✕
          </button>
        )}
      </div>
  
      <input
        placeholder="To"
        value={to}
        onChange={(e) => setTo(e.target.value)}
        style={{
          padding: "8px",
          borderRadius: 5,
          border: "1px solid #ccc"
        }}
      />
  
      <input
        placeholder="Subject"
        value={subject}
        onChange={(e) => setSubject(e.target.value)}
        style={{
          padding: "8px",
          borderRadius: 5,
          border: "1px solid #ccc"
        }}
      />
  
      <textarea
        placeholder="Message"
        value={body}
        onChange={(e) => setBody(e.target.value)}
        style={{
          padding: "8px",
          borderRadius: 5,
          border: "1px solid #ccc",
          minHeight: 120,
          resize: "none"
        }}
      />

      <div
        role="radiogroup"
        aria-label="Email encryption type"
        style={{
          display: "grid",
          gap: 8,
          padding: "10px",
          border: "1px solid #d1d5db",
          borderRadius: 6,
          background: "#f9fafb"
        }}
      >
        <label
          style={{
            display: "grid",
            gridTemplateColumns: "18px 1fr",
            gap: 8,
            alignItems: "start",
            cursor: "pointer"
          }}
        >
          <input
            type="radio"
            name="email-encryption"
            value="fully_encrypted"
            checked={encryptionMode === "fully_encrypted"}
            onChange={() => setEncryptionMode("fully_encrypted")}
            style={{ marginTop: 2 }}
          />
          <span>
            <strong>Fully encrypted</strong>
            <span
              style={{
                display: "block",
                color: "#4b5563",
                fontSize: 12,
                lineHeight: 1.35
              }}
            >
              Only Wayve users can decrypt and read this email inside Wayve.
            </span>
          </span>
        </label>

        <label
          style={{
            display: "grid",
            gridTemplateColumns: "18px 1fr",
            gap: 8,
            alignItems: "start",
            cursor: "pointer"
          }}
        >
          <input
            type="radio"
            name="email-encryption"
            value="standard"
            checked={encryptionMode === "standard"}
            onChange={() => setEncryptionMode("standard")}
            style={{ marginTop: 2 }}
          />
          <span>
            <strong>Standard encryption</strong>
            <span
              style={{
                display: "block",
                color: "#4b5563",
                fontSize: 12,
                lineHeight: 1.35
              }}
            >
              Sends normal email content that can also be viewed in Gmail.
            </span>
          </span>
        </label>
      </div>
  
      <button
        onClick={sendEmail}
        disabled={loading}
        style={{
          background: "#007bff",
          color: "white",
          padding: "10px",
          borderRadius: 5,
          border: "none",
          cursor: "pointer"
        }}
      >
        {loading ? "Sending..." : "Send"}
      </button>
  
      {status && (
        <div style={{
          fontSize: 12,
          color: status.includes("success") ? "green" : "red"
        }}>
          {status}
        </div>
      )}
  
    </div>
  );
}
