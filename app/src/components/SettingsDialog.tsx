import React, { useState } from 'react';
import { View, StyleSheet, ScrollView } from 'react-native';
import { Button, Dialog, Icon, Portal, Surface, Text, TextInput, useTheme } from 'react-native-paper';
import type { ConnectionStatus } from 'tpush_core';
import { translateState, normalizeChannel, normalizeServerUrl } from '../utils';

interface SettingsDialogProps {
  visible: boolean;
  onDismiss: () => void;
  connectionStatus: ConnectionStatus;
  initialServerUrl: string;
  initialChannel: string;
  initialSecret: string;
  onSave: (serverUrl: string, channel: string, secret: string) => void;
}

export function SettingsDialog({
  visible,
  onDismiss,
  connectionStatus,
  initialServerUrl,
  initialChannel,
  initialSecret,
  onSave,
}: SettingsDialogProps) {
  const [serverUrlDraft, setServerUrlDraft] = useState(initialServerUrl);
  const [channelDraft, setChannelDraft] = useState(initialChannel);
  const [channelSecretDraft, setChannelSecretDraft] = useState(initialSecret);
  const [secretVisible, setSecretVisible] = useState(false);
  const theme = useTheme();

  const handleSave = () => {
    onSave(
      normalizeServerUrl(serverUrlDraft),
      normalizeChannel(channelDraft),
      channelSecretDraft.trim()
    );
  };

  return (
    <Portal>
      <Dialog visible={visible} onDismiss={onDismiss} style={styles.dialog}>
        <Dialog.Title>连接设置</Dialog.Title>
        <Dialog.ScrollArea style={styles.scrollArea}>
          <ScrollView contentContainerStyle={styles.scrollContent}>
            <View style={styles.formGroup}>
              <TextInput
                label="服务端地址"
                value={serverUrlDraft}
                onChangeText={setServerUrlDraft}
                mode="outlined"
                keyboardType="url"
                autoCapitalize="none"
                autoCorrect={false}
                placeholder="例如 http://10.0.2.2:3000"
              />
              <TextInput
                label="频道名称"
                value={channelDraft}
                onChangeText={setChannelDraft}
                mode="outlined"
                autoCapitalize="none"
                autoCorrect={false}
              />
              <TextInput
                label="频道密钥（可选）"
                value={channelSecretDraft}
                onChangeText={setChannelSecretDraft}
                mode="outlined"
                secureTextEntry={!secretVisible}
                autoCapitalize="none"
                autoCorrect={false}
                right={
                  <TextInput.Icon
                    icon={secretVisible ? 'eye-off-outline' : 'eye-outline'}
                    onPress={() => setSecretVisible(!secretVisible)}
                  />
                }
              />
            </View>

            <View style={styles.diagnostics}>
              <Text variant="titleMedium" style={styles.sectionTitle}>
                连接诊断
              </Text>
              <DiagnosticCard
                icon="progress-wrench"
                title="后台服务"
                status={connectionStatus.serviceRunning ? "正在运行" : "未运行"}
                details={[
                  `服务进程：${connectionStatus.servicePid > 0 ? connectionStatus.servicePid : "未找到"}`,
                  `前端界面：${connectionStatus.frontendVisible ? "正在前台显示" : "不在前台"}`,
                  formatTimestamp("服务启动", connectionStatus.serviceStartedAt),
                  formatTimestamp("最近退到后台", connectionStatus.lastBackgroundAt),
                ]}
                error={connectionStatus.serviceRunning ? "" : "后台保活服务未检测到运行进程"}
              />
              <DiagnosticCard
                icon="memory"
                title="资源占用"
                status={`总计 ${formatMemory(connectionStatus.corePssKb + connectionStatus.frontendPssKb)}`}
                details={[
                  `核心服务：${formatMemory(connectionStatus.corePssKb)} PSS，Native ${formatMemory(connectionStatus.coreNativeHeapKb)}`,
                  `前端界面：${formatMemory(connectionStatus.frontendPssKb)} PSS，Native ${formatMemory(connectionStatus.frontendNativeHeapKb)}`,
                ]}
                error=""
              />
              <DiagnosticCard
                icon="information-outline"
                title="整体状态"
                status={connectionStatus.connected ? "已连接" : translateState(connectionStatus.state)}
                details={[
                  `服务端：${connectionStatus.serverBaseUrl || initialServerUrl || "未配置"}`,
                  `频道：${connectionStatus.channel || "default"}`,
                  `密钥：${connectionStatus.hasChannelSecret ? "已启用私有频道密钥" : "公开频道，无需密钥"}`,
                ]}
                error={connectionStatus.lastError}
              />
              <DiagnosticCard
                icon="access-point-network"
                title="实时通道"
                status={translateState(connectionStatus.wsState)}
                details={[
                  formatTimestamp("连接时间", connectionStatus.wsLastConnectedAt),
                  formatTimestamp("最近消息", connectionStatus.wsLastMessageAt),
                ]}
                error={connectionStatus.wsLastError}
              />
              <DiagnosticCard
                icon="sync"
                title="历史同步"
                status={translateState(connectionStatus.syncState)}
                details={[
                  `同步数量：${connectionStatus.syncLastCount} 条`,
                  formatTimestamp("最近执行", connectionStatus.syncLastAt),
                ]}
                error={connectionStatus.syncLastError}
              />
              <DiagnosticCard
                icon="lock-check-outline"
                title="解密状态"
                status={translateState(connectionStatus.decryptState)}
                details={[connectionStatus.hasChannelSecret ? "私有频道消息会在本机解密" : "公开频道消息不需要解密"]}
                error={connectionStatus.decryptLastError}
              />
              <DiagnosticCard
                icon="cellphone-link"
                title="设备注册"
                status={translateState(connectionStatus.registerState)}
                details={[formatTimestamp("注册时间", connectionStatus.registerLastAt)]}
                error={connectionStatus.registerLastError}
              />
            </View>
          </ScrollView>
        </Dialog.ScrollArea>
        <Dialog.Actions style={styles.actions}>
          <Button onPress={onDismiss}>取消</Button>
          <Button mode="contained" onPress={handleSave}>
            保存并重连
          </Button>
        </Dialog.Actions>
      </Dialog>
    </Portal>
  );
}

