mod client;
mod error;
mod server;

pub use client::*;
pub use error::*;
pub use server::*;

#[cfg(feature = "sync")]
fn read_bool<R: std::io::Read>(mut r: R) -> std::io::Result<bool> {
    Ok(if read_u8(r)? == 1 { true } else { false })
}

#[cfg(feature = "sync")]
fn write_bool<W: std::io::Write>(w: W, value: bool) -> std::io::Result<()> {
    write_u8(w, if value { 1 } else { 0 })
}

#[cfg(feature = "sync")]
fn read_u8<R: std::io::Read>(mut r: R) -> std::io::Result<u8> {
    let mut buf = [0_u8; 1];
    r.read_exact(&mut buf)?;

    Ok(buf[0])
}

#[cfg(feature = "sync")]
fn write_u8<W: std::io::Write>(mut w: W, value: u8) -> std::io::Result<()> {
    let buf = [value];
    w.write_all(&buf)
}

#[cfg(feature = "sync")]
fn read_string<R: std::io::Read>(mut r: R) -> Result<String, MessageError> {
    let length = read_u8(r)?;
    let mut buf = vec![0_u8; length.into()];
    r.read_exact(&mut buf)?;

    let s = String::from_utf8(buf)?;

    Ok(s)
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
async fn read_bool_tokio<R: tokio::io::AsyncReadExt + Unpin>(r: R) -> std::io::Result<bool> {
    Ok(if read_u8_tokio(r).await? == 1 {
        true
    } else {
        false
    })
}

#[cfg(feature = "tokio")]
async fn write_bool_tokio<W: tokio::io::AsyncWriteExt + std::marker::Unpin>(
    w: W,
    value: bool,
) -> std::io::Result<()> {
    write_u8_tokio(w, if value { 1 } else { 0 }).await
}

#[cfg(feature = "tokio")]
async fn read_u8_tokio<R: tokio::io::AsyncReadExt + Unpin>(mut r: R) -> std::io::Result<u8> {
    let mut buf = [0_u8; 1];
    r.read_exact(&mut buf).await?;

    Ok(buf[0])
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

#[cfg(feature = "tokio")]
async fn read_string_tokio<R: tokio::io::AsyncReadExt + Unpin>(
    mut r: R,
) -> Result<String, MessageError> {
    let length = read_u8_tokio(&mut r).await?;
    let mut buf = vec![0_u8; length.into()];
    r.read_exact(&mut buf).await?;

    let s = String::from_utf8(buf)?;

    Ok(s)
}
