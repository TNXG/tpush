export function translateState(state: string): string {
  const stateMap: Record<string, string> = {
    connected: "已连接",
    connecting: "正在连接",
    failed: "失败",
    idle: "未启动",
    ok: "正常",
    reconnecting: "正在重连",
    registering: "正在注册",
    syncing: "正在同步",
    "正在重连": "正在重连",
  };
  return stateMap[state] ?? state;
}

export function formatTime(label: string, timestamp: number): string {
  return timestamp > 0 ? `\n${label} ${new Date(timestamp).toLocaleTimeString()}` : "";
}

export function formatError(error: string): string {
  return error ? `\n错误：${error}` : "";
}

export function normalizeServerUrl(serverUrl: string): string {
  const trimmedUrl = serverUrl.trim().replace(/\/+$/, "");
  if (!trimmedUrl) {
    return "http://10.0.2.2:3000";
  }
  if (trimmedUrl.startsWith("http://") || trimmedUrl.startsWith("https://")) {
    return trimmedUrl;
  }
  return `http://${trimmedUrl}`;
}

export function normalizeChannel(channel: string): string {
  return channel.trim().replace(/[^\w-]/g, "") || "default";
}
