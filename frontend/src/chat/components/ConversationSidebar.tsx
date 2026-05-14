import type { ChatChannel, ChatUser } from "../../api/chat";
import type { ChannelRole, Conversation } from "../types";
import ChannelCreateForm from "./ChannelCreateForm";
import ChannelList from "./ChannelList";
import PersonalChatList from "./PersonalChatList";

type Props = {
  users: ChatUser[];
  channels: ChatChannel[];
  selectedConversation: Conversation | null;
  creatingChannel: boolean;
  channelName: string;
  inviteRole: ChannelRole;
  inviteEmails: string;
  channelError: string;
  onToggleCreateChannel: () => void;
  onChannelNameChange: (value: string) => void;
  onInviteRoleChange: (value: ChannelRole) => void;
  onInviteEmailsChange: (value: string) => void;
  onCancelCreateChannel: () => void;
  onCreateChannel: () => void;
  onSelectChannel: (channel: ChatChannel) => void;
  onJoinChannel: (channel: ChatChannel) => void;
  onSelectUser: (user: ChatUser) => void;
};

export default function ConversationSidebar({
  users,
  channels,
  selectedConversation,
  creatingChannel,
  channelName,
  inviteRole,
  inviteEmails,
  channelError,
  onToggleCreateChannel,
  onChannelNameChange,
  onInviteRoleChange,
  onInviteEmailsChange,
  onCancelCreateChannel,
  onCreateChannel,
  onSelectChannel,
  onJoinChannel,
  onSelectUser,
}: Props) {
  return (
    <aside className="user-list">
      <div className="chat-sidebar-header">
        <h3>Chat</h3>
        <button type="button" className="new-channel-btn" onClick={onToggleCreateChannel}>
          + Channel
        </button>
      </div>

      {creatingChannel && (
        <ChannelCreateForm
          channelName={channelName}
          inviteRole={inviteRole}
          inviteEmails={inviteEmails}
          error={channelError}
          onChannelNameChange={onChannelNameChange}
          onInviteRoleChange={onInviteRoleChange}
          onInviteEmailsChange={onInviteEmailsChange}
          onCancel={onCancelCreateChannel}
          onCreate={onCreateChannel}
        />
      )}

      <ChannelList
        channels={channels}
        selectedConversation={selectedConversation}
        onSelect={onSelectChannel}
        onJoin={onJoinChannel}
      />

      <PersonalChatList
        users={users}
        selectedConversation={selectedConversation}
        onSelect={onSelectUser}
      />
    </aside>
  );
}
