import React, { useRef } from 'react';
import { Animated, FlatList, PanResponder, Pressable, RefreshControl, StyleSheet, View } from 'react-native';
import { Avatar, Badge, Card, Icon, IconButton, Text, useTheme } from 'react-native-paper';
import type { Message } from 'tpush_core';

interface MessageListProps {
  messages: Message[];
  refreshing: boolean;
  onRefresh: () => void;
  onOpenMessage: (message: Message) => void;
  onRemoveMessage: (id: string) => void;
}

const ACTION_WIDTH = 96;
const ROW_RADIUS = 12;

export function MessageList({
  messages,
  refreshing,
  onRefresh,
  onOpenMessage,
  onRemoveMessage,
}: MessageListProps) {
  return (
    <FlatList<Message>
      contentContainerStyle={styles.list}
      data={messages}
      keyExtractor={(message) => message.id}
      refreshControl={
        <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
      }
      ListEmptyComponent={
        <View style={styles.emptyContainer}>
          <Icon source="inbox-outline" size={64} color="#94a3b8" />
          <Text variant="bodyLarge" style={styles.emptyText}>
            还没有收到推送消息
          </Text>
        </View>
      }
      renderItem={({ item }) => (
        <SwipeMessageRow
          message={item}
          onDelete={() => onRemoveMessage(item.id)}
          onOpen={() => onOpenMessage(item)}
        />
      )}
    />
  );
}

function SwipeMessageRow({
  message,
  onDelete,
  onOpen,
}: {
  message: Message;
  onDelete: () => void;
  onOpen: () => void;
}) {
  const theme = useTheme();
  const translateX = useRef(new Animated.Value(0)).current;
  const opened = useRef(false);
  const deleteOpacity = translateX.interpolate({
    inputRange: [-8, 0],
    outputRange: [1, 0],
    extrapolate: "clamp",
  });
  const snapRow = (toValue: number) => {
    opened.current = toValue < 0;
    Animated.spring(translateX, {
      toValue,
      useNativeDriver: true,
    }).start();
  };
  const panResponder = useRef(
    PanResponder.create({
      onMoveShouldSetPanResponder: (_, gestureState) =>
        Math.abs(gestureState.dx) > 12 && Math.abs(gestureState.dx) > Math.abs(gestureState.dy),
      onPanResponderMove: (_, gestureState) => {
        const baseOffset = opened.current ? -ACTION_WIDTH : 0;
        translateX.setValue(Math.max(-ACTION_WIDTH, Math.min(baseOffset + gestureState.dx, 0)));
      },
      onPanResponderRelease: (_, gestureState) => {
        const shouldOpen = gestureState.dx < -44 || (opened.current && gestureState.dx < 32);
        snapRow(shouldOpen ? -ACTION_WIDTH : 0);
      },
    }),
  ).current;

  const handlePress = () => {
    if (opened.current) {
      snapRow(0);
      return;
    }
    onOpen();
  };

  const handleDelete = () => {
    Animated.timing(translateX, {
      duration: 140,
      toValue: -ACTION_WIDTH,
      useNativeDriver: true,
    }).start(onDelete);
  };

  return (
    <View style={styles.swipeShell}>
      <Animated.View style={[styles.deleteBackground, { opacity: deleteOpacity }]}>
        <Pressable style={styles.deleteAction} onPress={handleDelete}>
          <IconButton
            icon="trash-can-outline"
            iconColor="#ffffff"
            size={22}
            style={styles.deleteIconButton}
          />
          <Text style={styles.deleteText}>删除</Text>
        </Pressable>
      </Animated.View>
      <Animated.View style={[styles.cardLayer, { transform: [{ translateX }] }]} {...panResponder.panHandlers}>
        <Card
          style={[styles.card, message.read && styles.cardRead, !message.read && styles.cardUnread]}
          onPress={handlePress}
        >
          <Card.Title
            title={message.title || message.kind}
            titleStyle={{ fontWeight: "bold" }}
            subtitle={new Date(message.received_at).toLocaleString()}
            left={(props) => (
              <Avatar.Icon
                {...props}
                icon="bell-outline"
                size={40}
                style={{ backgroundColor: theme.colors.primaryContainer }}
                color={theme.colors.primary}
              />
            )}
            right={(props) =>
              !message.read ? (
                <Badge {...props} style={{ marginRight: 16 }}>
                  新
                </Badge>
              ) : null
            }
          />
          <Card.Content>
            <Text variant="bodyMedium" style={{ lineHeight: 22, color: "#334155" }}>
              {message.content}
            </Text>
          </Card.Content>
        </Card>
      </Animated.View>
    </View>
  );
}

const styles = StyleSheet.create({
  list: {
    padding: 16,
    paddingBottom: 80,
  },
  emptyContainer: {
    alignItems: "center",
    justifyContent: "center",
    marginTop: 64,
  },
  emptyText: {
    color: "#64748b",
    marginTop: 16,
  },
  swipeShell: {
    backgroundColor: "#ffffff",
    borderRadius: ROW_RADIUS,
    marginBottom: 12,
    overflow: "hidden",
  },
  deleteBackground: {
    alignItems: "flex-end",
    bottom: 0,
    justifyContent: "center",
    position: "absolute",
    right: 0,
    top: 0,
    width: ACTION_WIDTH,
  },
  deleteAction: {
    alignItems: "center",
    backgroundColor: "#dc2626",
    borderBottomRightRadius: ROW_RADIUS,
    borderTopRightRadius: ROW_RADIUS,
    height: "100%",
    justifyContent: "center",
    width: ACTION_WIDTH,
  },
  deleteIconButton: {
    margin: 0,
  },
  deleteText: {
    color: "#ffffff",
    fontSize: 12,
    fontWeight: "800",
    marginTop: -2,
  },
  cardLayer: {
    borderRadius: ROW_RADIUS,
    overflow: "hidden",
  },
  card: {
    backgroundColor: "#ffffff",
    borderRadius: ROW_RADIUS,
  },
  cardUnread: {
    borderLeftWidth: 4,
    borderLeftColor: "#176b87",
  },
  cardRead: {
    opacity: 0.68,
  },
});
