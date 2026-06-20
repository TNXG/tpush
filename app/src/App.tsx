import React, { useCallback, useEffect, useState } from "react";
import { PermissionsAndroid, Platform, StatusBar, StyleSheet, View } from "react-native";
import { SafeAreaProvider, useSafeAreaInsets } from "react-native-safe-area-context";
import {
  Banner,
  Button,
  Dialog,
  MD3LightTheme,
  PaperProvider,
  Portal,
  Snackbar,
  Surface,
  Text,
} from "react-native-paper";
import {
  clearAll,
  deleteMessage,
  getChannel,
  getChannelSecret,
  getConnectionStatus,
  getDeviceId,
  getMessages,
  getServerBaseUrl,
  markRead,
  setChannelConfig,
  setServerBaseUrl,
  type ConnectionStatus,
  type Message,
} from "tpush_core";
import { PushAppBar } from "./components/PushAppBar";
import { SettingsDialog } from "./components/SettingsDialog";
import { MessageList } from "./components/MessageList";

const customTheme = {
  ...MD3LightTheme,
  colors: {
    ...MD3LightTheme.colors,
    primary: "#176b87",
    primaryContainer: "#e8f3f6",
    secondary: "#64748b",
    error: "#dc2626",
    success: "#22c55e",
    background: "#f7fafc",
    surface: "#ffffff",
  },
};

export default function AppWrapper(): React.JSX.Element {
  return (
    <SafeAreaProvider>
      <PaperProvider theme={customTheme}>
        <App />
      </PaperProvider>
    </SafeAreaProvider>
  );
}

function App(): React.JSX.Element {
  const [messages, setMessages] = useState<Message[]>([]);
  const [deviceId, setDeviceId] = useState("");
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>({
    connected: false,
    state: "idle",
    serverBaseUrl: "",
    channel: "default",
    hasChannelSecret: false,
    lastConnectedAt: 0,
    lastMessageAt: 0,
    lastError: "",
    registerState: "idle",
    registerLastAt: 0,
    registerLastError: "",
    syncState: "idle",
    syncLastAt: 0,
    syncLastCount: 0,
    syncLastError: "",
    wsState: "idle",
    wsLastConnectedAt: 0,
    wsLastMessageAt: 0,
    wsLastError: "",
    decryptState: "idle",
    decryptLastError: "",
    serviceRunning: false,
    serviceStartedAt: 0,
    serviceStoppedAt: 0,
    frontendVisible: false,
    lastForegroundAt: 0,
    lastBackgroundAt: 0,
    corePssKb: 0,
    frontendPssKb: 0,
    coreNativeHeapKb: 0,
    frontendNativeHeapKb: 0,
    servicePid: 0,
    frontendPid: 0,
  });

  const [statusMenuOpen, setStatusMenuOpen] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [clearDialogOpen, setClearDialogOpen] = useState(false);
  const [snackbarMessage, setSnackbarMessage] = useState("");

  const insets = useSafeAreaInsets();

  const refresh = useCallback(() => {
    setMessages(getMessages());
    setDeviceId(getDeviceId());
    setConnectionStatus(getConnectionStatus());
  }, []);

  useEffect(() => {
    requestNotificationPermission();
    refresh();
    const interval = setInterval(refresh, 2000);
    return () => clearInterval(interval);
  }, [refresh]);

  const onRefresh = useCallback(() => {
    setRefreshing(true);
    refresh();
    setTimeout(() => setRefreshing(false), 500);
  }, [refresh]);

  const openMessage = (message: Message) => {
    markRead(message.id);
    refresh();
  };

  const clearMessages = () => {
    clearAll();
    refresh();
    setClearDialogOpen(false);
    setSnackbarMessage("已清空所有消息");
  };

  const removeMessage = (id: string) => {
    deleteMessage(id);
    refresh();
  };

  const saveConnectionConfig = (serverUrl: string, channel: string, secret: string) => {
    setServerBaseUrl(serverUrl);
    setChannelConfig(channel, secret);
    setStatusMenuOpen(false);
    setSnackbarMessage("配置已保存，正在重新连接");
    setConnectionStatus({
      ...getConnectionStatus(),
      connected: false,
      state: "正在重连",
      serverBaseUrl: serverUrl,
      channel: channel,
      hasChannelSecret: secret.length > 0,
    });
  };

  const decryptFailed = connectionStatus.decryptState === "failed";

  return (
    <View style={[styles.screen]}>
      <StatusBar translucent backgroundColor="transparent" barStyle="dark-content" />
      <PushAppBar
        deviceId={deviceId}
        connectionStatus={connectionStatus}
        onOpenSettings={() => setStatusMenuOpen(true)}
        onRefresh={refresh}
        statusBarHeight={insets.top}
      />

      <View style={styles.bannerContainer}>
        <Banner
          visible={decryptFailed}
          actions={[
            {
              label: "重新配置",
              onPress: () => setStatusMenuOpen(true),
            },
          ]}
          icon="alert-circle-outline"
          style={{ backgroundColor: "#fef2f2" }}
        >
          密钥验证失败：{connectionStatus.decryptLastError}。请检查频道密钥配置。
        </Banner>
      </View>

      <SettingsDialog
        visible={statusMenuOpen}
        onDismiss={() => setStatusMenuOpen(false)}
        connectionStatus={connectionStatus}
        initialServerUrl={getServerBaseUrl()}
        initialChannel={getChannel()}
        initialSecret={getChannelSecret()}
        onSave={saveConnectionConfig}
      />

      <MessageList
        messages={messages}
        refreshing={refreshing}
        onRefresh={onRefresh}
        onOpenMessage={openMessage}
        onRemoveMessage={removeMessage}
      />

      <Surface style={[styles.footer, { paddingBottom: Math.max(insets.bottom, 16) }]} elevation={2}>
        <Button
          mode="contained-tonal"
          icon="trash-can-outline"
          onPress={() => setClearDialogOpen(true)}
          disabled={messages.length === 0}
          labelStyle={styles.clearButtonLabel}
        >
          清空全部消息
        </Button>
      </Surface>

      <Portal>
        <Dialog visible={clearDialogOpen} onDismiss={() => setClearDialogOpen(false)}>
          <Dialog.Title>清空消息</Dialog.Title>
          <Dialog.Content>
            <Text variant="bodyMedium">确定要清空所有消息历史吗？此操作不可恢复。</Text>
          </Dialog.Content>
          <Dialog.Actions>
            <Button onPress={() => setClearDialogOpen(false)}>取消</Button>
            <Button onPress={clearMessages} textColor={customTheme.colors.error}>
              清空
            </Button>
          </Dialog.Actions>
        </Dialog>
      </Portal>

      <Snackbar
        visible={snackbarMessage !== ""}
        onDismiss={() => setSnackbarMessage("")}
        duration={3000}
      >
        {snackbarMessage}
      </Snackbar>
    </View>
  );
}

function requestNotificationPermission() {
  if (Platform.OS !== "android" || Platform.Version < 33) {
    return;
  }

  PermissionsAndroid.request(PermissionsAndroid.PERMISSIONS.POST_NOTIFICATIONS).catch(() => {
    // 用户拒绝或系统异常时，后台服务仍会继续运行，只是系统通知可能不会弹出或发声。
  });
}

const styles = StyleSheet.create({
  screen: {
    backgroundColor: "#f7fafc",
    flex: 1,
  },
  bannerContainer: {
    zIndex: 10,
  },
  footer: {
    padding: 16,
    backgroundColor: "white",
  },
  clearButtonLabel: {
    flexShrink: 1,
  },
});
