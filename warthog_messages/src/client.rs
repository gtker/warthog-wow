use crate::MessageError;

pub enum ClientOpcodes {
    SessionKeyAnswer {
        name: String,
        session_key: Option<[u8; 40]>,
    },
    RegisterRealmReply {
        realm_id: Option<u8>,
    },
}

impl ClientOpcodes {
    const SESSION_KEY_ANSWER_OPCODE: u8 = 1;
    const REGISTER_REALM_REPLY_OPCODE: u8 = 5;

    #[cfg(feature = "sync")]
    pub fn read<R: std::io::Read>(mut r: R) -> Result<Self, MessageError> {
        let mut opcode = [0_u8; 1];
        r.read_exact(&mut opcode)?;

        Ok(match opcode[0] {
            Self::SESSION_KEY_ANSWER_OPCODE => {
                let name = crate::read_string(&mut r)?;

                let session_key = if crate::read_bool(&mut r)? {
                    let mut session_key = [0_u8; 40];
                    r.read_exact(&mut session_key)?;

                    Some(session_key)
                } else {
                    None
                };

                Self::SessionKeyAnswer { name, session_key }
            }
            Self::REGISTER_REALM_REPLY_OPCODE => {
                let success = crate::read_bool(&mut r)?;
                let realm_id = if success {
                    Some(crate::read_u8(&mut r)?)
                } else {
                    None
                };

                Self::RegisterRealmReply { realm_id }
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
            ClientOpcodes::RegisterRealmReply { realm_id } => {
                crate::write_u8(&mut w, Self::REGISTER_REALM_REPLY_OPCODE)?;

                if let Some(realm_id) = realm_id {
                    crate::write_bool(&mut w, true)?;

                    crate::write_u8(&mut w, *realm_id)?;
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
                let name = crate::read_string_tokio(&mut r).await?;

                let session_key = if crate::read_bool_tokio(&mut r).await? {
                    let mut session_key = [0_u8; 40];
                    r.read_exact(&mut session_key).await?;

                    Some(session_key)
                } else {
                    None
                };

                Self::SessionKeyAnswer { name, session_key }
            }
            Self::REGISTER_REALM_REPLY_OPCODE => {
                let success = crate::read_bool_tokio(&mut r).await?;
                let realm_id = if success {
                    Some(crate::read_u8_tokio(&mut r).await?)
                } else {
                    None
                };

                Self::RegisterRealmReply { realm_id }
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
            ClientOpcodes::RegisterRealmReply { realm_id } => {
                crate::write_u8_tokio(&mut w, Self::REGISTER_REALM_REPLY_OPCODE).await?;

                if let Some(realm_id) = realm_id {
                    crate::write_bool_tokio(&mut w, true).await?;

                    crate::write_u8_tokio(&mut w, *realm_id).await?;
                } else {
                    crate::write_bool_tokio(&mut w, false).await?;
                }
            }
        }

        Ok(())
    }
}
