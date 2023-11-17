use std::time::Duration;

use anyhow::Result;

use crate::connection::ProtocolMessage;
use crate::inputs::input_buffer::InputBuffer;
use crate::tick::Tick;
use crate::{
    ChannelKind, ChannelRegistry, PingChannel, Protocol, ReadBuffer, SyncMessage, TickManager,
    TimeManager,
};

use super::ping_manager::PingConfig;
use super::sync::SyncManager;

// TODO: this layer of indirection is annoying, is there a better way?
//  maybe just pass the inner connection to ping_manager? (but harder to test)
pub struct Connection<P: Protocol> {
    pub(crate) base: crate::Connection<P>,

    // pub(crate) ping_manager: PingManager,
    pub(crate) input_buffer: InputBuffer<P::Input>,
    pub(crate) sync_manager: SyncManager,
    // TODO: maybe don't do any replication until connection is synced?
}

impl<P: Protocol> Connection<P> {
    pub fn new(channel_registry: &ChannelRegistry, ping_config: &PingConfig) -> Self {
        Self {
            base: crate::Connection::new(channel_registry),
            // ping_manager: PingManager::new(ping_config),
            input_buffer: InputBuffer::default(),
            sync_manager: SyncManager::new(
                ping_config.sync_num_pings,
                ping_config.sync_ping_interval_ms,
            ),
        }
    }

    /// Add an input for the given tick
    pub fn add_input(&mut self, input: P::Input, tick: Tick) {
        self.input_buffer.buffer.push(&tick, input);
    }

    pub fn update(
        &mut self,
        delta: Duration,
        time_manager: &TimeManager,
        tick_manager: &TickManager,
    ) {
        self.base.update(time_manager, tick_manager);
        self.sync_manager.update(delta);
        // TODO: maybe prepare ping?
        // self.ping_manager.update(delta);

        // if not synced, keep doing syncing
        if !self.sync_manager.is_synced() {
            if let Some(sync_ping) = self
                .sync_manager
                .maybe_prepare_ping(time_manager, tick_manager)
            {
                let message = ProtocolMessage::Sync(SyncMessage::TimeSyncPing(sync_ping));
                let channel = ChannelKind::of::<PingChannel>();
                self.base
                    .message_manager
                    .buffer_send(message, channel)
                    .unwrap();
            }
        }
    }

    pub fn recv_packet(&mut self, reader: &mut impl ReadBuffer) -> Result<()> {
        let tick = self.base.recv_packet(reader)?;
        if tick > self.sync_manager.latest_received_server_tick {
            self.sync_manager.latest_received_server_tick = tick;
        }
        Ok(())
    }

    // pub fn buffer_ping(&mut self, time_manager: &TimeManager) -> Result<()> {
    //     if !self.ping_manager.should_send_ping() {
    //         return Ok(());
    //     }
    //
    //     let ping_message = self.ping_manager.prepare_ping(time_manager);
    //
    //     // info!("Sending ping {:?}", ping_message);
    //     trace!("Sending ping {:?}", ping_message);
    //
    //     let message = ProtocolMessage::Sync(SyncMessage::Ping(ping_message));
    //     let channel = ChannelKind::of::<DefaultUnreliableChannel>();
    //     self.base.message_manager.buffer_send(message, channel)
    // }

    // TODO: eventually call handle_ping and handle_pong directly from the connection
    //  without having to send to events

    // send pongs for every ping we received
    // pub fn buffer_pong(&mut self, time_manager: &TimeManager, ping: PingMessage) -> Result<()> {
    //     let pong_message = self.ping_manager.prepare_pong(time_manager, ping);
    //
    //     // info!("Sending ping {:?}", ping_message);
    //     trace!("Sending pong {:?}", pong_message);
    //     let message = ProtocolMessage::Sync(SyncMessage::Pong(pong_message));
    //     let channel = ChannelKind::of::<DefaultUnreliableChannel>();
    //     self.base.message_manager.buffer_send(message, channel)
    // }
}