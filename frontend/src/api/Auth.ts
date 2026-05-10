import { logger } from "../utils/logger";

const log = logger.scope("auth");
const API_BASE = import.meta.env.VITE_API_URL;

const newReqId = (): string => {
  const c = (globalThis as { crypto?: Crypto }).crypto;
  if (c && "randomUUID" in c) return c.randomUUID().slice(0, 8);
  return Math.random().toString(36).slice(2, 10);
};

export async function register(email: string, password: string, confirm: string) {
  const reqId = newReqId();
  log.info(`[${reqId}] register attempt`, { email });
  const start = performance.now();

  const res = await fetch(`${API_BASE}/api/register`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "X-Request-ID": reqId,
    },
    body: JSON.stringify({ email, password, confirm_password: confirm }),
  });

  const ms = Math.round(performance.now() - start);
  const data = await res.json();

  if (!res.ok) {
    log.warn(`[${reqId}] register rejected (${res.status} in ${ms}ms)`, {
      email,
      message: data?.message,
    });
    throw new Error(data.message || "Register failed");
  }

  log.info(`[${reqId}] register ok in ${ms}ms`, { email });
  return data;
}

export async function login(email: string, password: string) {
  const reqId = newReqId();
  log.info(`[${reqId}] login attempt`, { email });
  const start = performance.now();

  try {
    const res = await fetch(`${API_BASE}/api/login`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-Request-ID": reqId,
      },
      body: JSON.stringify({ email, password }),
    });

    const ms = Math.round(performance.now() - start);
    const text = await res.text();

    if (!res.ok) {
      log.warn(`[${reqId}] login rejected (${res.status} in ${ms}ms)`, { email });
      throw new Error(`Login failed: ${res.status} ${text}`);
    }

    log.info(`[${reqId}] login ok in ${ms}ms`, { email });
    return text ? JSON.parse(text) : {};
  } catch (err) {
    log.error(`[${reqId}] login error`, err);
    throw err;
  }
}

export async function forgotPassword(email: string) {
  const res = await fetch(`${API_BASE}/api/forgot-password`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email }),
  });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(data.message || "Request failed");
  return data;
}

export async function resetPassword(token: string, newPassword: string) {
  const res = await fetch(`${API_BASE}/api/reset-password`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ token, new_password: newPassword }),
  });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(data.message || "Reset failed");
  return data;
}

export async function changePassword(currentPassword: string, newPassword: string) {
  const token = localStorage.getItem("token");
  const res = await fetch(`${API_BASE}/api/profile/password`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify({ current_password: currentPassword, new_password: newPassword }),
  });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(data.message || "Change failed");
  return data;
}
