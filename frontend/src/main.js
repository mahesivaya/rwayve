import { jsx as _jsx } from "react/jsx-runtime";
import ReactDOM from "react-dom/client";
import App from "./App";
import { BrowserRouter } from "react-router-dom";
import { AuthProvider } from "./auth/AuthContext";
ReactDOM.createRoot(document.getElementById("root")).render(_jsx(BrowserRouter, { children: _jsx(AuthProvider, { children: _jsx(App, {}) }) }));
