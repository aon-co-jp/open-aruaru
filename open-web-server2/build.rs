fn main() -> Result<(), Box<dyn std::error::Error>> {
    // open-runo と同一の proto/runo.proto を正としてビルドする。
    // 変更時は open-runo/proto/runo.proto と手動同期すること
    // (将来的には共有 crate `runo-proto` へ切り出し予定)。
    tonic_build::compile_protos("proto/runo.proto")?;
    Ok(())
}
