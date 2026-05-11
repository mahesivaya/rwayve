import "@testing-library/jest-dom/vitest";
import { afterEach, vi } from "vitest";
import { cleanup } from "@testing-library/react";
// jsdom 29 + vitest 4 doesn't always expose a working Storage. Install a
// minimal in-memory polyfill on both `globalThis` and `window` so component
// code that touches `localStorage` directly Just Works in tests.
class MemoryStorage {
    store = new Map();
    get length() {
        return this.store.size;
    }
    clear() {
        this.store.clear();
    }
    getItem(key) {
        return this.store.has(key) ? this.store.get(key) : null;
    }
    setItem(key, value) {
        this.store.set(key, String(value));
    }
    removeItem(key) {
        this.store.delete(key);
    }
    key(index) {
        return Array.from(this.store.keys())[index] ?? null;
    }
}
const installStorage = (target, prop) => {
    Object.defineProperty(target, prop, {
        configurable: true,
        enumerable: true,
        writable: true,
        value: new MemoryStorage(),
    });
};
installStorage(globalThis, "localStorage");
installStorage(globalThis, "sessionStorage");
if (typeof window !== "undefined") {
    installStorage(window, "localStorage");
    installStorage(window, "sessionStorage");
}
afterEach(() => {
    cleanup();
    globalThis.localStorage.clear();
    if (typeof window !== "undefined") {
        window.localStorage.clear();
    }
    vi.restoreAllMocks();
});
// VITE_API_URL is read at module-load time; default it to a stable value
// so tests don't depend on a real .env file.
if (!import.meta.env.VITE_API_URL) {
    // @ts-expect-error mutating import.meta.env in tests
    import.meta.env.VITE_API_URL = "http://test.local";
}
