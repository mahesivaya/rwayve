import { BrowserRouter, Routes, Route } from "react-router-dom";

import Layout from "./components/Layout";
import Home from "./pages/Home";
import Emails from "./pages/Emails";
import Scheduler from "./pages/Scheduler";
import ProtectedRoute from "./components/ProtectedRoute";
import Login from "./auth/Login";
import Register from "./auth/Register";
import Chat from "./chat/Chat";

export default function App() {
  return (
    <BrowserRouter>
      <Routes>

        <Route path="/login" element={<Login />} />
        <Route path="/register" element={<Register />} />
        <Route path="/" element={<Layout />}>
        <Route index element={<Home />} />
        <Route path="emails" element={<ProtectedRoute><Emails /></ProtectedRoute>}/>
        <Route path="chat" element={<ProtectedRoute><Chat /></ProtectedRoute>}/>
        <Route path="scheduler" element={<ProtectedRoute><Scheduler /></ProtectedRoute>}/>
        </Route>
      </Routes>
    </BrowserRouter>
  );
}