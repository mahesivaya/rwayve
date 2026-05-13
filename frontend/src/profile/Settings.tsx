import { useEffect, useState } from "react";

import "./profile.css";

import { deleteAccount, getAccounts } from "../api/email";

type Account = {
  id: number;
  email: string;
};

export default function Settings() {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [loaded, setLoaded] = useState(false);
  const fetchAccounts = async () => {
    try {
      setAccounts(await getAccounts<Account>());

    } finally {
      setLoaded(true);
    }
  };

  useEffect(() => {
    void fetchAccounts();
  }, []);

  const remove = async (
    id: number,
    email: string
  ) => {
    if (
      !confirm(
        `Disconnect ${email}? Synced messages will be removed.`
      )
    ) {
      return;
    }

    try {
      await deleteAccount(id);

      setAccounts((prev) =>
        prev.filter(
          (a) => a.id !== id
        )
      );

    } catch {
      alert(
        "Failed to remove account"
      );
    }
  };

  return (
    <div className="settings-page">
      <div className="settings-card">

        <h2 className="settings-title">
          Settings & Privacy
        </h2>

        <div className="settings-section-title">
          Connected email accounts
        </div>

        {!loaded ? (
          <div className="settings-empty">
            Loading…
          </div>

        ) : accounts.length === 0 ? (
          <div className="settings-empty">
            No email accounts connected.
          </div>

        ) : (
          <div className="settings-list">
            {accounts.map((acc) => (
              <div
                key={acc.id}
                className="settings-account"
              >
                <span className="settings-account-icon">
                  📧
                </span>

                <span
                  className="settings-account-email"
                  title={acc.email}
                >
                  {acc.email}
                </span>

                <button
                  className="settings-account-delete"

                  onClick={() =>
                    void remove(
                      acc.id,
                      acc.email
                    )
                  }
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
