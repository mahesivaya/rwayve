import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { logger } from "../utils/logger";
import { useEffect, useState } from "react";
import "./scheduler.css";
import { getMeetings, createMeetingApi, updateMeetingApi, deleteMeetingApi } from "./SchedulerService";
export default function Scheduler() {
    const formatDateEST = (date) => {
        return new Intl.DateTimeFormat("en-CA", {
            timeZone: "America/New_York",
            year: "numeric",
            month: "2-digit",
            day: "2-digit",
        }).format(date);
    };
    const [view, setView] = useState("month");
    const [currentDate, setCurrentDate] = useState(new Date());
    const [events, setEvents] = useState([]);
    // modal
    const [showModal, setShowModal] = useState(false);
    const [editingEvent, setEditingEvent] = useState(null);
    const [title, setTitle] = useState("");
    const [start, setStart] = useState("09:00");
    const [end, setEnd] = useState("10:00");
    const [selectedDate, setSelectedDate] = useState(currentDate.toISOString().split("T")[0]);
    const [participants, setParticipants] = useState([]);
    const [emailInput, setEmailInput] = useState("");
    const deleteMeeting = async () => {
        if (!editingEvent)
            return;
        const confirmDelete = confirm("Delete this meeting?");
        if (!confirmDelete)
            return;
        try {
            await deleteMeetingApi(editingEvent.id);
            setShowModal(false);
            setEditingEvent(null);
            fetchMeetings();
        }
        catch (err) {
            logger.log("❌ Delete error", err);
        }
    };
    const slots = Array.from({ length: 48 }, (_, i) => i);
    // ================= HELPERS =================
    const toMinutes = (time) => {
        const [h, m] = time.split(":").map(Number);
        return h * 60 + m;
    };
    const fromTime = (time) => {
        const [h, m] = time.split(":").map(Number);
        return h * 60 + m;
    };
    const toTime = (mins) => {
        const h = Math.floor(mins / 60);
        const m = mins % 60;
        return `${h.toString().padStart(2, "0")}:${m
            .toString()
            .padStart(2, "0")}`;
    };
    // ================= PARTICIPANTS =================
    const addParticipant = () => {
        const email = emailInput.trim().toLowerCase();
        if (!email)
            return;
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
    const removeParticipant = (email) => {
        setParticipants(participants.filter((p) => p !== email));
    };
    // ================= FETCH =================
    const fetchMeetings = async () => {
        const data = await getMeetings();
        const formatted = data.map((m) => ({
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
        logger.log("🚀 sending participants:", finalParticipants);
        const payload = {
            title,
            date: selectedDate,
            start: toMinutes(start),
            end: toMinutes(end),
            participants: finalParticipants,
        };
        if (editingEvent) {
            await updateMeetingApi(editingEvent.id, payload);
        }
        else {
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
    const openEdit = (event) => {
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
    const changeMonth = (offset) => {
        const newDate = new Date(currentDate);
        newDate.setMonth(newDate.getMonth() + offset);
        setCurrentDate(newDate);
    };
    return (_jsxs("div", { className: "scheduler", children: [_jsxs("div", { className: "sidebar", children: [_jsxs("div", { className: "mini-header", children: [_jsx("button", { onClick: () => changeMonth(-1), children: "\u25C0" }), _jsx("span", { children: currentDate.toLocaleDateString("en-US", {
                                    month: "long",
                                    year: "numeric",
                                }) }), _jsx("button", { onClick: () => changeMonth(1), children: "\u25B6" })] }), _jsx("div", { className: "mini-calendar", children: [...Array(daysInMonth)].map((_, i) => {
                            const day = i + 1;
                            const d = new Date(year, month, day);
                            const isActive = d.toDateString() === currentDate.toDateString();
                            return (_jsx("div", { className: `mini-day ${isActive ? "active" : ""}`, onClick: () => {
                                    setCurrentDate(d);
                                    setView("day");
                                }, children: day }, i));
                        }) })] }), _jsxs("div", { className: "calendar", children: [_jsxs("div", { className: "calendar-header", children: [_jsx("button", { onClick: () => setView("day"), children: "Day" }), _jsx("button", { onClick: () => setView("week"), children: "Week" }), _jsx("button", { onClick: () => setView("month"), children: "Month" }), _jsx("button", { className: "create-btn", onClick: openCreate, children: "\u2795 Schedule" })] }), view === "day" && (_jsxs("div", { className: "day-view", children: [_jsx("h3", { className: "day-title", children: currentDate.toDateString() }), _jsx("div", { className: "day-slots", children: slots.map((slot) => {
                                    const mins = slot * 30;
                                    const timeLabel = `${Math.floor(mins / 60)
                                        .toString()
                                        .padStart(2, "0")}:${(mins % 60)
                                        .toString()
                                        .padStart(2, "0")}`;
                                    const dayDate = currentDate.toISOString().split("T")[0];
                                    const slotEvents = events.filter((e) => e.date === dayDate &&
                                        e.start >= mins &&
                                        e.start < mins + 30);
                                    return (_jsxs("div", { className: "time-row", children: [_jsx("div", { className: "time-label", children: timeLabel }), _jsx("div", { className: "time-events", children: slotEvents.map((e) => (_jsx("div", { className: "event", onClick: () => openEdit(e), children: e.title }, e.id))) })] }, slot));
                                }) })] })), view === "week" && (_jsxs("div", { className: "week-view", children: [_jsx("div", { className: "week-header", children: [...Array(7)].map((_, i) => {
                                    const d = new Date(currentDate);
                                    d.setDate(currentDate.getDate() - currentDate.getDay() + i);
                                    return (_jsx("div", { className: "week-day-header", children: d.toLocaleDateString("en-US", {
                                            weekday: "short",
                                            day: "numeric",
                                        }) }, i));
                                }) }), _jsx("div", { className: "week-grid", children: slots.map((slot) => {
                                    const mins = slot * 30;
                                    return (_jsxs("div", { className: "week-row", children: [_jsxs("div", { className: "time-label", children: [Math.floor(mins / 60)
                                                        .toString()
                                                        .padStart(2, "0"), ":", (mins % 60).toString().padStart(2, "0")] }), [...Array(7)].map((_, i) => {
                                                const d = new Date(currentDate);
                                                d.setDate(currentDate.getDate() - currentDate.getDay() + i);
                                                const dayDate = d.toISOString().split("T")[0];
                                                const slotEvents = events.filter((e) => e.date === dayDate &&
                                                    e.start >= mins &&
                                                    e.start < mins + 30);
                                                return (_jsx("div", { className: "week-cell", children: slotEvents.map((e) => (_jsx("div", { className: "event", onClick: () => openEdit(e), children: e.title }, e.id))) }, i));
                                            })] }, slot));
                                }) })] })), view === "month" && (_jsx("div", { className: "month-grid", children: [...Array(daysInMonth)].map((_, i) => {
                            const dayDate = new Date(year, month, i + 1)
                                .toISOString()
                                .split("T")[0];
                            return (_jsxs("div", { className: "day-cell", children: [_jsx("div", { className: "date", children: i + 1 }), _jsx("div", { className: "events", children: events
                                            .filter((e) => e.date === dayDate)
                                            .sort((a, b) => a.start - b.start) // ✅ sort by time
                                            .map((e) => (_jsxs("div", { className: "event", onClick: () => openEdit(e), children: [_jsxs("span", { className: "event-time", children: [Math.floor(e.start / 60), ":", (e.start % 60).toString().padStart(2, "0")] }), _jsx("span", { className: "event-title", children: e.title })] }, e.id))) })] }, i));
                        }) }))] }), showModal && (_jsx("div", { className: "modal-overlay", children: _jsxs("div", { className: "modal", children: [_jsx("button", { className: "close-btn", onClick: () => setShowModal(false), children: "\u2715" }), _jsx("h3", { children: editingEvent ? "Edit Meeting" : "Schedule Meeting" }), _jsxs("div", { className: "form", children: [_jsxs("div", { className: "form-group", children: [_jsx("label", { children: "Date" }), _jsx("input", { type: "date", value: selectedDate, onChange: (e) => setSelectedDate(e.target.value) })] }), _jsxs("div", { className: "form-group", children: [_jsx("label", { children: "Title" }), _jsx("input", { value: title, onChange: (e) => setTitle(e.target.value) })] }), _jsxs("div", { className: "form-group", children: [_jsx("label", { children: "Participants" }), _jsx("div", { className: "chips", children: participants.map((p) => (_jsxs("div", { className: "chip", children: [p, _jsx("span", { onClick: () => removeParticipant(p), children: "\u00D7" })] }, p))) }), _jsxs("div", { className: "participant-input", children: [_jsx("input", { type: "email", value: emailInput, onChange: (e) => setEmailInput(e.target.value) }), _jsx("button", { onClick: addParticipant, children: "Add" })] })] }), _jsxs("div", { className: "form-group", children: [_jsx("label", { children: "Start" }), _jsx("input", { type: "time", value: start, onChange: (e) => setStart(e.target.value) })] }), _jsxs("div", { className: "form-group", children: [_jsx("label", { children: "End" }), _jsx("input", { type: "time", value: end, onChange: (e) => setEnd(e.target.value) })] })] }), _jsxs("div", { className: "modal-actions", children: [editingEvent && (_jsx("button", { className: "delete-btn", onClick: deleteMeeting, children: "Delete" })), _jsx("button", { onClick: saveMeeting, children: editingEvent ? "Update" : "Save" }), _jsx("button", { onClick: () => setShowModal(false), children: "Cancel" })] })] }) }))] }));
}
