use crate::{
    data_combination::data_points::data_point::{
        CalcWasActivePastNDays,
        CalculateDataPoints,
        DataPointMetadata,
    },
    data_provision::analytics_event::AnalyticsEvent,
};

impl CalcWasActivePastNDays {
    pub fn new(metadata: DataPointMetadata, events: Vec<AnalyticsEvent>) -> CalcWasActivePastNDays {
        CalcWasActivePastNDays { metadata, events }
    }
}

impl CalculateDataPoints for CalcWasActivePastNDays {
    fn metadata(&self) -> DataPointMetadata {
        self.metadata
    }

    fn calculate(&self) -> Vec<u32> {
        vec![!self.events.is_empty() as u32]
    }
}
