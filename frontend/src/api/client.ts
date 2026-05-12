import { API_BASE } from "../config/env";

type ApiOptions =
  RequestInit & {
    auth?: boolean;
  };

export async function apiFetch(
  path: string,
  options: ApiOptions = {}
) {
  const {
    auth = true,
    headers,
    ...rest
  } = options;

  const token =
    localStorage.getItem(
      "token"
    );

  const response =
    await fetch(
      `${API_BASE}${path}`,
      {
        ...rest,

        headers: {
          "Content-Type":
            "application/json",

          ...(auth && token
            ? {
                Authorization:
                  `Bearer ${token}`,
              }
            : {}),

          ...headers,
        },
      }
    );

  // ================= GLOBAL ERRORS =================

  if (
    response.status === 401
  ) {
    console.error(
      "Unauthorized"
    );

    localStorage.removeItem(
      "token"
    );

    // Avoid jsdom/Vitest
    // navigation crashes.

    if (
      import.meta.env.MODE !==
      "test"
    ) {
      window.location.href =
        "/login";
    }

    throw new Error(
      "Unauthorized"
    );
  }

  if (!response.ok) {
    let message =
      "Request failed";

    try {
      const data =
        await response
          .clone()
          .json();

      message =
        data.error ||
        data.message ||
        message;

    } catch {
      // ignore json parse errors
    }

    throw new Error(
      message
    );
  }

  return response;
}