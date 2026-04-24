import { createContext, useContext, useEffect, useState } from "react";
import { savePrivateKey, loadPrivateKey } from "../crypto/keyStore";

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

// 🔥 JWT decode
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

  // 🔐 Generate + Save Keys (ONLY ONCE)
  const setupEncryption = async (token: string) => {
    try {
      const existingKey = await loadPrivateKey();

      if (existingKey) {
        console.log("🔑 Key already exists in IndexedDB");
        return;
      }

      console.log("🔐 Generating new key pair...");

      const keyPair = await crypto.subtle.generateKey(
        {
          name: "RSA-OAEP",
          modulusLength: 2048,
          publicExponent: new Uint8Array([1, 0, 1]),
          hash: "SHA-256",
        },
        true,
        ["encrypt", "decrypt"]
      );

      // 🔐 Save private key
      await savePrivateKey(keyPair.privateKey);

      // 📤 Export public key
      const publicKey = await crypto.subtle.exportKey("spki", keyPair.publicKey);

      // 🔥 Save public key to backend
      await fetch("/api/save-public-key", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify({
          public_key: Array.from(new Uint8Array(publicKey)),
        }),
      });

      console.log("✅ Encryption setup complete");

    } catch (err) {
      console.error("❌ Encryption setup failed", err);
    }
  };



  useEffect(() => {
    const initAuth = async () => {
      const params = new URLSearchParams(window.location.search);
  
      let token = localStorage.getItem("token");
      const tokenFromUrl = params.get("token");
  
      // 1) Prefer token from URL (OAuth)
      if (tokenFromUrl) {
        console.log("🔐 Restoring token from OAuth");
        localStorage.setItem("token", tokenFromUrl);
        token = tokenFromUrl;
  
        // remove only token param, keep connected=true
        params.delete("token");
        const newUrl = `/emails?${params.toString()}`;
        window.history.replaceState({}, document.title, newUrl);
      }
  
      // 2) If still no token → stop
      if (!token) {
        console.warn("⚠️ No token found");
        setLoading(false);
        return;
      }
  
      try {
        // 3) Call /api/me with the SAME token variable
        const res = await fetch("/api/me", {
          headers: { Authorization: `Bearer ${token}` },
        });
  
        if (res.status === 401) {
          logout();
          return;
        }
  
        if (!res.ok) {
          const txt = await res.text();
          console.error("❌ /api/me failed:", res.status, txt);
          setLoading(false);
          return;
        }
  
        const data = await res.json();
        setUser({ email: data.email, id: data.id });
  
        // 4) Ensure keys exist
        await setupEncryption(token);
      } catch (err) {
        console.error("❌ Network error:", err);
      }
  
      setLoading(false);
    };
  
    initAuth();
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

    // ❌ DO NOT call setupEncryption here
    // it runs in useEffect already
    setupEncryption(token);
  };

  // 🔥 Logout (FIXED)
  const logout = () => {
    localStorage.removeItem("token");

    // ❗ DO NOT delete private key (important for decrypt)
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

// 🔥 Hook
export function useAuth() {
  const context = useContext(AuthContext);

  if (!context) {
    throw new Error("useAuth must be used inside AuthProvider");
  }

  return context;
}