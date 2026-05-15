import type { CalendarItem } from "./types";

export const CALENDAR_STORAGE_KEY = "wayve.scheduler.calendars";
export const EVENT_CALENDAR_STORAGE_KEY = "wayve.scheduler.eventCalendars";

export const DEFAULT_CALENDARS: CalendarItem[] = [
  { id: "office", name: "Office Calendar", color: "#1a73e8", visible: true },
  { id: "personal", name: "Personal Calendar", color: "#34a853", visible: true },
  { id: "holiday", name: "Holiday Calendar", color: "#fbbc04", visible: true },
];

export const CALENDAR_COLORS = [
  "#1a73e8",
  "#34a853",
  "#fbbc04",
  "#a142f4",
  "#fa7b17",
  "#24c1e0",
  "#e8710a",
];

export const DEFAULT_VISIBLE_START_HOUR = 6;
export const DAY_SLOTS = Array.from({ length: 48 }, (_, index) => index);
