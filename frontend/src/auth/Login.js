import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { logger } from "../utils/logger";
import { useEffect, useState } from "react";
import { login } from "../api/Auth";
import { useNavigate, useSearchParams } from "react-router-dom";
import { useAuth } from "../auth/AuthContext";
import { API_BASE } from "../config";
import "./login.css";
export default function Login() {
    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");
    const [error, setError] = useState("");
    const [params] = useSearchParams();
    const navigate = useNavigate();
    const { login: authLogin } = useAuth();
    // Surface OAuth-side errors that the backend redirected here with.
    useEffect(() => {
        if (params.get("error") === "email_exists") {
            setError("This email is already registered with a password. Please login.");
        }
    }, [params]);
    const handleLogin = async (e) => {
        e.preventDefault();
        setError("");
        try {
            const data = await login(email, password);
            if (!data || !data.token) {
                throw new Error("No token returned");
            }
            localStorage.setItem("token", data.token);
            authLogin(data.token);
            navigate("/emails");
        }
        catch (err) {
            logger.error(err);
            setError("Login failed. Check your credentials.");
        }
    };
    const handleGoogle = () => {
        window.location.href = `${API_BASE}/gmail/login?mode=signup`;
    };
    return (_jsx("div", { className: "login-page", children: _jsxs("form", { className: "login-card", onSubmit: handleLogin, children: [_jsx("h2", { children: "Welcome back \uD83D\uDC4B" }), _jsx("p", { className: "subtitle", children: "Login to your Wayve account" }), _jsx("input", { type: "email", value: email, onChange: (e) => setEmail(e.target.value), placeholder: "Email", required: true }), _jsx("input", { type: "password", value: password, onChange: (e) => setPassword(e.target.value), placeholder: "Password", required: true }), _jsx("button", { type: "submit", children: "Login" }), _jsx("div", { className: "auth-divider", children: _jsx("span", { children: "or" }) }), _jsx("button", { type: "button", className: "google-btn", onClick: handleGoogle, children: "Continue with Gmail" }), error && _jsx("p", { className: "error", children: error }), _jsx("p", { className: "switch-auth", children: _jsx("span", { onClick: () => navigate("/forgot-password"), children: "Forgot password?" }) }), _jsxs("p", { className: "switch-auth", children: ["Don\u2019t have an account?", " ", _jsx("span", { onClick: () => navigate("/register"), children: "Register" })] })] }) }));
}
