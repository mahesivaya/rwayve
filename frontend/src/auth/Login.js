import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { useState } from "react";
import { login } from "../api/Auth";
import { useNavigate } from "react-router-dom";
import { useAuth } from "../auth/AuthContext";
import "./login.css";
export default function Login() {
    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");
    const navigate = useNavigate();
    const { login: authLogin } = useAuth();
    const handleLogin = async (e) => {
        e.preventDefault();
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
            console.error(err);
            alert("Login failed");
        }
    };
    return (_jsx("div", { className: "login-page", children: _jsxs("form", { className: "login-card", onSubmit: handleLogin, children: [_jsx("h2", { children: "Welcome back \uD83D\uDC4B" }), _jsx("p", { className: "subtitle", children: "Login to your Wayve account" }), _jsx("input", { type: "email", value: email, onChange: (e) => setEmail(e.target.value), placeholder: "Email", required: true }), _jsx("input", { type: "password", value: password, onChange: (e) => setPassword(e.target.value), placeholder: "Password", required: true }), _jsx("button", { type: "submit", children: "Login" }), _jsxs("p", { className: "switch-auth", children: ["Don\u2019t have an account?", " ", _jsx("span", { onClick: () => navigate("/register"), children: "Register" })] })] }) }));
}
