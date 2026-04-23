import { jsx as _jsx } from "react/jsx-runtime";
export default function EventBlock({ event }) {
    const top = (event.start % 60) * 2; // pixels per minute
    const height = (event.end - event.start) * 2;
    return (_jsx("div", { className: "event-block", style: {
            position: "absolute", // ✅ REQUIRED
            top: `${top}px`,
            height: `${height}px`,
            left: "2px",
            right: "2px",
        }, children: event.title }));
}
