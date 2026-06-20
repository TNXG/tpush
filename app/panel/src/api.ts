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

export type DeviceItem = {
  deviceId: string;
  channel: string;
  deviceName: string;
  systemName: string;
  systemVersion: string;
  appVersion: string;
  lastSeenAt: string;
  lastWsConnectedAt: string | null;
  online: boolean;
};

let token: string | null = localStorage.getItem("tpush_token");

export function getToken(): string | null {
  return token;
}

export function setToken(t: string | null) {
  token = t;
  if (t) {
    localStorage.setItem("tpush_token", t);
  } else {
    localStorage.removeItem("tpush_token");
  }
}

async function authFetch(url: string, options?: RequestInit): Promise<Response> {
  const headers: Record<string, string> = {
    ...(options?.headers as Record<string, string> | undefined),
  };
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }
  const response = await fetch(url, { ...options, headers });
  if (response.status === 401) {
    setToken(null);
    throw new Error("登录已过期，请重新登录");
  }
  return response;
}

export async function loginApi(username: string, password: string): Promise<string> {
  const formData = new URLSearchParams();
  formData.set("username", username);
  formData.set("password", password);

  const response = await fetch("/api/admin/login", {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: formData.toString(),
  });
  if (!response.ok) {
    const data = await response.json();
    throw new Error(data.error || "登录失败");
  }
  const data = await response.json();
  return data.token;
}

export const fetchMessages = async (): Promise<MessageHistoryItem[]> => {
  const response = await authFetch("/api/messages");
  if (!response.ok) {
    throw new Error(await response.text());
  }
  return response.json();
};

export const fetchChannels = async (): Promise<ChannelItem[]> => {
  const response = await authFetch("/api/channels");
  if (!response.ok) {
    throw new Error(await response.text());
  }
  return response.json();
};

export const fetchDevices = async (): Promise<DeviceItem[]> => {
  const response = await authFetch("/api/devices");
  if (!response.ok) {
    throw new Error(await response.text());
  }
  return response.json();
};

export const saveChannel = async (name: string, key: string): Promise<ChannelItem> => {
  const response = await authFetch("/api/channels", {
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
  const response = await authFetch(`/api/channels/${encodeURIComponent(name)}`, {
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
  const response = await authFetch("/api/push", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ channel, title, content, extras }),
  });
  if (!response.ok) {
    throw new Error(await response.text());
  }
};

export const deleteMessages = async (ids: string[]): Promise<void> => {
  const response = await authFetch("/api/messages", {
    method: "DELETE",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ ids }),
  });
  if (!response.ok) {
    throw new Error(await response.text());
  }
};
