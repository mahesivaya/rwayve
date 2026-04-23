import { jsx as _jsx } from "react/jsx-runtime";
import { Navigate, Outlet } from "react-router-dom";
import { useAuth } from "../auth/AuthContext";
export default function ProtectedRoute() {
    const { user } = useAuth();
    if (!user) {
        return _jsx(Navigate, { to: "/" });
    }
    return _jsx(Outlet, {});
}
