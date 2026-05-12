import { API_BASE } from "../config/env";

type ApiOptions =
  RequestInit & {
    auth?: boolean;

    // Preserve backend 401
    // messages without forcing
    // logout/redirect.
    preserve401?: boolean;
  };

export async function apiFetch(
  path: string,
  options: ApiOptions = {}
) {
  const {
    auth = true,

    preserve401 = false,

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

  // ================= 401 =================

  if (
    response.status === 401
  ) {
    let message =
      "Unauthorized";

    try {
      const data =
        await response
          .clone()
          .json();

      message =
        data?.error ||
        data?.message ||
        message;

    } catch {
      // ignore
    }

    // Some endpoints intentionally
    // return 401 without invalidating
    // the session.
    //
    // Example:
    // - wrong current password
    // - MFA challenge
    // - partial auth flows

    if (preserve401) {
      throw new Error(
        message
      );
    }

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
      message
    );
  }

  // ================= OTHER ERRORS =================

  if (!response.ok) {
    let message =
      "Request failed";

    try {
      const data =
        await response
          .clone()
          .json();

      message =
        data?.error ||
        data?.message ||
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