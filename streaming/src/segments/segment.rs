use crate::config::SegmentConfig;
use crate::message::Message;
use crate::segments::time_index::TimeIndex;
use ringbuffer::AllocRingBuffer;
use std::sync::Arc;

pub const LOG_EXTENSION: &str = "log";
pub const INDEX_EXTENSION: &str = "index";
pub const TIME_INDEX_EXTENSION: &str = "timeindex";
pub const MAX_SIZE_BYTES: u32 = 1_000_000_000;

// TODO: Move messages buffer to partition and remove from segment
#[derive(Debug)]
pub struct Segment {
    pub partition_id: u32,
    pub start_offset: u64,
    pub current_offset: u64,
    pub end_offset: u64,
    pub partition_path: String,
    pub index_path: String,
    pub log_path: String,
    pub time_index_path: String,
    pub messages: AllocRingBuffer<Arc<Message>>,
    pub unsaved_messages_count: u32,
    pub next_saved_message_index: u32,
    pub current_size_bytes: u32,
    pub saved_bytes: u32,
    pub should_increment_offset: bool,
    pub config: Arc<SegmentConfig>,
    pub time_indexes: Vec<TimeIndex>,
}

impl Segment {
    pub fn create(
        partition_id: u32,
        start_offset: u64,
        partition_path: &str,
        config: Arc<SegmentConfig>,
    ) -> Segment {
        let index_path = format!(
            "{}/{:0>20}.{}",
            partition_path, start_offset, INDEX_EXTENSION
        );
        let time_index_path = format!(
            "{}/{:0>20}.{}",
            partition_path, start_offset, TIME_INDEX_EXTENSION
        );
        let log_path = format!("{}/{:0>20}.{}", partition_path, start_offset, LOG_EXTENSION);

        Segment {
            partition_id,
            start_offset,
            current_offset: start_offset,
            end_offset: 0,
            partition_path: partition_path.to_string(),
            index_path,
            time_index_path,
            log_path,
            messages: AllocRingBuffer::with_capacity(config.messages_buffer as usize),
            unsaved_messages_count: 0,
            next_saved_message_index: 0,
            current_size_bytes: 0,
            saved_bytes: 0,
            should_increment_offset: false,
            config,
            time_indexes: Vec::new(),
        }
    }

    pub fn is_full(&self) -> bool {
        self.current_size_bytes >= self.config.size_bytes
    }
}