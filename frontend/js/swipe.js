/**
 * Attaches pointer-event-based swipe handling to a card element.
 * onSwipe(direction) is called with 'right' (like) or 'left' (pass).
 */
export function attachSwipe(card, onSwipe) {
  let startX = 0;
  let startY = 0;
  let dragging = false;
  const THRESHOLD = 100;

  card.addEventListener('pointerdown', e => {
    if (e.button !== 0 && e.pointerType === 'mouse') return;
    dragging = true;
    startX = e.clientX;
    startY = e.clientY;
    card.setPointerCapture(e.pointerId);
    card.style.transition = 'none';
  });

  card.addEventListener('pointermove', e => {
    if (!dragging) return;
    const dx = e.clientX - startX;
    const dy = e.clientY - startY;
    const rotate = dx * 0.05;
    card.style.transform = `translateX(${dx}px) translateY(${dy * 0.3}px) rotate(${rotate}deg)`;

    // Show like / nope labels
    const likeLabel = card.querySelector('.card-label.like');
    const nopeLabel = card.querySelector('.card-label.nope');
    const ratio = Math.min(Math.abs(dx) / THRESHOLD, 1);
    if (dx > 0) {
      likeLabel.style.opacity = ratio;
      nopeLabel.style.opacity = 0;
    } else {
      nopeLabel.style.opacity = ratio;
      likeLabel.style.opacity = 0;
    }
  });

  card.addEventListener('pointerup', e => {
    if (!dragging) return;
    dragging = false;
    const dx = e.clientX - startX;

    if (Math.abs(dx) >= THRESHOLD) {
      // Commit: fly off screen
      const dir = dx > 0 ? 'right' : 'left';
      const offscreen = dx > 0 ? window.innerWidth + 200 : -(window.innerWidth + 200);
      card.style.transition = 'transform 0.35s ease-out, opacity 0.35s ease-out';
      card.style.transform = `translateX(${offscreen}px) rotate(${dx * 0.05}deg)`;
      card.style.opacity = '0';
      card.addEventListener('transitionend', () => card.remove(), { once: true });
      onSwipe(dir);
    } else {
      // Snap back
      card.style.transition = 'transform 0.3s ease-out';
      card.style.transform = '';
      const likeLabel = card.querySelector('.card-label.like');
      const nopeLabel = card.querySelector('.card-label.nope');
      if (likeLabel) likeLabel.style.opacity = 0;
      if (nopeLabel) nopeLabel.style.opacity = 0;
    }
  });

  card.addEventListener('pointercancel', () => {
    if (!dragging) return;
    dragging = false;
    card.style.transition = 'transform 0.3s ease-out';
    card.style.transform = '';
  });
}
