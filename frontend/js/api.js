const BASE = '';

async function request(method, path, body) {
  const opts = {
    method,
    credentials: 'include',
    headers: {},
  };
  if (body instanceof FormData) {
    opts.body = body;
  } else if (body !== undefined) {
    opts.headers['Content-Type'] = 'application/json';
    opts.body = JSON.stringify(body);
  }
  const res = await fetch(BASE + path, opts);
  if (res.status === 401) {
    window.location.hash = '#/login';
    throw new Error('unauthorized');
  }
  return res;
}

export async function getMe() {
  const res = await fetch('/user/me', { credentials: 'include' });
  if (res.status === 401) return null;
  if (!res.ok) throw new Error('getMe failed');
  return res.json();
}

export async function updateProfile(formData) {
  const res = await request('POST', '/user/profile', formData);
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export async function getCompatibleProfiles() {
  const res = await request('GET', '/profiles/compatible');
  if (!res.ok) throw new Error('getCompatibleProfiles failed');
  return res.json();
}

export async function getProfile(id) {
  const res = await request('GET', `/profiles/${id}`);
  if (!res.ok) throw new Error('getProfile failed');
  return res.json();
}

export async function submitLike(liked_id, is_like) {
  const res = await request('POST', '/like', { liked_id, is_like });
  return res; // caller checks status
}

export async function getMatches() {
  const res = await request('GET', '/matches');
  if (!res.ok) throw new Error('getMatches failed');
  return res.json();
}

export async function getMessages(userId) {
  const res = await request('GET', `/messages/${userId}`);
  if (!res.ok) throw new Error('getMessages failed');
  return res.json();
}

export async function sendMessage(recipient_id, content, image) {
  const fd = new FormData();
  fd.append('recipient_id', String(recipient_id));
  fd.append('content', content || '');
  if (image) fd.append('image', image);
  const res = await request('POST', '/message', fd);
  if (!res.ok) throw new Error('sendMessage failed');
  return res.json();
}

export function profileImageUrl(id) {
  return `/profiles/${id}/image`;
}

export function messageImageUrl(messageId) {
  return `/messages/${messageId}/image`;
}
