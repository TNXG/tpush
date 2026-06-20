import React from 'react';
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
    <Appbar.Header elevated style={{ backgroundColor: theme.colors.surface, paddingTop: statusBarHeight }}>
      <Appbar.Content
        title="TPush"
        titleStyle={{ fontWeight: 'bold' }}
        subtitle={deviceId || '正在生成设备 ID...'}
        subtitleStyle={{ fontSize: 12, opacity: 0.8 }}
        style={{ flexShrink: 1 }}
      />
      <Chip
        onPress={onOpenSettings}
        style={{ marginRight: 8, backgroundColor: theme.colors.surfaceVariant, flexShrink: 1 }}
        textStyle={{ color: statusColor, fontWeight: 'bold' }}
        icon={statusIcon}
        ellipsizeMode="tail"
      >
        {statusLabel}
      </Chip>
      <Appbar.Action
        icon="refresh"
        onPress={onRefresh}
      />
    </Appbar.Header>
  );
}
