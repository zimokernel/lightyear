use crate::client::io::{ClientIoEventReceiver, ClientNetworkEventSender};
use crate::transport::dummy::DummyIo;
use crate::transport::error::Error as TransportError;
use crate::transport::io::IoState;
use crate::transport::local::{LocalChannel, LocalChannelBuilder};
use crate::transport::udp::{UdpSocket, UdpSocketBuilder};

use enum_dispatch::enum_dispatch;

/// Transport combines a PacketSender and a PacketReceiver
///
/// This trait is used to abstract the raw transport layer that sends and receives packets.
/// There are multiple implementations of this trait, such as UdpSocket etc.
#[enum_dispatch]
pub(crate) trait ClientTransportBuilder: Send + Sync {
    /// Attempt to connect to the remote
    fn connect(
        self,
    ) -> Result<
        (
            ClientTransportEnum,
            IoState,
            Option<ClientIoEventReceiver>,
            Option<ClientNetworkEventSender>,
        ),
        TransportError,
    >;
}

#[enum_dispatch(ClientTransportBuilder)]
pub(crate) enum ClientTransportBuilderEnum {
    UdpSocket(UdpSocketBuilder),
    LocalChannel(LocalChannelBuilder),
    Dummy(DummyIo),
}

#[allow(clippy::large_enum_variant)]
#[enum_dispatch(Transport)]
pub(crate) enum ClientTransportEnum {
    UdpSocket(UdpSocket),
    LocalChannel(LocalChannel),
    Dummy(DummyIo),
}
