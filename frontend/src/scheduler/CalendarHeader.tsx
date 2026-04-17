import { useMemo } from "react";

type Props = {
  view: "day" | "week" | "month";
  setView: (v: "day" | "week" | "month") => void;
  currentDate: Date;
  setCurrentDate: (d: Date) => void;
};

export default function CalendarHeader({
  view,
  setView,
  currentDate,        // ✅ MUST be here
  setCurrentDate,
}: Props) {

  const label = useMemo(() => {
    if (view === "month") {
      return currentDate.toLocaleDateString("en-US", {
        month: "long",
        year: "numeric",
      });
    }

    if (view === "week") {
      const start = new Date(currentDate);
      start.setDate(currentDate.getDate() - currentDate.getDay());

      const end = new Date(start);
      end.setDate(start.getDate() + 6);

      return `${start.toLocaleDateString("en-US", {
        month: "short",
        day: "numeric",
      })} - ${end.toLocaleDateString("en-US", {
        month: "short",
        day: "numeric",
      })}`;
    }

    return currentDate.toLocaleDateString("en-US", {
      weekday: "long",
      month: "long",
      day: "numeric",
    });
  }, [currentDate, view]);

  return <div>{label}</div>;
}