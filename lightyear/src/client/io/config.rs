use crate::client::io::transport::{ClientTransportBuilder, ClientTransportBuilderEnum};
use crate::client::io::{Io, IoContext};
use crate::prelude::CompressionConfig;
use crate::transport::config::SharedIoConfig;
use crate::transport::dummy::DummyIo;
use crate::transport::error::Result;
use crate::transport::io::{BaseIo, IoStats};
use crate::transport::local::LocalChannelBuilder;
#[cfg(feature = "zstd")]
use crate::transport::middleware::compression::zstd::compression::ZstdCompressor;
#[cfg(feature = "zstd")]
use crate::transport::middleware::compression::zstd::decompression::ZstdDecompressor;
use crate::transport::middleware::conditioner::LinkConditioner;
use crate::transport::middleware::PacketReceiverWrapper;
use crate::transport::udp::UdpSocketBuilder;

use crate::transport::{BoxedReceiver, Transport, LOCAL_SOCKET};
use bevy::prelude::TypePath;
use crossbeam_channel::{Receiver, Sender};
use std::net::SocketAddr;

/// Use this to configure the [`Transport`] that will be used to establish a connection with the
/// server.
#[derive(Clone, Debug, TypePath)]
pub enum ClientTransport {
    /// Use a [`UdpSocket`](std::net::UdpSocket)
    UdpSocket(SocketAddr),

    /// Use a crossbeam_channel as a transport. This is useful for testing.
    /// This is mostly for clients.
    LocalChannel {
        recv: Receiver<Vec<u8>>,
        send: Sender<Vec<u8>>,
    },
    /// Dummy transport if the connection handles its own io (for example steam sockets)
    Dummy,
}

impl ClientTransport {
    pub(super) fn build(self) -> ClientTransportBuilderEnum {
        match self {
            ClientTransport::UdpSocket(addr) => {
                ClientTransportBuilderEnum::UdpSocket(UdpSocketBuilder { local_addr: addr })
            }
           
            ClientTransport::LocalChannel { recv, send } => {
                ClientTransportBuilderEnum::LocalChannel(LocalChannelBuilder { recv, send })
            }
            ClientTransport::Dummy => ClientTransportBuilderEnum::Dummy(DummyIo),
        }
    }
}

impl Default for ClientTransport {
    fn default() -> Self {
        ClientTransport::UdpSocket(LOCAL_SOCKET)
    }
}

impl SharedIoConfig<ClientTransport> {
    pub fn connect(self) -> Result<Io> {
        let (transport, state, io_rx, network_tx) = self.transport.build().connect()?;
        let local_addr = transport.local_addr();
        #[allow(unused_mut)]
        let (mut sender, receiver) = transport.split();
        #[allow(unused_mut)]
        let mut receiver: BoxedReceiver = if let Some(conditioner_config) = self.conditioner {
            let conditioner = LinkConditioner::new(conditioner_config);
            Box::new(conditioner.wrap(receiver))
        } else {
            Box::new(receiver)
        };
        match self.compression {
            CompressionConfig::None => {}
            #[cfg(feature = "zstd")]
            CompressionConfig::Zstd { level } => {
                use crate::transport::middleware::PacketSenderWrapper;
                let compressor = ZstdCompressor::new(level);
                sender = Box::new(compressor.wrap(sender));
                let decompressor = ZstdDecompressor::new();
                receiver = Box::new(decompressor.wrap(receiver));
            }
            #[cfg(feature = "lz4")]
            CompressionConfig::Lz4 => {
                use crate::transport::middleware::PacketSenderWrapper;
                let compressor =
                    crate::transport::middleware::compression::lz4::Compressor::default();
                sender = Box::new(compressor.wrap(sender));
                let decompressor =
                    crate::transport::middleware::compression::lz4::Decompressor::default();
                receiver = Box::new(decompressor.wrap(receiver));
            }
        }
        Ok(BaseIo {
            local_addr,
            sender,
            receiver,
            state,
            stats: IoStats::default(),
            context: IoContext {
                event_sender: network_tx,
                event_receiver: io_rx,
            },
        })
    }
}
