import { useEffect, useState } from "react";
import "./scheduler.css";

export default function Scheduler() {
  const [view, setView] = useState<"day" | "week" | "month">("month");
  const [currentDate, setCurrentDate] = useState(new Date());
  const [events, setEvents] = useState<any[]>([]);

  // modal
  const [showModal, setShowModal] = useState(false);
  const [title, setTitle] = useState("");
  const [start, setStart] = useState("09:00");
  const [end, setEnd] = useState("10:00");
  const [selectedDate, setSelectedDate] = useState(
    currentDate.toISOString().split("T")[0]
  );

  const slots = Array.from({ length: 48 }, (_, i) => i);

  // 🔥 helpers
  const toMinutes = (time: string) => {
    const [h, m] = time.split(":").map(Number);
    return h * 60 + m;
  };

  const fromTime = (time: string) => {
    const [h, m] = time.split(":").map(Number);
    return h * 60 + m;
  };

  // 🔥 FETCH EVENTS
  const fetchMeetings = async () => {
    const token = localStorage.getItem("token");
  
    const res = await fetch("http://localhost:8080/api/meetings", {
      headers: {
        Authorization: `Bearer ${token}`,
      },
    });
  
    if (!res.ok) {
      console.log("❌ API error", res.status);
      return;
    }
  
    const data = await res.json();
  
    const formatted = data.map((m: any) => ({
      id: m.id,
      title: m.title,
      date: m.date,
      start: fromTime(m.start_time),
      end: fromTime(m.end_time),
    }));
  
    setEvents(formatted);
  };

  useEffect(() => {
    fetchMeetings();
  }, []);

  // 🔥 CREATE EVENT
  const createMeeting = async () => {
    const token = localStorage.getItem("token");
  
    await fetch("http://localhost:8080/api/meetings", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify({
        title,
        date: selectedDate,
        start: toMinutes(start),
        end: toMinutes(end),
      }),
    });
  
    setShowModal(false);
    setTitle("");
    fetchMeetings();
  };

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

          <button className="create-btn" onClick={() => setShowModal(true)}>
            ➕ Schedule
          </button>
        </div>

        {/* ================= MONTH ================= */}
        {view === "month" && (
          <div className="month-grid">

            {[...Array(firstDay)].map((_, i) => (
              <div key={"empty" + i} className="day-cell empty" />
            ))}

            {[...Array(daysInMonth)].map((_, i) => {
              const dayDate = new Date(year, month, i + 1)
                .toISOString()
                .split("T")[0];

              return (
                <div key={i} className="day-cell">
                  <div className="date">{i + 1}</div>

                  {events
                    .filter((e) => e.date === dayDate)
                    .map((e) => (
                      <div key={e.id} className="event blue">
                        {e.title}
                      </div>
                    ))}
                </div>
              );
            })}

          </div>
        )}

        {/* ================= WEEK ================= */}
        {view === "week" && (
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
              {[...Array(7)].map((_, i) => {
                const day = new Date(currentDate);
                day.setDate(currentDate.getDate() - currentDate.getDay() + i);

                const dayStr = day.toISOString().split("T")[0];

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

                      {/* EVENTS */}
                      {events
                        .filter((e) => e.date === dayStr)
                        .map((e) => {
                          const top = (e.start / 30) * 30;
                          const height = ((e.end - e.start) / 30) * 30;

                          return (
                            <div
                              key={e.id}
                              className="event-block"
                              style={{
                                top,
                                height,
                              }}
                            >
                              {e.title}
                            </div>
                          );
                        })}
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

                  {events
                    .filter(
                      (e) =>
                        e.date === currentDate.toISOString().split("T")[0]
                    )
                    .map((e) => {
                      const top = (e.start / 30) * 30;
                      const height = ((e.end - e.start) / 30) * 30;

                      return (
                        <div
                          key={e.id}
                          className="event-block"
                          style={{ top, height }}
                        >
                          {e.title}
                        </div>
                      );
                    })}
                </div>

              </div>
            </div>

          </div>
        )}

      </div>

      {/* ================= MODAL ================= */}
      {showModal && (
        <div className="modal-overlay">
          <div className="modal">
            <h3>Schedule Meeting</h3>
            <label>Date</label>
            
            <input
              type="date"
              value={selectedDate}
              onChange={(e) => setSelectedDate(e.target.value)}
            />

            <input
              placeholder="Title"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
            />

            <label>Start</label>
            <input
              type="time"
              value={start}
              onChange={(e) => setStart(e.target.value)}
            />

            <label>End</label>
            <input
              type="time"
              value={end}
              onChange={(e) => setEnd(e.target.value)}
            />

            <div className="modal-actions">
              <button onClick={createMeeting}>Save</button>
              <button onClick={() => setShowModal(false)}>Cancel</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}