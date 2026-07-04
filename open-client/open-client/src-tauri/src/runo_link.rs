//! runo_link — open-client → open-runo への QUIC 直結クライアント。
//!
//! open-web-server を経由せず、GAME/リアルタイム系はここから
//! open-runo (udp/4433) へ直接三重コミットを投げる (レイテンシ最小化)。
//! ワイヤ形式は open-runo::quic_ingress と同一 (u32長さ前置き + bincode)。
//! 業務系 (Web UI 等) は従来どおり open-web-server 経由 (HTTPS) でも接続可能。

use anyhow::Result;
pub use aruaru_common::{Commit, CommitOutcome, Priority, ReplicaAck};
use std::sync::Arc;

pub struct RunoLink {
    endpoint: quinn::Endpoint,
    server_addr: std::net::SocketAddr,
    server_name: String,
}

impl RunoLink {
    /// 開発用: 証明書検証をスキップする接続 (本番は社内CA証明書へ差し替え必須)
    pub fn connect_insecure(server_addr: std::net::SocketAddr, server_name: &str) -> Result<Self> {
        let mut endpoint = quinn::Endpoint::client("0.0.0.0:0".parse()?)?;
        let crypto = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoVerify))
            .with_no_client_auth();
        let client_config =
            quinn::ClientConfig::new(Arc::new(quinn::crypto::rustls::QuicClientConfig::try_from(crypto)?));
        endpoint.set_default_client_config(client_config);
        Ok(Self { endpoint, server_addr, server_name: server_name.to_string() })
    }

    pub async fn commit(&self, c: Commit) -> Result<CommitOutcome> {
        let conn = self.endpoint.connect(self.server_addr, &self.server_name)?.await?;
        let (mut send, mut recv) = conn.open_bi().await?;

        let body = bincode::serialize(&c)?;
        send.write_all(&(body.len() as u32).to_le_bytes()).await?;
        send.write_all(&body).await?;
        send.finish()?;

        let mut len_buf = [0u8; 4];
        recv.read_exact(&mut len_buf).await?;
        let len = u32::from_le_bytes(len_buf) as usize;
        let mut resp = vec![0u8; len];
        recv.read_exact(&mut resp).await?;
        Ok(bincode::deserialize(&resp)?)
    }
}

/// 開発専用の証明書検証スキップ実装。本番ビルドでは使用しないこと。
#[derive(Debug)]
struct NoVerify;
impl rustls::client::danger::ServerCertVerifier for NoVerify {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self, _m: &[u8], _c: &rustls::pki_types::CertificateDer<'_>, _s: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn verify_tls13_signature(
        &self, _m: &[u8], _c: &rustls::pki_types::CertificateDer<'_>, _s: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider().signature_verification_algorithms.supported_schemes()
    }
}
