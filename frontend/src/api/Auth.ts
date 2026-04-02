
export async function register(email: string, password: string, confirm: string) {
  const res = await fetch("http://localhost:8080/api/register", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      email,
      password,
      confirm_password: confirm,
    }),
  });

  // 🔥 IMPORTANT: read response
  const data = await res.json();

  // 🔥 handle backend errors
  if (!res.ok) {
    throw new Error(data.message || "Register failed");
  }

  return data; // ✅ THIS FIXES YOUR BUG
}



export async function login(email: string, password: string) {
  try {
    const res = await fetch("http://localhost:8080/api/login", {
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