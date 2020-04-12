use crate::app::format::write;
use crate::app::parse::parser::{HeaderCollection, ParseLogLevel, ParsedFragment, Response};
use crate::app::sequence::Sequence;
use crate::master::task::{MasterTask, ResponseError, ResponseResult};
use crate::transport::{ReaderType, WriterType};

use crate::app::header::ResponseHeader;
use crate::app::parse::error::ObjectParseError;
use crate::link::error::LinkError;
use crate::master::handlers::ResponseHandler;
use crate::master::unsolicited::UnsolicitedHandler;
use crate::util::cursor::{WriteCursor, WriteError};
use std::time::Duration;
use tokio::prelude::{AsyncRead, AsyncWrite};
use tokio::time::Instant;

struct ResponseCount {
    count: usize,
}

impl ResponseCount {
    pub(crate) fn new() -> Self {
        Self { count: 0 }
    }

    pub(crate) fn reset(&mut self) {
        self.count = 0
    }

    pub(crate) fn is_none(&self) -> bool {
        self.count == 0
    }

    pub(crate) fn increment(&mut self) {
        self.count += 1
    }
}

pub struct TaskRunner {
    seq: Sequence,
    reply_timeout: Duration,
    count: ResponseCount,
    unsolicited_handler: UnsolicitedHandler,
    buffer: [u8; 2048],
}

