extern crate env_logger;
extern crate futures;
extern crate netlink_sys;

use futures::{Future, Sink, Stream};
use netlink_sys::constants::{NLM_F_DUMP, NLM_F_REQUEST};
use netlink_sys::rtnl::{LinkHeader, LinkMessage, Message, RtnlMessage};
use netlink_sys::{NetlinkCodec, NetlinkFlags, NetlinkFramed, Protocol, SocketAddr, TokioSocket};

fn main() {
    env_logger::init();
    let mut socket = TokioSocket::new(Protocol::Route).unwrap();
    let _port_number = socket.bind_auto().unwrap().port_number();
    socket.connect(&SocketAddr::new(0, 0)).unwrap();
    let stream = NetlinkFramed::new(socket, NetlinkCodec::<Message>::new());

    let mut packet: Message =
        RtnlMessage::GetLink(LinkMessage::from_parts(LinkHeader::new(), vec![])).into();
    packet
        .header_mut()
        .set_flags(NetlinkFlags::from(NLM_F_DUMP | NLM_F_REQUEST))
        .set_sequence_number(1);
    packet.finalize();
    let mut buf = vec![0; packet.header().length() as usize];
    packet.to_bytes(&mut buf[..]).unwrap();

    println!(">>> {:?}", packet);
    let stream = stream.send((packet, SocketAddr::new(0, 0))).wait().unwrap();

    stream
        .for_each(|(packet, _addr)| {
            println!("<<< {:?}", packet);
            Ok(())
        })
        .wait()
        .unwrap();
}