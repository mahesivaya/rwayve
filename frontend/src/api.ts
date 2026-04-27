const API_BASE = import.meta.env.VITE_API_URL;

export const apiFetch = async (endpoint: string, options: any = {}) => {
  const token = localStorage.getItem("token");

  return fetch(`${API_BASE}${endpoint}`, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
      ...(options.headers || {}),
    },
  });
};