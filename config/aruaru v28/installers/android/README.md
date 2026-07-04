# Android installer plan

v28のAndroid版はTauriを使わず、`apps/aruaru-mobile` の薄いPWA UIを Android WebView / Capacitor / Gradle のいずれかで包む想定です。

中核処理はスマホ内ではなく、Rust + Poem GraphQL Gateway / Cosmo / VPS worker に置きます。

優先順:

1. PWAとしてホーム画面追加
2. CapacitorでAPK/AAB化
3. オフラインキャッシュはService Workerで保持
