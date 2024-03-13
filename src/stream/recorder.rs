use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::{env, io};

const DEFAULT_NET_RECORDING_FILE_INBOUND: &str = "stream_inbound.rec";
const DEFAULT_NET_RECORDING_FILE_OUTBOUND: &str = "stream_outbound.rec";

pub struct Recorder {
    inbound: Box<dyn Write>,
    outbound: Box<dyn Write>,
}

impl Recorder {
    pub fn new() -> io::Result<Self> {
        let file_in = env::var(DEFAULT_NET_RECORDING_FILE_INBOUND).unwrap_or("stream_inbound.rec".to_owned());
        let file_out = env::var(DEFAULT_NET_RECORDING_FILE_OUTBOUND).unwrap_or("stream_outbound.rec".to_owned());
        let inbound = Box::new(BufWriter::new(File::create(file_in)?));
        let outbound = Box::new(BufWriter::new(File::create(file_out)?));
        Ok(Self { inbound, outbound })
    }
    fn record_inbound(&mut self, buf: &[u8]) -> io::Result<()> {
        self.inbound.write_all(buf)?;
        self.inbound.flush()
    }
    fn record_outbound(&mut self, buf: &[u8]) -> io::Result<()> {
        self.outbound.write_all(buf)?;
        self.outbound.flush()
    }
}

pub struct Recorded<S> {
    inner: S,
    recorder: Recorder,
}

impl<S> Recorded<S> {
    pub fn new(stream: S, recorder: Recorder) -> Recorded<S> {
        Self {
            inner: stream,
            recorder,
        }
    }
}

impl<S: Read + Write> Read for Recorded<S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read = self.inner.read(buf)?;
        self.recorder.record_inbound(&buf[..read])?;
        Ok(read)
    }
}

impl<S: Read + Write> Write for Recorded<S> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let wrote = self.inner.write(buf)?;
        self.recorder.record_outbound(&buf[..wrote])?;
        Ok(wrote)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

pub trait Record {
    fn record(self) -> Recorded<Self>
    where
        Self: Sized;
}

impl<T> Record for T
where
    T: Read + Write,
{
    fn record(self) -> Recorded<Self>
    where
        Self: Sized,
    {
        Recorded::new(self, Recorder::new().unwrap())
    }
}
