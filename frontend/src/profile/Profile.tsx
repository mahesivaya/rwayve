import { useEffect, useState } from "react";
import "./profile.css";

const API_BASE = import.meta.env.VITE_API_URL;

const authHeaders = () => {
  const token = localStorage.getItem("token");
  return {
    "Content-Type": "application/json",
    Authorization: `Bearer ${token}`,
  };
};

type ProfileData = {
  id: number;
  email: string;
  first_name: string | null;
  last_name: string | null;
};

export default function Profile() {
  const [profile, setProfile] = useState<ProfileData | null>(null);
  const [firstName, setFirstName] = useState("");
  const [lastName, setLastName] = useState("");
  const [saving, setSaving] = useState(false);
  const [status, setStatus] = useState<string | null>(null);

  useEffect(() => {
    const load = async () => {
      const res = await fetch(`${API_BASE}/api/profile`, { headers: authHeaders() });
      if (!res.ok) return;
      const data: ProfileData = await res.json();
      setProfile(data);
      setFirstName(data.first_name ?? "");
      setLastName(data.last_name ?? "");
    };
    load();
  }, []);

  useEffect(() => {
    if (!status) return;
    const t = setTimeout(() => setStatus(null), 2000);
    return () => clearTimeout(t);
  }, [status]);

  const save = async () => {
    setSaving(true);
    try {
      const res = await fetch(`${API_BASE}/api/profile`, {
        method: "PUT",
        headers: authHeaders(),
        body: JSON.stringify({ first_name: firstName, last_name: lastName }),
      });

      if (!res.ok) throw new Error(await res.text());
      const data: ProfileData = await res.json();
      setProfile(data);
      setStatus("Saved ✓");
    } catch {
      setStatus("Save failed");
    } finally {
      setSaving(false);
    }
  };

  if (!profile) {
    return (
      <div className="profile-page">
        <div className="profile-loading">Loading…</div>
      </div>
    );
  }

  return (
    <div className="profile-page">
      <div className="profile-card">
        <h2 className="profile-title">My Profile</h2>

        <div className="profile-row">
          <label>Username</label>
          <div className="profile-readonly">{profile.email}</div>
        </div>

        <div className="profile-row">
          <label htmlFor="profile-first">First name</label>
          <input
            id="profile-first"
            value={firstName}
            onChange={(e) => setFirstName(e.target.value)}
            placeholder="First name"
          />
        </div>

        <div className="profile-row">
          <label htmlFor="profile-last">Last name</label>
          <input
            id="profile-last"
            value={lastName}
            onChange={(e) => setLastName(e.target.value)}
            placeholder="Last name"
          />
        </div>

        <div className="profile-actions">
          <button
            className="profile-save"
            onClick={save}
            disabled={saving}
          >
            {saving ? "Saving…" : "Save"}
          </button>
          {status && <span className="profile-status">{status}</span>}
        </div>
      </div>
    </div>
  );
}
