use crate::MessageError;

pub enum ClientOpcodes {
    SessionKeyAnswer {
        name: String,
        session_key: Option<[u8; 40]>,
    },
}

impl ClientOpcodes {
    const SESSION_KEY_ANSWER_OPCODE: u8 = 1;

    #[cfg(feature = "sync")]
    pub fn read<R: std::io::Read>(mut r: R) -> Result<Self, MessageError> {
        let mut opcode = [0_u8; 1];
        r.read_exact(&mut opcode)?;

        Ok(match opcode[0] {
            Self::SESSION_KEY_ANSWER_OPCODE => {
                let mut length = [0_u8; 1];
                r.read_exact(&mut length)?;

                let mut name = vec![0_u8; length[0].into()];
                r.read_exact(&mut name)?;
                let name = String::from_utf8(name)?;

                let mut session_key_found = [0_u8; 1];
                r.read_exact(&mut session_key_found)?;
                let session_key = if session_key_found[0] == 1 {
                    let mut session_key = [0_u8; 40];
                    r.read_exact(&mut session_key)?;

                    Some(session_key)
                } else {
                    None
                };

                Self::SessionKeyAnswer { name, session_key }
            }
            v => return Err(MessageError::InvalidOpcode(v)),
        })
    }

    #[cfg(feature = "sync")]
    pub fn write<W: std::io::Write>(&mut self, mut w: W) -> std::io::Result<()> {
        match self {
            ClientOpcodes::SessionKeyAnswer { name, session_key } => {
                crate::write_u8(&mut w, Self::SESSION_KEY_ANSWER_OPCODE)?;

                crate::write_string(&mut w, &name)?;

                if let Some(session_key) = session_key {
                    crate::write_bool(&mut w, true)?;

                    w.write_all(session_key)?;
                } else {
                    crate::write_bool(&mut w, false)?;
                }
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
            Self::SESSION_KEY_ANSWER_OPCODE => {
                let mut length = [0_u8; 1];
                r.read_exact(&mut length).await?;

                let mut name = vec![0_u8; length[0].into()];
                r.read_exact(&mut name).await?;
                let name = String::from_utf8(name)?;

                let mut session_key_found = [0_u8; 1];
                r.read_exact(&mut session_key_found).await?;
                let session_key = if session_key_found[0] == 1 {
                    let mut session_key = [0_u8; 40];
                    r.read_exact(&mut session_key).await?;

                    Some(session_key)
                } else {
                    None
                };

                Self::SessionKeyAnswer { name, session_key }
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
            ClientOpcodes::SessionKeyAnswer { name, session_key } => {
                crate::write_u8_tokio(&mut w, Self::SESSION_KEY_ANSWER_OPCODE).await?;

                crate::write_string_tokio(&mut w, &name).await?;

                if let Some(session_key) = session_key {
                    crate::write_bool_tokio(&mut w, true).await?;

                    w.write_all(session_key).await?;
                } else {
                    crate::write_bool_tokio(&mut w, false).await?;
                }
            }
        }

        Ok(())
    }
}
