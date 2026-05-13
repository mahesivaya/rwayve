import { useNavigate } from "react-router-dom";
import "../home/home.css";

export default function Business() {
  const navigate = useNavigate();

  return (
    <div className="home business-welcome">
      <h1>Hello wayve business</h1>
      <div className="auth-buttons">
        <button onClick={() => navigate("/")}>Home</button>
      </div>
    </div>
  );
}
