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

    const ws = new WebSocket(`ws://localhost:8080/ws/call?user_id=${currentUserId}`);
    socketRef.current = ws;

    ws.onopen = () => {
      console.log("✅ WebSocket connected");
    };

    ws.onclose = () => {
      console.log("❌ WebSocket closed");
    };

    ws.onmessage = async (event) => {
      const data = JSON.parse(event.data);

      // 📞 Incoming call
      if (data.type === "call_request") {
        setIncomingCall(data);
        return;
      }

      if (data.type === "call_accept") {
        await startWebRTC(true, data.from, data.callType || "video");
        setInCall(true);
        return;
      }

      if (data.type === "call_reject") {
        alert("❌ Call rejected");
        return;
      }

      let pc = pcRef.current;

        if (!pc) {
          console.log("⚠️ PC not ready, creating...");
          await startWebRTC(false, data.from, data.callType || "video");
          pc = pcRef.current;
        }

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

    return () => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.close();
      }
    };
  }, [currentUserId]);

  // 🚀 WebRTC setup
  const startWebRTC = async (
    isCaller: boolean,
    targetId: number,
    callType: "audio" | "video"
  ) => {
    const pc = new RTCPeerConnection({
      iceServers: [{ urls: "stun:stun.l.google.com:19302" }],
    });

    pcRef.current = pc;

    const stream = await navigator.mediaDevices.getUserMedia({
      video: callType === "video",
      audio: true,
    }
  );

    if (localVideo.current) {
      localVideo.current.srcObject = stream;
    }

    stream.getTracks().forEach(track => pc.addTrack(track, stream));

    pc.ontrack = (event) => {
      if (remoteVideo.current) {
        remoteVideo.current.srcObject = event.streams[0];
      }
    };

    pc.onicecandidate = (event) => {
      if (event.candidate) {
        socketRef.current?.send(JSON.stringify({
          type: "IceCandidate",
          to: targetId,
          candidate: event.candidate,
        }));
      }
    };

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

  // 📞 Start call (SAFE)
  const startCall = (callType: "audio" | "video") => {
    if (!selectedUser) return;
  
    const ws = socketRef.current;
  
    if (!ws || ws.readyState !== WebSocket.OPEN) {
      console.warn("⚠️ WebSocket not ready");
      return;
    }
  
    console.log(`📞 Calling ${selectedUser.email} [ID: ${selectedUser.id}] (${callType})`);
  
    ws.send(JSON.stringify({
      type: "call_request",     // ✅ always string
      to: selectedUser.id,
      from: currentUserId,      // 🔥 ADD THIS (important)
      callType: callType,       // ✅ no conflict
    }));
  };

  // ✅ Accept
  const acceptCall = async () => {
    if (!incomingCall) return;

    socketRef.current?.send(JSON.stringify({
      type: "call_accept",
      to: incomingCall.from,
      from: currentUserId, 
      callType: incomingCall.callType || "video",
    }));

    await startWebRTC(false, incomingCall.from, incomingCall.callType || "video");

    setIncomingCall(null);
    setInCall(true);
  };

  // ❌ Reject
  const rejectCall = () => {
    socketRef.current?.send(JSON.stringify({
      type: "call_reject",
      to: incomingCall.from,
      from: currentUserId, 
    }));

    setIncomingCall(null);
  };

  // 🛑 End
  const endCall = () => {
    pcRef.current?.close();
    pcRef.current = null;

    if (localVideo.current) localVideo.current.srcObject = null;
    if (remoteVideo.current) remoteVideo.current.srcObject = null;

    setInCall(false);
  };

  return (
    <div className="call-container">

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

      <div className="call-area">
        <div className="videos">
          <video ref={remoteVideo} autoPlay playsInline className="remote-video" />
          <video ref={localVideo} autoPlay playsInline muted className="local-video" />
        </div>

        <div className="controls">
          {!inCall ? (
            <div className="call-buttons">
              <button className="audio-btn" onClick={() => startCall("audio")} disabled={!selectedUser}>📞</button>
              <button className="video-btn" onClick={() => startCall("video")} disabled={!selectedUser}>🎥</button>
            </div>
          ) : (
            <button className="end" onClick={endCall}>❌ End Call</button>
          )}
        </div>
      </div>
    </div>
  );
}