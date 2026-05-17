import { useState, useEffect, useRef, useCallback } from "react";
import { 
  getAccounts, 
  getEmails, 
  getEmail, 
  getEmailAttachments, 
  getEmailBody, 
  getAllEmailAttachments,
  deleteEmail as deleteEmailRequest,
} from "../api/email";
import { decryptWayveBodyIfNeeded, emailBodyErrorMessage } from "./bodyUtils";
import { EmailAccount, EmailItem, EmailAttachment } from "./types";

export function useEmailInbox(user_id: number | undefined, normalizedSearchQuery: string) {
  const [accounts, setAccounts] = useState<EmailAccount[]>([]);
  const [emails, setEmails] = useState<EmailItem[]>([]);
  const [selectedEmail, setSelectedEmail] = useState<EmailItem | null>(null);
  const [activeAccount, setActiveAccount] = useState<number | null>(null);
  const [activeFolder, setActiveFolder] = useState<"inbox" | "sent">("inbox");
  const [hasMore, setHasMore] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [refreshTick, setRefreshTick] = useState(0);
  const [viewMode, setViewMode] = useState<"email" | "files">("email");
  const [files, setFiles] = useState<EmailAttachment[]>([]);
  const [filesLoading, setFilesLoading] = useState(false);
  const [filesError, setFilesError] = useState<string | null>(null);

  const emailCache = useRef<Record<number, EmailItem>>({});

  const fetchAccounts = useCallback(async () => {
    try {
      const data = await getAccounts<EmailAccount>();
      setAccounts(Array.isArray(data) ? data : []);
    } catch (err) {
      console.error("Fetch accounts failed", err);
    }
  }, []);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void fetchAccounts();
    }, 0);

    return () => window.clearTimeout(timer);
  }, [fetchAccounts]);

  useEffect(() => {
    emailCache.current = {};
  }, [activeAccount, user_id]);

  useEffect(() => {
    const fetchInitialEmails = async () => {
      const { emails: data, hasMore: hasMorePage } = await getEmails<EmailItem>({
        folder: activeFolder,
        accountId: activeAccount,
        query: normalizedSearchQuery,
      });
      setEmails(data);
      setHasMore(hasMorePage || data.length === 50);
      setSelectedEmail(null);
    };
    void fetchInitialEmails();
  }, [activeAccount, activeFolder, refreshTick, normalizedSearchQuery]);

  const loadMore = async () => {
    if (!hasMore || emails.length === 0 || loadingMore) return;
    setLoadingMore(true);
    try {
      const last = emails[emails.length - 1];
      const before = Math.floor(new Date(last.created_at).getTime() / 1000);
      const { emails: data, hasMore: hasMorePage } = await getEmails<EmailItem>({
        folder: activeFolder,
        accountId: activeAccount,
        query: normalizedSearchQuery,
        before,
        beforeId: last.id,
      });
      setEmails((prev) => [...prev, ...data]);
      setHasMore(hasMorePage);
    } finally {
      setLoadingMore(false);
    }
  };

  const openFiles = async () => {
    if (viewMode === "files") {
      setViewMode("email");
      return;
    }
    setViewMode("files");
    setFilesLoading(true);
    setFilesError(null);
    try {
      const data = await getAllEmailAttachments();
      setFiles(data);
    } catch (err) {
      setFilesError(err instanceof Error ? err.message : "Failed to load files");
    } finally {
      setFilesLoading(false);
    }
  };

  const openEmail = async (email: EmailItem) => {
    setViewMode("email");
    const openedEmail = { ...email, is_read: true };
    setEmails((prev) =>
      prev.map((item) => (item.id === email.id ? { ...item, is_read: true } : item))
    );
    if (emailCache.current[email.id]) {
      const cached = { ...emailCache.current[email.id], is_read: true };
      emailCache.current[email.id] = cached;
      setSelectedEmail(cached);
      return;
    }

    try {
      const data = await getEmail<EmailItem>(email.id);
      const emailWithListFields = { ...openedEmail, ...data, is_read: true };
      if (data.body) {
        const decryptedBody = await decryptWayveBodyIfNeeded(emailWithListFields.body || "", user_id);
        let attachments = await getEmailAttachments(email.id);
        if (!data.attachments_checked) {
          await getEmailBody(email.id);
          attachments = await getEmailAttachments(email.id);
        }
        const full = { ...emailWithListFields, body: decryptedBody, attachments };
        emailCache.current[email.id] = full;
        setSelectedEmail(full);
      } else {
        setSelectedEmail({ ...emailWithListFields, _bodyLoading: true });
        const { body } = await getEmailBody(email.id);
        const decryptedBody = await decryptWayveBodyIfNeeded(body || "", user_id);
        const attachments = await getEmailAttachments(email.id);
        const merged = { ...emailWithListFields, body: decryptedBody, attachments, _bodyLoading: false };
        emailCache.current[email.id] = merged;
        setSelectedEmail((cur) => (cur?.id === email.id ? merged : cur));
      }
    } catch (err) {
      setSelectedEmail({
        ...openedEmail,
        body: "",
        _bodyError: emailBodyErrorMessage(err),
      });
    }
  };

  const deleteEmail = async (emailId: number) => {
    await deleteEmailRequest(emailId);
    delete emailCache.current[emailId];
    setEmails((prev) => prev.filter((email) => email.id !== emailId));
    setSelectedEmail((cur) => (cur?.id === emailId ? null : cur));
  };

  return {
    accounts,
    emails,
    selectedEmail,
    setSelectedEmail,
    activeAccount,
    setActiveAccount,
    activeFolder,
    setActiveFolder,
    hasMore,
    loadingMore,
    viewMode,
    setViewMode,
    files,
    filesLoading,
    filesError,
    fetchAccounts,
    setRefreshTick,
    loadMore,
    openFiles,
    openEmail,
    deleteEmail,
  };
}
