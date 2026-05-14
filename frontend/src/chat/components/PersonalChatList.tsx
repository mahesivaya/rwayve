import type { ChatUser } from "../../api/chat";
import type { Conversation } from "../types";

type Props = {
  users: ChatUser[];
  selectedConversation: Conversation | null;
  onSelect: (user: ChatUser) => void;
};

export default function PersonalChatList({
  users,
  selectedConversation,
  onSelect,
}: Props) {
  return (
    <>
      <div className="conversation-section-title">People</div>
      {users.map((u) => (
        <button
          key={u.id}
          type="button"
          className={`conversation-item ${
            selectedConversation?.type === "user" &&
            selectedConversation.user.id === u.id
              ? "active"
              : ""
          }`}
          onClick={() => onSelect(u)}
        >
          <span className="conversation-icon">@</span>
          <span className="conversation-main">
            <span className="conversation-name">{u.email}</span>
          </span>
        </button>
      ))}
    </>
  );
}
