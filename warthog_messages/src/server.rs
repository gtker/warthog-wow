use crate::error::MessageError;

pub enum ServerOpcodes {
    RequestSessionKey { name: String },
}

impl ServerOpcodes {
    const REQUEST_SESSION_KEY_OPCODE: u8 = 0;

    #[cfg(feature = "sync")]
    pub fn read<R: std::io::Read>(mut r: R) -> Result<Self, MessageError> {
        let mut opcode = [0_u8; 1];
        r.read_exact(&mut opcode)?;

        Ok(match opcode[0] {
            Self::REQUEST_SESSION_KEY_OPCODE => {
                let name = crate::read_string(r)?;

                Self::RequestSessionKey { name }
            }
            v => return Err(MessageError::InvalidOpcode(v)),
        })
    }

    #[cfg(feature = "sync")]
    pub fn write<W: std::io::Write>(&mut self, mut w: W) -> std::io::Result<()> {
        match self {
            ServerOpcodes::RequestSessionKey { name } => {
                crate::write_u8(&mut w, Self::REQUEST_SESSION_KEY_OPCODE)?;

                crate::write_string(&mut w, &name)?;
            }
        }

        Ok(())
    }

    #[cfg(feature = "tokio")]
    pub async fn tokio_read<R: tokio::io::AsyncReadExt + Unpin>(
        mut r: R,
    ) -> Result<Self, MessageError> {
        let mut opcode = [0_u8; 1];
        r.read_exact(&mut opcode).await?;

        Ok(match opcode[0] {
            Self::REQUEST_SESSION_KEY_OPCODE => {
                let name = crate::read_string_tokio(r).await?;

                Self::RequestSessionKey { name }
            }
            v => return Err(MessageError::InvalidOpcode(v)),
        })
    }

    #[cfg(feature = "tokio")]
    pub async fn tokio_write<W: tokio::io::AsyncWriteExt + Unpin>(
        &mut self,
        mut w: W,
    ) -> std::io::Result<()> {
        match self {
            ServerOpcodes::RequestSessionKey { name } => {
                crate::write_u8_tokio(&mut w, Self::REQUEST_SESSION_KEY_OPCODE).await?;

                crate::write_string_tokio(&mut w, &name).await?;
            }
        }

        Ok(())
    }
}
