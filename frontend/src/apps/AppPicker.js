import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { APP_LIST } from "./registry";
import "./apps.css";
export default function AppPicker({ onPick, title = "Choose an app" }) {
    return (_jsxs("div", { className: "app-picker", children: [_jsx("h3", { className: "app-picker-title", children: title }), _jsx("div", { className: "app-picker-grid", children: APP_LIST.map((app) => (_jsxs("button", { className: "app-picker-card", onClick: () => onPick(app.key), children: [_jsx("div", { className: "app-picker-icon", children: app.icon }), _jsx("div", { className: "app-picker-label", children: app.label })] }, app.key))) })] }));
}
