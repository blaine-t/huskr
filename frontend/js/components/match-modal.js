import { profileImageUrl } from '../api.js';
import { navigate } from '../router.js';

let _matchedUserId = null;

export function showMatchModal(me, matchedUser) {
  _matchedUserId = matchedUser.id;

  document.getElementById('match-other-name').textContent =
    matchedUser.full_name || matchedUser.display_name || 'them';

  setAvatar('match-my-avatar', me);
  setAvatar('match-their-avatar', matchedUser);

  document.getElementById('match-modal').classList.remove('hidden');

  document.getElementById('match-msg-btn').onclick = () => {
    hideMatchModal();
    navigate(`#/messages/${_matchedUserId}`);
  };

  document.getElementById('match-dismiss-btn').onclick = hideMatchModal;
}

function hideMatchModal() {
  document.getElementById('match-modal').classList.add('hidden');
}

function setAvatar(elementId, user) {
  const el = document.getElementById(elementId);
  if (user.image_key) {
    el.innerHTML = `<img src="${profileImageUrl(user.id)}" alt="" />`;
  } else {
    el.textContent = '🎓';
  }
}
