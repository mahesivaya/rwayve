import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { useState } from "react";
import { register } from "../api/Auth";
import { useAuth } from "../auth/AuthContext";
import { useNavigate } from "react-router-dom";
import "./login.css"; // ✅ reuse styles
export default function Register() {
    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");
    const [confirm, setConfirm] = useState("");
    const [error, setError] = useState("");
    const { login } = useAuth();
    const navigate = useNavigate();
    const handleRegister = async (e) => {
        e.preventDefault();
        setError("");
        // ✅ basic validation
        if (password !== confirm) {
            setError("Passwords do not match");
            return;
        }
        try {
            const data = await register(email, password, confirm);
            if (!data || !data.token) {
                throw new Error("No token returned from server");
            }
            localStorage.setItem("token", data.token);
            login(data.token);
            navigate("/emails");
        }
        catch (err) {
            setError(err.message || "Registration failed");
        }
    };
    return (_jsx("div", { className: "login-page", children: _jsxs("form", { className: "login-card", onSubmit: handleRegister, children: [_jsx("h2", { children: "Create account \uD83D\uDE80" }), _jsx("p", { className: "subtitle", children: "Join Wayve to get started" }), _jsx("input", { type: "email", placeholder: "Email", value: email, onChange: (e) => setEmail(e.target.value), required: true }), _jsx("input", { type: "password", placeholder: "Password", value: password, onChange: (e) => setPassword(e.target.value), required: true }), _jsx("input", { type: "password", placeholder: "Confirm Password", value: confirm, onChange: (e) => setConfirm(e.target.value), required: true }), _jsx("button", { type: "submit", children: "Register" }), error && _jsx("p", { className: "error", children: error }), _jsxs("p", { className: "switch-auth", children: ["Already have an account?", " ", _jsx("span", { onClick: () => navigate("/login"), children: "Login" })] })] }) }));
}
