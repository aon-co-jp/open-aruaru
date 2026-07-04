use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Framework {
    WordPress, Laravel, Symfony, CakePHP,
    NodeJs, NextJs, NuxtJs, Deno,
    Rust, Go,
    Python, Django, Flask, FastApi,
    Ruby, Rails,
    StaticSite,
    #[serde(other)]
    Other,
}

#[derive(Debug, Serialize)]
pub struct Recommendation {
    pub webserver:  &'static str,
    pub reason:     &'static str,
    pub confidence: u8,
}

pub fn recommend(fw: &Framework) -> Recommendation {
    match fw {
        Framework::WordPress => Recommendation {
            webserver: "apache",
            reason: ".htaccess ネイティブ対応・mod_rewrite が WordPress と相性最良",
            confidence: 90,
        },
        Framework::Laravel | Framework::Symfony | Framework::CakePHP => Recommendation {
            webserver: "nginx",
            reason: "try_files による高速 URL ルーティング。PHP-FPM との組み合わせが推奨",
            confidence: 80,
        },
        Framework::NodeJs | Framework::NextJs | Framework::NuxtJs | Framework::Deno => Recommendation {
            webserver: "nginx",
            reason: "リバースプロキシとして Nginx が最適。静的ファイルも高速配信",
            confidence: 95,
        },
        Framework::Rust | Framework::Go => Recommendation {
            webserver: "nginx",
            reason: "アプリが HTTP サーバー内蔵のため Nginx はリバースプロキシのみ担当。最軽量構成",
            confidence: 92,
        },
        Framework::Python | Framework::Django | Framework::Flask | Framework::FastApi => Recommendation {
            webserver: "nginx",
            reason: "Nginx + Gunicorn/uvicorn が業界標準。WSGI/ASGI 連携が容易",
            confidence: 88,
        },
        Framework::Ruby | Framework::Rails => Recommendation {
            webserver: "nginx",
            reason: "Nginx + Puma が Rails 公式推奨構成",
            confidence: 88,
        },
        Framework::StaticSite => Recommendation {
            webserver: "nginx",
            reason: "静的ファイル配信は Nginx が圧倒的に高速・省リソース",
            confidence: 98,
        },
        Framework::Other => Recommendation {
            webserver: "nginx",
            reason: "汎用性・パフォーマンスの観点から Nginx を推奨。Apache も選択可能",
            confidence: 65,
        },
    }
}
