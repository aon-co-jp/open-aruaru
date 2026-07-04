import 'bootstrap/dist/css/bootstrap.min.css';
import 'bootstrap/dist/js/bootstrap.bundle.min.js';
import './styles/aruaru.scss';

import { renderFeatures } from './views/features';
import { renderHandoff } from './views/handoff';
import { renderConnectWizard } from './views/connect-wizard';
import { renderDashboard } from './views/dashboard';
import { renderDomainSettings } from './views/domains';
import { renderEasyweb } from './views/easyweb';
import { renderHome } from './views/home';
import { renderHttps } from './views/https';
import { renderLogin } from './views/login';
import { renderRegister } from './views/register';

const root = document.getElementById('app')!;

const routes: Record<string, (el: HTMLElement) => void | Promise<void>> = {
  '/': renderHome,
  '/register': renderRegister,
  '/login': renderLogin,
  '/features': renderFeatures,
  '/handoff': renderHandoff,
  '/wizard': renderConnectWizard,
  '/dashboard': renderDashboard,
  '/settings/domain': renderDomainSettings,
  '/settings/easyweb': renderEasyweb,
  '/settings/https': renderHttps,
};

function currentPath(): string {
  const hash = location.hash.replace(/^#/, '') || '/';
  return hash.startsWith('/') ? hash : '/' + hash;
}

async function render(): Promise<void> {
  const path = currentPath();
  const handler = routes[path] ?? renderHome;
  root.innerHTML = '<div class="text-center py-5 text-muted">読み込み中…</div>';
  await handler(root);
}

window.addEventListener('hashchange', () => void render());
void render();
