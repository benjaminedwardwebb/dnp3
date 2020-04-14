use crate::app::format::write::{start_request, HeaderWriter};
use crate::app::gen::enums::FunctionCode;
use crate::app::header::{Control, ResponseHeader};
use crate::app::parse::parser::HeaderCollection;
use crate::app::sequence::Sequence;
use crate::master::handlers::ReadTaskHandler;
use crate::master::runner::TaskError;
use crate::master::task::TaskStatus;
use crate::master::types::ClassScan;
use crate::util::cursor::{WriteCursor, WriteError};

#[derive(Copy, Clone)]
pub enum ReadRequest {
    ClassScan(ClassScan),
}

impl ReadRequest {
    pub fn class_scan(scan: ClassScan) -> Self {
        ReadRequest::ClassScan(scan)
    }

    pub(crate) fn format(self, writer: &mut HeaderWriter) -> Result<(), WriteError> {
        match self {
            ReadRequest::ClassScan(scan) => scan.write(writer),
        }
    }
}

pub(crate) struct ReadTask {
    pub(crate) request: ReadRequest,
    pub(crate) handler: Box<dyn ReadTaskHandler>,
}

impl ReadTask {
    pub(crate) fn format(&self, seq: Sequence, cursor: &mut WriteCursor) -> Result<(), WriteError> {
        let mut writer = start_request(Control::request(seq), FunctionCode::Read, cursor)?;
        self.request.format(&mut writer)
    }

    pub(crate) fn handle(
        &mut self,
        source: u16,
        response: ResponseHeader,
        headers: HeaderCollection,
    ) -> TaskStatus {
        self.handler.handle(source, response, headers);
        if response.control.fin {
            TaskStatus::Complete
        } else {
            TaskStatus::ReadNextResponse
        }
    }

    pub(crate) fn on_error(&mut self, error: TaskError) {
        self.handler.on_error(error)
    }
}
