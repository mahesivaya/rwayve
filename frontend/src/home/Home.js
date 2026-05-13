import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { useAuth } from "../auth/AuthContext";
import { useNavigate } from "react-router-dom";
import { useGlobalSearch } from "../search/SearchContext";
import "./home.css";
export default function Home() {
    const { user, logout } = useAuth();
    const { normalizedSearchQuery } = useGlobalSearch();
    const navigate = useNavigate();
    const cards = [
        { path: "/emails", title: "📧 Emails", description: "View and send emails" },
        { path: "/chat", title: "💬 Chat", description: "Real-time messaging" },
        { path: "/call", title: " 📞  🎥 Call", description: "Real-time calling" },
        { path: "/scheduler", title: "📅 Scheduler", description: "Manage your meetings" },
        { path: "/drive", title: "📁 Drive", description: "Store and manage files" },
        { path: "/notes", title: "📝 Notes", description: "Store and manage notes" },
        { path: "/aichat", title: "✨ AI Chat", description: "Chat with AI" },
    ];
    const visibleCards = normalizedSearchQuery
        ? cards.filter((card) => [card.title, card.description]
            .join(" ")
            .toLowerCase()
            .includes(normalizedSearchQuery))
        : cards;
    if (!user) {
        return (_jsxs("div", { className: "home", children: [_jsx("h1", { children: "Welcome to Wayve \uD83D\uDE80" }), _jsx("p", { children: "Your all-in-one platform for Email, Chat, and Scheduling." }), _jsxs("div", { className: "auth-buttons", children: [_jsx("button", { onClick: () => navigate("/login"), children: "Login" }), _jsx("button", { onClick: () => navigate("/register"), children: "Register" }), _jsx("button", { onClick: () => navigate("/business"), children: "Business" })] })] }));
    }
    return (_jsxs("div", { className: "dashboard", children: [_jsx("div", { className: "dashboard-header", children: _jsxs("h2", { children: ["Welcome, ", user.email, " \uD83D\uDC4B"] }) }), _jsxs("div", { className: "dashboard-grid", children: [visibleCards.map((card) => (_jsxs("div", { className: "card", onClick: () => navigate(card.path), children: [_jsx("h3", { children: card.title }), _jsx("p", { children: card.description })] }, card.path))), _jsxs("div", { className: "card", onClick: () => navigate("/business"), children: [_jsx("h3", { children: "Business" }), _jsx("p", { children: "Welcome to Wayve Business" })] })] })] }));
}
