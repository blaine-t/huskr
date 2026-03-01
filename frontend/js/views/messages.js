import { getMatches, getMessages, sendMessage, getMe, profileImageUrl, messageImageUrl } from '../api.js';
import { renderNavbar } from '../components/navbar.js';
import { navigate } from '../router.js';

let pollTimer = null;

export async function renderMessages(container, hash) {
  renderNavbar('#/messages');

  // Parse optional chat target from hash: #/messages/123
  const parts = hash.split('/');
  const targetId = parts.length >= 3 ? Number(parts[2]) : null;

  container.innerHTML = `
    <div id="messages-view">
      <div id="messages-list-panel">
        <div id="match-list-header"><h1>Matches</h1></div>
        <ul id="match-list"><li><div class="spinner"></div></li></ul>
      </div>
      <div id="chat-view"></div>
    </div>
  `;

  if (pollTimer) { clearInterval(pollTimer); pollTimer = null; }

  let me, matches;
  try {
    [me, matches] = await Promise.all([getMe(), getMatches()]);
  } catch {
    navigate('#/login');
    return;
  }
  if (!me) { navigate('#/login'); return; }

  renderMatchList(matches, me);

  if (targetId) {
    const match = matches.find(m => m.user.id === targetId);
    if (match) openChat(match.user, me);
  }
}

function renderMatchList(matches, me) {
  const list = document.getElementById('match-list');
  if (!list) return;

  if (matches.length === 0) {
    list.innerHTML = `
      <li>
        <div class="match-empty">
          <h2>No matches yet</h2>
          <p>Keep swiping to find your people!</p>
        </div>
      </li>`;
    return;
  }

  list.innerHTML = matches.map(m => {
    const user = m.user;
    const name = user.full_name || user.display_name || 'Unknown';
    const avatarHtml = user.image_key
      ? `<img src="${profileImageUrl(user.id)}" alt="" />`
      : '🎓';
    return `
      <li class="match-item" data-user-id="${user.id}">
        <div class="match-avatar">${avatarHtml}</div>
        <div class="match-info">
          <div class="match-name">${escHtml(name)}</div>
          <div class="match-preview">Tap to chat</div>
        </div>
      </li>`;
  }).join('');

  list.querySelectorAll('.match-item').forEach(item => {
    item.addEventListener('click', () => {
      const userId = Number(item.dataset.userId);
      const match = matches.find(m => m.user.id === userId);
      if (match) {
        window.history.replaceState(null, '', `#/messages/${userId}`);
        openChat(match.user, me);
      }
    });
  });
}

function openChat(user, me) {
  const panel = document.getElementById('messages-list-panel');
  const chatView = document.getElementById('chat-view');
  if (!panel || !chatView) return;

  panel.classList.add('hidden');
  chatView.classList.add('visible');

  const name = user.full_name || user.display_name || 'Unknown';
  const avatarHtml = user.image_key
    ? `<img src="${profileImageUrl(user.id)}" alt="" />`
    : '🎓';

  chatView.innerHTML = `
    <div id="chat-header">
      <button id="chat-back-btn">
        <svg viewBox="0 0 24 24"><polyline points="15 18 9 12 15 6"/></svg>
      </button>
      <div id="chat-other-avatar">${avatarHtml}</div>
      <div id="chat-other-name">${escHtml(name)}</div>
    </div>
    <div id="chat-messages"></div>
    <div id="chat-image-preview" hidden>
      <img id="chat-image-preview-img" src="" alt="preview" />
      <button id="chat-image-clear-btn" title="Remove image">
        <svg viewBox="0 0 24 24"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
      </button>
    </div>
    <div id="chat-input-row">
      <input id="chat-file-input" type="file" accept="image/*" hidden />
      <button id="chat-attach-btn" title="Attach image">
        <svg viewBox="0 0 24 24"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
      </button>
      <input id="chat-input" type="text" placeholder="Type a message…" autocomplete="off" />
      <button id="chat-send-btn">
        <svg viewBox="0 0 24 24"><line x1="22" y1="2" x2="11" y2="13"/><polygon points="22 2 15 22 11 13 2 9 22 2"/></svg>
      </button>
    </div>
  `;

  document.getElementById('chat-back-btn').addEventListener('click', () => {
    if (pollTimer) { clearInterval(pollTimer); pollTimer = null; }
    window.history.replaceState(null, '', '#/messages');
    chatView.classList.remove('visible');
    chatView.innerHTML = '';
    panel.classList.remove('hidden');
  });

  async function loadMessages() {
    try {
      const msgs = await getMessages(user.id);
      renderChatMessages(msgs, me.id);
    } catch { /* ignore */ }
  }

  let pendingImage = null;

  document.getElementById('chat-attach-btn').addEventListener('click', () => {
    document.getElementById('chat-file-input').click();
  });

  document.getElementById('chat-file-input').addEventListener('change', e => {
    const file = e.target.files[0];
    if (!file) return;
    pendingImage = file;
    const preview = document.getElementById('chat-image-preview');
    const previewImg = document.getElementById('chat-image-preview-img');
    previewImg.src = URL.createObjectURL(file);
    preview.hidden = false;
    e.target.value = '';
  });

  document.getElementById('chat-image-clear-btn').addEventListener('click', () => {
    pendingImage = null;
    const previewImg = document.getElementById('chat-image-preview-img');
    URL.revokeObjectURL(previewImg.src);
    previewImg.src = '';
    document.getElementById('chat-image-preview').hidden = true;
  });

  document.getElementById('chat-send-btn').addEventListener('click', doSend);
  document.getElementById('chat-input').addEventListener('keydown', e => {
    if (e.key === 'Enter') doSend();
  });

  async function doSend() {
    const input = document.getElementById('chat-input');
    if (!input) return;
    const content = input.value.trim();
    if (!content && !pendingImage) return;
    input.value = '';
    const imageToSend = pendingImage;
    pendingImage = null;
    const previewImg = document.getElementById('chat-image-preview-img');
    if (previewImg.src) URL.revokeObjectURL(previewImg.src);
    previewImg.src = '';
    document.getElementById('chat-image-preview').hidden = true;
    try {
      await sendMessage(user.id, content, imageToSend);
      await loadMessages();
    } catch { /* ignore */ }
  }

  loadMessages();
  pollTimer = setInterval(loadMessages, 5000);
}

function renderChatMessages(messages, myId) {
  const container = document.getElementById('chat-messages');
  if (!container) return;
  container.innerHTML = messages.map(msg => {
    const mine = msg.sender_id === myId;
    const textHtml = msg.content ? `<span>${escHtml(msg.content)}</span>` : '';
    const imgHtml = msg.image_key
      ? `<img src="${messageImageUrl(msg.id)}" alt="image" class="bubble-img" />`
      : '';
    return `
      <div class="bubble-row ${mine ? 'mine' : 'theirs'}">
        <div class="bubble">${imgHtml}${textHtml}</div>
      </div>`;
  }).join('');
  container.scrollTop = container.scrollHeight;
}

function escHtml(str) {
  return String(str)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}
