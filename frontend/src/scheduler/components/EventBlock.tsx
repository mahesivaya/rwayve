export default function EventBlock({ event }: any) {
    const top = (event.start % 60) * 2; // position inside hour
    const height = (event.end - event.start) * 2;
  
    return (
      <div
        className="event-block"
        style={{
          top,
          height,
        }}
      >
        {event.title}
      </div>
    );
  }