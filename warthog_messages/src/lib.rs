mod client;
mod error;
mod server;

pub use client::*;
pub use error::*;
pub use server::*;

#[cfg(feature = "sync")]
fn write_bool<W: std::io::Write>(w: W, value: bool) -> std::io::Result<()> {
    write_u8(w, if value { 1 } else { 0 })
}

#[cfg(feature = "sync")]
fn write_u8<W: std::io::Write>(mut w: W, value: u8) -> std::io::Result<()> {
    let buf = [value];
    w.write_all(&buf)
}

#[cfg(feature = "sync")]
fn write_string<W: std::io::Write>(mut w: W, value: &str) -> std::io::Result<()> {
    write_u8(&mut w, value.len() as u8)?;

    for b in value.as_bytes() {
        write_u8(&mut w, *b)?;
    }

    Ok(())
}

#[cfg(feature = "tokio")]
async fn write_bool_tokio<W: tokio::io::AsyncWriteExt + std::marker::Unpin>(
    w: W,
    value: bool,
) -> std::io::Result<()> {
    write_u8_tokio(w, if value { 1 } else { 0 }).await
}

#[cfg(feature = "tokio")]
async fn write_u8_tokio<W: tokio::io::AsyncWriteExt + std::marker::Unpin>(
    mut w: W,
    value: u8,
) -> std::io::Result<()> {
    let buf = [value];
    w.write_all(&buf).await
}

#[cfg(feature = "tokio")]
async fn write_string_tokio<W: tokio::io::AsyncWriteExt + std::marker::Unpin>(
    mut w: W,
    value: &str,
) -> std::io::Result<()> {
    write_u8_tokio(&mut w, value.len() as u8).await?;

    for b in value.as_bytes() {
        write_u8_tokio(&mut w, *b).await?;
    }

    Ok(())
}
