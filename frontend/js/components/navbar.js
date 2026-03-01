import { navigate } from '../router.js';

const TABS = [
  {
    id: 'feed',
    hash: '#/feed',
    label: 'Discover',
    icon: `<svg viewBox="0 0 24 24"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/></svg>`,
  },
  {
    id: 'messages',
    hash: '#/messages',
    label: 'Matches',
    icon: `<svg viewBox="0 0 24 24"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>`,
  },
  {
    id: 'profile',
    hash: '#/profile',
    label: 'Profile',
    icon: `<svg viewBox="0 0 24 24"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>`,
  },
];

export function renderNavbar(activeHash) {
  const nav = document.getElementById('navbar');
  nav.classList.remove('hidden');
  nav.innerHTML = TABS.map(t => `
    <button class="nav-tab ${activeHash.startsWith(t.hash) ? 'active' : ''}" data-hash="${t.hash}">
      ${t.icon}
      <span>${t.label}</span>
    </button>
  `).join('');

  nav.querySelectorAll('.nav-tab').forEach(btn => {
    btn.addEventListener('click', () => navigate(btn.dataset.hash));
  });
}

export function hideNavbar() {
  const nav = document.getElementById('navbar');
  nav.classList.add('hidden');
  nav.innerHTML = '';
}
