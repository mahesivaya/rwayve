export const API_BASE =
  import.meta.env.VITE_API_URL ||
  "";

const configuredWsBase =
  import.meta.env.VITE_WS_URL ||
  import.meta.env.VITE_WS_BASE_URL ||
  `${window.location.protocol === "https:" ? "wss" : "ws"}://${window.location.host}`;

export const WS_BASE =
  configuredWsBase.startsWith("ws")
    ? configuredWsBase
    : `ws://${configuredWsBase}`;
