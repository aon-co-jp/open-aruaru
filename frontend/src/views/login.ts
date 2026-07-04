import { aruaruApi } from '../api';
import { alertBox, layout, magicLinkBlock } from '../components/layout';

export function renderLogin(root: HTMLElement): void {
  root.innerHTML = layout(
    'ログイン',
    `
<p class="text-muted">登録メール宛にログインリンクを送信します。</p>
<form id="login-form" class="card border-secondary">
  <div class="card-body">
    <label class="form-label">メール</label>
    <input class="form-control mb-3" type="email" name="email" required />
    <button class="btn btn-primary" type="submit">ログインリンクを送る</button>
    <div id="login-result" class="mt-3"></div>
  </div>
</form>`
  );

  const form = document.getElementById('login-form') as HTMLFormElement;
  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    const email = new FormData(form).get('email') as string;
    const box = document.getElementById('login-result')!;
    const data = await aruaruApi.login(email);
    if (data.ok) {
      box.innerHTML = data.magic_link
        ? magicLinkBlock(data.magic_link)
        : alertBox('success', 'リンクを発行しました。');
    } else {
      box.innerHTML = alertBox('danger', data.error || 'error');
    }
  });
}
