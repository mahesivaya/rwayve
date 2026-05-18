import { useNavigate } from "react-router-dom";
import "../home/home.css";

export default function Organization() {
  const navigate = useNavigate();

  return (
    <div className="home organization-welcome">
      <h1>Hello Wayve</h1>
      <div className="auth-buttons">
        <button onClick={() => navigate("/")}>Home</button>
      </div>
    </div>
  );
}
