# API KEY 引き継ぎヘルパーの使い方

## 目的

Claude Opus、OpenAI / ChatGPT、Gemini、DeepSeek などのAPI型LLMを aruaru-ai で使うために、API KEYを安全に設定します。

## 一時的に使う場合

現在のPowerShellウィンドウだけに保存します。PCに長期保存しません。

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manual\aruaru-api-key-import-template.ps1 -Provider anthropic -Scope process
```

OpenAIの場合:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manual\aruaru-api-key-import-template.ps1 -Provider openai -Scope process
```

## ユーザー環境変数に保存する場合

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manual\aruaru-api-key-import-template.ps1 -Provider anthropic -Scope user
```

保存後はPowerShellまたはaruaru-desktopを再起動してください。

## 注意

- API KEYをREADMEやGitに入れないでください。
- 他人のAPI KEYを使わないでください。
- 有料APIは利用料金が発生します。
- aruaru-aiはログにAPI KEYを出さない設計にします。
