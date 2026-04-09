import { Routes, Route, Navigate } from "react-router-dom";
import Layout from "./components/Layout";
import ProtectedRoute from "./components/ProtectedRoute";
import Register from "./auth/Register";
import Login from "./auth/Login";
import Home from "./home/Home";
import Emails from "./emails/Emails";
import Chat from "./chat/Chat";
import Scheduler from "./scheduler/Scheduler";
import Drive from "./drive/DriveBox";
import { useAuth } from "./auth/AuthContext";

export default function App() {
  const { user } = useAuth();

  return (
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

    <Route path="/home" element={<Home />} /> ✅
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
  );
}