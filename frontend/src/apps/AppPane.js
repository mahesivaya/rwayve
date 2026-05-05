import { jsx as _jsx, Fragment as _Fragment, jsxs as _jsxs } from "react/jsx-runtime";
import { Suspense } from "react";
import { APP_REGISTRY } from "./registry";
import AppPicker from "./AppPicker";
import "./apps.css";
export default function AppPane({ appKey, side, onPick, onClose }) {
    const app = appKey ? APP_REGISTRY[appKey] : null;
    return (_jsxs("div", { className: "app-pane", "data-side": side, children: [_jsxs("div", { className: "app-pane-header", children: [_jsx("div", { className: "app-pane-title", children: app ? (_jsxs(_Fragment, { children: [_jsx("span", { className: "app-pane-icon", children: app.icon }), _jsx("span", { children: app.label })] })) : (_jsx("span", { className: "app-pane-empty", children: "Empty pane" })) }), _jsx("button", { className: "app-pane-close", onClick: onClose, title: "Close pane", "aria-label": "Close pane", children: "\u00D7" })] }), _jsx("div", { className: "app-pane-body", children: app ? (_jsx(Suspense, { fallback: _jsx("div", { className: "app-pane-loading", children: "Loading\u2026" }), children: _jsx(app.Component, {}) })) : (_jsx(AppPicker, { onPick: onPick, title: "Open app in this pane" })) })] }));
}
