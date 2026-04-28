import { Routes, Route, Navigate } from "react-router-dom";
import { lazy, Suspense } from "react";

import Layout from "./components/Layout";
import ProtectedRoute from "./components/ProtectedRoute";
import Register from "./auth/Register";
import Login from "./auth/Login";
import { useAuth } from "./auth/AuthContext";

// 🔥 Lazy loaded pages
const Home = lazy(() => import("./home/Home"));
const Emails = lazy(() => import("./emails/Emails"));
const Chat = lazy(() => import("./chat/Chat"));
const Scheduler = lazy(() => import("./scheduler/Scheduler"));
const Drive = lazy(() => import("./drive/DriveBox"));

export default function App() {
  const { user } = useAuth();

  return (
    <Suspense fallback={<div>Loading...</div>}>
      <Routes>

        {/* ROOT */}
        <Route
          path="/"
          element={<Navigate to={user ? "/home" : "/login"} />}
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

        {/* PROTECTED */}
        <Route element={<ProtectedRoute />}>
          <Route element={<Layout />}>

            <Route path="/home" element={<Home />} />
            <Route path="/emails" element={<Emails />} />
            <Route path="/chat" element={<Chat />} />
            <Route path="/scheduler" element={<Scheduler />} />
            <Route path="/drive" element={<Drive />} />

          </Route>
        </Route>

        {/* FALLBACK */}
        <Route
          path="*"
          element={<Navigate to={user ? "/home" : "/login"} />}
        />

      </Routes>
    </Suspense>
  );
}