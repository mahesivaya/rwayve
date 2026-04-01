import { useState } from "react";

type Event = {
  id: number;
  title: string;
  start: number;
  end: number;
};

export default function TimeGrid({
  events,
  onCreateEvent,
}: any) {

  // ✅ MOVE HERE (inside component)
  const [dragStart, setDragStart] = useState<number | null>(null);

  const hours = Array.from({ length: 24 }, (_, i) => i);

  const getMinutesFromEvent = (
    e: React.MouseEvent<HTMLDivElement>,
    hour: number
  ) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const y = e.clientY - rect.top;

    const minutes = Math.floor((y / rect.height) * 60);

    return hour * 60 + minutes;
  };

  const handleMouseDown = (
    e: React.MouseEvent<HTMLDivElement>,
    hour: number
  ) => {
    const start = getMinutesFromEvent(e, hour);
    setDragStart(start);
  };

  const handleMouseUp = (
    e: React.MouseEvent<HTMLDivElement>,
    hour: number
  ) => {
    if (dragStart !== null) {
      const end = getMinutesFromEvent(e, hour);

      const newEvent = {
        id: Date.now(),
        start: Math.min(dragStart, end),
        end: Math.max(dragStart, end),
        title: "New Event",
      };

      onCreateEvent(newEvent); // 🔥 send to backend
      setDragStart(null);
    }
  };

  return (
    <div className="time-grid">
      {hours.map((hour) => (
        <div key={hour} className="hour-row">
          <div className="time-label">{hour}:00</div>

          <div
            className="hour-cell"
            onMouseDown={(e) => handleMouseDown(e, hour)}
            onMouseUp={(e) => handleMouseUp(e, hour)}
          >
            {events
              .filter((e: Event) => Math.floor(e.start / 60) === hour)
              .map((e: Event) => (
                <EventBlock key={e.id} event={e} />
              ))}
          </div>
        </div>
      ))}
    </div>
  );
}