import { useEffect, useRef, useState } from "react";
import { useAuth } from "../auth/AuthContext";
import "./call.css";

type User = {
  id: number;
  email: string;
};

export default function Call() {
  const { user } = useAuth();
  const currentUserId = user?.id;

  const [users, setUsers] = useState<User[]>([]);
  const [selectedUser, setSelectedUser] = useState<User | null>(null);

  const [incomingCall, setIncomingCall] = useState<any>(null);
  const [inCall, setInCall] = useState(false);

  const localVideo = useRef<HTMLVideoElement>(null);
  const remoteVideo = useRef<HTMLVideoElement>(null);

  const socketRef = useRef<WebSocket | null>(null);
  const pcRef = useRef<RTCPeerConnection | null>(null);

  // 🔹 Load users
  useEffect(() => {
    fetch("/api/users")
      .then(res => res.json())
      .then(setUsers)
      .catch(console.error);
  }, []);

  // 🔹 WebSocket connect
  useEffect(() => {
    if (!currentUserId) return;

    const ws = new WebSocket(`ws://${window.location.host}/ws/call?user_id=${currentUserId}`);
    socketRef.current = ws;

    ws.onmessage = async (event) => {
      const data = JSON.parse(event.data);

      // 📞 Incoming call
      if (data.type === "call_request") {
        setIncomingCall(data);
        return;
      }

      if (data.type === "call_accept") {
        await startWebRTC(true, data.from);
        setInCall(true);
        return;
      }

      if (data.type === "call_reject") {
        alert("❌ Call rejected");
        return;
      }

      const pc = pcRef.current;
      if (!pc) return;

      // 📥 OFFER
      if (data.type === "Offer") {
        await pc.setRemoteDescription({ type: "offer", sdp: data.sdp });

        const answer = await pc.createAnswer();
        await pc.setLocalDescription(answer);

        socketRef.current?.send(JSON.stringify({
          type: "Answer",
          to: data.from,
          sdp: answer.sdp,
        }));
      }

      // 📥 ANSWER
      if (data.type === "Answer") {
        await pc.setRemoteDescription({ type: "answer", sdp: data.sdp });
      }

      // ❄️ ICE
      if (data.type === "IceCandidate") {
        if (data.candidate) {
          await pc.addIceCandidate(data.candidate);
        }
      }
    };

    return () => ws.close();
  }, [currentUserId]);

  // 🚀 WebRTC setup (caller + receiver)
  const startWebRTC = async (isCaller: boolean, targetId: number) => {
    const pc = new RTCPeerConnection({
      iceServers: [{ urls: "stun:stun.l.google.com:19302" }],
    });

    pcRef.current = pc;

    const stream = await navigator.mediaDevices.getUserMedia({
      video: true,
      audio: true,
    });

    if (localVideo.current) {
      localVideo.current.srcObject = stream;
    }

    stream.getTracks().forEach(track => pc.addTrack(track, stream));

    // 🎬 Remote stream
    pc.ontrack = (event) => {
      if (remoteVideo.current) {
        remoteVideo.current.srcObject = event.streams[0];
      }
    };

    // ❄️ ICE
    pc.onicecandidate = (event) => {
      if (event.candidate) {
        socketRef.current?.send(JSON.stringify({
          type: "IceCandidate",
          to: targetId,
          candidate: event.candidate,
        }));
      }
    };

    // 📤 Only caller sends offer
    if (isCaller) {
      const offer = await pc.createOffer();
      await pc.setLocalDescription(offer);

      socketRef.current?.send(JSON.stringify({
        type: "Offer",
        to: targetId,
        sdp: offer.sdp,
      }));
    }
  };

  // 📞 Start call (STEP 1 → send request)
  const startCall = () => {
    if (!selectedUser) return;

    socketRef.current?.send(JSON.stringify({
      type: "call_request",
      to: selectedUser.id,
    }));
  };

  // ✅ Accept call
  const acceptCall = async () => {
    if (!incomingCall) return;

    socketRef.current?.send(JSON.stringify({
      type: "call_accept",
      to: incomingCall.from,
    }));

    await startWebRTC(false, incomingCall.from);

    setIncomingCall(null);
    setInCall(true);
  };

  // ❌ Reject call
  const rejectCall = () => {
    socketRef.current?.send(JSON.stringify({
      type: "call_reject",
      to: incomingCall.from,
    }));

    setIncomingCall(null);
  };

  // 🛑 End call
  const endCall = () => {
    pcRef.current?.close();
    pcRef.current = null;

    if (localVideo.current) localVideo.current.srcObject = null;
    if (remoteVideo.current) remoteVideo.current.srcObject = null;

    setInCall(false);
  };

  return (
    <div className="call-container">

      {/* 📞 Incoming Call Popup */}
      {incomingCall && (
        <div className="incoming-call">
          <h3>📞 Incoming Call</h3>
          <p>User {incomingCall.from} is calling...</p>

          <div className="call-actions">
            <button className="accept" onClick={acceptCall}>Accept</button>
            <button className="reject" onClick={rejectCall}>Reject</button>
          </div>
        </div>
      )}

      {/* LEFT: USERS */}
      <div className="call-users">
        <h3>Users</h3>
        {users.map(u => (
          <div
            key={u.id}
            className={`user-item ${selectedUser?.id === u.id ? "active" : ""}`}
            onClick={() => setSelectedUser(u)}
          >
            {u.email}
          </div>
        ))}
      </div>

      {/* RIGHT: CALL AREA */}
      <div className="call-area">
  <div className="videos">
    <video ref={remoteVideo} autoPlay playsInline className="remote-video" />
    <video ref={localVideo} autoPlay playsInline muted className="local-video" />
  </div>

  <div className="controls">
    {!inCall ? (
      <div className="call-buttons">
        {/* 📞 Audio Call */}
        <button
          className="audio-btn"
          onClick={() => startCall("audio")}
          disabled={!selectedUser}
        >
          📞
        </button>

        {/* 🎥 Video Call */}
        <button
          className="video-btn"
          onClick={() => startCall("video")}
          disabled={!selectedUser}
        >
          🎥
        </button>
      </div>
    ) : (
      <button className="end" onClick={endCall}>
        ❌ End Call
      </button>
    )}
  </div>
</div>
    </div>
  );
}