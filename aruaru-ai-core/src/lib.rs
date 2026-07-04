//! aruaru-ai-core — AI ライブラリ
//! 独立サービスとして起動しない。aruaru-easyweb に組み込んで使用。

pub mod agent;      // Anthropic API 接続・コマンド生成
pub mod prompt;     // プロンプトテンプレート
pub mod routes;     // Poem ルート（aruaru-easyweb にマウント）

pub use agent::{connect_anthropic, get_plan_info, generate_commands, PlanInfo};
