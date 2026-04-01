const API_BASE = "http://localhost:8080";

export async function register(
  email: string,
  password: string,
  confirm_password: string
) {
  try {
    const res = await fetch(`${API_BASE}/api/register`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email, password, confirm_password }),
    });

    const text = await res.text();

    if (!res.ok) {
      let message = "Register failed";
  
      try {
        const json = JSON.parse(text);
        message = json.message || message;
      } catch {
        message = text;
      }
  
      throw new Error(message); // ✅ send clean message
    }
  } catch (err) {
    console.error("register error:", err);
    throw err;
  }
}



export async function login(email: string, password: string) {
  try {
    const res = await fetch(`${API_BASE}/api/login`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email, password }),
    });

    const text = await res.text();

    if (!res.ok) {
      throw new Error(`Login failed: ${res.status} ${text}`);
    }

    return text ? JSON.parse(text) : {};
  } catch (err) {
    console.error("login error:", err);
    throw err;
  }
}