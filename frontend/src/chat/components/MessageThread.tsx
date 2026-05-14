import type { ChatChannel, ChatMessage } from "../../api/chat";
import { formatTime, getStatusIcon } from "../utils";

type Props = {
  messages: ChatMessage[];
  selectedChannel: ChatChannel | null;
  currentUserId?: number;
};

export default function MessageThread({
  messages,
  selectedChannel,
  currentUserId,
}: Props) {
  if (selectedChannel && !selectedChannel.is_member) {
    return (
      <div className="messages">
        <div className="channel-join-empty">
          <strong>{selectedChannel.name}</strong>
          <span>
            {selectedChannel.visibility === "public"
              ? "Join this public channel to read and write messages."
              : "Request admin approval to join this private channel."}
          </span>
        </div>
      </div>
    );
  }

  return (
    <div className="messages">
      {messages.map((msg, i) => {
        const mine = msg.sender_id === currentUserId;
        return (
          <div key={msg.message_id ?? i} className={`message ${mine ? "me" : ""}`}>
            <div className={`bubble ${mine ? "me" : "other"}`}>
              <div>{msg.content}</div>
              <div className="message-meta">
                {formatTime(msg.created_at)} {mine && getStatusIcon(msg.status)}
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}
