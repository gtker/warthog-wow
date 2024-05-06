use crate::error::MessageError;

pub enum ServerOpcodes {
    RequestSessionKey {
        name: String,
    },
    RegisterRealm {
        name: String,
        address: String,
        population: f32,
        locked: bool,
        flags: u8,
        category: u8,
        realm_type: u8,
        version_major: u8,
        version_minor: u8,
        version_patch: u8,
        version_build: u16,
    },
}

impl ServerOpcodes {
    const REQUEST_SESSION_KEY_OPCODE: u8 = 0;
    const REGISTER_REALM_OPCODE: u8 = 4;

    #[cfg(feature = "sync")]
    pub fn read<R: std::io::Read>(mut r: R) -> Result<Self, MessageError> {
        let mut opcode = [0_u8; 1];
        r.read_exact(&mut opcode)?;

        Ok(match opcode[0] {
            Self::REQUEST_SESSION_KEY_OPCODE => {
                let name = crate::read_string(r)?;

                Self::RequestSessionKey { name }
            }
            Self::REGISTER_REALM_OPCODE => {
                let name = crate::read_string(&mut r)?;

                let address = crate::read_string(&mut r)?;

                let population = crate::read_f32(&mut r)?;

                let locked = crate::read_bool(&mut r)?;

                let flags = crate::read_u8(&mut r)?;

                let category = crate::read_u8(&mut r)?;

                let realm_type = crate::read_u8(&mut r)?;

                let version_major = crate::read_u8(&mut r)?;
                let version_minor = crate::read_u8(&mut r)?;
                let version_patch = crate::read_u8(&mut r)?;
                let version_build = crate::read_u16(&mut r)?;

                Self::RegisterRealm {
                    name,
                    address,
                    population,
                    locked,
                    flags,
                    category,
                    realm_type,
                    version_major,
                    version_minor,
                    version_patch,
                    version_build,
                }
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
            ServerOpcodes::RegisterRealm {
                name,
                address,
                population,
                locked,
                flags,
                category,
                realm_type,
                version_major,
                version_minor,
                version_patch,
                version_build,
            } => {
                crate::write_u8(&mut w, Self::REGISTER_REALM_OPCODE)?;

                crate::write_string(&mut w, name)?;

                crate::write_string(&mut w, address)?;

                crate::write_f32(&mut w, *population)?;

                crate::write_bool(&mut w, *locked)?;

                crate::write_u8(&mut w, *flags)?;

                crate::write_u8(&mut w, *category)?;

                crate::write_u8(&mut w, *realm_type)?;

                crate::write_u8(&mut w, *version_major)?;
                crate::write_u8(&mut w, *version_minor)?;
                crate::write_u8(&mut w, *version_patch)?;
                crate::write_u16(&mut w, *version_build)?;
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
            Self::REGISTER_REALM_OPCODE => {
                let name = crate::read_string_tokio(&mut r).await?;

                let address = crate::read_string_tokio(&mut r).await?;

                let population = crate::read_f32_tokio(&mut r).await?;

                let locked = crate::read_bool_tokio(&mut r).await?;

                let flags = crate::read_u8_tokio(&mut r).await?;

                let category = crate::read_u8_tokio(&mut r).await?;

                let realm_type = crate::read_u8_tokio(&mut r).await?;

                let version_major = crate::read_u8_tokio(&mut r).await?;
                let version_minor = crate::read_u8_tokio(&mut r).await?;
                let version_patch = crate::read_u8_tokio(&mut r).await?;
                let version_build = crate::read_u16_tokio(&mut r).await?;

                Self::RegisterRealm {
                    name,
                    address,
                    population,
                    locked,
                    flags,
                    category,
                    realm_type,
                    version_major,
                    version_minor,
                    version_patch,
                    version_build,
                }
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
            ServerOpcodes::RegisterRealm {
                name,
                address,
                population,
                locked,
                flags,
                category,
                realm_type,
                version_major,
                version_minor,
                version_patch,
                version_build,
            } => {
                crate::write_u8_tokio(&mut w, Self::REGISTER_REALM_OPCODE).await?;

                crate::write_string_tokio(&mut w, name).await?;

                crate::write_string_tokio(&mut w, address).await?;

                crate::write_f32_tokio(&mut w, *population).await?;

                crate::write_bool_tokio(&mut w, *locked).await?;

                crate::write_u8_tokio(&mut w, *flags).await?;

                crate::write_u8_tokio(&mut w, *category).await?;

                crate::write_u8_tokio(&mut w, *realm_type).await?;

                crate::write_u8_tokio(&mut w, *version_major).await?;
                crate::write_u8_tokio(&mut w, *version_minor).await?;
                crate::write_u8_tokio(&mut w, *version_patch).await?;
                crate::write_u16_tokio(&mut w, *version_build).await?;
            }
        }

        Ok(())
    }
}
