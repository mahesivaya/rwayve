import { useState } from "react";
import "./scheduler.css";

export default function Scheduler() {
  const [view, setView] = useState<"day" | "week" | "month">("month");
  const [currentDate, setCurrentDate] = useState(new Date());

  const slots = Array.from({ length: 48 }, (_, i) => i); // 30-min slots

  // 🔥 Month helpers
  const year = currentDate.getFullYear();
  const month = currentDate.getMonth();

  const daysInMonth = new Date(year, month + 1, 0).getDate();
  const firstDay = new Date(year, month, 1).getDay();

  return (
    <div className="scheduler">

      {/* SIDEBAR */}
      <div className="sidebar">
        <h3>
          {currentDate.toLocaleDateString("en-US", {
            month: "long",
            year: "numeric",
          })}
        </h3>

        <div className="mini-calendar">
          {[...Array(daysInMonth)].map((_, i) => (
            <div key={i} className="mini-day">{i + 1}</div>
          ))}
        </div>

      </div>

      {/* MAIN */}
      <div className="calendar">

        {/* HEADER */}
        <div className="calendar-header">
          <button onClick={() => setView("day")}>Day</button>
          <button onClick={() => setView("week")}>Week</button>
          <button onClick={() => setView("month")}>Month</button>
        </div>

        {/* ================= MONTH ================= */}
        {view === "month" && (
          <div className="month-grid">

            {/* empty cells */}
            {[...Array(firstDay)].map((_, i) => (
              <div key={"empty" + i} className="day-cell empty" />
            ))}

            {/* days */}
            {[...Array(daysInMonth)].map((_, i) => (
              <div key={i} className="day-cell">
                <div className="date">{i + 1}</div>
              </div>
            ))}

          </div>
        )}

        {/* ================= WEEK ================= */}
        {view === "week" && (
          <div className="week-container">

            {/* TIME COLUMN */}
            <div className="time-column">
              {slots.map((slot) => {
                const hour = Math.floor(slot / 2);
                const min = slot % 2 === 0 ? "00" : "30";

                return (
                  <div key={slot} className="time-cell">
                    {min === "00" ? `${hour}:00` : ""}
                  </div>
                );
              })}
            </div>

            {/* DAYS */}
            <div className="week-grid">
              {[...Array(7)].map((_, i) => {
                const day = new Date(currentDate);
                day.setDate(currentDate.getDate() - currentDate.getDay() + i);

                return (
                  <div key={i} className="week-day-column">

                    <div className="week-day-header">
                      {day.toLocaleDateString("en-US", {
                        weekday: "short",
                        day: "numeric",
                      })}
                    </div>

                    <div className="day-cells">
                      {slots.map((slot) => (
                        <div key={slot} className="day-cell-slot" />
                      ))}
                    </div>

                  </div>
                );
              })}
            </div>

          </div>
        )}

        {/* ================= DAY ================= */}
        {view === "day" && (
          <div className="week-container">

            <div className="time-column">
              {slots.map((slot) => {
                const hour = Math.floor(slot / 2);
                const min = slot % 2 === 0 ? "00" : "30";

                return (
                  <div key={slot} className="time-cell">
                    {min === "00" ? `${hour}:00` : ""}
                  </div>
                );
              })}
            </div>

            <div className="week-grid">
              <div className="week-day-column">

                <div className="week-day-header">
                  {currentDate.toLocaleDateString("en-US", {
                    weekday: "long",
                    day: "numeric",
                  })}
                </div>

                <div className="day-cells">
                  {slots.map((slot) => (
                    <div key={slot} className="day-cell-slot" />
                  ))}
                </div>

              </div>
            </div>

          </div>
        )}

      </div>
    </div>
  );
}