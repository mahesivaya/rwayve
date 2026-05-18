import type { ChatChannel } from "../../api/chat";

type Props = {
  title: string;
  selectedChannel: ChatChannel | null;
  settingsOpen: boolean;
  onBack: () => void;
  onToggleSettings: () => void;
  onJoinChannel: (channel: ChatChannel) => void;
};

export default function ChatHeader({
  title,
  selectedChannel,
  settingsOpen,
  onBack,
  onToggleSettings,
  onJoinChannel,
}: Props) {
  return (
    <div className="chat-header">
      <div className="chat-header-main">
        <button
          type="button"
          className="chat-back-btn"
          onClick={onBack}
          aria-label="Back to conversations"
        >
          ‹
        </button>
        <div className="chat-header-copy">
          <h3>{title}</h3>
          {selectedChannel && (
            <span>
              {selectedChannel.visibility} channel
              {selectedChannel.is_member
                ? ` · ${[
                    ...selectedChannel.member_emails,
                    ...(selectedChannel.invite_emails ?? []).map(
                      (email) => `${email} invited`,
                    ),
                  ].join(", ")}`
                : selectedChannel.join_status === "pending"
                  ? " · request pending"
                  : " · join to read and write messages"}
            </span>
          )}
        </div>

        {selectedChannel && (
          <div className="chat-header-actions">
            {!selectedChannel.is_member && (
              <button
                type="button"
                className="channel-header-join"
                disabled={selectedChannel.join_status === "pending"}
                onClick={() => onJoinChannel(selectedChannel)}
              >
                {selectedChannel.join_status === "pending"
                  ? "Pending"
                  : selectedChannel.visibility === "public"
                    ? "Join"
                    : "Request"}
              </button>
            )}
            {selectedChannel.is_member && (
              <button
                type="button"
                className={`channel-settings-btn ${settingsOpen ? "active" : ""}`}
                onClick={onToggleSettings}
                title="Channel settings"
                aria-label="Channel settings"
              >
                ⚙
              </button>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
