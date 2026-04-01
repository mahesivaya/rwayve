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
  currentDate,
  setCurrentDate,
}: Props) {

  // 🔥 Format current date
  const label = useMemo(() => {
    return currentDate.toLocaleDateString("en-US", {
      month: "long",
      year: "numeric",
    });
  }, [currentDate]);

  // 🔥 Navigation logic
  const goPrev = () => {
    const newDate = new Date(currentDate);

    if (view === "month") newDate.setMonth(newDate.getMonth() - 1);
    if (view === "week") newDate.setDate(newDate.getDate() - 7);
    if (view === "day") newDate.setDate(newDate.getDate() - 1);

    setCurrentDate(newDate);
  };

  const goNext = () => {
    const newDate = new Date(currentDate);

    if (view === "month") newDate.setMonth(newDate.getMonth() + 1);
    if (view === "week") newDate.setDate(newDate.getDate() + 7);
    if (view === "day") newDate.setDate(newDate.getDate() + 1);

    setCurrentDate(newDate);
  };

  const goToday = () => {
    setCurrentDate(new Date());
  };

  return (
    <div className="calendar-header">

      {/* LEFT CONTROLS */}
      <div className="nav-controls">
        <button onClick={goToday}>Today</button>
        <button onClick={goPrev}>◀</button>
        <button onClick={goNext}>▶</button>
      </div>

      {/* CENTER TITLE */}
      <div className="calendar-title">
        {label}
      </div>

      {/* RIGHT VIEW SWITCH */}
      <div className="view-switch">
        <button
          className={view === "day" ? "active" : ""}
          onClick={() => setView("day")}
        >
          Day
        </button>

        <button
          className={view === "week" ? "active" : ""}
          onClick={() => setView("week")}
        >
          Week
        </button>

        <button
          className={view === "month" ? "active" : ""}
          onClick={() => setView("month")}
        >
          Month
        </button>
      </div>

    </div>
  );
}