use crate::app::enums::FunctionCode;
use crate::app::sequence::Sequence;
use crate::outstation::tests::harness::{Event, EventHandle};
use crate::outstation::traits::{BroadcastAction, OutstationInformation};

pub(crate) struct MockOutstationInformation {
    events: EventHandle,
}

impl MockOutstationInformation {
    pub(crate) fn new(events: EventHandle) -> Box<dyn OutstationInformation> {
        Box::new(Self { events })
    }
}

impl OutstationInformation for MockOutstationInformation {
    fn broadcast_received(&mut self, function: FunctionCode, action: BroadcastAction) {
        self.events.push(Event::BroadcastReceived(function, action))
    }

    fn expect_solicited_confirm(&mut self, ecsn: Sequence) {
        self.events.push(Event::ExpectSolicitedConfirm(ecsn))
    }

    fn solicited_confirm_timeout(&mut self, ecsn: Sequence) {
        self.events.push(Event::SolicitedConfirmTimeout(ecsn))
    }

    fn solicited_confirm_received(&mut self, ecsn: Sequence) {
        self.events.push(Event::SolicitedConfirmReceived(ecsn))
    }

    fn clear_restart_iin(&mut self) {
        self.events.push(Event::ClearRestartIIN)
    }
}