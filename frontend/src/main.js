import { jsx as _jsx } from "react/jsx-runtime";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";
import { BrowserRouter } from "react-router-dom";
import { AuthProvider } from "./auth/AuthContext";
import { installDevLog } from "./utils/devlog";
installDevLog();
ReactDOM.createRoot(document.getElementById("root")).render(_jsx(BrowserRouter, { children: _jsx(AuthProvider, { children: _jsx(App, {}) }) }));
