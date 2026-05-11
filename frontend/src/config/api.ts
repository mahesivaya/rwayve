import { logger } from "src/utils/logger";

const log = logger.scope("api");
const API_BASE = import.meta.env.VITE_API_URL;

const newReqId = (): string => {
  // Short, sortable, no-deps id for correlation. Backend reads X-Request-ID.
  const c = (globalThis as { crypto?: Crypto }).crypto;
  if (c && "randomUUID" in c) return c.randomUUID().slice(0, 8);
  return Math.random().toString(36).slice(2, 10);
};

export const apiFetch = async (
  endpoint: string,
  options: RequestInit = {}
) => {
  const token = localStorage.getItem("token");
  const method = options.method || "GET";
  const reqId = newReqId();

  log.debug(`→ [${reqId}] ${method} ${endpoint}`);
  const start = performance.now();

  let res: Response;
  try {
    res = await fetch(`${API_BASE}${endpoint}`, {
      ...options,
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
        "X-Request-ID": reqId,
        ...(options.headers || {}),
      },
    });
  } catch (err) {
    log.error(`✗ [${reqId}] ${method} ${endpoint} (network)`, err);
    throw err;
  }

  const ms = Math.round(performance.now() - start);

  if (res.status === 401) {
    log.warn(`← [${reqId}] ${method} ${endpoint} 401 in ${ms}ms — clearing token`);
    localStorage.removeItem("token");
    window.location.href = "/login";
    throw new Error("Unauthorized");
  }

  const line = `← [${reqId}] ${method} ${endpoint} ${res.status} in ${ms}ms`;
  if (!res.ok) log.warn(line);
  else log.debug(line);

  return res;
};
