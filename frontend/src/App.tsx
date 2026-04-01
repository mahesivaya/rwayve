import { Routes, Route, Navigate } from "react-router-dom";
import Layout from "./components/Layout";
import ProtectedRoute from "./components/ProtectedRoute";

import Login from "./auth/Login";
import Home from "./pages/Home";
import Emails from "./pages/Emails";
import Chat from "./chat/Chat";
import Scheduler from "./scheduler/Scheduler";

import { useAuth } from "./auth/AuthContext";

export default function App() {
  const { user } = useAuth();

  return (
    <Routes>
      {/* 🔓 Public */}
      <Route
        path="/login"
        element={user ? <Navigate to="/emails" /> : <Login />}
      />
      <Route path="/" element={<Home />} />
      <Route element={<Layout />}>

      {/* 🔐 Protected */}
      <Route element={<ProtectedRoute />}>
          <Route path="emails" element={<Emails />} />
          <Route path="chat" element={<Chat />} />
          <Route path="scheduler" element={<Scheduler />} />
        </Route>
      </Route>

      {/* fallback */}
      <Route
        path="*"
        element={<Navigate to={user ? "/emails" : "/login"} />}
      />
    </Routes>
  );
}