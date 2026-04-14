import { createContext, useContext, useEffect, useState } from "react";

type UserType = {
  email: string;
  id: number;
};

type AuthType = {
  user: UserType | null;
  login: (token: string) => void;
  logout: () => void;
};

const AuthContext = createContext<AuthType | null>(null);

// 🔥 Manual JWT decode (NO LIBRARY)
const parseJwt = (token: string) => {
  try {
    return JSON.parse(atob(token.split(".")[1]));
  } catch {
    return null;
  }
};

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<UserType | null>(null);
  const [loading, setLoading] = useState(true);

  // 🔥 Restore user on refresh
  // Inside AuthProvider
  useEffect(() => {
    const checkAuth = async () => {
      const token = localStorage.getItem("token");
      if (!token) {
        setLoading(false);
        return;
      }
      try {
        const res = await fetch("/api/me", {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        });
        if (res.status === 401) {
          // 🔥 ONLY logout if backend says invalid
          logout();
          return;
        }
        if (!res.ok) {
          // ❌ don't logout on server errors
          console.warn("Server error, keeping user logged in");
          setLoading(false);
          return;
        }
        const data = await res.json();
        setUser({
          email: data.email,
          id: data.id,
        });
      } catch (err) {
        console.warn("Network error, keeping session");
      }
    setLoading(false);
    };
    checkAuth();
  }, []);

  // 🔥 Login
  const login = (token: string) => {
    localStorage.setItem("token", token);

    const decoded = parseJwt(token);

    if (decoded) {
      setUser({
        email: decoded.email,
        id: decoded.sub,
      });
    }
  };

  // 🔥 Logout
  const logout = () => {
    localStorage.removeItem("token");
    setUser(null);
    window.location.href = "/login";
  };

  if (loading) return <div>Loading...</div>;

  return (
    <AuthContext.Provider value={{ user, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);

  if (!context) {
    throw new Error("useAuth must be used inside AuthProvider");
  }

  return context;
}