use crate::server::io::{ServerIoEventReceiver, ServerNetworkEventSender};
use crate::transport::channels::Channels;
use crate::transport::dummy::DummyIo;
use crate::transport::error::Result;
use crate::transport::io::IoState;
use crate::transport::udp::{UdpSocket, UdpSocketBuilder};

use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub(crate) trait ServerTransportBuilder: Send + Sync {
    /// Attempt to listen for incoming connections
    fn start(
        self,
    ) -> Result<(
        ServerTransportEnum,
        IoState,
        Option<ServerIoEventReceiver>,
        Option<ServerNetworkEventSender>,
    )>;
}

#[enum_dispatch(ServerTransportBuilder)]
pub(crate) enum ServerTransportBuilderEnum {
    UdpSocket(UdpSocketBuilder),

    Channels(Channels),
    Dummy(DummyIo),
}

#[allow(clippy::large_enum_variant)]
#[enum_dispatch(Transport)]
pub(crate) enum ServerTransportEnum {
    UdpSocket(UdpSocket),

    Channels(Channels),
    Dummy(DummyIo),
}
