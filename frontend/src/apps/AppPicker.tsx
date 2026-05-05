import { APP_LIST, type AppKey } from "./registry";
import "./apps.css";

type Props = {
  onPick: (key: AppKey) => void;
  title?: string;
};

export default function AppPicker({ onPick, title = "Choose an app" }: Props) {
  return (
    <div className="app-picker">
      <h3 className="app-picker-title">{title}</h3>
      <div className="app-picker-grid">
        {APP_LIST.map((app) => (
          <button
            key={app.key}
            className="app-picker-card"
            onClick={() => onPick(app.key)}
          >
            <div className="app-picker-icon">{app.icon}</div>
            <div className="app-picker-label">{app.label}</div>
          </button>
        ))}
      </div>
    </div>
  );
}
