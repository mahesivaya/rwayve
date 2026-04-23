// import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
// import { useEffect, useRef, useState } from "react";
// import { useAuth } from "../auth/AuthContext";
// import "./call.css";
// export default function Call() {
//     const { user } = useAuth();
//     const currentUserId = user?.id;
//     const [users, setUsers] = useState([]);
//     const [selectedUser, setSelectedUser] = useState(null);
//     const [incomingCall, setIncomingCall] = useState(null);
//     const [inCall, setInCall] = useState(false);
//     const localVideo = useRef(null);
//     const remoteVideo = useRef(null);
//     const socketRef = useRef(null);
//     const pcRef = useRef(null);
//     // 🔹 Load users
//     useEffect(() => {
//         fetch("/api/users")
//             .then(res => res.json())
//             .then(setUsers)
//             .catch(console.error);
//     }, []);
//     // 🔹 WebSocket connect
//     useEffect(() => {
//         if (!currentUserId)
//             return;
//         const ws = new WebSocket(`ws://localhost:8080/ws/call?user_id=${currentUserId}`);
//         socketRef.current = ws;
//         ws.onopen = () => {
//             console.log("✅ WebSocket connected");
//         };
//         ws.onclose = () => {
//             console.log("❌ WebSocket closed");
//         };
//         ws.onmessage = async (event) => {
//             const data = JSON.parse(event.data);
//             // 📞 Incoming call
//             if (data.type === "call_request") {
//                 setIncomingCall(data);
//                 return;
//             }
//             if (data.type === "call_accept") {
//                 await startWebRTC(true, data.from, data.callType || "video");
//                 setInCall(true);
//                 return;
//             }
//             if (data.type === "call_reject") {
//                 alert("❌ Call rejected");
//                 return;
//             }
//             let pc = pcRef.current;
//             if (!pc) {
//                 console.log("⚠️ PC not ready, creating...");
//                 await startWebRTC(false, data.from, data.callType || "video");
//                 pc = pcRef.current;
//             }
//             // 📥 OFFER
//             if (data.type === "Offer") {
//                 // await pc.setRemoteDescription({ type: "offer", sdp: data.sdp });
//                 const answer = // await pc.createAnswer();
                 
//                 // await pc.setLocalDescription(answer);
//                 socketRef.current?.send(JSON.stringify({
//                     type: "Answer",
//                     to: data.from,
//                     sdp: answer.sdp,
//                 }));
//             }
//             // 📥 ANSWER
//             if (data.type === "Answer") {
//                 // await pc.setRemoteDescription({ type: "answer", sdp: data.sdp });
//             }
//             // ❄️ ICE
//             if (data.type === "IceCandidate") {
//                 if (data.candidate) {
//                     // await pc.addIceCandidate(data.candidate);
//                 }
//             }
//         };
//         return () => {
//             if (ws.readyState === WebSocket.OPEN) {
//                 ws.close();
//             }
//         };
//     }, [currentUserId]);
//     // 🚀 WebRTC setup
//     // const startWebRTC = async (
//     //   isCaller: boolean,
//     //   targetId: number,
//     //   callType: "audio" | "video"
//     // ) => {
//     //   const pc = new RTCPeerConnection({
//     //     iceServers: [{ urls: "stun:stun.l.google.com:19302" }],
//     //   });
//     //   pcRef.current = pc;
//     //   const stream = await navigator.mediaDevices.getUserMedia({
//     //     video: callType === "video",
//     //     audio: true,
//     //   }
//     // );
//     if (localVideo.current) {
//         localVideo.current.srcObject = stream;
//     }
//     stream.getTracks().forEach(track => pc.addTrack(track, stream));
//     pc.ontrack = (event) => {
//         if (remoteVideo.current) {
//             remoteVideo.current.srcObject = event.streams[0];
//         }
//     };
//     pc.onicecandidate = (event) => {
//         if (event.candidate) {
//             socketRef.current?.send(JSON.stringify({
//                 type: "IceCandidate",
//                 to: targetId,
//                 candidate: event.candidate,
//             }));
//         }
//     };
//     if (isCaller) {
//         const offer = // await pc.createOffer();
         
//         // await pc.setLocalDescription(offer);
//         socketRef.current?.send(JSON.stringify({
//             type: "Offer",
//             to: targetId,
//             sdp: offer.sdp,
//         }));
//     }
// }
// ;
// // 📞 Start call (SAFE)
// const startCall = (callType) => {
//     if (!selectedUser)
//         return;
//     const ws = socketRef.current;
//     if (!ws || ws.readyState !== WebSocket.OPEN) {
//         console.warn("⚠️ WebSocket not ready");
//         return;
//     }
//     console.log(`📞 Calling ${selectedUser.email} [ID: ${selectedUser.id}] (${callType})`);
//     ws.send(JSON.stringify({
//         type: "call_request", // ✅ always string
//         to: selectedUser.id,
//         from: currentUserId, // 🔥 ADD THIS (important)
//         callType: callType, // ✅ no conflict
//     }));
// };
// // ✅ Accept
// const acceptCall = async () => {
//     if (!incomingCall)
//         return;
//     socketRef.current?.send(JSON.stringify({
//         type: "call_accept",
//         to: incomingCall.from,
//         from: currentUserId,
//         callType: incomingCall.callType || "video",
//     }));
//     await startWebRTC(false, incomingCall.from, incomingCall.callType || "video");
//     setIncomingCall(null);
//     setInCall(true);
// };
// // ❌ Reject
// const rejectCall = () => {
//     socketRef.current?.send(JSON.stringify({
//         type: "call_reject",
//         to: incomingCall.from,
//         from: currentUserId,
//     }));
//     setIncomingCall(null);
// };
// // 🛑 End
// const endCall = () => {
//     pcRef.current?.close();
//     pcRef.current = null;
//     if (localVideo.current)
//         localVideo.current.srcObject = null;
//     if (remoteVideo.current)
//         remoteVideo.current.srcObject = null;
//     setInCall(false);
// };
// return (_jsxs("div", { className: "call-container", children: [incomingCall && (_jsxs("div", { className: "incoming-call", children: [_jsx("h3", { children: "\uD83D\uDCDE Incoming Call" }), _jsxs("p", { children: ["User ", incomingCall.from, " is calling..."] }), _jsxs("div", { className: "call-actions", children: [_jsx("button", { className: "accept", onClick: acceptCall, children: "Accept" }), _jsx("button", { className: "reject", onClick: rejectCall, children: "Reject" })] })] })), _jsxs("div", { className: "call-users", children: [_jsx("h3", { children: "Users" }), users.map(u => (_jsx("div", { className: `user-item ${selectedUser?.id === u.id ? "active" : ""}`, onClick: () => setSelectedUser(u), children: u.email }, u.id)))] }), _jsxs("div", { className: "call-area", children: [_jsxs("div", { className: "videos", children: [_jsx("video", { ref: remoteVideo, autoPlay: true, playsInline: true, className: "remote-video" }), _jsx("video", { ref: localVideo, autoPlay: true, playsInline: true, muted: true, className: "local-video" })] }), _jsx("div", { className: "controls", children: !inCall ? (_jsxs("div", { className: "call-buttons", children: [_jsx("button", { className: "audio-btn", onClick: () => startCall("audio"), disabled: !selectedUser, children: "\uD83D\uDCDE" }), _jsx("button", { className: "video-btn", onClick: () => startCall("video"), disabled: !selectedUser, children: "\uD83C\uDFA5" })] })) : (_jsx("button", { className: "end", onClick: endCall, children: "\u274C End Call" })) })] })] }));
