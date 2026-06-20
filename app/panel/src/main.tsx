import { For, Show, createResource, createSignal } from "solid-js";
import { render } from "solid-js/web";
import {
  type ChannelItem,
  deleteChannel,
  deleteMessages as deleteMessagesByIds,
  fetchChannels,
  fetchMessages,
  saveChannel as saveChannelApi,
  sendPush,
} from "./api";
import "./style.css";

function App() {
  const [messages, { refetch }] = createResource(fetchMessages);
  const [channels, { refetch: refetchChannels }] = createResource(fetchChannels);
  const [channel, setChannel] = createSignal("default");
  const [channelKey, setChannelKey] = createSignal("");
  const [newChannelName, setNewChannelName] = createSignal("");
  const [newChannelKey, setNewChannelKey] = createSignal("");
  const [keyVisible, setKeyVisible] = createSignal(false);
  const [selectedMessageIds, setSelectedMessageIds] = createSignal<string[]>([]);
  const [title, setTitle] = createSignal("TPush");
  const [content, setContent] = createSignal("");
  const [extras, setExtras] = createSignal("{}");
  const [isSubmitting, setIsSubmitting] = createSignal(false);
  const [error, setError] = createSignal("");

  const submitPush = async (event: SubmitEvent) => {
    event.preventDefault();
    setError("");
    setIsSubmitting(true);

    try {
      await sendPush(channel(), title(), content(), JSON.parse(extras() || "{}"));
      setContent("");
      await refetch();
    } catch (caughtError) {
      setError(caughtError instanceof Error ? caughtError.message : String(caughtError));
    } finally {
      setIsSubmitting(false);
    }
  };

  const selectChannel = (item: ChannelItem) => {
    setChannel(item.name);
    setChannelKey(item.key);
    setNewChannelName(item.name);
    setNewChannelKey(item.key);
  };

  const generateRandomKey = () => {
    const bytes = new Uint8Array(32);
    crypto.getRandomValues(bytes);
    const key = btoa(String.fromCharCode(...bytes))
      .replace(/\+/g, "-")
      .replace(/\//g, "_")
      .replace(/=+$/g, "");
    setNewChannelKey(key);
  };

  const toggleMessage = (id: string) => {
    setSelectedMessageIds((ids) =>
      ids.includes(id) ? ids.filter((selectedId) => selectedId !== id) : [...ids, id],
    );
  };

  const toggleAllMessages = () => {
    const ids = (messages() ?? []).map((message) => message.id);
    setSelectedMessageIds(selectedMessageIds().length === ids.length ? [] : ids);
  };

  const deleteMessages = async (ids: string[]) => {
    if (ids.length === 0) {
      return;
    }
    setError("");
    setIsSubmitting(true);
    try {
      await deleteMessagesByIds(ids);
      setSelectedMessageIds([]);
      await refetch();
    } catch (caughtError) {
      setError(caughtError instanceof Error ? caughtError.message : String(caughtError));
    } finally {
      setIsSubmitting(false);
    }
  };

  const deleteSelectedMessages = () => deleteMessages(selectedMessageIds());

  const saveChannel = async (event: SubmitEvent) => {
    event.preventDefault();
    setError("");
    setIsSubmitting(true);

    try {
      const savedChannel = await saveChannelApi(newChannelName(), newChannelKey());
      setChannel(savedChannel.name);
      setChannelKey(savedChannel.key);
      setNewChannelName(savedChannel.name);
      setNewChannelKey(savedChannel.key);
      await refetchChannels();
    } catch (caughtError) {
      setError(caughtError instanceof Error ? caughtError.message : String(caughtError));
    } finally {
      setIsSubmitting(false);
    }
  };

  const removeCurrentChannel = async () => {
    const name = newChannelName().trim() || channel();
    if (!name || !confirm(`删除频道 ${name}？该频道内消息会全部清空。`)) {
      return;
    }
    setError("");
    setIsSubmitting(true);
    try {
      await deleteChannel(name);
      setChannel("default");
      setChannelKey("");
      setNewChannelName("");
      setNewChannelKey("");
      setSelectedMessageIds([]);
      await Promise.all([refetchChannels(), refetch()]);
    } catch (caughtError) {
      setError(caughtError instanceof Error ? caughtError.message : String(caughtError));
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <main class="shell">
      <section class="composer">
        <div class="header-section">
          <span class="material-symbols-rounded brand-icon">hub</span>
          <div>
            <h1>TPush</h1>
            <p>实时频道推送管理面板</p>
          </div>
        </div>

        <form class="channel-form" onSubmit={saveChannel}>
          <div class="form-title">
            <span class="material-symbols-rounded">vpn_key</span>
            <h2>频道管理</h2>
          </div>
          <label>
            频道名称
            <input
              value={newChannelName()}
              onInput={(event) => setNewChannelName(event.currentTarget.value)}
              placeholder="例如 team_notice"
            />
          </label>
          <label>
            频道密钥
            <div class="secret-row">
              <input
                value={newChannelKey()}
                onInput={(event) => setNewChannelKey(event.currentTarget.value)}
                placeholder="留空表示公开频道"
                type={keyVisible() ? "text" : "password"}
              />
              <button type="button" class="icon-button" onClick={() => setKeyVisible(!keyVisible())}>
                <span class="material-symbols-rounded">{keyVisible() ? "visibility_off" : "visibility"}</span>
              </button>
            </div>
          </label>
          <div class="button-row">
            <button type="button" class="btn-secondary" onClick={generateRandomKey}>
              <span class="material-symbols-rounded">casino</span>
              随机密钥
            </button>
            <button type="button" class="btn-secondary" onClick={() => setNewChannelKey("")}>
              <span class="material-symbols-rounded">public</span>
              设为公开
            </button>
            <button type="submit" disabled={isSubmitting() || !newChannelName().trim()}>
              <span class="material-symbols-rounded">save</span>
              保存频道
            </button>
            <button
              type="button"
              class="btn-danger"
              disabled={isSubmitting() || !newChannelName().trim()}
              onClick={removeCurrentChannel}
            >
              <span class="material-symbols-rounded">delete</span>
              删除频道
            </button>
          </div>
        </form>

        <div class="channels">
          <For each={channels() ?? []}>
            {(item) => (
              <div
                class={item.name === channel() ? "channel active" : "channel"}
                onClick={() => selectChannel(item)}
              >
                <span class="channel-name">
                  <span class="material-symbols-rounded">{item.name === channel() ? "radio_button_checked" : "radio_button_unchecked"}</span>
                  {item.name}
                </span>
                <small>{item.key ? "私有" : "公开"}</small>
              </div>
            )}
          </For>
        </div>

        <form class="push-form" onSubmit={submitPush}>
          <div class="form-title">
            <span class="material-symbols-rounded">send</span>
            <h2>发送推送</h2>
          </div>
          <label>
            目标频道
            <input value={channel()} onInput={(event) => setChannel(event.currentTarget.value)} />
          </label>
          <label>
            频道密钥状态
            <input
              value={channelKey() ? "私有频道，服务端会加密下发" : "公开频道，不需要密钥"}
              readonly
            />
          </label>
          <label>
            通知标题
            <input value={title()} onInput={(event) => setTitle(event.currentTarget.value)} />
          </label>
          <label>
            消息内容
            <textarea
              value={content()}
              onInput={(event) => setContent(event.currentTarget.value)}
              placeholder="输入要推送给频道的消息"
              rows={4}
            />
          </label>
          <label>
            Extras JSON
            <textarea value={extras()} onInput={(event) => setExtras(event.currentTarget.value)} rows={3} />
          </label>
          <button type="submit" disabled={isSubmitting() || !content().trim()}>
            <span class="material-symbols-rounded">{isSubmitting() ? "hourglass_top" : "send"}</span>
            {isSubmitting() ? "发送中" : "发送推送"}
          </button>
        </form>

        <Show when={error()}>
          <div class="error">
            <span class="material-symbols-rounded">error</span>
            {error()}
          </div>
        </Show>
      </section>

      <section class="history">
        <div class="history-header">
          <div>
            <h2>消息历史</h2>
            <p>最近 200 条服务端推送记录</p>
          </div>
          <button type="button" class="btn-secondary" onClick={() => refetch()}>
            <span class="material-symbols-rounded">refresh</span>
            刷新
          </button>
        </div>
        <div class="message-toolbar">
          <label class="select-all">
            <input
              type="checkbox"
              checked={(messages() ?? []).length > 0 && selectedMessageIds().length === (messages() ?? []).length}
              onChange={toggleAllMessages}
            />
            已选择 {selectedMessageIds().length} 条
          </label>
          <button
            type="button"
            class="btn-danger"
            disabled={selectedMessageIds().length === 0 || isSubmitting()}
            onClick={deleteSelectedMessages}
          >
            <span class="material-symbols-rounded">delete</span>
            批量删除
          </button>
        </div>

        <Show when={!messages.loading} fallback={<div class="muted">正在加载消息...</div>}>
          <For each={messages() ?? []} fallback={<div class="muted">暂无消息，发送一条推送后会显示在这里。</div>}>
            {(message) => (
              <article class={selectedMessageIds().includes(message.id) ? "message selected" : "message"}>
                <div class="message-header">
                  <label class="message-select">
                    <input
                      type="checkbox"
                      checked={selectedMessageIds().includes(message.id)}
                      onChange={() => toggleMessage(message.id)}
                    />
                    <h3>{message.title}</h3>
                  </label>
                  <div class="message-actions">
                    <time>{new Date(message.created_at).toLocaleString()}</time>
                    <button
                      type="button"
                      class="icon-button danger"
                      onClick={() => deleteMessages([message.id])}
                    >
                      <span class="material-symbols-rounded">delete</span>
                    </button>
                  </div>
                </div>
                <div class="message-channel">
                  <span class="material-symbols-rounded">tag</span>
                  {message.channel}
                </div>
                <p>{message.content}</p>
                <Show when={message.extras && message.extras !== "{}"}>
                  <code>{message.extras}</code>
                </Show>
              </article>
            )}
          </For>
        </Show>
      </section>
    </main>
  );
}

render(() => <App />, document.getElementById("root")!);
