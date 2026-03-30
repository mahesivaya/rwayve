// src/pages/Home.tsx
import { useAuth } from "../auth/AuthContext";

export default function Home() {
  const { user } = useAuth();

  return (
    <div className="home">
      <h1>Welcome to Wayve 🚀</h1>

      {user ? (
        <>
          <p>You are logged in as <b>{user.email}</b></p>
          <p>Go to Emails, Chat, or Scheduler to start.</p>
        </>
      ) : (
        <>
          <p>Your all-in-one platform for Email, Chat, and Scheduling.</p>
          <p>Please login to continue.</p>
        </>
      )}
    </div>
  );
}