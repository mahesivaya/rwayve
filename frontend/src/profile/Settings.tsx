import { useEffect, useState } from "react";
import "./profile.css";

import {API_BASE} from "../config/env";

const authHeaders = () => {
  const token = localStorage.getItem("token");
  return {
    "Content-Type": "application/json",
    Authorization: `Bearer ${token}`,
  };
};

type Account = { id: number; email: string };

export default function Settings() {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [loaded, setLoaded] = useState(false);

  const fetchAccounts = async () => {
    const res = await fetch(`${API_BASE}/api/accounts`, { headers: authHeaders() });
    if (!res.ok) {
      setLoaded(true);
      return;
    }
    const data: Account[] = await res.json();
    setAccounts(data);
    setLoaded(true);
  };

  useEffect(() => {
    fetchAccounts();
  }, []);

  const remove = async (id: number, email: string) => {
    if (!confirm(`Disconnect ${email}? Synced messages will be removed.`)) return;

    const res = await fetch(`${API_BASE}/api/accounts/${id}`, {
      method: "DELETE",
      headers: authHeaders(),
    });

    if (res.ok) {
      setAccounts((prev) => prev.filter((a) => a.id !== id));
    } else {
      alert("Failed to remove account");
    }
  };

  return (
    <div className="settings-page">
      <div className="settings-card">
        <h2 className="settings-title">Settings & Privacy</h2>

        <div className="settings-section-title">Connected email accounts</div>

        {!loaded ? (
          <div className="settings-empty">Loading…</div>
        ) : accounts.length === 0 ? (
          <div className="settings-empty">No email accounts connected.</div>
        ) : (
          <div className="settings-list">
            {accounts.map((acc) => (
              <div key={acc.id} className="settings-account">
                <span className="settings-account-icon">📧</span>
                <span className="settings-account-email" title={acc.email}>
                  {acc.email}
                </span>
                <button
                  className="settings-account-delete"
                  onClick={() => remove(acc.id, acc.email)}
                >
                  Remove
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
