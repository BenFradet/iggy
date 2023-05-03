use crate::message::Message;
use crate::partitions::partition::Partition;
use crate::segments::segment::Segment;
use shared::error::Error;
use std::sync::Arc;
use tracing::trace;

const EMPTY_MESSAGES: Vec<Arc<Message>> = vec![];

impl Partition {
    pub async fn get_messages(&self, offset: u64, count: u32) -> Result<Vec<Arc<Message>>, Error> {
        if self.segments.is_empty() {
            return Ok(EMPTY_MESSAGES);
        }

        let mut end_offset = offset + (count - 1) as u64;
        let max_offset = self.segments.last().unwrap().current_offset;
        if end_offset > max_offset {
            end_offset = max_offset;
        }

        let segments = self
            .segments
            .iter()
            .filter(|segment| {
                (segment.start_offset >= offset && segment.current_offset <= end_offset)
                    || (segment.start_offset <= offset && segment.current_offset >= offset)
                    || (segment.start_offset <= end_offset && segment.current_offset >= end_offset)
            })
            .collect::<Vec<&Segment>>();

        if segments.is_empty() {
            return Ok(EMPTY_MESSAGES);
        }

        if segments.len() == 1 {
            let segment = segments.first().unwrap();
            let messages = segment.get_messages(offset, count).await?;
            return Ok(messages);
        }

        let mut messages = Vec::new();
        for segment in segments {
            let segment_messages = segment.get_messages(offset, count).await?;
            for message in segment_messages {
                messages.push(message);
            }
        }

        Ok(messages)
    }

    pub async fn append_messages(&mut self, messages: Vec<Message>) -> Result<(), Error> {
        let segment = self.segments.last_mut();
        if segment.is_none() {
            return Err(Error::SegmentNotFound);
        }

        let segment = segment.unwrap();
        if segment.is_full() {
            trace!(
                "Current segment is full, creating new segment for partition with ID: {}",
                self.id
            );
            let start_offset = segment.end_offset + 1;
            let mut new_segment = Segment::create(
                self.id,
                start_offset,
                &self.path,
                self.config.segment.clone(),
            );
            new_segment.persist().await?;
            self.segments.push(new_segment);
            self.segments
                .sort_by(|a, b| a.start_offset.cmp(&b.start_offset));
        }

        let segment = self.segments.last_mut();
        segment.unwrap().append_messages(messages).await
    }
}