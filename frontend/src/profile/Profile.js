import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";
import { useEffect, useState } from "react";
import { changePassword } from "../api/Auth";
import "./profile.css";
import { apiFetch } from "../api/client";
const authHeaders = () => {
    const token = localStorage.getItem("token");
    return {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
    };
};
export default function Profile() {
    const [profile, setProfile] = useState(null);
    const [firstName, setFirstName] = useState("");
    const [lastName, setLastName] = useState("");
    const [saving, setSaving] = useState(false);
    const [status, setStatus] = useState(null);
    const [showPwForm, setShowPwForm] = useState(false);
    const [currentPw, setCurrentPw] = useState("");
    const [newPw, setNewPw] = useState("");
    const [confirmPw, setConfirmPw] = useState("");
    const [pwSaving, setPwSaving] = useState(false);
    const [pwStatus, setPwStatus] = useState(null);
    useEffect(() => {
        const load = async () => {
            try {
                const res = await apiFetch(`/api/profile`);
                const data = await res.json();
                setProfile(data);
                setFirstName(data.first_name ?? "");
                setLastName(data.last_name ?? "");
            }
            catch (err) {
                console.error(err);
            }
        };
        void load();
    }, []);
    useEffect(() => {
        if (!status)
            return;
        const t = setTimeout(() => setStatus(null), 2000);
        return () => clearTimeout(t);
    }, [status]);
    useEffect(() => {
        if (!pwStatus)
            return;
        const t = setTimeout(() => setPwStatus(null), 2500);
        return () => clearTimeout(t);
    }, [pwStatus]);
    const submitPasswordChange = async () => {
        const isCreatingPassword = profile?.auth_provider === "google";
        if (newPw !== confirmPw) {
            setPwStatus("New passwords do not match");
            return;
        }
        if (newPw.length < 6) {
            setPwStatus("Password must be at least 6 characters");
            return;
        }
        setPwSaving(true);
        try {
            await changePassword(isCreatingPassword ? null : currentPw, newPw);
            setPwStatus(isCreatingPassword ? "Password created ✓" : "Password updated ✓");
            setCurrentPw("");
            setNewPw("");
            setConfirmPw("");
            setShowPwForm(false);
            setProfile((prev) => prev ? { ...prev, auth_provider: "local" } : prev);
        }
        catch (err) {
            setPwStatus(err instanceof Error ? err.message : "Update failed");
        }
        finally {
            setPwSaving(false);
        }
    };
    const save = async () => {
        setSaving(true);
        try {
            const res = await apiFetch(`/api/profile`, {
                method: "PUT",
                body: JSON.stringify({ first_name: firstName, last_name: lastName }),
            });
            const data = await res.json();
            setProfile(data);
            setStatus("Saved ✓");
        }
        catch {
            setStatus("Save failed");
        }
        finally {
            setSaving(false);
        }
    };
    if (!profile) {
        return (_jsx("div", { className: "profile-page", children: _jsx("div", { className: "profile-loading", children: "Loading\u2026" }) }));
    }
    return (_jsx("div", { className: "profile-page", children: _jsxs("div", { className: "profile-card", children: [_jsx("h2", { className: "profile-title", children: "My Profile" }), _jsxs("div", { className: "profile-row", children: [_jsx("label", { children: "Username" }), _jsx("div", { className: "profile-readonly", children: profile.email })] }), _jsxs("div", { className: "profile-row", children: [_jsx("label", { htmlFor: "profile-first", children: "First name" }), _jsx("input", { id: "profile-first", value: firstName, onChange: (e) => setFirstName(e.target.value), placeholder: "First name" })] }), _jsxs("div", { className: "profile-row", children: [_jsx("label", { htmlFor: "profile-last", children: "Last name" }), _jsx("input", { id: "profile-last", value: lastName, onChange: (e) => setLastName(e.target.value), placeholder: "Last name" })] }), _jsxs("div", { className: "profile-actions", children: [_jsx("button", { className: "profile-save", onClick: save, disabled: saving, children: saving ? "Saving…" : "Save" }), status && _jsx("span", { className: "profile-status", children: status })] }), _jsxs("div", { className: "profile-password-section", children: [_jsx("h3", { className: "profile-section-title", children: "Password" }), !showPwForm ? (_jsx("button", { type: "button", className: "profile-save", onClick: () => setShowPwForm(true), children: profile.auth_provider === "google"
                                ? "Create Password"
                                : "Change Password" })) : (_jsxs(_Fragment, { children: [profile.auth_provider !== "google" && (_jsxs("div", { className: "profile-row", children: [_jsx("label", { htmlFor: "profile-current-pw", children: "Current password" }), _jsx("input", { id: "profile-current-pw", type: "password", value: currentPw, onChange: (e) => setCurrentPw(e.target.value) })] })), _jsxs("div", { className: "profile-row", children: [_jsx("label", { htmlFor: "profile-new-pw", children: profile.auth_provider === "google"
                                                ? "Password"
                                                : "New password" }), _jsx("input", { id: "profile-new-pw", type: "password", value: newPw, onChange: (e) => setNewPw(e.target.value) })] }), _jsxs("div", { className: "profile-row", children: [_jsx("label", { htmlFor: "profile-confirm-pw", children: profile.auth_provider === "google"
                                                ? "Confirm password"
                                                : "Confirm new password" }), _jsx("input", { id: "profile-confirm-pw", type: "password", value: confirmPw, onChange: (e) => setConfirmPw(e.target.value) })] }), _jsxs("div", { className: "profile-actions", children: [_jsx("button", { type: "button", className: "profile-save", onClick: submitPasswordChange, disabled: pwSaving, children: pwSaving
                                                ? "Saving…"
                                                : profile.auth_provider === "google"
                                                    ? "Create password"
                                                    : "Update password" }), _jsx("button", { type: "button", className: "profile-cancel", onClick: () => {
                                                setShowPwForm(false);
                                                setCurrentPw("");
                                                setNewPw("");
                                                setConfirmPw("");
                                            }, disabled: pwSaving, children: "Cancel" })] })] })), pwStatus && _jsx("p", { className: "profile-status", children: pwStatus })] })] }) }));
}
