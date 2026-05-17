import { useEffect, useRef, useState } from "react";
import "./emails.css";
import "./loadMore.css";
import "../files/emailFiles.css";
import SendEmail from "./SendEmail";
import Modal from "../components/Modal";
import { EmailSidebar } from "./EmailSidebar";
import { EmailList } from "./EmailList";
import { EmailDetail } from "./EmailDetail";
import { useEmailInbox } from "./useEmailInbox";
import { getGmailConnectUrl, getOutlookConnectUrl } from "../api/email";
import { useAuth } from "../auth/useAuth";
import { useGlobalSearch } from "../search/SearchContext";

export default function Emails() {
  const { user } = useAuth();
  const { normalizedSearchQuery } = useGlobalSearch();
  
  const {
    accounts, emails, selectedEmail, setSelectedEmail, activeAccount, 
    setActiveAccount, activeFolder, setActiveFolder, hasMore, loadingMore,
    viewMode, setViewMode, files, filesLoading, filesError, 
    fetchAccounts, setRefreshTick, loadMore, openFiles, openEmail
  } = useEmailInbox(user?.id, normalizedSearchQuery);

  const [composeOpen, setComposeOpen] = useState(false);

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

  const showList =
    !isNarrow || (selectedEmail === null && viewMode === "email");
  const showDetail =
    !isNarrow || selectedEmail !== null || viewMode === "files";

  const composeAccountId =
    activeAccount ?? accounts.find((account) => account?.id !== undefined)?.id ?? null;

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
  }, [fetchAccounts, setRefreshTick]);

  // ================= ADD ACCOUNT =================
  const addAccount = async () => {
    const url = await getGmailConnectUrl();
    window.location.href = url;
  };

  const addOutlookAccount = async () => {
    const url = await getOutlookConnectUrl();
    window.location.href = url;
  };

  // ================= UI =================
  return (
    <div ref={mainRef} className={`main ${isNarrow ? "narrow" : ""}`}>
      <EmailSidebar
        accounts={accounts}
        activeAccount={activeAccount}
        setActiveAccount={setActiveAccount}
        activeFolder={activeFolder}
        setActiveFolder={(f) => { setViewMode("email"); setActiveFolder(f); }}
        viewMode={viewMode}
        onOpenFiles={openFiles}
        onAddGmail={addAccount}
        onAddOutlook={addOutlookAccount}
        onCompose={() => setComposeOpen(true)}
        composeDisabled={accounts.length === 0}
      />

      {showList && (
        <EmailList
          emails={emails}
          selectedEmailId={selectedEmail?.id ?? null}
          onOpenEmail={openEmail}
          hasMore={hasMore}
          loadMore={loadMore}
          loadingMore={loadingMore}
        />
      )}

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

      {showDetail && (
        <EmailDetail
          selectedEmail={selectedEmail}
          viewMode={viewMode}
          isNarrow={isNarrow}
          onBack={() => { setViewMode("email"); setSelectedEmail(null); }}
          files={files}
          filesLoading={filesLoading}
          filesError={filesError}
          normalizedSearchQuery={normalizedSearchQuery}
        />
      )}
    </div>
  );
}
