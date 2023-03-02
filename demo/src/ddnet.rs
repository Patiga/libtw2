use gamenet_ddnet::snap_obj;
use std::io;
use thiserror::Error;
use warn::Warn;
use warn::wrap;

use crate::RawChunk;
use crate::reader;
use crate::format;

#[derive(Error, Debug)]
pub enum ReadError {
    #[error(transparent)]
    Inner(#[from] reader::ReadError),
    #[error("{0:?}")]
    Snap(snapshot::snap::Error),
    #[error("{0:?}")]
    Gamenet(gamenet_common::error::Error)
}

pub struct DemoReader {
    raw: reader::Reader,
    delta_reader: snapshot::DeltaReader,
    delta: snapshot::Delta,
}

#[derive(Debug)]
pub enum Warning {
    Demo(format::Warning),
    Snapshot(snapshot::format::Warning),
    Packer(packer::Warning)
}

impl From<format::Warning> for Warning {
    fn from(w: format::Warning) -> Self {
        Warning::Demo(w)
    }
}
impl From<snapshot::format::Warning> for Warning {
    fn from(w: snapshot::format::Warning) -> Self {
        Warning::Snapshot(w)
    }
}
impl From<packer::Warning> for Warning {
    fn from(w: packer::Warning) -> Self {
        Warning::Packer(w)
    }
}

impl DemoReader {
    pub fn new<R, W>(data: R, warn: &mut W) -> Result<Self, ReadError>
    where R: io::Read + io::Seek + 'static, W: Warn<Warning>
    {
        let reader = reader::Reader::new(data, wrap(warn))?;
        Ok(DemoReader {
            raw: reader,
            delta_reader: snapshot::DeltaReader::new(),
            delta: snapshot::Delta::new(),
        })
    }

    pub fn next_chunk<W: Warn<Warning>>(&mut self, warn: &mut W) -> Result<Option<()>, ReadError> {
        match self.raw.read_chunk(wrap(warn))? {
            None => return Ok(None),
            Some(RawChunk::SnapshotDelta(dt)) => {
                let mut unpacker = packer::Unpacker::new(dt);
                let obj_size = snap_obj::obj_size;
                self.delta_reader.read(wrap(warn), &mut self.delta, obj_size, &mut unpacker)
                    .map_err(ReadError::Snap)?;
            },
            Some(RawChunk::Message(msg)) => {
                let mut unpacker = packer::Unpacker::new_from_demo(msg);
                let _ = gamenet_ddnet::msg::Game::decode(wrap(warn), &mut unpacker)
                    .map_err(|err| ReadError::Gamenet(err))?;
                println!("YEP");
            }
            _ => {},
        }
        Ok(Some(()))
    }
}
