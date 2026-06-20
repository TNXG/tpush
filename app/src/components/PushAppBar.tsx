import React from 'react';
import { StyleSheet, View } from 'react-native';
import { Appbar, Chip, useTheme } from 'react-native-paper';
import { translateState } from '../utils';

interface PushAppBarProps {
  deviceId: string;
  connectionStatus: {
    connected: boolean;
    state: string;
    decryptState: string;
  };
  onOpenSettings: () => void;
  onRefresh: () => void;
  statusBarHeight: number;
}

export function PushAppBar({
  deviceId,
  connectionStatus,
  onOpenSettings,
  onRefresh,
  statusBarHeight,
}: PushAppBarProps) {
  const theme = useTheme();
  
  const decryptFailed = connectionStatus.decryptState === 'failed';
  const statusColor = decryptFailed
    ? theme.colors.error
    : connectionStatus.connected
    ? theme.colors.primary
    : theme.colors.secondary;
    
  const statusLabel = decryptFailed
    ? '解密失败'
    : connectionStatus.connected
    ? '已连接'
    : translateState(connectionStatus.state);
    
  const statusIcon = decryptFailed
    ? 'alert-circle-outline'
    : connectionStatus.connected
    ? 'check-circle-outline'
    : 'circle-outline';

  return (
    <Appbar.Header
      elevated
      statusBarHeight={statusBarHeight}
      style={[styles.header, { backgroundColor: theme.colors.surface }]}
    >
      <Appbar.Content
        title="TPush"
        titleStyle={styles.title}
        subtitle={deviceId || '正在生成设备 ID...'}
        subtitleStyle={styles.subtitle}
        style={styles.content}
      />
      <View style={styles.statusContainer}>
        <Chip
          onPress={onOpenSettings}
          style={[styles.statusChip, { backgroundColor: theme.colors.surfaceVariant }]}
          textStyle={[styles.statusText, { color: statusColor }]}
          icon={statusIcon}
          ellipsizeMode="tail"
        >
          {statusLabel}
        </Chip>
      </View>
      <Appbar.Action
        icon="refresh"
        onPress={onRefresh}
      />
    </Appbar.Header>
  );
}

const styles = StyleSheet.create({
  header: {
    minHeight: 64,
  },
  content: {
    flex: 1,
    minWidth: 0,
  },
  title: {
    fontWeight: 'bold',
  },
  subtitle: {
    fontSize: 12,
    opacity: 0.8,
  },
  statusContainer: {
    flexShrink: 0,
    maxWidth: 136,
  },
  statusChip: {
    marginRight: 8,
  },
  statusText: {
    fontWeight: 'bold',
  },
});
