export function formatDateLocal(date: Date) {
  const y = date.getFullYear();
  const m = (date.getMonth() + 1).toString().padStart(2, "0");
  const d = date.getDate().toString().padStart(2, "0");
  return `${y}-${m}-${d}`;
}

export function toMinutes(time: string) {
  const [h, m] = time.split(":").map(Number);
  return h * 60 + m;
}

export function fromTime(time: string) {
  const [h, m] = time.split(":").map(Number);
  return h * 60 + m;
}

export function toTime(mins: number) {
  const h = Math.floor(mins / 60);
  const m = mins % 60;
  return `${h.toString().padStart(2, "0")}:${m.toString().padStart(2, "0")}`;
}

export function todayStr() {
  return formatDateLocal(new Date());
}

export function nowTimeStr() {
  const d = new Date();
  return `${d.getHours().toString().padStart(2, "0")}:${d
    .getMinutes()
    .toString()
    .padStart(2, "0")}`;
}

export function addMinutesToTime(time: string, minutes: number) {
  return toTime(Math.min(toMinutes(time) + minutes, 23 * 60 + 59));
}

export function formatHour(mins: number) {
  const date = new Date();
  date.setHours(Math.floor(mins / 60), mins % 60, 0, 0);
  return date.toLocaleTimeString([], { hour: "numeric" });
}
