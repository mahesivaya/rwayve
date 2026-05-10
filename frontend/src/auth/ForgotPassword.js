import { jsx as _jsx, Fragment as _Fragment, jsxs as _jsxs } from "react/jsx-runtime";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { forgotPassword } from "../api/Auth";
import "./login.css";
export default function ForgotPassword() {
    const [email, setEmail] = useState("");
    const [status, setStatus] = useState("idle");
    const [error, setError] = useState("");
    const navigate = useNavigate();
    const submit = async (e) => {
        e.preventDefault();
        setError("");
        setStatus("sending");
        try {
            await forgotPassword(email);
            setStatus("sent");
        }
        catch (err) {
            setStatus("idle");
            setError(err instanceof Error ? err.message : "Request failed");
        }
    };
    return (_jsx("div", { className: "login-page", children: _jsxs("form", { className: "login-card", onSubmit: submit, children: [_jsx("h2", { children: "Forgot password?" }), _jsx("p", { className: "subtitle", children: "Enter your email and we'll send you a reset link." }), status === "sent" ? (_jsx("p", { className: "subtitle", children: "If that account exists, a reset link has been sent. Check your inbox \u2014 the link expires in 30 minutes." })) : (_jsxs(_Fragment, { children: [_jsx("input", { type: "email", placeholder: "Email", value: email, onChange: (e) => setEmail(e.target.value), required: true }), _jsx("button", { type: "submit", disabled: status === "sending", children: status === "sending" ? "Sending…" : "Send reset link" }), error && _jsx("p", { className: "error", children: error })] })), _jsx("p", { className: "switch-auth", children: _jsx("span", { onClick: () => navigate("/login"), children: "Back to login" }) })] }) }));
}
