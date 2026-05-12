import { useEffect, useRef, useState } from "react";
import "./emails.css";
import "./loadMore.css";
import SendEmail from "./SendEmail";

import {API_BASE} from "../config/env";
import { apiFetch } from "@/api/client";

type Email = {
  id: number;
  sender: string;
  receiver: string;
  subject: string;
  preview?: string;
  body?: string;
  created_at: string;
};

export default function Emails() {
  const [accounts, setAccounts] = useState<any[]>([]);
  const [emails, setEmails] = useState<any[]>([]);
  const [selectedEmail, setSelectedEmail] = useState<any | null>(null);

  const [activeAccount, setActiveAccount] = useState<number | null>(null);
  const [activeFolder, setActiveFolder] = useState<"inbox" | "sent">("inbox");

  const [hasMore, setHasMore] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);

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
      const res = await apiFetch("api/accounts");
      const data = await res.json();
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
      window.history.replaceState({}, "", "/emails");
    }
  }, []);

  // ================= ADD ACCOUNT =================
  const addAccount = () => {
    const token = localStorage.getItem("token");
    if (!token) return;
    window.location.href = `${API_BASE}/gmail/login?token=${encodeURIComponent(token)}`;
  };

  // ================= FETCH EMAILS =================
  useEffect(() => {
    const fetchEmails = async () => {

      let url = `api/emails?folder=${activeFolder}`;

      if (activeAccount !== null) {
        url += `&account_id=${activeAccount}`;
      }

      const res = await apiFetch(url);

      const data = await res.json();

      setEmails(data);
      setHasMore(data.length === 50);
      setSelectedEmail(null);
    };

    void fetchEmails();
  }, [activeAccount, activeFolder]);

  // ================= LOAD MORE =================
  const loadMore = async () => {
    if (!hasMore || emails.length === 0) return;

    setLoadingMore(true);

    try{
    const last = emails[emails.length - 1];

    const before = Math.floor(new Date(last.created_at).getTime() / 1000);
    const before_id = last.id;

    let url = `/api/emails?folder=${activeFolder}&before=${before}&before_id=${before_id}`;

    if (activeAccount !== null) {
      url += `&account_id=${activeAccount}`;
    }

    const res = await apiFetch(url);

    const data = await res.json();

    setEmails((prev) => [...prev, ...data]);
    setHasMore(data.length === 50);
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

    // 1) Show metadata immediately. Body may be empty if body_worker hasn't
    //    fetched it yet — render the placeholder via bodyLoading.
    const res = await apiFetch(`/api/emails/${email.id}`);
    const data = await res.json();
    setSelectedEmail({ ...data, _bodyLoading: !data.body });

    // 2) If body wasn't ready, hit the on-demand endpoint. Backend triggers a
    //    Gmail fetch, encrypts, persists, and returns the body.
    if (!data.body) {
      try {
        const bodyRes = await apiFetch(`/api/emails/${email.id}/body`);
        
        const { body } = await bodyRes.json();
        const merged = { ...data, body, _bodyLoading: false };
        emailCache.current[email.id] = merged;
        // Only update if user hasn't navigated away to a different email.
        setSelectedEmail((cur: any) => (cur && cur.id === email.id ? merged : cur));
        return;
    
      } catch {
        setSelectedEmail((cur: any) =>
          cur && cur.id === email.id ? { ...cur, _bodyLoading: false, _bodyError: true } : cur
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
                {loadingMore ? "Loading..." : "Load More"}
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
                  <p className="email-body-error">Failed to load email body. Try again.</p>
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
