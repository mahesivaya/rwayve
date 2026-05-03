import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { logger } from "../utils/logger";
import { useEffect, useState } from "react";
import "./scheduler.css";
import { getMeetings, createMeetingApi, updateMeetingApi, deleteMeetingApi } from "./SchedulerService";
export default function Scheduler() {
    // Format a Date as YYYY-MM-DD in the user's local timezone. Calendars
    // should always show events in the viewer's wall clock — using a hardcoded
    // zone here desyncs the displayed date from the date sent on create, which
    // caused the "Meeting cannot be in the past" 400.
    const formatDateLocal = (date) => {
        const y = date.getFullYear();
        const m = (date.getMonth() + 1).toString().padStart(2, "0");
        const d = date.getDate().toString().padStart(2, "0");
        return `${y}-${m}-${d}`;
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
    const [selectedDate, setSelectedDate] = useState(formatDateLocal(currentDate));
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
    const todayStr = () => formatDateLocal(new Date());
    const nowTimeStr = () => {
        const d = new Date();
        return `${d.getHours().toString().padStart(2, "0")}:${d
            .getMinutes()
            .toString()
            .padStart(2, "0")}`;
    };
    const addMinutesToTime = (t, n) => toTime(Math.min(toMinutes(t) + n, 23 * 60 + 59));
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
            participants: m.participants ?? [],
            zoom_join_url: m.zoom_join_url ?? null,
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
            if (selectedDate < todayStr() ||
                (selectedDate === todayStr() && startMins <= nowMins)) {
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
        setParticipants(event.participants ?? []);
        setEmailInput("");
        setShowModal(true);
    };
    const openCreate = (date, startTime) => {
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
    const openDay = (date) => {
        setCurrentDate(date);
        setView("day");
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
                        }) })] }), _jsxs("div", { className: "calendar", children: [_jsxs("div", { className: "calendar-header", children: [_jsx("button", { onClick: () => setView("day"), children: "Day" }), _jsx("button", { onClick: () => setView("week"), children: "Week" }), _jsx("button", { onClick: () => setView("month"), children: "Month" }), _jsx("button", { className: "create-btn", onClick: () => openCreate(), children: "\u2795 Schedule" })] }), view === "day" && (_jsxs("div", { className: "day-view", children: [_jsx("h3", { className: "day-title", children: currentDate.toDateString() }), _jsx("div", { className: "day-slots", children: slots.map((slot) => {
                                    const mins = slot * 30;
                                    const timeLabel = `${Math.floor(mins / 60)
                                        .toString()
                                        .padStart(2, "0")}:${(mins % 60)
                                        .toString()
                                        .padStart(2, "0")}`;
                                    const dayDate = formatDateLocal(currentDate);
                                    const slotEvents = events.filter((e) => e.date === dayDate &&
                                        e.start >= mins &&
                                        e.start < mins + 30);
                                    return (_jsxs("div", { className: "time-row", onClick: () => openCreate(dayDate, timeLabel), children: [_jsx("div", { className: "time-label", children: timeLabel }), _jsx("div", { className: "time-events", children: slotEvents.map((e) => (_jsx("div", { className: "event", onClick: (ev) => {
                                                        ev.stopPropagation();
                                                        openEdit(e);
                                                    }, children: e.title }, e.id))) })] }, slot));
                                }) })] })), view === "week" && (_jsxs("div", { className: "week-view", children: [_jsxs("div", { className: "week-header", children: [_jsx("div", { className: "time-spacer", children: "Time" }), [...Array(7)].map((_, i) => {
                                        const d = new Date(currentDate);
                                        d.setDate(currentDate.getDate() - currentDate.getDay() + i);
                                        const isToday = formatDateLocal(d) === todayStr();
                                        return (_jsxs("div", { className: `week-day-header${isToday ? " is-today" : ""}`, children: [_jsx("span", { className: "week-day-name", children: d.toLocaleDateString("en-US", { weekday: "short" }) }), _jsx("span", { className: "week-day-num", children: d.getDate() })] }, i));
                                    })] }), _jsx("div", { className: "week-grid", children: slots.map((slot) => {
                                    const mins = slot * 30;
                                    return (_jsxs("div", { className: "week-row", children: [_jsxs("div", { className: "time-label", children: [Math.floor(mins / 60)
                                                        .toString()
                                                        .padStart(2, "0"), ":", (mins % 60).toString().padStart(2, "0")] }), [...Array(7)].map((_, i) => {
                                                const d = new Date(currentDate);
                                                d.setDate(currentDate.getDate() - currentDate.getDay() + i);
                                                const dayDate = formatDateLocal(d);
                                                const slotTime = `${Math.floor(mins / 60)
                                                    .toString()
                                                    .padStart(2, "0")}:${(mins % 60)
                                                    .toString()
                                                    .padStart(2, "0")}`;
                                                const slotEvents = events.filter((e) => e.date === dayDate &&
                                                    e.start >= mins &&
                                                    e.start < mins + 30);
                                                return (_jsx("div", { className: "week-cell", onClick: () => openCreate(dayDate, slotTime), children: slotEvents.map((e) => (_jsx("div", { className: "event", onClick: (ev) => {
                                                            ev.stopPropagation();
                                                            openEdit(e);
                                                        }, children: e.title }, e.id))) }, i));
                                            })] }, slot));
                                }) })] })), view === "month" && (_jsxs("div", { className: "month-view", children: [_jsx("div", { className: "month-weekday-row", children: ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"].map((d) => (_jsx("div", { className: "month-weekday", children: d }, d))) }), _jsxs("div", { className: "month-grid", children: [[...Array(new Date(year, month, 1).getDay())].map((_, i) => (_jsx("div", { className: "day-cell empty" }, `pad-${i}`))), [...Array(daysInMonth)].map((_, i) => {
                                        const cellDate = new Date(year, month, i + 1);
                                        const dayDate = formatDateLocal(cellDate);
                                        const isToday = dayDate === todayStr();
                                        return (_jsxs("div", { className: `day-cell${isToday ? " is-today" : ""}`, onClick: () => openDay(cellDate), children: [_jsx("div", { className: "date", children: i + 1 }), _jsx("div", { className: "events", children: events
                                                        .filter((e) => e.date === dayDate)
                                                        .sort((a, b) => a.start - b.start)
                                                        .map((e) => (_jsxs("div", { className: "event", onClick: (ev) => {
                                                            ev.stopPropagation();
                                                            openEdit(e);
                                                        }, children: [_jsxs("span", { className: "event-time", children: [Math.floor(e.start / 60), ":", (e.start % 60).toString().padStart(2, "0")] }), _jsx("span", { className: "event-title", children: e.title })] }, e.id))) })] }, i));
                                    })] })] }))] }), showModal && (_jsx("div", { className: "modal-overlay", children: _jsxs("div", { className: "modal", children: [_jsx("button", { className: "close-btn", onClick: () => setShowModal(false), children: "\u2715" }), _jsx("h3", { children: editingEvent ? "Edit Meeting" : "Schedule Meeting" }), _jsxs("div", { className: "form", children: [_jsxs("div", { className: "form-group", children: [_jsx("label", { children: "Date" }), _jsx("input", { type: "date", value: selectedDate, min: editingEvent ? undefined : todayStr(), onChange: (e) => {
                                                const v = e.target.value;
                                                if (!editingEvent && v && v < todayStr()) {
                                                    setSelectedDate(todayStr());
                                                }
                                                else {
                                                    setSelectedDate(v);
                                                }
                                            } })] }), _jsxs("div", { className: "form-group", children: [_jsx("label", { children: "Title" }), _jsx("input", { value: title, onChange: (e) => setTitle(e.target.value) })] }), _jsxs("div", { className: "form-group", children: [_jsx("label", { children: "Participants" }), _jsx("div", { className: "chips", children: participants.map((p) => (_jsxs("div", { className: "chip", children: [p, _jsx("span", { onClick: () => removeParticipant(p), children: "\u00D7" })] }, p))) }), _jsxs("div", { className: "participant-input", children: [_jsx("input", { type: "email", value: emailInput, onChange: (e) => setEmailInput(e.target.value) }), _jsx("button", { onClick: addParticipant, children: "Add" })] })] }), _jsxs("div", { className: "form-group", children: [_jsx("label", { children: "Start" }), _jsx("input", { type: "time", value: start, min: !editingEvent && selectedDate === todayStr()
                                                ? nowTimeStr()
                                                : undefined, onChange: (e) => {
                                                let v = e.target.value;
                                                if (!editingEvent && selectedDate === todayStr() && v < nowTimeStr()) {
                                                    v = nowTimeStr();
                                                }
                                                setStart(v);
                                                if (toMinutes(end) <= toMinutes(v)) {
                                                    setEnd(addMinutesToTime(v, 30));
                                                }
                                            } })] }), _jsxs("div", { className: "form-group", children: [_jsx("label", { children: "End" }), _jsx("input", { type: "time", value: end, min: addMinutesToTime(start, 1), onChange: (e) => {
                                                const v = e.target.value;
                                                if (toMinutes(v) <= toMinutes(start)) {
                                                    setEnd(addMinutesToTime(start, 30));
                                                }
                                                else {
                                                    setEnd(v);
                                                }
                                            } })] }), editingEvent?.zoom_join_url && (_jsxs("div", { className: "form-group", children: [_jsx("label", { children: "Zoom link" }), _jsx("a", { href: editingEvent.zoom_join_url, target: "_blank", rel: "noopener noreferrer", children: editingEvent.zoom_join_url })] }))] }), _jsxs("div", { className: "modal-actions", children: [editingEvent && (_jsx("button", { className: "delete-btn", onClick: deleteMeeting, children: "Delete" })), _jsx("button", { onClick: saveMeeting, children: editingEvent ? "Update" : "Save" }), _jsx("button", { onClick: () => setShowModal(false), children: "Cancel" })] })] }) }))] }));
}
