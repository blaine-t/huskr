import { getMe, updateProfile, profileImageUrl } from '../api.js';
import { renderNavbar } from '../components/navbar.js';
import { showToast } from '../components/toast.js';
import { navigate } from '../router.js';

export async function renderProfile(container) {
  renderNavbar('#/profile');
  container.innerHTML = `<div class="spinner"></div>`;

  let me;
  try {
    me = await getMe();
  } catch {
    navigate('#/login');
    return;
  }
  if (!me) { navigate('#/login'); return; }

  const interests = [...(me.interests || [])];

  container.innerHTML = `
    <div id="profile-view">
      <h1>Your Profile</h1>

      <div class="avatar-wrapper">
        <div class="avatar-circle" id="avatar-preview">
          ${me.image_key
            ? `<img src="${profileImageUrl(me.id)}" alt="Profile photo" />`
            : '🎓'}
        </div>
        <button class="upload-btn" id="upload-btn">Change photo</button>
        <input type="file" id="photo-file" accept="image/*" style="display:none" />
      </div>

      <div class="form-group">
        <label for="fullname-input">Full Name</label>
        <input id="fullname-input" type="text" placeholder="Your name"
          value="${escHtml(me.full_name || me.display_name || '')}" />
      </div>

      <div class="form-group">
        <label for="age-input">Age</label>
        <input id="age-input" type="number" min="16" max="99" placeholder="Your age"
          value="${me.age || ''}" />
      </div>

      <div class="form-group">
        <label for="major-input">Major</label>
        <input id="major-input" type="text" placeholder="e.g. Computer Science"
          value="${escHtml(me.major || '')}" />
      </div>

      <div class="form-group">
        <label for="bio-input">Bio</label>
        <textarea id="bio-input" rows="3" placeholder="Tell people about yourself...">${escHtml(me.bio || '')}</textarea>
      </div>

      <div class="form-group">
        <label>Interests</label>
        <div id="interest-input-wrapper">
          <div id="interest-tags"></div>
          <input id="interest-input" type="text" placeholder="Type + Enter to add" />
        </div>
        <span class="interest-hint">Press Enter or comma to add an interest</span>
      </div>

      <button id="profile-save-btn" class="btn-primary">Save Profile</button>
      <button id="profile-logout-btn">Log Out</button>
    </div>
  `;

  renderInterestTags();

  // Interest input
  const interestInput = document.getElementById('interest-input');
  interestInput.addEventListener('keydown', e => {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      const val = interestInput.value.trim().replace(/,$/, '');
      if (val && !interests.includes(val)) {
        interests.push(val);
        renderInterestTags();
      }
      interestInput.value = '';
    }
  });

  // Photo upload
  document.getElementById('upload-btn').addEventListener('click', () => {
    document.getElementById('photo-file').click();
  });
  document.getElementById('photo-file').addEventListener('change', e => {
    const file = e.target.files[0];
    if (!file) return;
    const url = URL.createObjectURL(file);
    document.getElementById('avatar-preview').innerHTML = `<img src="${url}" alt="" />`;
  });

  // Save
  document.getElementById('profile-save-btn').addEventListener('click', async () => {
    const btn = document.getElementById('profile-save-btn');
    btn.disabled = true;
    btn.textContent = 'Saving…';

    const fd = new FormData();
    fd.append('full_name', document.getElementById('fullname-input').value.trim());
    fd.append('age', document.getElementById('age-input').value || '');
    fd.append('major', document.getElementById('major-input').value.trim());
    fd.append('bio', document.getElementById('bio-input').value.trim());
    interests.forEach(i => fd.append('interests', i));

    const fileInput = document.getElementById('photo-file');
    if (fileInput.files[0]) {
      fd.append('image', fileInput.files[0]);
    }

    try {
      await updateProfile(fd);
      showToast('Profile saved!');
    } catch (err) {
      showToast('Error saving: ' + err.message);
    } finally {
      btn.disabled = false;
      btn.textContent = 'Save Profile';
    }
  });

  // Logout
  document.getElementById('profile-logout-btn').addEventListener('click', () => {
    window.location.href = '/auth/logout';
  });

  function renderInterestTags() {
    const container = document.getElementById('interest-tags');
    if (!container) return;
    container.innerHTML = interests.map((t, i) => `
      <span class="tag">
        ${escHtml(t)}
        <button class="remove-tag" data-index="${i}" title="Remove">×</button>
      </span>
    `).join('');
    container.querySelectorAll('.remove-tag').forEach(btn => {
      btn.addEventListener('click', () => {
        interests.splice(Number(btn.dataset.index), 1);
        renderInterestTags();
      });
    });
  }
}

function escHtml(str) {
  return String(str)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}