function DiagnosticCard({
  icon,
  title,
  status,
  details,
  error,
}: {
  icon: string;
  title: string;
  status: string;
  details: string[];
  error: string;
}) {
  const theme = useTheme();
  const visibleDetails = details.filter(Boolean);
  const hasError = error.trim().length > 0;

  return (
    <Surface
      style={[
        styles.diagnosticCard,
        {
          borderColor: hasError ? theme.colors.error : theme.colors.outlineVariant,
          backgroundColor: hasError ? theme.colors.errorContainer : theme.colors.surface,
        },
      ]}
      elevation={0}
    >
      <View style={styles.diagnosticHeader}>
        <Icon source={icon} size={22} color={hasError ? theme.colors.error : theme.colors.primary} />
        <Text variant="titleSmall" style={styles.diagnosticTitle}>
          {title}
        </Text>
        <Text
          variant="labelMedium"
          style={[
            styles.statusPill,
            {
              color: hasError ? theme.colors.onErrorContainer : theme.colors.onPrimaryContainer,
              backgroundColor: hasError ? theme.colors.errorContainer : theme.colors.primaryContainer,
            },
          ]}
        >
          {status}
        </Text>
      </View>
      {visibleDetails.map((detail) => (
        <Text key={detail} variant="bodySmall" style={styles.diagnosticDetail}>
          {detail}
        </Text>
      ))}
      {hasError ? (
        <Text variant="bodySmall" style={[styles.errorText, { color: theme.colors.error }]}>
          错误：{error}
        </Text>
      ) : null}
    </Surface>
  );
}

function formatTimestamp(label: string, timestamp: number): string {
  return timestamp > 0 ? `${label}：${new Date(timestamp).toLocaleString()}` : `${label}：暂无`;
}

function formatMemory(kb: number): string {
  if (kb <= 0) {
    return "0 MB";
  }
  return `${(kb / 1024).toFixed(1)} MB`;
}

const styles = StyleSheet.create({
  dialog: {
    maxHeight: '90%',
  },
  scrollArea: {
    paddingHorizontal: 0,
  },
  scrollContent: {
    paddingHorizontal: 24,
    paddingVertical: 8,
  },
  formGroup: {
    gap: 16,
    marginBottom: 16,
  },
  diagnostics: {
    gap: 10,
  },
  sectionTitle: {
    paddingHorizontal: 0,
    fontWeight: 'bold',
    marginBottom: 2,
  },
  diagnosticCard: {
    borderRadius: 8,
    borderWidth: 1,
    padding: 12,
  },
  diagnosticHeader: {
    alignItems: 'center',
    flexDirection: 'row',
    gap: 8,
    marginBottom: 8,
  },
  diagnosticTitle: {
    flex: 1,
    fontWeight: '700',
  },
  statusPill: {
    borderRadius: 999,
    overflow: 'hidden',
    paddingHorizontal: 10,
    paddingVertical: 4,
  },
  diagnosticDetail: {
    color: '#475569',
    lineHeight: 20,
    marginLeft: 30,
  },
  errorText: {
    lineHeight: 20,
    marginLeft: 30,
    marginTop: 4,
  },
  actions: {
    paddingHorizontal: 24,
    paddingBottom: 16,
  }
});
