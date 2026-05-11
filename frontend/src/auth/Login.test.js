import { jsx as _jsx } from "react/jsx-runtime";
import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import Login from "./Login";
import { AuthProvider } from "./AuthContext";
vi.mock("../api/Auth", () => ({
    login: vi.fn(),
}));
import { login as apiLogin } from "../api/Auth";
const renderAt = (initialEntries) => render(_jsx(MemoryRouter, { initialEntries: initialEntries, children: _jsx(AuthProvider, { children: _jsx(Login, {}) }) }));
describe("Login page", () => {
    afterEach(() => {
        vi.clearAllMocks();
    });
    it("submits credentials and stores the token on success", async () => {
        apiLogin
            .mockResolvedValue({ token: "jwt-1" });
        renderAt(["/login"]);
        await userEvent.type(screen.getByPlaceholderText("Email"), "a@b.c");
        await userEvent.type(screen.getByPlaceholderText("Password"), "pw");
        await userEvent.click(screen.getByRole("button", { name: /^login$/i }));
        await waitFor(() => {
            expect(localStorage.getItem("token")).toBe("jwt-1");
        });
        expect(apiLogin).toHaveBeenCalledWith("a@b.c", "pw");
    });
    it("shows inline error on auth failure", async () => {
        apiLogin
            .mockRejectedValue(new Error("bad creds"));
        renderAt(["/login"]);
        await userEvent.type(screen.getByPlaceholderText("Email"), "a@b.c");
        await userEvent.type(screen.getByPlaceholderText("Password"), "wrong");
        await userEvent.click(screen.getByRole("button", { name: /^login$/i }));
        expect(await screen.findByText(/login failed/i)).toBeInTheDocument();
    });
    it("shows email_exists banner when redirected from OAuth", () => {
        renderAt(["/login?error=email_exists"]);
        expect(screen.getByText(/already registered with a password/i)).toBeInTheDocument();
    });
    it("renders the Continue with Gmail button and Forgot password link", () => {
        renderAt(["/login"]);
        expect(screen.getByRole("button", { name: /continue with gmail/i })).toBeInTheDocument();
        expect(screen.getByText(/forgot password/i)).toBeInTheDocument();
    });
});
