import { getCompatibleProfiles, submitLike, getMe } from '../api.js';
import { renderNavbar } from '../components/navbar.js';
import { buildCard } from '../components/card.js';
import { showMatchModal } from '../components/match-modal.js';
import { attachSwipe } from '../swipe.js';
import { navigate } from '../router.js';

export async function renderFeed(container) {
  renderNavbar('#/feed');
  container.innerHTML = `<div id="feed-view"><div class="spinner"></div></div>`;

  let me, profiles;
  try {
    [me, profiles] = await Promise.all([getMe(), getCompatibleProfiles()]);
  } catch {
    navigate('#/login');
    return;
  }
  if (!me) { navigate('#/login'); return; }

  const feedEl = document.getElementById('feed-view');
  feedEl.innerHTML = `
    <div id="card-stack"></div>
    <div id="feed-empty" style="display:none;">
      <h2>You're all caught up! 🎉</h2>
      <p>No more profiles right now.<br>Check back later!</p>
      <button class="btn-primary" id="refresh-btn">Refresh</button>
    </div>
    <div id="card-actions">
      <button class="action-btn pass-btn" id="pass-btn" title="Pass">
        <svg viewBox="0 0 24 24"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
      </button>
      <button class="action-btn like-btn" id="like-btn" title="Like">
        <svg viewBox="0 0 24 24"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/></svg>
      </button>
    </div>
  `;

  document.getElementById('refresh-btn')?.addEventListener('click', () => renderFeed(container));

  const stack = document.getElementById('card-stack');
  // profiles[0] = front, profiles[1] = back (peek)
  let queue = [...profiles];

  function showEmptyState() {
    stack.innerHTML = '';
    document.getElementById('feed-empty').style.display = 'block';
    document.getElementById('card-actions').style.display = 'none';
  }

  function renderTopCards() {
    // Remove old cards that are no longer at top
    while (stack.firstChild) stack.removeChild(stack.firstChild);

    if (queue.length === 0) {
      showEmptyState();
      return;
    }

    // Back card (index 1)
    if (queue.length > 1) {
      const backCard = buildCard(queue[1], 'back');
      stack.appendChild(backCard);
    }
    // Front card (index 0)
    const frontCard = buildCard(queue[0], 'front');
    stack.appendChild(frontCard);

    attachSwipe(frontCard, dir => handleSwipe(dir, queue[0]));
  }

  async function handleSwipe(direction, user) {
    const isLike = direction === 'right';
    queue.shift();
    renderTopCards();
    try {
      const res = await submitLike(user.id, isLike);
      if (res.status === 201 && isLike) {
        showMatchModal(me, user);
      }
    } catch {
      // silent
    }
  }

  document.getElementById('pass-btn').addEventListener('click', () => {
    if (queue.length === 0) return;
    triggerCardSwipe('left');
  });

  document.getElementById('like-btn').addEventListener('click', () => {
    if (queue.length === 0) return;
    triggerCardSwipe('right');
  });

  function triggerCardSwipe(dir) {
    const frontCard = stack.querySelector('.profile-card.front');
    if (!frontCard) return;
    const user = queue[0];
    const offscreen = dir === 'right' ? window.innerWidth + 200 : -(window.innerWidth + 200);
    frontCard.style.transition = 'transform 0.35s ease-out, opacity 0.35s ease-out';
    frontCard.style.transform = `translateX(${offscreen}px) rotate(${dir === 'right' ? 15 : -15}deg)`;
    frontCard.style.opacity = '0';
    frontCard.addEventListener('transitionend', () => frontCard.remove(), { once: true });
    handleSwipe(dir, user);
  }

  renderTopCards();
}
