import { createContext, useContext, useEffect, useState } from "react";
import { savePrivateKey, loadPrivateKey } from "../crypto/keyStore";
import { logger } from "../utils/logger";

const log = logger.scope("auth");

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

type Claims = { sub: number; email: string; exp?: number };

// Decode JWT payload. Returns null on malformed/expired tokens so callers
// can treat a stale token the same as no token.
const parseJwt = (token: string): Claims | null => {
  try {
    const claims = JSON.parse(atob(token.split(".")[1])) as Claims;
    if (typeof claims.exp === "number" && claims.exp * 1000 < Date.now()) {
      return null;
    }
    return claims;
  } catch {
    return null;
  }
};

// Resolve the boot-time token: prefer the OAuth redirect token, otherwise
// reuse the stored one. Side-effect: persists the OAuth token and cleans
// it out of the URL. Runs synchronously before first render so we can
// optimistically populate `user` without flashing a loading screen.
const resolveBootToken = (): string | null => {
  const params = new URLSearchParams(window.location.search);
  const tokenFromUrl = params.get("token");

  if (tokenFromUrl) {
    log.info("restoring token from OAuth redirect");
    localStorage.setItem("token", tokenFromUrl);
    params.delete("token");
    const qs = params.toString();
    window.history.replaceState({}, document.title, qs ? `/emails?${qs}` : "/emails");
    return tokenFromUrl;
  }

  return localStorage.getItem("token");
};

export function AuthProvider({ children }: { children: React.ReactNode }) {
  // Optimistic init: trust a non-expired JWT immediately so the app renders
  // without a round-trip. /api/me below confirms it and logs us out on 401.
  const [user, setUser] = useState<UserType | null>(() => {
    const token = resolveBootToken();
    if (!token) return null;
    const claims = parseJwt(token);
    return claims ? { email: claims.email, id: claims.sub } : null;
  });

  const setupEncryption = async (token: string) => {
    try {
      const existingKey = await loadPrivateKey();

      if (existingKey) {
        log.debug("private key already in IndexedDB");
        return;
      }

      log.info("generating new RSA key pair");

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

      await savePrivateKey(keyPair.privateKey);

      const publicKey = await crypto.subtle.exportKey("spki", keyPair.publicKey);

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

      log.info("encryption setup complete");
    } catch (err) {
      log.error("encryption setup failed", err);
    }
  };

  useEffect(() => {
    const token = localStorage.getItem("token");
    if (!token) return;

    // Validate in the background. AbortController makes StrictMode's
    // double-mount in dev clean up the first request instead of racing.
    const ctrl = new AbortController();

    (async () => {
      try {
        const res = await fetch("/api/me", {
          headers: { Authorization: `Bearer ${token}` },
          signal: ctrl.signal,
        });

        if (res.status === 401) {
          log.warn("/api/me rejected stored token; logging out");
          localStorage.removeItem("token");
          setUser(null);
          window.location.href = "/login";
          return;
        }

        if (!res.ok) {
          const txt = await res.text();
          log.error("/api/me failed", { status: res.status, body: txt });
          return;
        }

        const data = await res.json();
        // Only patch state if the server sees a different user — avoids a
        // pointless re-render when the optimistic claims already matched.
        setUser((prev) =>
          prev && prev.id === data.id && prev.email === data.email
            ? prev
            : { email: data.email, id: data.id }
        );

        setupEncryption(token).catch((err) =>
          log.error("background encryption setup failed", err)
        );
      } catch (err) {
        if ((err as { name?: string }).name === "AbortError") return;
        log.error("auth init network error", err);
      }
    })();

    return () => ctrl.abort();
  }, []);

  const login = (token: string) => {
    localStorage.setItem("token", token);

    const decoded = parseJwt(token);

    if (decoded) {
      setUser({
        email: decoded.email,
        id: decoded.sub,
      });
    }

    setupEncryption(token).catch((err) =>
      log.error("background encryption setup failed", err)
    );
  };

  const logout = () => {
    localStorage.removeItem("token");
    setUser(null);
    window.location.href = "/login";
  };

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
