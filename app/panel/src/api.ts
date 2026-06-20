export type MessageHistoryItem = {
  id: string;
  channel: string;
  title: string;
  content: string;
  extras: string;
  delivery_status: string;
  created_at: string;
};

export type ChannelItem = {
  id: string;
  name: string;
  key: string;
  created_at: string;
  updated_at: string;
};

export const fetchMessages = async (): Promise<MessageHistoryItem[]> => {
  const response = await fetch("/api/messages");
  if (!response.ok) {
    throw new Error(await response.text());
  }
  return response.json();
};

export const fetchChannels = async (): Promise<ChannelItem[]> => {
  const response = await fetch("/api/channels");
  if (!response.ok) {
    throw new Error(await response.text());
  }
  return response.json();
};

export const saveChannel = async (name: string, key: string): Promise<ChannelItem> => {
  const response = await fetch("/api/channels", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ name, key }),
  });
  if (!response.ok) {
    throw new Error(await response.text());
  }
  return response.json();
};

export const deleteChannel = async (name: string): Promise<void> => {
  const response = await fetch(`/api/channels/${encodeURIComponent(name)}`, {
    method: "DELETE",
  });
  if (!response.ok) {
    throw new Error(await response.text());
  }
};

export const sendPush = async (
  channel: string,
  title: string,
  content: string,
  extras: unknown,
): Promise<void> => {
  const response = await fetch("/api/push", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ channel, title, content, extras }),
  });
  if (!response.ok) {
    throw new Error(await response.text());
  }
};

export const deleteMessages = async (ids: string[]): Promise<void> => {
  const response = await fetch("/api/messages", {
    method: "DELETE",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ ids }),
  });
  if (!response.ok) {
    throw new Error(await response.text());
  }
};
