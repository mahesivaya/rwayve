import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import Profile from "../../profile/Profile";

vi.mock("../../api/Auth", () => ({
  changePassword: vi.fn(),
}));
import { changePassword as apiChange } from "../../api/Auth";

type ProfileShape = {
  id: number;
  email: string;
  first_name: string | null;
  last_name: string | null;
  auth_provider: string;
};

const mockProfileFetch = (profile: ProfileShape) => {
  vi.stubGlobal(
    "fetch",
    vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => profile,
      text: async () => JSON.stringify(profile),
    } as unknown as Response),
  );
};

describe("Profile page", () => {
  beforeEach(() => {
    localStorage.setItem("token", "jwt");
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    vi.clearAllMocks();
  });

  it("shows the Change Password button for local-auth users", async () => {
    mockProfileFetch({
      id: 1,
      email: "a@b.c",
      first_name: "A",
      last_name: "B",
      auth_provider: "local",
    });

    render(<Profile />);

    expect(
      await screen.findByRole("button", { name: /change password/i }),
    ).toBeInTheDocument();
  });

  it("shows Create Password for Google-auth users", async () => {
    mockProfileFetch({
      id: 2,
      email: "g@b.c",
      first_name: null,
      last_name: null,
      auth_provider: "google",
    });

    render(<Profile />);

    // Wait for profile load (the email row appearing in the readonly cell).
    await screen.findByText("g@b.c");

    expect(
      screen.queryByRole("button", { name: /change password/i }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /create password/i }),
    ).toBeInTheDocument();
  });

  it("creates password for Google-auth users without current password", async () => {
    mockProfileFetch({
      id: 2,
      email: "g@b.c",
      first_name: null,
      last_name: null,
      auth_provider: "google",
    });
    (apiChange as unknown as { mockResolvedValue: (v: unknown) => void })
      .mockResolvedValue({ message: "Password updated" });

    render(<Profile />);

    await userEvent.click(
      await screen.findByRole("button", { name: /create password/i }),
    );

    expect(screen.queryByLabelText(/current password/i)).not.toBeInTheDocument();

    await userEvent.type(screen.getByLabelText(/^password$/i), "fresh-pw");
    await userEvent.type(
      screen.getByLabelText(/confirm password/i),
      "fresh-pw",
    );
    await userEvent.click(
      screen.getByRole("button", { name: /create password/i }),
    );

    await waitFor(() => {
      expect(apiChange).toHaveBeenCalledWith(null, "fresh-pw");
    });
    expect(
      await screen.findByText(/password created/i),
    ).toBeInTheDocument();
  });

  it("submits change-password form and shows success", async () => {
    mockProfileFetch({
      id: 1,
      email: "a@b.c",
      first_name: "A",
      last_name: "B",
      auth_provider: "local",
    });
    (apiChange as unknown as { mockResolvedValue: (v: unknown) => void })
      .mockResolvedValue({ message: "Password updated" });

    render(<Profile />);

    const openBtn = await screen.findByRole("button", {
      name: /change password/i,
    });
    await userEvent.click(openBtn);

    await userEvent.type(
      screen.getByLabelText(/current password/i),
      "old-pw",
    );
    await userEvent.type(screen.getByLabelText(/^new password$/i), "fresh-pw");
    await userEvent.type(
      screen.getByLabelText(/confirm new password/i),
      "fresh-pw",
    );
    await userEvent.click(
      screen.getByRole("button", { name: /update password/i }),
    );

    await waitFor(() => {
      expect(apiChange).toHaveBeenCalledWith("old-pw", "fresh-pw");
    });
    expect(
      await screen.findByText(/password updated/i),
    ).toBeInTheDocument();
  });

  it("rejects mismatched passwords client-side", async () => {
    mockProfileFetch({
      id: 1,
      email: "a@b.c",
      first_name: "",
      last_name: "",
      auth_provider: "local",
    });

    render(<Profile />);
    await userEvent.click(
      await screen.findByRole("button", { name: /change password/i }),
    );

    await userEvent.type(screen.getByLabelText(/current password/i), "x");
    await userEvent.type(screen.getByLabelText(/^new password$/i), "abcdef");
    await userEvent.type(
      screen.getByLabelText(/confirm new password/i),
      "different",
    );
    await userEvent.click(
      screen.getByRole("button", { name: /update password/i }),
    );

    expect(
      await screen.findByText(/passwords do not match/i),
    ).toBeInTheDocument();
    expect(apiChange).not.toHaveBeenCalled();
  });
});
