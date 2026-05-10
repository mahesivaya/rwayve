import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { AuthProvider, useAuth } from "./AuthContext";

// Tiny consumer that surfaces auth state for assertions.
function AuthProbe() {
  const { user } = useAuth();
  return (
    <div>
      <span data-testid="user-email">{user?.email ?? "anon"}</span>
      <span data-testid="user-id">{user?.id ?? -1}</span>
      <span data-testid="path">{window.location.pathname}</span>
      <span data-testid="search">{window.location.search}</span>
    </div>
  );
}

// HS256 JWT signed with secret "secret" so the parser succeeds.
// payload: { "sub": 99, "email": "alice@example.com", "exp": 9999999999 }
const VALID_JWT =
  "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9." +
  "eyJzdWIiOjk5LCJlbWFpbCI6ImFsaWNlQGV4YW1wbGUuY29tIiwiZXhwIjo5OTk5OTk5OTk5fQ." +
  "Yfk2GANHfoqcl3T1jbBhHptPj0xK_e3pGE9pq5VtZ8I";

const renderProvider = (initialUrl: string) => {
  // jsdom's location is read-only via assignment; replaceState works.
  window.history.replaceState({}, "", initialUrl);
  return render(
    <MemoryRouter>
      <AuthProvider>
        <AuthProbe />
      </AuthProvider>
    </MemoryRouter>,
  );
};

describe("AuthContext.resolveBootToken", () => {
  beforeEach(() => {
    localStorage.clear();
    // /api/me succeeds by default so the background validator doesn't trip.
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: true,
        status: 200,
        json: async () => ({ id: 99, email: "alice@example.com" }),
      } as unknown as Response),
    );
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("does NOT consume ?token= on /reset-password (the bug we fixed)", () => {
    renderProvider(`/reset-password?token=${"a".repeat(64)}`);

    // localStorage must remain empty — this is the core regression check.
    expect(localStorage.getItem("token")).toBeNull();
    expect(screen.getByTestId("user-email").textContent).toBe("anon");

    // URL must keep the reset token so ResetPassword.tsx can read it.
    expect(window.location.pathname).toBe("/reset-password");
    expect(window.location.search).toContain("token=");
  });

  it("consumes ?token= when ?signup=true marker is present (Google signup)", () => {
    renderProvider(`/home?signup=true&token=${VALID_JWT}`);

    expect(localStorage.getItem("token")).toBe(VALID_JWT);
    expect(screen.getByTestId("user-email").textContent).toBe(
      "alice@example.com",
    );
    expect(screen.getByTestId("user-id").textContent).toBe("99");

    // token query should be stripped from the URL.
    expect(window.location.search).not.toContain("token=");
  });

  it("consumes ?token= when ?connected=true marker is present (Gmail connect)", () => {
    renderProvider(`/emails?connected=true&token=${VALID_JWT}`);
    expect(localStorage.getItem("token")).toBe(VALID_JWT);
    expect(screen.getByTestId("user-email").textContent).toBe(
      "alice@example.com",
    );
  });

  it("uses stored localStorage token if present and no URL token", () => {
    localStorage.setItem("token", VALID_JWT);
    renderProvider("/home");
    expect(screen.getByTestId("user-email").textContent).toBe(
      "alice@example.com",
    );
  });

  it("clears stored token on /api/me 401 but does NOT hard-redirect", async () => {
    localStorage.setItem("token", VALID_JWT);
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: false,
        status: 401,
        text: async () => "",
        json: async () => ({}),
      } as unknown as Response),
    );

    renderProvider("/reset-password?token=abc");

    await waitFor(() => {
      expect(localStorage.getItem("token")).toBeNull();
    });
    // Path must remain /reset-password — the old bug used to redirect to /login.
    expect(window.location.pathname).toBe("/reset-password");
  });
});
