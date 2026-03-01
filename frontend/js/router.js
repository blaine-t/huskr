const routes = {};

export function register(pattern, handler) {
  routes[pattern] = handler;
}

export function navigate(hash) {
  window.location.hash = hash;
}

export function start() {
  window.addEventListener('hashchange', dispatch);
  dispatch();
}

function dispatch() {
  const hash = window.location.hash || '#/feed';
  // Find exact match first, then prefix match
  let handler = routes[hash];
  if (!handler) {
    for (const pattern of Object.keys(routes)) {
      if (hash.startsWith(pattern + '/') || hash === pattern) {
        handler = routes[pattern];
        break;
      }
    }
  }
  if (handler) {
    handler(hash);
  } else {
    // Default to feed
    navigate('#/feed');
  }
}

export function currentHash() {
  return window.location.hash || '#/feed';
}
