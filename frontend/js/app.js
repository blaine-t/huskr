import { getMe } from './api.js';
import { register, start, navigate } from './router.js';
import { renderLogin } from './views/login.js';
import { renderFeed } from './views/feed.js';
import { renderProfile } from './views/profile.js';
import { renderMessages } from './views/messages.js';

const container = document.getElementById('view-container');

// Check auth before routing
async function boot() {
  const me = await getMe();

  if (!me) {
    // Not logged in — always show login
    register('#/login', () => renderLogin(container));
    register('#/feed',     () => navigate('#/login'));
    register('#/profile',  () => navigate('#/login'));
    register('#/messages', () => navigate('#/login'));
    navigate('#/login');
    start();
    return;
  }

  // Logged in — set up all routes
  register('#/login', () => navigate('#/feed'));
  register('#/feed',     () => renderFeed(container));
  register('#/profile',  () => renderProfile(container));
  register('#/messages', hash => renderMessages(container, hash));

  // If no hash or login hash, determine where to go
  const hash = window.location.hash;
  if (!hash || hash === '#/' || hash === '#/login') {
    // First-time users (no profile info) → go to profile page
    const needsProfile = !me.full_name && !me.bio && !me.major;
    navigate(needsProfile ? '#/profile' : '#/feed');
  }

  start();
}

boot().catch(err => {
  console.error('Boot failed:', err);
  container.innerHTML = `
    <div class="center-content">
      <p style="color:var(--coral)">Something went wrong. Please refresh.</p>
    </div>`;
});
