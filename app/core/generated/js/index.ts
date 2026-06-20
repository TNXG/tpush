export type Message = {
  id: string;
  title: string;
  content: string;
  payload: string;
  kind: string;
  received_at: string;
  read: boolean;
};

export type ConnectionStatus = {
  connected: boolean;
  state: string;
  serverBaseUrl: string;
  channel: string;
  hasChannelSecret: boolean;
  lastConnectedAt: number;
  lastMessageAt: number;
  lastError: string;
  registerState: string;
  registerLastAt: number;
  registerLastError: string;
  syncState: string;
  syncLastAt: number;
  syncLastCount: number;
  syncLastError: string;
  wsState: string;
  wsLastConnectedAt: number;
  wsLastMessageAt: number;
  wsLastError: string;
  decryptState: string;
  decryptLastError: string;
  serviceRunning: boolean;
  serviceStartedAt: number;
  serviceStoppedAt: number;
  frontendVisible: boolean;
  lastForegroundAt: number;
  lastBackgroundAt: number;
  corePssKb: number;
  frontendPssKb: number;
  coreNativeHeapKb: number;
  frontendNativeHeapKb: number;
  servicePid: number;
  frontendPid: number;
};

type TPushBinding = {
  getMessages: () => Message[];
  markRead: (id: string) => void;
  getDeviceId: () => string;
  getServerBaseUrl: () => string;
  setServerBaseUrl: (serverBaseUrl: string) => void;
  getChannel: () => string;
  getChannelSecret: () => string;
  setChannelConfig: (channel: string, channelSecret: string) => void;
  deleteMessage: (id: string) => void;
  getConnectionStatusJson: () => string;
  clearAll: () => void;
};

const loadBinding = (): TPushBinding => {
  const { NativeModules } = require("react-native");
  const binding = NativeModules.TPush;
  if (!binding) {
    throw new Error("TPush UniFFI React Native binding is not installed");
  }
  return binding;
};

export const getMessages = (): Message[] => loadBinding().getMessages();

export const markRead = (id: string): void => loadBinding().markRead(id);

export const getDeviceId = (): string => loadBinding().getDeviceId();

export const getServerBaseUrl = (): string => loadBinding().getServerBaseUrl();

export const setServerBaseUrl = (serverBaseUrl: string): void =>
  loadBinding().setServerBaseUrl(serverBaseUrl);

export const getChannel = (): string => loadBinding().getChannel();

export const getChannelSecret = (): string => loadBinding().getChannelSecret();

export const setChannelConfig = (channel: string, channelSecret: string): void =>
  loadBinding().setChannelConfig(channel, channelSecret);

export const deleteMessage = (id: string): void => loadBinding().deleteMessage(id);

export const getConnectionStatus = (): ConnectionStatus =>
  JSON.parse(loadBinding().getConnectionStatusJson()) as ConnectionStatus;

export const clearAll = (): void => loadBinding().clearAll();
