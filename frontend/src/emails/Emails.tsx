import { useEffect, useRef, useState } from "react";
import "./emails.css";
import "./loadMore.css";
import SendEmail from "./SendEmail";

import {
  getAccounts,
  getEmail,
  getEmailBody,
  getEmails,
  getGmailLoginUrl,
} from "../api/email";
import { decryptMessage } from "../crypto/crypto";
import { loadPrivateKey } from "../crypto/keyStore";
import { useAuth } from "../auth/AuthContext";
import { useGlobalSearch } from "../search/SearchContext";

type Email = {
  id: number;
  sender: string;
  receiver: string;
  subject: string;
  preview?: string;
  body?: string;
  created_at: string;
};

type WayveEncryptedBody = {
  type: "wayve_encrypted";
  data: number[];
  key: number[];
  iv: number[];
};

const WAYVE_SECURE_PREFIX = "WAYVE_SECURE_V1";

function normalizeEmailBody(body: string) {
  if (!/[<&][a-zA-Z#/!]/.test(body)) {
    return body;
  }

  const doc = new DOMParser().parseFromString(body, "text/html");

  doc
    .querySelectorAll("script, style, noscript, svg")
    .forEach((node) => node.remove());

  doc
    .querySelectorAll("br")
    .forEach((node) => node.replaceWith(doc.createTextNode("\n")));

  doc
    .querySelectorAll("p, div, section, article, header, footer, tr, table")
    .forEach((node) => node.append(doc.createTextNode("\n")));

  doc
    .querySelectorAll("li")
    .forEach((node) => node.prepend(doc.createTextNode("\n- ")));

  const text = doc.body.textContent || body;

  return text
    .replace(/\u00a0/g, " ")
    .replace(/[ \t]+\n/g, "\n")
    .replace(/\n[ \t]+/g, "\n")
    .replace(/[ \t]{2,}/g, " ")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

function parseWayveEncryptedBody(body: string): WayveEncryptedBody | null {
  const trimmed = normalizeEmailBody(body).trim();

  if (!trimmed.startsWith(WAYVE_SECURE_PREFIX)) {
    return null;
  }

  const jsonStart = trimmed.indexOf("{");
  if (jsonStart === -1) {
    throw new Error("Encrypted Wayve email is missing its payload");
  }

  const jsonEnd = trimmed.lastIndexOf("}");
  if (jsonEnd < jsonStart) {
    throw new Error("Encrypted Wayve email payload is incomplete");
  }

  const parsed = JSON.parse(trimmed.slice(jsonStart, jsonEnd + 1));

  if (
    parsed?.type !== "wayve_encrypted" ||
    !Array.isArray(parsed.data) ||
    !Array.isArray(parsed.key) ||
    !Array.isArray(parsed.iv)
  ) {
    throw new Error("Encrypted Wayve email payload is invalid");
  }

  return parsed;
}

function emailBodyErrorMessage(err: unknown) {
  const message = err instanceof Error ? err.message : "";

  if (
    message.includes("private key") ||
    message.includes("decrypt") ||
    message.includes("operation failed")
  ) {
    return "Unable to decrypt this fully encrypted email on this device. Sign out and back in to refresh your Wayve encryption key, then ask the sender to resend it.";
  }

  if (message) {
    return message;
  }

  return "Failed to load email body. Try again.";
}

async function decryptWayveBodyIfNeeded(
  body: string,
  userId?: number | null
): Promise<string> {
  const encrypted = parseWayveEncryptedBody(body);

  if (!encrypted) {
    return normalizeEmailBody(body);
  }

  const privateKeys: CryptoKey[] = [];
  const scopedPrivateKey = await loadPrivateKey(userId);

  if (scopedPrivateKey) {
    privateKeys.push(scopedPrivateKey);
  }

  if (userId) {
    const legacyPrivateKey = await loadPrivateKey();
    if (legacyPrivateKey && legacyPrivateKey !== scopedPrivateKey) {
      privateKeys.push(legacyPrivateKey);
    }
  }

  if (privateKeys.length === 0) {
    throw new Error("This device does not have your Wayve private key");
  }

  let lastError: unknown = null;

  for (const privateKey of privateKeys) {
    try {
      return await decryptMessage(
        new Uint8Array(encrypted.data),
        new Uint8Array(encrypted.key),
        new Uint8Array(encrypted.iv),
        privateKey
      ).then(normalizeEmailBody);
    } catch (err) {
      lastError = err;
    }
  }

  throw lastError || new Error("Unable to decrypt Wayve email");
}

export default function Emails() {
  const { user } = useAuth();
  const { normalizedSearchQuery } = useGlobalSearch();
  const [accounts, setAccounts] = useState<any[]>([]);
  const [emails, setEmails] = useState<any[]>([]);
  const [selectedEmail, setSelectedEmail] = useState<any | null>(null);

  const [activeAccount, setActiveAccount] = useState<number | null>(null);
  const [activeFolder, setActiveFolder] = useState<"inbox" | "sent">("inbox");

  const [hasMore, setHasMore] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [refreshTick, setRefreshTick] = useState(0);

  const [composeOpen, setComposeOpen] = useState(false);

  const emailCache = useRef<{ [key: number]: any }>({});

  // ================= NARROW MODE (split-pane / small viewport) =================
  // When the container is narrow (e.g. rendered inside the split view), we
  // collapse the 3-pane layout to a stacked one: show the list OR the detail,
  // not both. The threshold is the container width — independent of viewport
  // size, so this also responds correctly to a resized split.
  const mainRef = useRef<HTMLDivElement>(null);
  const [isNarrow, setIsNarrow] = useState(false);

  useEffect(() => {
    const el = mainRef.current;
    if (!el) return;
    const ro = new ResizeObserver((entries) => {
      for (const entry of entries) {
        setIsNarrow(entry.contentRect.width < 800);
      }
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const showList = !isNarrow || selectedEmail === null;
  const showDetail = !isNarrow || selectedEmail !== null;

  // ================= FETCH ACCOUNTS =================
  const fetchAccounts = async () => {
    try{
      const data = await getAccounts();
      setAccounts(data);
    }
    catch(err){
    console.error(err)
    }
  };

  useEffect(() => {
    fetchAccounts();
  }, []);

  // ================= HANDLE OAUTH RETURN =================
  // After /oauth/callback redirects back with ?connected=true, refresh the
  // account list so the newly linked account shows up immediately. The 30s
  // sync worker will import its emails on the next tick.
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    if (params.get("connected") === "true") {
      fetchAccounts();
      setRefreshTick((tick) => tick + 1);
      window.history.replaceState({}, "", "/emails");

      let attempts = 0;
      const poll = window.setInterval(() => {
        attempts += 1;
        setRefreshTick((tick) => tick + 1);

        if (attempts >= 12) {
          window.clearInterval(poll);
        }
      }, 2000);

      return () => window.clearInterval(poll);
    }
  }, []);

  // ================= ADD ACCOUNT =================
  const addAccount = () => {
    const token = localStorage.getItem("token");
    if (!token) return;
    window.location.href = getGmailLoginUrl(token);
  };

  // ================= FETCH EMAILS =================
  useEffect(() => {
    const fetchEmails = async () => {
      const { emails: data, hasMore: hasMorePage } = await getEmails({
        folder: activeFolder,
        accountId: activeAccount,
        query: normalizedSearchQuery,
      });

      setEmails(data);
      setHasMore(hasMorePage || data.length === 50);
      setSelectedEmail(null);
    };

    void fetchEmails();
  }, [activeAccount, activeFolder, refreshTick, normalizedSearchQuery]);

  // ================= LOAD MORE =================
  const loadMore = async () => {
    if (!hasMore || emails.length === 0) return;

    setLoadingMore(true);

    try{
    const last = emails[emails.length - 1];

    const before = Math.floor(new Date(last.created_at).getTime() / 1000);
    const before_id = last.id;

    const { emails: data, hasMore: hasMorePage } = await getEmails({
      folder: activeFolder,
      accountId: activeAccount,
      query: normalizedSearchQuery,
      before,
      beforeId: before_id,
    });

    setEmails((prev) => [...prev, ...data]);
    setHasMore(hasMorePage);
  }finally{
    setLoadingMore(false);
    }
  };

  // ================= OPEN EMAIL =================
  const openEmail = async (email: any) => {
    if (emailCache.current[email.id]) {
      setSelectedEmail(emailCache.current[email.id]);
      return;
    }

    let data: any;

    try {
      // 1) Show metadata immediately. Body may be empty if body_worker hasn't
      //    fetched it yet — render the placeholder via bodyLoading.
      data = await getEmail(email.id);
    } catch (err) {
      console.error("Email detail load failed", err);
      setSelectedEmail({
        ...email,
        body: "",
        _bodyError: "Failed to load email body. Try again.",
      });
      return;
    }

    if (data.body) {
      try {
        const decryptedBody = await decryptWayveBodyIfNeeded(data.body, user?.id);
        const decryptedData = { ...data, body: decryptedBody };
        emailCache.current[email.id] = decryptedData;
        setSelectedEmail(decryptedData);
        return;
      } catch (err) {
        console.error("Wayve email decrypt failed", err);
        setSelectedEmail({
          ...data,
          body: "",
          _bodyError: emailBodyErrorMessage(err),
        });
        return;
      }
    }

    setSelectedEmail({ ...data, _bodyLoading: true });

    // 2) If body wasn't ready, hit the on-demand endpoint. Backend triggers a
    //    Gmail fetch, encrypts, persists, and returns the body.
    if (!data.body) {
      try {
        const { body } = await getEmailBody(email.id);
        const decryptedBody = await decryptWayveBodyIfNeeded(body || "", user?.id);
        const merged = { ...data, body: decryptedBody, _bodyLoading: false };
        emailCache.current[email.id] = merged;
        // Only update if user hasn't navigated away to a different email.
        setSelectedEmail((cur: any) => (cur && cur.id === email.id ? merged : cur));
        return;
    
      } catch (err) {
        console.error("Wayve email body load/decrypt failed", err);
        setSelectedEmail((cur: any) =>
          cur && cur.id === email.id
            ? {
                ...cur,
                _bodyLoading: false,
                _bodyError: emailBodyErrorMessage(err),
              }
            : cur
        );
      }
      return;
    }

    emailCache.current[email.id] = data;
  };

  // ================= UI =================
  return (
    <div ref={mainRef} className={`main ${isNarrow ? "narrow" : ""}`}>

      {/* ================= SIDEBAR ================= */}
      <div className="sidebar">
        <button
          className="compose-btn"
          onClick={() => setComposeOpen(true)}
          disabled={accounts.length === 0}
          title={accounts.length === 0 ? "Add an account first" : "Compose"}
        >
          Compose
        </button>

        <div className="mail-section-title">Accounts</div>

        <button
          className={`filter-btn ${activeAccount === null ? "active" : ""}`}
          onClick={() => setActiveAccount(null)}
        >
          🌐 All Accounts
        </button>


        {accounts.map((acc) => (
          <button
            key={acc.id}
            className={`filter-btn ${activeAccount === acc.id ? "active" : ""}`}
            onClick={() => setActiveAccount(acc.id)}
          >
            {acc.email}
          </button>
        ))}

        <button className="add-email-btn" onClick={addAccount}>
          ➕ Add Account
        </button>

        <div className="mail-section-title">Folders</div>

        <div className="mail-filters">
          <button
            className={`filter-btn ${activeFolder === "inbox" ? "active" : ""}`}
            onClick={() => setActiveFolder("inbox")}
          >
            📥 Inbox
          </button>

          <button
            className={`filter-btn ${activeFolder === "sent" ? "active" : ""}`}
            onClick={() => setActiveFolder("sent")}
          >
            📤 Sent
          </button>
        </div>
      </div>

      {/* ================= EMAIL LIST ================= */}
      {showList && (
        <div className="email-list">
          {emails.map((email: any) => (
            <div
              key={email.id}
              className={`email-item ${
                selectedEmail?.id === email.id ? "active" : ""
              }`}
              onClick={() => openEmail(email)}
            >
              <div className="email-top">
                <span className="email-sender">{email.sender}</span>
                <span className="email-time">
                  {new Date(email.created_at).toLocaleTimeString()}
                </span>
              </div>

              <div className="email-subject">{email.subject}</div>

              <div className="email-preview">
                {email.preview || ""}
              </div>
            </div>
          ))}

          {hasMore && (
            <div className="load-more-wrap">
              <button
                className="load-more-btn"
                onClick={loadMore}
                disabled={loadingMore}
              >
                {loadingMore ? "Loading..." : "Show more emails"}
              </button>
            </div>
          )}
        </div>
      )}

      {/* ================= COMPOSE MODAL ================= */}
      {composeOpen && accounts.length > 0 && (
        <div
          onClick={() => setComposeOpen(false)}
          style={{
            position: "fixed",
            inset: 0,
            background: "rgba(0,0,0,0.4)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 1000,
          }}
        >
          <div
            onClick={(e) => e.stopPropagation()}
            style={{
              background: "#fff",
              padding: 20,
              borderRadius: 8,
              width: 480,
              maxWidth: "90vw",
              boxShadow: "0 10px 30px rgba(0,0,0,0.2)",
            }}
          >
            <SendEmail
              accountId={activeAccount ?? accounts[0].id}
              onClose={() => setComposeOpen(false)}
            />
          </div>
        </div>
      )}

      {/* ================= EMAIL DETAIL ================= */}
      {showDetail && (
        <div className="email-detail">
          {isNarrow && selectedEmail && (
            <button
              className="email-detail-back"
              onClick={() => setSelectedEmail(null)}
              title="Close email"
              aria-label="Close email"
            >
              ✕ Close email
            </button>
          )}

          {!selectedEmail ? (
            <p>Select an email</p>
          ) : (
            <>
              <h2>{selectedEmail.subject}</h2>

              <p><b>From:</b> {selectedEmail.sender}</p>
              <p><b>To:</b> {selectedEmail.receiver}</p>

              <div className="email-body">
                {selectedEmail._bodyLoading ? (
                  <div className="email-body-loading">
                    <span className="spinner" aria-hidden="true" />
                    <span>Loading email …</span>
                  </div>
                ) : selectedEmail._bodyError ? (
                  <p className="email-body-error">
                    {typeof selectedEmail._bodyError === "string"
                      ? selectedEmail._bodyError
                      : "Failed to load email body. Try again."}
                  </p>
                ) : (
                  selectedEmail.body
                )}
              </div>
            </>
          )}
        </div>
      )}

    </div>
  );
}
