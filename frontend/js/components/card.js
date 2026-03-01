import { profileImageUrl } from '../api.js';

export function buildCard(user, className = '') {
  const card = document.createElement('div');
  card.className = `profile-card ${className}`;
  card.dataset.userId = user.id;

  const name = user.full_name || user.display_name || 'Unknown';
  const age = user.age ? `, ${user.age}` : '';
  const major = user.major || '';
  const bio = user.bio || '';
  const tags = (user.interests || []).slice(0, 5);

  const bgHtml = user.image_key
    ? `<div class="card-bg" style="background-image:url('${profileImageUrl(user.id)}')"></div>`
    : `<div class="card-bg-placeholder"><span>🎓</span></div>`;

  card.innerHTML = `
    ${bgHtml}
    <div class="card-gradient"></div>
    <div class="card-label like">LIKE</div>
    <div class="card-label nope">NOPE</div>
    <div class="card-info">
      <div class="card-name">${escHtml(name)}${age}</div>
      ${major ? `<div class="card-major">${escHtml(major)}</div>` : ''}
      ${bio ? `<div class="card-bio">${escHtml(bio)}</div>` : ''}
      ${tags.length ? `<div class="card-tags">${tags.map(t => `<span class="card-tag">${escHtml(t)}</span>`).join('')}</div>` : ''}
    </div>
  `;

  return card;
}

function escHtml(str) {
  return String(str)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}
