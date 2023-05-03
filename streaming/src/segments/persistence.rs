use crate::segments::segment::Segment;
use crate::segments::{log, time_index};
use ringbuffer::{AllocRingBuffer, RingBufferWrite};
use shared::error::Error;
use std::sync::Arc;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tracing::info;

const RELATIVE_START_OFFSET: u32 = 0;

impl Segment {
    pub async fn load(&mut self) -> Result<(), Error> {
        info!(
            "Loading segment from disk for offset: {} and partition with ID: {}...",
            self.start_offset, self.partition_id
        );
        let mut log_file = Segment::open_file(&self.log_path, false).await;
        let mut time_index_file = Segment::open_file(&self.time_index_path, false).await;
        let file_size = log_file.metadata().await.unwrap().len() as u32;

        info!(
            "Loading time indexes from segment log file for start offset: {} and partition ID: {}...",
            self.start_offset, self.partition_id
        );

        self.time_indexes = time_index::load(&mut time_index_file).await?;

        info!(
            "Loading messages from segment log file for start offset: {} and partition ID: {}...",
            self.start_offset, self.partition_id
        );

        let relative_end_offset = (self.end_offset - self.start_offset) as u32;
        let messages = log::load(&mut log_file, RELATIVE_START_OFFSET, relative_end_offset).await?;
        if messages.is_empty() {
            return Ok(());
        }

        self.current_offset = messages.last().unwrap().offset;
        self.next_saved_message_index += messages.len() as u32;

        let mut buffered_messages =
            AllocRingBuffer::with_capacity(self.config.messages_buffer as usize);

        for message in messages {
            buffered_messages.push(Arc::new(message));
        }

        self.messages = buffered_messages;
        self.should_increment_offset = self.current_offset > 0;
        self.current_size_bytes = file_size;
        self.saved_bytes = self.current_size_bytes;

        info!(
            "Loaded {} bytes from segment log file with start offset {}, current offset: {}, and partition ID: {}.",
            self.current_size_bytes, self.start_offset, self.current_offset, self.partition_id
        );

        Ok(())
    }

    pub async fn persist(&mut self) -> Result<(), Error> {
        info!("Saving segment with start offset: {}", self.start_offset);
        if File::create(&self.log_path).await.is_err() {
            return Err(Error::CannotCreatePartitionSegmentLogFile(
                self.log_path.clone(),
            ));
        }

        if File::create(&self.time_index_path).await.is_err() {
            return Err(Error::CannotCreatePartitionSegmentTimeIndexFile(
                self.log_path.clone(),
            ));
        }

        let index_file = File::create(&self.index_path).await;
        if index_file.is_err() {
            return Err(Error::CannotCreatePartitionSegmentIndexFile(
                self.log_path.clone(),
            ));
        }

        let mut index_file = index_file.unwrap();
        let zero_index = 0u64.to_le_bytes();
        if index_file.write_all(&zero_index).await.is_err() {
            return Err(Error::CannotSaveIndexToSegment);
        }

        info!(
            "Created partition segment log file for start offset: {} and partition with ID: {} and path: {}.",
            self.start_offset, self.partition_id, self.log_path
        );

        Ok(())
    }

    pub async fn open_file(path: &str, append: bool) -> File {
        OpenOptions::new()
            .read(true)
            .append(append)
            .open(path)
            .await
            .unwrap()
    }
}