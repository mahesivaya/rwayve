import { lazy, type ComponentType, type LazyExoticComponent } from "react";

export type AppKey = "emails" | "chat" | "scheduler" | "drive" | "call";

export type AppDef = {
  key: AppKey;
  label: string;
  icon: string;
  path: string;
  Component: LazyExoticComponent<ComponentType>;
};

export const APP_REGISTRY: Record<AppKey, AppDef> = {
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

export const APP_LIST: AppDef[] = Object.values(APP_REGISTRY);

export function isAppKey(value: string | undefined): value is AppKey {
  return value !== undefined && value in APP_REGISTRY;
}
