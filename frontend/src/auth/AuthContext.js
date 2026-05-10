import { jsx as _jsx } from "react/jsx-runtime";
import { createContext, useContext, useEffect, useState } from "react";
import { savePrivateKey, loadPrivateKey } from "../crypto/keyStore";
import { logger } from "../utils/logger";
const log = logger.scope("auth");
const AuthContext = createContext(null);
// Decode JWT payload. Returns null on malformed/expired tokens so callers
// can treat a stale token the same as no token.
const parseJwt = (token) => {
    try {
        const claims = JSON.parse(atob(token.split(".")[1]));
        if (typeof claims.exp === "number" && claims.exp * 1000 < Date.now()) {
            return null;
        }
        return claims;
    }
    catch {
        return null;
    }
};
// Resolve the boot-time token: prefer the OAuth redirect token, otherwise
// reuse the stored one. Side-effect: persists the OAuth token and cleans
// it out of the URL. Runs synchronously before first render so we can
// optimistically populate `user` without flashing a loading screen.
//
// IMPORTANT: only consume `?token=` when an OAuth marker is also present.
// Other features (like password reset) also use `?token=` on their own URLs
// and must not have it stolen here.
const resolveBootToken = () => {
    const params = new URLSearchParams(window.location.search);
    const tokenFromUrl = params.get("token");
    const isOAuthLanding = params.has("signup") || params.has("connected");
    if (tokenFromUrl && isOAuthLanding) {
        log.info("restoring token from OAuth redirect");
        localStorage.setItem("token", tokenFromUrl);
        params.delete("token");
        const qs = params.toString();
        const path = window.location.pathname || "/home";
        window.history.replaceState({}, document.title, qs ? `${path}?${qs}` : path);
        return tokenFromUrl;
    }
    return localStorage.getItem("token");
};
export function AuthProvider({ children }) {
    // Optimistic init: trust a non-expired JWT immediately so the app renders
    // without a round-trip. /api/me below confirms it and logs us out on 401.
    const [user, setUser] = useState(() => {
        const token = resolveBootToken();
        if (!token)
            return null;
        const claims = parseJwt(token);
        return claims ? { email: claims.email, id: claims.sub } : null;
    });
    const setupEncryption = async (token) => {
        try {
            const existingKey = await loadPrivateKey();
            if (existingKey) {
                log.debug("private key already in IndexedDB");
                return;
            }
            log.info("generating new RSA key pair");
            const keyPair = await crypto.subtle.generateKey({
                name: "RSA-OAEP",
                modulusLength: 2048,
                publicExponent: new Uint8Array([1, 0, 1]),
                hash: "SHA-256",
            }, true, ["encrypt", "decrypt"]);
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
        }
        catch (err) {
            log.error("encryption setup failed", err);
        }
    };
    useEffect(() => {
        const token = localStorage.getItem("token");
        if (!token)
            return;
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
                    log.warn("/api/me rejected stored token; clearing session");
                    localStorage.removeItem("token");
                    setUser(null);
                    // No hard redirect: ProtectedRoute already sends unauthenticated
                    // users away from protected pages, and public pages (/login,
                    // /register, /reset-password, ...) must stay rendered.
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
                setUser((prev) => prev && prev.id === data.id && prev.email === data.email
                    ? prev
                    : { email: data.email, id: data.id });
                setupEncryption(token).catch((err) => log.error("background encryption setup failed", err));
            }
            catch (err) {
                if (err.name === "AbortError")
                    return;
                log.error("auth init network error", err);
            }
        })();
        return () => ctrl.abort();
    }, []);
    const login = (token) => {
        localStorage.setItem("token", token);
        const decoded = parseJwt(token);
        if (decoded) {
            setUser({
                email: decoded.email,
                id: decoded.sub,
            });
        }
        setupEncryption(token).catch((err) => log.error("background encryption setup failed", err));
    };
    const logout = () => {
        localStorage.removeItem("token");
        setUser(null);
        window.location.href = "/login";
    };
    return (_jsx(AuthContext.Provider, { value: { user, login, logout }, children: children }));
}
export function useAuth() {
    const context = useContext(AuthContext);
    if (!context) {
        throw new Error("useAuth must be used inside AuthProvider");
    }
    return context;
}
