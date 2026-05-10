import { jsx as _jsx, Fragment as _Fragment, jsxs as _jsxs } from "react/jsx-runtime";
import { useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { resetPassword } from "../api/Auth";
import "./login.css";
export default function ResetPassword() {
    const [params] = useSearchParams();
    const token = params.get("token") ?? "";
    const [password, setPassword] = useState("");
    const [confirm, setConfirm] = useState("");
    const [status, setStatus] = useState("idle");
    const [error, setError] = useState("");
    const navigate = useNavigate();
    const submit = async (e) => {
        e.preventDefault();
        setError("");
        if (!token) {
            setError("Missing reset token.");
            return;
        }
        if (password !== confirm) {
            setError("Passwords do not match.");
            return;
        }
        if (password.length < 6) {
            setError("Password must be at least 6 characters.");
            return;
        }
        setStatus("saving");
        try {
            await resetPassword(token, password);
            setStatus("done");
            setTimeout(() => navigate("/login"), 1500);
        }
        catch (err) {
            setStatus("idle");
            setError(err instanceof Error ? err.message : "Reset failed");
        }
    };
    return (_jsx("div", { className: "login-page", children: _jsxs("form", { className: "login-card", onSubmit: submit, children: [_jsx("h2", { children: "Set a new password" }), status === "done" ? (_jsx("p", { className: "subtitle", children: "Password updated. Redirecting to login\u2026" })) : (_jsxs(_Fragment, { children: [_jsx("input", { type: "password", placeholder: "New password", value: password, onChange: (e) => setPassword(e.target.value), required: true }), _jsx("input", { type: "password", placeholder: "Confirm new password", value: confirm, onChange: (e) => setConfirm(e.target.value), required: true }), _jsx("button", { type: "submit", disabled: status === "saving", children: status === "saving" ? "Saving…" : "Update password" }), error && _jsx("p", { className: "error", children: error })] })), _jsx("p", { className: "switch-auth", children: _jsx("span", { onClick: () => navigate("/login"), children: "Back to login" }) })] }) }));
}
