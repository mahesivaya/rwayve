import { logger } from "../utils/logger";
import { useEffect, useState } from "react";
import "./scheduler.css";
import { getMeetings, createMeetingApi, updateMeetingApi, deleteMeetingApi } from "./SchedulerService";


export default function Scheduler() {

  // Format a Date as YYYY-MM-DD in the user's local timezone. Calendars
  // should always show events in the viewer's wall clock — using a hardcoded
  // zone here desyncs the displayed date from the date sent on create, which
  // caused the "Meeting cannot be in the past" 400.
  const formatDateLocal = (date: Date) => {
    const y = date.getFullYear();
    const m = (date.getMonth() + 1).toString().padStart(2, "0");
    const d = date.getDate().toString().padStart(2, "0");
    return `${y}-${m}-${d}`;
  };
  
  const [view, setView] = useState<"day" | "week" | "month">("month");
  const [currentDate, setCurrentDate] = useState(new Date());
  const [events, setEvents] = useState<any[]>([]);

  // modal
  const [showModal, setShowModal] = useState(false);
  const [editingEvent, setEditingEvent] = useState<any | null>(null);

  const [title, setTitle] = useState("");
  const [start, setStart] = useState("09:00");
  const [end, setEnd] = useState("10:00");
  const [selectedDate, setSelectedDate] = useState(
    formatDateLocal(currentDate)
  );

  const [participants, setParticipants] = useState<string[]>([]);
  const [emailInput, setEmailInput] = useState("");

  const deleteMeeting = async () => {
    if (!editingEvent) return;
  
    const confirmDelete = confirm("Delete this meeting?");
    if (!confirmDelete) return;
  
    try {
      await deleteMeetingApi(editingEvent.id);
  
      setShowModal(false);
      setEditingEvent(null);
      fetchMeetings();
    } catch (err) {
      logger.log("❌ Delete error", err);
    }
  };

  const slots = Array.from({ length: 48 }, (_, i) => i);

  // ================= HELPERS =================
  const toMinutes = (time: string) => {
    const [h, m] = time.split(":").map(Number);
    return h * 60 + m;
  };

  const fromTime = (time: string) => {
    const [h, m] = time.split(":").map(Number);
    return h * 60 + m;
  };

  const toTime = (mins: number) => {
    const h = Math.floor(mins / 60);
    const m = mins % 60;
    return `${h.toString().padStart(2, "0")}:${m
      .toString()
      .padStart(2, "0")}`;
  };

  const todayStr = () => formatDateLocal(new Date());
  const nowTimeStr = () => {
    const d = new Date();
    return `${d.getHours().toString().padStart(2, "0")}:${d
      .getMinutes()
      .toString()
      .padStart(2, "0")}`;
  };
  const addMinutesToTime = (t: string, n: number) =>
    toTime(Math.min(toMinutes(t) + n, 23 * 60 + 59));

  // ================= PARTICIPANTS =================
  const addParticipant = () => {
    const email = emailInput.trim().toLowerCase();
  
    if (!email) return;
  
    // better validation
    if (!email.includes("@") || !email.includes(".")) {
      alert("Enter a valid email");
      return;
    }
  
    if (!participants.includes(email)) {
      setParticipants([...participants, email]);
    }
  
    setEmailInput("");
  };

  const removeParticipant = (email: string) => {
    setParticipants(participants.filter((p) => p !== email));
  };

  // ================= FETCH =================
  const fetchMeetings = async () => {
    const data = await getMeetings();

    const formatted = data.map((m: any) => ({
      id: m.id,
      title: m.title,
      date: m.date,
      start: fromTime(m.start_time),
      end: fromTime(m.end_time),
      participants: m.participants ?? [],
    }));

    setEvents(formatted);
  };

  useEffect(() => {
    fetchMeetings();
  }, []);

  // ================= CREATE / UPDATE =================
  const saveMeeting = async () => {
    const startMins = toMinutes(start);
    const endMins = toMinutes(end);

    if (!editingEvent) {
      const nowMins = toMinutes(nowTimeStr());
      if (
        selectedDate < todayStr() ||
        (selectedDate === todayStr() && startMins <= nowMins)
      ) {
        alert("Cannot create a meeting in the past");
        return;
      }
    }
    if (endMins <= startMins) {
      alert("End time must be after start time");
      return;
    }

    let finalParticipants = [...participants];

    // auto-add typed email if not added
    const email = emailInput.trim().toLowerCase();
    if (email && email.includes("@") && email.includes(".")) {
      if (!finalParticipants.includes(email)) {
        finalParticipants.push(email);
      }
    }

    logger.log("🚀 sending participants:", finalParticipants);

    const payload = {
      title,
      date: selectedDate,
      start: startMins,
      end: endMins,
      participants: finalParticipants,
    };

    if (editingEvent) {
      await updateMeetingApi(editingEvent.id, payload);
    } else {
      await createMeetingApi(payload);
    }

    resetModal();
    fetchMeetings();
  };

  const resetModal = () => {
    setShowModal(false);
    setEditingEvent(null);
    setTitle("");
    setParticipants([]);
  };

  // ================= EDIT =================
  const openEdit = (event: any) => {
    setEditingEvent(event);

    setTitle(event.title);
    setSelectedDate(event.date);
    setStart(toTime(event.start));
    setEnd(toTime(event.end));
    setParticipants(event.participants ?? []);
    setEmailInput("");

    setShowModal(true);
  };

  const openCreate = (date?: string, startTime?: string) => {
    setEditingEvent(null);
    setTitle("");
    setParticipants([]);
    setEmailInput("");
    const baseStart = startTime ?? addMinutesToTime(nowTimeStr(), 0);
    setSelectedDate(date ?? todayStr());
    setStart(baseStart);
    setEnd(addMinutesToTime(baseStart, startTime ? 30 : 60));
    setShowModal(true);
  };

  const openDay = (date: Date) => {
    setCurrentDate(date);
    setView("day");
  };

  // ================= MINI CALENDAR =================
  const year = currentDate.getFullYear();
  const month = currentDate.getMonth();
  const daysInMonth = new Date(year, month + 1, 0).getDate();

  const changeMonth = (offset: number) => {
    const newDate = new Date(currentDate);
    newDate.setMonth(newDate.getMonth() + offset);
    setCurrentDate(newDate);
  };

  return (
    <div className="scheduler">

      {/* SIDEBAR */}
      <div className="sidebar">

        <div className="mini-header">
          <button onClick={() => changeMonth(-1)}>◀</button>
          <span>
            {currentDate.toLocaleDateString("en-US", {
              month: "long",
              year: "numeric",
            })}
          </span>
          <button onClick={() => changeMonth(1)}>▶</button>
        </div>

        <div className="mini-calendar">
          {[...Array(daysInMonth)].map((_, i) => {
            const day = i + 1;
            const d = new Date(year, month, day);

            const isActive =
              d.toDateString() === currentDate.toDateString();

            return (
              <div
                key={i}
                className={`mini-day ${isActive ? "active" : ""}`}
                onClick={() => {
                  setCurrentDate(d);
                  setView("day");
                }}
              >
                {day}
              </div>
            );
          })}
        </div>

      </div>





      {/* MAIN */}
      <div className="calendar">

        <div className="calendar-header">
          <button onClick={() => setView("day")}>Day</button>
          <button onClick={() => setView("week")}>Week</button>
          <button onClick={() => setView("month")}>Month</button>

          <button className="create-btn" onClick={() => openCreate()}>
            ➕ Schedule
          </button>
        </div>


{/* DAY VIEW */}
{view === "day" && (
  <div className="day-view">

    <h3 className="day-title">
      {currentDate.toDateString()}
    </h3>

    <div className="day-slots">
      {slots.map((slot) => {
        const mins = slot * 30;

        const timeLabel = `${Math.floor(mins / 60)
          .toString()
          .padStart(2, "0")}:${(mins % 60)
          .toString()
          .padStart(2, "0")}`;

        const dayDate = formatDateLocal(currentDate);

        const slotEvents = events.filter(
          (e) =>
            e.date === dayDate &&
            e.start >= mins &&
            e.start < mins + 30
        );

        return (
          <div
            key={slot}
            className="time-row"
            onClick={() => openCreate(dayDate, timeLabel)}
          >
            <div className="time-label">{timeLabel}</div>

            <div className="time-events">
              {slotEvents.map((e) => (
                <div
                  key={e.id}
                  className="event"
                  onClick={(ev) => {
                    ev.stopPropagation();
                    openEdit(e);
                  }}
                >
                  {e.title}
                </div>
              ))}
            </div>
          </div>
        );
      })}
    </div>

  </div>
)}




{/* WEEK VIEW */}
{view === "week" && (
  <div className="week-view">

    <div className="week-header">
      <div className="time-spacer">Time</div>
      {[...Array(7)].map((_, i) => {
        const d = new Date(currentDate);
        d.setDate(currentDate.getDate() - currentDate.getDay() + i);
        const isToday = formatDateLocal(d) === todayStr();

        return (
          <div
            key={i}
            className={`week-day-header${isToday ? " is-today" : ""}`}
          >
            <span className="week-day-name">
              {d.toLocaleDateString("en-US", { weekday: "short" })}
            </span>
            <span className="week-day-num">{d.getDate()}</span>
          </div>
        );
      })}
    </div>

    <div className="week-grid">
      {slots.map((slot) => {
        const mins = slot * 30;

        return (
          <div key={slot} className="week-row">

            <div className="time-label">
              {Math.floor(mins / 60)
                .toString()
                .padStart(2, "0")}
              :
              {(mins % 60).toString().padStart(2, "0")}
            </div>

            {[...Array(7)].map((_, i) => {
              const d = new Date(currentDate);
              d.setDate(currentDate.getDate() - currentDate.getDay() + i);

              const dayDate = formatDateLocal(d);
              const slotTime = `${Math.floor(mins / 60)
                .toString()
                .padStart(2, "0")}:${(mins % 60)
                .toString()
                .padStart(2, "0")}`;

              const slotEvents = events.filter(
                (e) =>
                  e.date === dayDate &&
                  e.start >= mins &&
                  e.start < mins + 30
              );

              return (
                <div
                  key={i}
                  className="week-cell"
                  onClick={() => openCreate(dayDate, slotTime)}
                >
                  {slotEvents.map((e) => (
                    <div
                      key={e.id}
                      className="event"
                      onClick={(ev) => {
                        ev.stopPropagation();
                        openEdit(e);
                      }}
                    >
                      {e.title}
                    </div>
                  ))}
                </div>
              );
            })}
          </div>
        );
      })}
    </div>

  </div>
)}
        

        {/* MONTH VIEW */}
        {view === "month" && (
          <div className="month-view">
            <div className="month-weekday-row">
              {["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"].map((d) => (
                <div key={d} className="month-weekday">{d}</div>
              ))}
            </div>
            <div className="month-grid">
              {[...Array(new Date(year, month, 1).getDay())].map((_, i) => (
                <div key={`pad-${i}`} className="day-cell empty" />
              ))}
              {[...Array(daysInMonth)].map((_, i) => {
                const cellDate = new Date(year, month, i + 1);
                const dayDate = formatDateLocal(cellDate);
                const isToday = dayDate === todayStr();
                return (
                  <div
                    key={i}
                    className={`day-cell${isToday ? " is-today" : ""}`}
                    onClick={() => openDay(cellDate)}
                  >
                    <div className="date">{i + 1}</div>
                    <div className="events">
                      {events
                        .filter((e) => e.date === dayDate)
                        .sort((a, b) => a.start - b.start)
                        .map((e) => (
                          <div
                            key={e.id}
                            className="event"
                            onClick={(ev) => {
                              ev.stopPropagation();
                              openEdit(e);
                            }}
                          >
                            <span className="event-time">
                              {Math.floor(e.start / 60)}:
                              {(e.start % 60).toString().padStart(2, "0")}
                            </span>
                            <span className="event-title">{e.title}</span>
                          </div>
                        ))}
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        )}

      </div>



      {/* MODAL */}
      {showModal && (
        <div className="modal-overlay">
          <div className="modal">

            <button
              className="close-btn"
              onClick={() => setShowModal(false)}
            >
              ✕
            </button>

            <h3>
              {editingEvent ? "Edit Meeting" : "Schedule Meeting"}
            </h3>

            <div className="form">

              <div className="form-group">
                <label>Date</label>
                <input
                  type="date"
                  value={selectedDate}
                  min={editingEvent ? undefined : todayStr()}
                  onChange={(e) => {
                    const v = e.target.value;
                    if (!editingEvent && v && v < todayStr()) {
                      setSelectedDate(todayStr());
                    } else {
                      setSelectedDate(v);
                    }
                  }}
                />
              </div>

              <div className="form-group">
                <label>Title</label>
                <input
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                />
              </div>

              <div className="form-group">
                <label>Participants</label>

                <div className="chips">
                  {participants.map((p) => (
                    <div key={p} className="chip">
                      {p}
                      <span onClick={() => removeParticipant(p)}>×</span>
                    </div>
                  ))}
                </div>

                <div className="participant-input">
                  <input
                    type="email"
                    value={emailInput}
                    onChange={(e) => setEmailInput(e.target.value)}
                  />
                  <button onClick={addParticipant}>Add</button>
                </div>
              </div>

              <div className="form-group">
                <label>Start</label>
                <input
                  type="time"
                  value={start}
                  min={
                    !editingEvent && selectedDate === todayStr()
                      ? nowTimeStr()
                      : undefined
                  }
                  onChange={(e) => {
                    let v = e.target.value;
                    if (!editingEvent && selectedDate === todayStr() && v < nowTimeStr()) {
                      v = nowTimeStr();
                    }
                    setStart(v);
                    if (toMinutes(end) <= toMinutes(v)) {
                      setEnd(addMinutesToTime(v, 30));
                    }
                  }}
                />
              </div>

              <div className="form-group">
                <label>End</label>
                <input
                  type="time"
                  value={end}
                  min={addMinutesToTime(start, 1)}
                  onChange={(e) => {
                    const v = e.target.value;
                    if (toMinutes(v) <= toMinutes(start)) {
                      setEnd(addMinutesToTime(start, 30));
                    } else {
                      setEnd(v);
                    }
                  }}
                />
              </div>

            </div>

            <div className="modal-actions">
                {editingEvent && (
              <button className="delete-btn" onClick={deleteMeeting}>
                Delete
              </button>
                )}
              <button onClick={saveMeeting}>
                {editingEvent ? "Update" : "Save"}
              </button>
              <button onClick={() => setShowModal(false)}>
                Cancel
              </button>
            </div>

          </div>
        </div>
      )}
    </div>
  );
}