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
import { getGmailConnectUrl, getOutlookConnectUrl, updateAccountDisplayName } from "../api/email";
import { useAuth } from "../auth/useAuth";
import { useGlobalSearch } from "../search/SearchContext";

const ACCOUNT_NAME_STORAGE_KEY = "rwayve.emailAccountNames";

export default function Emails() {
  const { user } = useAuth();
  const { normalizedSearchQuery, emailViewLayout } = useGlobalSearch();
  
  const {
    accounts, emails, selectedEmail, setSelectedEmail, activeAccount, 
    setActiveAccount, activeFolder, setActiveFolder, hasMore, loadingMore,
    viewMode, setViewMode, files, filesLoading, filesError, 
    fetchAccounts, setRefreshTick, loadMore, openFiles, openEmail
  } = useEmailInbox(user?.id, normalizedSearchQuery);

  const [composeOpen, setComposeOpen] = useState(false);
  const [accountNameOverrides, setAccountNameOverrides] = useState<Record<number, string>>(() => {
    try {
      const stored = localStorage.getItem(ACCOUNT_NAME_STORAGE_KEY);
      return stored ? JSON.parse(stored) as Record<number, string> : {};
    } catch {
      return {};
    }
  });

  // ================= NARROW MODE (split-pane / small viewport) =================
  // When the container is narrow (e.g. rendered inside the split view), we
  // collapse the 3-pane layout to a stacked one: show the list OR the detail,
  // not both. The threshold is the container width — independent of viewport
  // size, so this also responds correctly to a resized split.
  const mainRef = useRef<HTMLDivElement>(null);
  const sidebarDraggingRef = useRef(false);
  const [isNarrow, setIsNarrow] = useState(false);
  const [sidebarWidth, setSidebarWidth] = useState<number>(() => {
    const stored = localStorage.getItem("rwayve.emailSidebar.width");
    const parsed = stored ? Number(stored) : NaN;
    return Number.isFinite(parsed) ? Math.min(360, Math.max(180, parsed)) : 220;
  });

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

  useEffect(() => {
    localStorage.setItem("rwayve.emailSidebar.width", String(sidebarWidth));
  }, [sidebarWidth]);

  useEffect(() => {
    function onMove(e: MouseEvent) {
      if (!sidebarDraggingRef.current || !mainRef.current) return;
      const rect = mainRef.current.getBoundingClientRect();
      const nextWidth = e.clientX - rect.left;
      setSidebarWidth(Math.min(360, Math.max(180, nextWidth)));
    }

    function onUp() {
      if (!sidebarDraggingRef.current) return;
      sidebarDraggingRef.current = false;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    }

    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
    return () => {
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
  }, []);

  function startSidebarResize() {
    sidebarDraggingRef.current = true;
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
  }

  const useSingleColumn = isNarrow;
  const showList =
    viewMode === "email" &&
    (
      (emailViewLayout === "list" && selectedEmail === null) ||
      (emailViewLayout === "split" && (!useSingleColumn || selectedEmail === null))
    );
  const showDetail =
    viewMode === "files" ||
    (emailViewLayout === "list" && selectedEmail !== null) ||
    (emailViewLayout === "split" && (!useSingleColumn || selectedEmail !== null));

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

  const renameAccount = async (accountId: number, displayName: string | null) => {
    const nextOverrides = { ...accountNameOverrides };

    if (displayName) {
      nextOverrides[accountId] = displayName;
    } else {
      delete nextOverrides[accountId];
    }

    setAccountNameOverrides(nextOverrides);
    localStorage.setItem(ACCOUNT_NAME_STORAGE_KEY, JSON.stringify(nextOverrides));

    try {
      await updateAccountDisplayName(accountId, displayName);
      await fetchAccounts();
    } catch (err) {
      console.warn("Account name saved locally; backend update failed", err);
    }
  };

  const displayedAccounts = accounts.map((account) => ({
    ...account,
    display_name: accountNameOverrides[account.id] ?? account.display_name,
  }));

  // ================= UI =================
  return (
    <div
      ref={mainRef}
      className={[
        "main",
        isNarrow ? "narrow" : "",
        emailViewLayout === "list" ? "email-list-view" : "email-split-view",
      ].filter(Boolean).join(" ")}
    >
      <EmailSidebar
        accounts={displayedAccounts}
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
        width={sidebarWidth}
        onRenameAccount={renameAccount}
      />

      <div
        className="email-sidebar-resizer"
        onMouseDown={startSidebarResize}
        role="separator"
        aria-orientation="vertical"
        aria-label="Resize email sidebar"
        title="Drag to resize sidebar"
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
          isNarrow={useSingleColumn || emailViewLayout === "list"}
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
