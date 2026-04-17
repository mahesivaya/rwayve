import { useEffect, useState } from "react";
import "./scheduler.css";
import { getMeetings, createMeetingApi, updateMeetingApi, deleteMeetingApi } from "./SchedulerService";


export default function Scheduler() {

  const formatDateEST = (date: Date) => {
    return new Intl.DateTimeFormat("en-CA", {
      timeZone: "America/New_York",
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
    }).format(date);
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
    currentDate.toISOString().split("T")[0]
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
      console.log("❌ Delete error", err);
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
    }));

    setEvents(formatted);
  };

  useEffect(() => {
    fetchMeetings();
  }, []);

  // ================= CREATE / UPDATE =================
  const saveMeeting = async () => {
    let finalParticipants = [...participants];
  
    // auto-add typed email if not added
    const email = emailInput.trim().toLowerCase();
    if (email && email.includes("@") && email.includes(".")) {
      if (!finalParticipants.includes(email)) {
        finalParticipants.push(email);
      }
    }
  
    console.log("🚀 sending participants:", finalParticipants);
  
    const payload = {
      title,
      date: selectedDate,
      start: toMinutes(start),
      end: toMinutes(end),
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

    setShowModal(true);
  };

  const openCreate = () => {
    setEditingEvent(null);
    setTitle("");
    setParticipants([]);
    setShowModal(true);
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

          <button className="create-btn" onClick={openCreate}>
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

        const dayDate = currentDate.toISOString().split("T")[0];

        const slotEvents = events.filter(
          (e) =>
            e.date === dayDate &&
            e.start >= mins &&
            e.start < mins + 30
        );

        return (
          <div key={slot} className="time-row">
            <div className="time-label">{timeLabel}</div>

            <div className="time-events">
              {slotEvents.map((e) => (
                <div
                  key={e.id}
                  className="event"
                  onClick={() => openEdit(e)}
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
      {[...Array(7)].map((_, i) => {
        const d = new Date(currentDate);
        d.setDate(currentDate.getDate() - currentDate.getDay() + i);

        return (
          <div key={i} className="week-day-header">
            {d.toLocaleDateString("en-US", {
              weekday: "short",
              day: "numeric",
            })}
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

              const dayDate = d.toISOString().split("T")[0];

              const slotEvents = events.filter(
                (e) =>
                  e.date === dayDate &&
                  e.start >= mins &&
                  e.start < mins + 30
              );

              return (
                <div key={i} className="week-cell">
                  {slotEvents.map((e) => (
                    <div
                      key={e.id}
                      className="event"
                      onClick={() => openEdit(e)}
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
          <div className="month-grid">
            {[...Array(daysInMonth)].map((_, i) => {
              const dayDate = new Date(year, month, i + 1)
                .toISOString()
                .split("T")[0];

              return (
                <div key={i} className="day-cell">
                <div className="date">{i + 1}</div>

                <div className="events">
                  {events
                    .filter((e) => e.date === dayDate)
                    .sort((a, b) => a.start - b.start) // ✅ sort by time
                    .map((e) => (
                      <div
                        key={e.id}
                        className="event"
                        onClick={() => openEdit(e)}
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
                  onChange={(e) => setSelectedDate(e.target.value)}
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
                  onChange={(e) => setStart(e.target.value)}
                />
              </div>

              <div className="form-group">
                <label>End</label>
                <input
                  type="time"
                  value={end}
                  onChange={(e) => setEnd(e.target.value)}
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