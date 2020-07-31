use super::InternalEvent;
use metrics::counter;

#[derive(Debug)]
pub struct VicscriptEventProcessed;

impl InternalEvent for VicscriptEventProcessed {
    fn emit_metrics(&self) {
        counter!("events_processed", 1,
            "component_kind" => "transform",
            "component_type" => "vicscript",
        );
    }
}

#[derive(Debug)]
pub struct VicscriptFailedMapping {
    pub error: String,
}

impl InternalEvent for VicscriptFailedMapping {
    fn emit_logs(&self) {
        warn!(
            message = "Mapping failed with event",
            %self.error,
            rate_limit_secs = 30
        )
    }

    fn emit_metrics(&self) {
        counter!("processing_error", 1,
            "component_kind" => "transform",
            "component_type" => "vicscript",
            "error_type" => "failed_mapping",
        );
    }
}