impl TaskRunner {
    pub fn new(reply_timeout: Duration, unsolicited_handler: Box<dyn ResponseHandler>) -> Self {
        Self {
            seq: Sequence::default(),
            reply_timeout,
            count: ResponseCount::new(),
            unsolicited_handler: UnsolicitedHandler::new(unsolicited_handler),
            buffer: [0; 2048],
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TaskError {
    Lower(LinkError),
    MalformedResponse(ObjectParseError),
    BadResponse(ResponseError),
    NeverReceivedFir,
    UnexpectedFir,
    BadSequence,
    MultiFragmentResponse,
    ResponseTimeout,
    WriteError,
}

impl From<WriteError> for TaskError {
    fn from(_: WriteError) -> Self {
        TaskError::WriteError
    }
}

impl From<LinkError> for TaskError {
    fn from(err: LinkError) -> Self {
        TaskError::Lower(err)
    }
}

impl From<tokio::time::Elapsed> for TaskError {
    fn from(_: tokio::time::Elapsed) -> Self {
        TaskError::ResponseTimeout
    }
}

impl From<ObjectParseError> for TaskError {
    fn from(err: ObjectParseError) -> Self {
        TaskError::MalformedResponse(err)
    }
}

impl From<ResponseError> for TaskError {
    fn from(err: ResponseError) -> Self {
        TaskError::BadResponse(err)
    }
}

impl TaskRunner {
    async fn confirm<T>(
        level: ParseLogLevel,
        io: &mut T,
        destination: u16,
        seq: Sequence,
        writer: &mut WriterType,
    ) -> Result<(), LinkError>
    where
        T: AsyncWrite + Unpin,
    {
        let mut buffer: [u8; 2] = [0; 2];
        let mut cursor = WriteCursor::new(&mut buffer);
        write::confirm_solicited(seq, &mut cursor)?;
        writer
            .write(level, io, destination, cursor.written())
            .await?;
        Ok(())
    }

    async fn handle_non_read_response<T>(
        &mut self,
        level: ParseLogLevel,
        io: &mut T,
        header: ResponseHeader,
        objects: HeaderCollection<'_>,
        task: &mut MasterTask,
        writer: &mut WriterType,
    ) -> Result<ResponseResult, TaskError>
    where
        T: AsyncWrite + Unpin,
    {
        if header.control.seq.value() != self.seq.previous() {
            return Err(TaskError::BadSequence);
        }

        if !(header.control.is_fir_and_fin()) {
            return Err(TaskError::MultiFragmentResponse);
        }

        // non-read responses REALLY shouldn't request confirmation
        // but we'll confirm them if requested and log
        if header.control.con {
            log::warn!("received response requesting confirmation to non-read request");
            Self::confirm(level, io, task.destination, header.control.seq, writer).await?;
        }

        Ok(task.details.handle(header, objects)?)
    }

    async fn handle_read_response<T>(
        &mut self,
        level: ParseLogLevel,
        io: &mut T,
        header: ResponseHeader,
        objects: HeaderCollection<'_>,
        task: &mut MasterTask,
        writer: &mut WriterType,
    ) -> Result<ResponseResult, TaskError>
    where
        T: AsyncWrite + Unpin,
    {
        // validate the sequence number
        if header.control.seq.value() != self.seq.previous() {
            return Err(TaskError::BadSequence);
        }

        if header.control.fir && !self.count.is_none() {
            return Err(TaskError::UnexpectedFir);
        }

        if !header.control.fir && self.count.is_none() {
            return Err(TaskError::NeverReceivedFir);
        }

        if !header.control.fin && !header.control.con {
            log::warn!("received non-FIN response NOT requesting confirmation")
        }

        self.count.increment();

        // write a confirmation if required
        if header.control.con {
            Self::confirm(level, io, task.destination, header.control.seq, writer).await?;
        }

        let result = task.details.handle(header, objects)?;

        if !header.control.fin {
            self.seq.increment();
        }

        Ok(result)
    }

    async fn handle_response<T>(
        &mut self,
        level: ParseLogLevel,
        io: &mut T,
        response: &Response<'_>,
        task: &mut MasterTask,
        writer: &mut WriterType,
    ) -> Result<ResponseResult, TaskError>
    where
        T: AsyncWrite + Unpin,
    {
        let objects = response.objects?;

        if task.details.is_read_request() {
            self.handle_read_response(level, io, response.header, objects, task, writer)
                .await
        } else {
            self.handle_non_read_response(level, io, response.header, objects, task, writer)
                .await
        }
    }

    pub async fn run<T>(
        &mut self,
        level: ParseLogLevel,
        io: &mut T,
        task: &mut MasterTask,
        writer: &mut WriterType,
        reader: &mut ReaderType,
    ) -> Result<(), TaskError>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        self.count.reset();
        // format the request
        let seq = self.seq.increment();
        let mut cursor = WriteCursor::new(&mut self.buffer);
        task.details.format(seq, &mut cursor)?;
        writer
            .write(level, io, task.destination, cursor.written())
            .await?;

        let mut deadline = Instant::now() + self.reply_timeout;

        // now enter a loop to read responses
        loop {
            tokio::time::timeout_at(deadline, reader.read(io)).await??;

            if let Some(fragment) = reader.peek() {
                if let Ok(parsed) = ParsedFragment::parse(level.receive(), fragment.data) {
                    match parsed.to_response() {
                        Err(err) => log::warn!("{}", err),
                        Ok(response) => {
                            if response.header.unsolicited {
                                self.unsolicited_handler
                                    .handle(level, fragment.address, response, io, writer)
                                    .await?;
                            } else {
                                match self
                                    .handle_response(level, io, &response, task, writer)
                                    .await?
                                {
                                    ResponseResult::Success => {
                                        if response.header.control.fin {
                                            return Ok(());
                                        }
                                        // continue to next iteration of the loop, read another reply
                                        deadline = Instant::now() + self.reply_timeout;
                                    }
                                };
                            }
                        }
                    }
                };
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::master::handlers::NullResponseHandler;
    use crate::master::types::ClassScan;
    use crate::transport::mocks::{MockReader, MockWriter};
    use tokio_test::io::Builder;

    #[test]
    fn performs_multi_fragmented_class_scan() {
        let mut task =
            MasterTask::class_scan(1024, ClassScan::class1(), NullResponseHandler::create());

        let mut runner = TaskRunner::new(Duration::from_secs(1), NullResponseHandler::create());

        let mut io = Builder::new()
            .write(&[0xC0, 0x01, 0x3C, 0x02, 0x06])
            // FIR=1, FIN=0, CON=1, SEQ = 0
            .read(&[0xA0, 0x81, 0x00, 0x00])
            // confirm
            .write(&[0xC0, 0x00])
            // FIR=0, FIN=0, CON=1, SEQ = 1
            .read(&[0x21, 0x81, 0x00, 0x00])
            // confirm
            .write(&[0xC1, 0x00])
            // FIR=0, FIN=1, CON=0, SEQ = 2
            .read(&[0x42, 0x81, 0x00, 0x00])
            .build();

        let mut writer = MockWriter::mock();
        let mut reader = MockReader::mock();
        tokio_test::block_on(runner.run(
            ParseLogLevel::Nothing,
            &mut io,
            &mut task,
            &mut writer,
            &mut reader,
        ))
        .unwrap();
    }
}
