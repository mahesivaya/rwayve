import { Routes, Route } from "react-router-dom";
import Emails from "./pages/Emails";

function App() {
  return (
    <Routes>
      <Route path="/" element={<Emails />} />
      <Route path="/emails" element={<Emails />} />
    </Routes>
  );
}

export default App;