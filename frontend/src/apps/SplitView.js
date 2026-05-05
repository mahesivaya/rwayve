import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { useEffect, useRef, useState } from "react";
import "./apps.css";
const MIN_PCT = 15;
const MAX_PCT = 85;
const STORAGE_KEY = "rwayve.splitView.leftPct";
export default function SplitView({ left, right }) {
    const containerRef = useRef(null);
    const draggingRef = useRef(false);
    const [leftPct, setLeftPct] = useState(() => {
        const stored = localStorage.getItem(STORAGE_KEY);
        const parsed = stored ? Number(stored) : NaN;
        return Number.isFinite(parsed) && parsed >= MIN_PCT && parsed <= MAX_PCT
            ? parsed
            : 50;
    });
    useEffect(() => {
        localStorage.setItem(STORAGE_KEY, String(leftPct));
    }, [leftPct]);
    useEffect(() => {
        function onMove(e) {
            if (!draggingRef.current || !containerRef.current)
                return;
            const rect = containerRef.current.getBoundingClientRect();
            const pct = ((e.clientX - rect.left) / rect.width) * 100;
            const clamped = Math.min(MAX_PCT, Math.max(MIN_PCT, pct));
            setLeftPct(clamped);
        }
        function onUp() {
            if (draggingRef.current) {
                draggingRef.current = false;
                document.body.style.cursor = "";
                document.body.style.userSelect = "";
            }
        }
        window.addEventListener("mousemove", onMove);
        window.addEventListener("mouseup", onUp);
        return () => {
            window.removeEventListener("mousemove", onMove);
            window.removeEventListener("mouseup", onUp);
        };
    }, []);
    function startDrag() {
        draggingRef.current = true;
        document.body.style.cursor = "col-resize";
        document.body.style.userSelect = "none";
    }
    return (_jsxs("div", { className: "split-view", ref: containerRef, children: [_jsx("div", { className: "split-view-pane", style: { width: `${leftPct}%` }, children: left }), _jsx("div", { className: "split-view-divider", onMouseDown: startDrag, role: "separator", "aria-orientation": "vertical", title: "Drag to resize" }), _jsx("div", { className: "split-view-pane", style: { width: `${100 - leftPct}%` }, children: right })] }));
}
