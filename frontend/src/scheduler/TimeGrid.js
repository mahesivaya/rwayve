import { jsxs as _jsxs, jsx as _jsx } from "react/jsx-runtime";
import { useState } from "react";
export default function TimeGrid({ events, onCreateEvent }) {
    const [dragStart, setDragStart] = useState(null);
    const hours = Array.from({ length: 24 }, (_, i) => i);
    const getMinutesFromEvent = (e, hour) => {
        const rect = e.currentTarget.getBoundingClientRect();
        const y = e.clientY - rect.top;
        const minutes = Math.floor((y / rect.height) * 60);
        return hour * 60 + minutes;
    };
    const handleMouseDown = (e, hour) => {
        setDragStart(getMinutesFromEvent(e, hour));
    };
    const handleMouseUp = (e, hour) => {
        if (dragStart === null)
            return;
        const end = getMinutesFromEvent(e, hour);
        const newEvent = {
            id: Date.now(),
            start: Math.min(dragStart, end),
            end: Math.max(dragStart, end),
            title: "New Event",
        };
        onCreateEvent(newEvent);
        setDragStart(null);
    };
    return (_jsx("div", { className: "time-grid", children: hours.map((hour) => (_jsxs("div", { className: "hour-row", children: [_jsxs("div", { className: "time-label", children: [hour, ":00"] }), _jsx("div", { className: "hour-cell", onMouseDown: (e) => handleMouseDown(e, hour), onMouseUp: (e) => handleMouseUp(e, hour), children: events
                        .filter((ev) => Math.floor(ev.start / 60) === hour)
                        .map((ev) => (_jsx("div", { className: "event-block", style: {
                            top: (ev.start % 60),
                            height: ev.end - ev.start,
                        }, children: ev.title }, ev.id))) })] }, hour))) }));
}
