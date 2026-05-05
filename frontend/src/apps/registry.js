import { lazy } from "react";
export const APP_REGISTRY = {
    emails: {
        key: "emails",
        label: "Emails",
        icon: "📧",
        path: "/emails",
        Component: lazy(() => import("../emails/Emails")),
    },
    chat: {
        key: "chat",
        label: "Chat",
        icon: "💬",
        path: "/chat",
        Component: lazy(() => import("../chat/Chat")),
    },
    scheduler: {
        key: "scheduler",
        label: "Scheduler",
        icon: "📅",
        path: "/scheduler",
        Component: lazy(() => import("../scheduler/Scheduler")),
    },
    drive: {
        key: "drive",
        label: "Drive",
        icon: "📁",
        path: "/drive",
        Component: lazy(() => import("../drive/DriveBox")),
    },
    call: {
        key: "call",
        label: "Call",
        icon: "📞",
        path: "/call",
        Component: lazy(() => import("../call/Call")),
    },
};
export const APP_LIST = Object.values(APP_REGISTRY);
export function isAppKey(value) {
    return value !== undefined && value in APP_REGISTRY;
}
