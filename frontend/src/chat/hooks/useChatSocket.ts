import { useEffect, useRef, useState, type RefObject } from "react";
import { getAuthToken } from "../../auth/token";
import type { ChatMessage } from "../../api/chat";
import { WS_BASE } from "../../config/env";
import { logger } from "../../utils/logger";
import type { Conversation } from "../types";

type User = {
  id: number;
};

export function useChatSocket(
  user: User | null | undefined,
  selectedRef: RefObject<Conversation | null>,
  onMessage: (message: ChatMessage) => void,
) {
  const wsRef = useRef<WebSocket | null>(null);
  const [readyState, setReadyState] = useState<number>(WebSocket.CLOSED);

  useEffect(() => {
    if (!user) {
      setReadyState(WebSocket.CLOSED);
      return;
    }

    const token = getAuthToken() ?? "";
    const ws = new WebSocket(
      `${WS_BASE}/ws/chat?token=${encodeURIComponent(token)}`,
    );
    wsRef.current = ws;
    setReadyState(ws.readyState);

    ws.onopen = () => {
      setReadyState(ws.readyState);
      logger.log("✅ WS connected");
    };

    ws.onmessage = (event) => {
      const msg: ChatMessage & { type?: string } = JSON.parse(event.data);
      if (msg.type === "status_update" || msg.sender_id === user.id) return;

      if (messageBelongsToSelectedConversation(msg, selectedRef.current)) {
        onMessage(msg);
      }
    };

    ws.onclose = () => {
      setReadyState(ws.readyState);
      logger.log("❌ WS disconnected");
    };

    ws.onerror = () => {
      setReadyState(ws.readyState);
    };

    return () => {
      setReadyState(WebSocket.CLOSED);
      ws.close();
    };
  }, [onMessage, selectedRef, user]);

  return {
    wsRef,
    isConnected: readyState === WebSocket.OPEN,
  };
}

function messageBelongsToSelectedConversation(
  msg: ChatMessage,
  conversation: Conversation | null,
) {
  if (conversation?.type === "channel") {
    return msg.channel_id === conversation.channel.id;
  }

  return (
    conversation?.type === "user" &&
    !msg.channel_id &&
    (msg.sender_id === conversation.user.id || msg.receiver_id === conversation.user.id)
  );
}
