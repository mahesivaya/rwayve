import { jsx as _jsx } from "react/jsx-runtime";
import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import ForgotPassword from "../../auth/ForgotPassword";
vi.mock("../../api/Auth", () => ({
    forgotPassword: vi.fn(),
}));
import { forgotPassword as apiForgot } from "../../api/Auth";
describe("ForgotPassword page", () => {
    afterEach(() => {
        vi.clearAllMocks();
    });
    it("submits the email and shows the success notice", async () => {
        apiForgot
            .mockResolvedValue({ message: "ok" });
        render(_jsx(MemoryRouter, { children: _jsx(ForgotPassword, {}) }));
        await userEvent.type(screen.getByPlaceholderText("Email"), "a@b.c");
        await userEvent.click(screen.getByRole("button", { name: /send reset link/i }));
        await waitFor(() => {
            expect(apiForgot).toHaveBeenCalledWith("a@b.c");
        });
        expect(await screen.findByText(/if that account exists/i)).toBeInTheDocument();
    });
    it("shows server error inline on failure", async () => {
        apiForgot
            .mockRejectedValue(new Error("server boom"));
        render(_jsx(MemoryRouter, { children: _jsx(ForgotPassword, {}) }));
        await userEvent.type(screen.getByPlaceholderText("Email"), "a@b.c");
        await userEvent.click(screen.getByRole("button", { name: /send reset link/i }));
        expect(await screen.findByText(/server boom/i)).toBeInTheDocument();
    });
});
