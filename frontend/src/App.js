import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { Routes, Route, Navigate } from "react-router-dom";
import Layout from "./components/Layout";
import ProtectedRoute from "./components/ProtectedRoute";
import Register from "./auth/Register";
import Login from "./auth/Login";
import Home from "./home/Home";
import Emails from "./emails/Emails";
import Chat from "./chat/Chat";
// import Call from "./call/Call";
import Scheduler from "./scheduler/Scheduler";
import Drive from "./drive/DriveBox";
import { useAuth } from "./auth/AuthContext";
export default function App() {
    const { user } = useAuth();
    return (_jsxs(Routes, { children: [_jsx(Route, { path: "/", element: _jsx(Navigate, { to: user ? "/home" : "/login" }) }), _jsx(Route, { path: "/login", element: user ? _jsx(Navigate, { to: "/home" }) : _jsx(Login, {}) }), _jsx(Route, { path: "/register", element: user ? _jsx(Navigate, { to: "/home" }) : _jsx(Register, {}) }), _jsx(Route, { element: _jsx(ProtectedRoute, {}), children: _jsxs(Route, { element: _jsx(Layout, {}), children: [_jsx(Route, { path: "/home", element: _jsx(Home, {}) }), _jsx(Route, { path: "/emails", element: _jsx(Emails, {}) }), _jsx(Route, { path: "/chat", element: _jsx(Chat, {}) }), _jsx(Route, { path: "/chat", element: _jsx(Chat, {}) }), _jsx(Route, { path: "/scheduler", element: _jsx(Scheduler, {}) }), _jsx(Route, { path: "/drive", element: _jsx(Drive, {}) })] }) }), _jsx(Route, { path: "*", element: _jsx(Navigate, { to: user ? "/home" : "/login" }) })] }));
}
