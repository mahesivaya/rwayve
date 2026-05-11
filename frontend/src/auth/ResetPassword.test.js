import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import ResetPassword from "./ResetPassword";
vi.mock("../api/Auth", () => ({
    resetPassword: vi.fn(),
}));
import { resetPassword as apiReset } from "../api/Auth";
const renderAt = (url) => render(_jsx(MemoryRouter, { initialEntries: [url], children: _jsxs(Routes, { children: [_jsx(Route, { path: "/reset-password", element: _jsx(ResetPassword, {}) }), _jsx(Route, { path: "/login", element: _jsx("div", { children: "Login page" }) })] }) }));
describe("ResetPassword page", () => {
    afterEach(() => {
        vi.clearAllMocks();
    });
    it("rejects mismatched passwords client-side", async () => {
        renderAt("/reset-password?token=tok123");
        await userEvent.type(screen.getByPlaceholderText("New password"), "abcdef");
        await userEvent.type(screen.getByPlaceholderText("Confirm new password"), "different");
        await userEvent.click(screen.getByRole("button", { name: /update password/i }));
        expect(await screen.findByText(/passwords do not match/i)).toBeInTheDocument();
        expect(apiReset).not.toHaveBeenCalled();
    });
    it("rejects short passwords client-side", async () => {
        renderAt("/reset-password?token=tok123");
        await userEvent.type(screen.getByPlaceholderText("New password"), "ab");
        await userEvent.type(screen.getByPlaceholderText("Confirm new password"), "ab");
        await userEvent.click(screen.getByRole("button", { name: /update password/i }));
        expect(await screen.findByText(/at least 6 characters/i)).toBeInTheDocument();
    });
    it("submits token + password and shows confirmation", async () => {
        apiReset
            .mockResolvedValue({ message: "ok" });
        renderAt("/reset-password?token=tok123");
        await userEvent.type(screen.getByPlaceholderText("New password"), "fresh-pw");
        await userEvent.type(screen.getByPlaceholderText("Confirm new password"), "fresh-pw");
        await userEvent.click(screen.getByRole("button", { name: /update password/i }));
        await waitFor(() => {
            expect(apiReset).toHaveBeenCalledWith("tok123", "fresh-pw");
        });
        expect(await screen.findByText(/password updated/i)).toBeInTheDocument();
    });
    it("shows missing-token error if URL has no token", async () => {
        renderAt("/reset-password");
        await userEvent.type(screen.getByPlaceholderText("New password"), "fresh-pw");
        await userEvent.type(screen.getByPlaceholderText("Confirm new password"), "fresh-pw");
        await userEvent.click(screen.getByRole("button", { name: /update password/i }));
        expect(await screen.findByText(/missing reset token/i)).toBeInTheDocument();
    });
});
