import type { ChannelRole } from "../types";
import { roleFromValue } from "../utils";

type Props = {
  channelName: string;
  inviteRole: ChannelRole;
  inviteEmails: string;
  error: string;
  onChannelNameChange: (value: string) => void;
  onInviteRoleChange: (value: ChannelRole) => void;
  onInviteEmailsChange: (value: string) => void;
  onCancel: () => void;
  onCreate: () => void;
};

export default function ChannelCreateForm({
  channelName,
  inviteRole,
  inviteEmails,
  error,
  onChannelNameChange,
  onInviteRoleChange,
  onInviteEmailsChange,
  onCancel,
  onCreate,
}: Props) {
  return (
    <div className="channel-form">
      <label className="channel-field">
        <span>Channel name</span>
        <input
          value={channelName}
          onChange={(e) => onChannelNameChange(e.target.value)}
          placeholder="project-updates"
        />
      </label>

      <label className="channel-field">
        <span>Invitees role</span>
        <select
          value={inviteRole}
          onChange={(e) => onInviteRoleChange(roleFromValue(e.target.value))}
        >
          <option value="user">User</option>
          <option value="admin">Admin</option>
        </select>
      </label>

      <label className="channel-field">
        <span>Invitee emails</span>
        <textarea
          value={inviteEmails}
          onChange={(e) => onInviteEmailsChange(e.target.value)}
          placeholder="alex@example.com, priya@example.com"
        />
      </label>

      {error && <div className="channel-error">{error}</div>}

      <div className="channel-form-actions">
        <button type="button" onClick={onCancel}>
          Cancel
        </button>
        <button type="button" className="primary" onClick={onCreate}>
          Create
        </button>
      </div>
    </div>
  );
}
