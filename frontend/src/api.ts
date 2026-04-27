const API_BASE = import.meta.env.VITE_API_URL;

export const apiFetch = async (
    endpoint: string,
    options: RequestInit = {}
  ) => {
    const token = localStorage.getItem("token");
  
    const res = await fetch(`${API_BASE}${endpoint}`, {
      ...options,
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
        ...(options.headers || {}),
      },
    });
  
    // 🔐 Global 401 handling
    if (res.status === 401) {
      localStorage.removeItem("token");
      window.location.href = "/login";
      throw new Error("Unauthorized");
    }
  
    return res;
  };