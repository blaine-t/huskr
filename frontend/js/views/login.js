import { hideNavbar } from '../components/navbar.js';

export function renderLogin(container) {
  hideNavbar();
  container.innerHTML = `
    <div class="center-content" style="min-height:100vh; padding-bottom:0;">
      <div style="max-width:340px; width:100%;">
        <h1 style="font-size:3rem; font-weight:900; color:var(--coral); margin-bottom:0.25rem;">🐾 Huskr</h1>
        <p style="color:var(--text-muted); font-size:1.1rem; margin-bottom:3rem;">Find your people on campus</p>
        <a href="/auth/login" style="display:block;">
          <button class="btn-primary" style="width:100%; padding:0.9rem; font-size:1rem; border-radius:var(--radius-sm);">
            Sign in with Microsoft
          </button>
        </a>
        <p style="font-size:0.8rem; color:var(--text-muted); margin-top:1.5rem; text-align:center;">
          Use your university Microsoft account
        </p>
      </div>
    </div>
  `;
}
