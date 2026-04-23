import { jsx as _jsx, Fragment as _Fragment, jsxs as _jsxs } from "react/jsx-runtime";
import { useAuth } from "../auth/AuthContext";
import { useNavigate } from "react-router-dom";
export default function Header() {
    const { user, logout } = useAuth();
    const navigate = useNavigate();
    return (_jsx("div", { children: user ? (_jsxs(_Fragment, { children: [_jsx("span", { children: user.email }), _jsx("button", { onClick: () => {
                        logout();
                        navigate("/login");
                    }, children: "Logout" })] })) : (_jsx("button", { children: "Login" })) }));
}
