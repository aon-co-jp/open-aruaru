import { aruaruApi } from '../api';
import { alertBox, layout, magicLinkBlock } from '../components/layout';

export function renderRegister(root: HTMLElement): void {
  root.innerHTML = layout(
    'メール登録',
    `
<p class="text-muted">パスワード不要 — ログインリンクをメール（または画面）でお届けします。</p>
<form id="register-form" class="card border-secondary">
  <div class="card-body">
    <label class="form-label">メールアドレス</label>
    <input class="form-control mb-3" type="email" name="email" required placeholder="you@example.com" />
    <button class="btn btn-primary" type="submit">登録してログインリンクを受け取る</button>
    <div id="register-result" class="mt-3"></div>
  </div>
</form>`
  );

  const form = document.getElementById('register-form') as HTMLFormElement;
  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    const email = new FormData(form).get('email') as string;
    const box = document.getElementById('register-result')!;
    const data = await aruaruApi.register(email);
    if (data.ok) {
      box.innerHTML =
        alertBox('success', '登録しました。') +
        (data.magic_link ? magicLinkBlock(data.magic_link) : '');
    } else {
      box.innerHTML = alertBox('danger', data.error || 'error');
    }
  });
}
