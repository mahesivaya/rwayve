import { Routes, Route, Navigate } from "react-router-dom";
import { lazy, Suspense } from "react";

import Layout from "./components/Layout";
import ProtectedRoute from "./components/ProtectedRoute";
import Register from "./auth/Register";
import Login from "./auth/Login";
import ForgotPassword from "./auth/ForgotPassword";
import ResetPassword from "./auth/ResetPassword";
import { useAuth } from "./auth/AuthContext";

// 🔥 Lazy loaded pages
const Home = lazy(() => import("./home/Home"));
const Emails = lazy(() => import("./emails/Emails"));
const Chat = lazy(() => import("./chat/Chat"));
const Call = lazy(() => import("./call/Call"));
const Scheduler = lazy(() => import("./scheduler/Scheduler"));
const Drive = lazy(() => import("./drive/DriveBox"));
const Notes = lazy(() => import("./notes/Notes"));
const AIChat = lazy(() => import("./aichat/AIChat"));
const Profile = lazy(() => import("./profile/Profile"));
const Settings = lazy(() => import("./profile/Settings"));
const Business = lazy(() => import("./business/Business"));
const EmailFiles = lazy(() => import("./files/EmailFiles"));

export default function App() {
  const { user } = useAuth();

  return (
    <Suspense fallback={<div>Loading...</div>}>
      <Routes>

        {/* ROOT */}
        <Route
          path="/"
          element={<Home />}
        />
        <Route
          path="/Home"
          element={<Navigate to="/" replace />}
        />

        {/* PUBLIC */}
        <Route
          path="/login"
          element={user ? <Navigate to="/home" /> : <Login />}
        />
        <Route
          path="/register"
          element={user ? <Navigate to="/home" /> : <Register />}
        />
        <Route path="/forgot-password" element={<ForgotPassword />} />
        <Route path="/reset-password" element={<ResetPassword />} />
        <Route path="/business" element={<Business />} />

        {/* PROTECTED */}
        <Route element={<ProtectedRoute />}>
          <Route element={<Layout />}>

            <Route path="/home" element={<Home />} />
            <Route path="/emails" element={<Emails />} />
            <Route path="/email-files" element={<EmailFiles />} />
            <Route path="/chat" element={<Chat />} />
            <Route path="/call" element={<Call />} />
            <Route path="/scheduler" element={<Scheduler />} />
            <Route path="/drive" element={<Drive />} />
            <Route path="/notes" element={<Notes />} />
            <Route path="/aichat" element={<AIChat />} />
            <Route path="/profile" element={<Profile />} />
            <Route path="/settings" element={<Settings />} />

          </Route>
        </Route>

        {/* FALLBACK */}
        <Route
          path="*"
          element={<Navigate to={user ? "/home" : "/"} replace />}
        />

      </Routes>
    </Suspense>
  );
}
