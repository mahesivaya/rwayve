export const API_BASE =
  import.meta.env.VITE_API_URL ||
  "http://localhost:8080";

const configuredWsBase =
  import.meta.env.VITE_WS_URL ||
  import.meta.env.VITE_WS_BASE_URL ||
  "ws://localhost:8080";

export const WS_BASE =
  configuredWsBase.startsWith("ws")
    ? configuredWsBase
    : `ws://${configuredWsBase}`;
