import { useCallback, useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import "./emails.css";
import "./loadMore.css";
import SendEmail from "./SendEmail";
import {
  decryptWayveBodyIfNeeded,
  emailBodyErrorMessage,
} from "./bodyUtils";
import Modal from "../components/Modal";
import { formatFileSize, renderEmailBody } from "./renderUtils";

import {
  downloadEmailAttachment,
  type EmailAttachment,
  getAccounts,
  getEmail,
  getEmailAttachments,
  getEmailBody,
  getEmails,
  getGmailConnectUrl,
} from "../api/email";
import { useAuth } from "../auth/useAuth";
import { useGlobalSearch } from "../search/SearchContext";

type EmailAccount = {
  id: number;
  email: string;
};

type EmailItem = {
  id: number;
  subject?: string | null;
  sender?: string | null;
  receiver?: string | null;
  preview?: string | null;
  body?: string | null;
  created_at: string;
  has_attachments?: boolean;
  attachments_checked?: boolean;
  attachments?: EmailAttachment[];
  zoom_join_url?: string | null;
  _bodyLoading?: boolean;
  _bodyError?: unknown;
};

export default function Emails() {
  const { user } = useAuth();
  const { normalizedSearchQuery } = useGlobalSearch();
  const navigate = useNavigate();
  const [accounts, setAccounts] = useState<EmailAccount[]>([]);
  const [emails, setEmails] = useState<EmailItem[]>([]);
  const [selectedEmail, setSelectedEmail] = useState<EmailItem | null>(null);

  const [activeAccount, setActiveAccount] = useState<number | null>(null);
  const [activeFolder, setActiveFolder] = useState<"inbox" | "sent">("inbox");

  const [hasMore, setHasMore] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [refreshTick, setRefreshTick] = useState(0);

  const [composeOpen, setComposeOpen] = useState(false);

  const emailCache = useRef<Record<number, EmailItem>>({});

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
  const composeAccountId =
    activeAccount ?? accounts.find((account) => account?.id !== undefined)?.id ?? null;

  // ================= FETCH ACCOUNTS =================
  const fetchAccounts = useCallback(async () => {
    try{
      const data = await getAccounts<EmailAccount>();
      setAccounts(
        Array.isArray(data)
          ? data.filter(
              (account): account is EmailAccount =>
                account !== null &&
                typeof account === "object" &&
                typeof account.id === "number" &&
                typeof account.email === "string"
            )
          : []
      );
    }
    catch(err){
    console.error(err)
    }
  }, []);

  useEffect(() => {
    void fetchAccounts();
  }, [fetchAccounts]);

  // ================= CACHE MANAGEMENT =================
  // Clear the decryption cache when switching accounts or user identity changes
  // (e.g. on logout) to ensure security and prevent stale data.
  useEffect(() => {
    emailCache.current = {};
  }, [activeAccount, user?.id]);

  // ================= HANDLE OAUTH RETURN =================
  // After /oauth/callback redirects back with #connected=true, refresh the
  // account list so the newly linked account shows up immediately. The 30s
  // sync worker will import its emails on the next tick.
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const hashParams = new URLSearchParams(window.location.hash.slice(1));
    if (
      params.get("connected") === "true" ||
      hashParams.get("connected") === "true"
    ) {
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
  }, [fetchAccounts]);

  // ================= ADD ACCOUNT =================
  const addAccount = async () => {
    const url = await getGmailConnectUrl();
    window.location.href = url;
  };

  // ================= FETCH EMAILS =================
  useEffect(() => {
    const fetchEmails = async () => {
      const { emails: data, hasMore: hasMorePage } = await getEmails<EmailItem>({
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

    const { emails: data, hasMore: hasMorePage } = await getEmails<EmailItem>({
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
  const openEmail = async (email: EmailItem) => {
    if (emailCache.current[email.id]) {
      setSelectedEmail(emailCache.current[email.id]);
      return;
    }

    let data: EmailItem;
    let attachments: EmailAttachment[] = [];

    try {
      // 1) Show metadata immediately. Body may be empty if body_worker hasn't
      //    fetched it yet — render the placeholder via bodyLoading.
      data = await getEmail<EmailItem>(email.id);
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
        attachments = await getEmailAttachments(email.id);
        if (!data.attachments_checked) {
          await getEmailBody(email.id);
          attachments = await getEmailAttachments(email.id);
        }
        const decryptedData = { ...data, body: decryptedBody, attachments };
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
        attachments = await getEmailAttachments(email.id);
        const merged = { ...data, body: decryptedBody, attachments, _bodyLoading: false };
        emailCache.current[email.id] = merged;
        // Only update if user hasn't navigated away to a different email.
        setSelectedEmail((cur) => (cur && cur.id === email.id ? merged : cur));
        return;
    
      } catch (err) {
        console.error("Wayve email body load/decrypt failed", err);
        setSelectedEmail((cur) =>
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

          <button className="filter-btn" onClick={() => navigate("/email-files")}>
            📎 Files
          </button>
        </div>
      </div>

      {/* ================= EMAIL LIST ================= */}
      {showList && (
        <div className="email-list">
          {emails.map((email) => (
            <div
              key={email.id}
              className={`email-item ${
                selectedEmail?.id === email.id ? "active" : ""
              }`}
              onClick={() => openEmail(email)}
            >
              <div className="email-top">
                <span className="email-sender">{email.sender}</span>
                <span className="email-row-meta">
                  {email.has_attachments && (
                    <span className="email-attachment-pin" title="Has attachments">
                      📎
                    </span>
                  )}
                  <span className="email-time">
                    {new Date(email.created_at).toLocaleTimeString()}
                  </span>
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
      <Modal
        isOpen={composeOpen && composeAccountId !== null}
        onClose={() => setComposeOpen(false)}
        title="New Message"
      >
        {composeAccountId !== null && (
          <SendEmail
            accountId={composeAccountId}
            onClose={() => setComposeOpen(false)}
          />
        )}
      </Modal>

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
                  renderEmailBody(selectedEmail.body || "")
                )}
              </div>

              {(selectedEmail.attachments?.length ?? 0) > 0 && (
                <div className="email-attachments">
                  <div className="email-attachments-title">Attachments</div>
                  {selectedEmail.attachments?.map((attachment: EmailAttachment) => (
                    <button
                      key={attachment.id}
                      className="email-attachment"
                      onClick={() => downloadEmailAttachment(attachment)}
                    >
                      <span className="email-attachment-icon">📎</span>
                      <span className="email-attachment-name">
                        {attachment.filename}
                      </span>
                      <span className="email-attachment-size">
                        {formatFileSize(attachment.size)}
                      </span>
                    </button>
                  ))}
                </div>
              )}
            </>
          )}
        </div>
      )}

    </div>
  );
}
