import { BrowserRouter, Routes, Route } from "react-router-dom";

import Layout from "./components/Layout";
import Home from "./pages/Home";
import Emails from "./pages/Emails";
import Chat from "./pages/Chat";
import Scheduler from "./pages/Scheduler";
import ProtectedRoute from "./components/ProtectedRoute";
import Login from "./auth/Login";
import Register from "./auth/Register"; // ✅ ADD THIS

export default function App() {
  return (
    <BrowserRouter>
      <Routes>

        {/* ✅ Public route */}
        <Route path="/login" element={<Login />} />
        <Route path="/register" element={<Register />} />

        {/* ✅ Protected layout */}
        <Route path="/" element={<Layout />}>

          <Route index element={<Home />} />

          <Route
            path="emails"
            element={
              <ProtectedRoute>
                <Emails />
              </ProtectedRoute>
            }
          />

          <Route
            path="chat"
            element={
              <ProtectedRoute>
                <Chat />
              </ProtectedRoute>
            }
          />

          <Route
            path="scheduler"
            element={
              <ProtectedRoute>
                <Scheduler />
              </ProtectedRoute>
            }
          />

        </Route>

      </Routes>
    </BrowserRouter>
  );
}