use std::net::SocketAddr;

use amethyst::{
    core::SystemDesc,
    ecs::{Read, ReadExpect, System, World, Write},
    network::simulation::{
        DeliveryRequirement, NetworkSimulationEvent, TransportResource, UrgencyRequirement,
    },
    shred::{ResourceId, SystemData},
    shrev::{EventChannel, ReaderId},
};
use derivative::Derivative;
use derive_new::new;
use log::{error, warn};
use network_join_model::{config::SessionServerConfig, NetworkJoinEvent};

/// Builds a `SessionJoinRequestSystem`.
#[derive(Debug, Default, new)]
pub struct SessionJoinRequestSystemDesc {
    /// Configuration needed to connect to the session server.
    pub session_server_config: SessionServerConfig,
}

impl<'a, 'b> SystemDesc<'a, 'b, SessionJoinRequestSystem> for SessionJoinRequestSystemDesc {
    fn build(self, world: &mut World) -> SessionJoinRequestSystem {
        <SessionJoinRequestSystem as System<'_>>::SystemData::setup(world);

        let network_join_event_rid = world
            .fetch_mut::<EventChannel<NetworkJoinEvent>>()
            .register_reader();
        let network_simulation_event_rid = world
            .fetch_mut::<EventChannel<NetworkSimulationEvent>>()
            .register_reader();

        world.insert(self.session_server_config);

        SessionJoinRequestSystem::new(network_join_event_rid, network_simulation_event_rid)
    }
}

/// Sends requests to a game server to join a session.
#[derive(Debug, new)]
pub struct SessionJoinRequestSystem {
    /// Reader ID for the `NetworkJoinEvent` channel.
    network_join_event_rid: ReaderId<NetworkJoinEvent>,
    /// Reader ID for the `NetworkSimulationEvent` channel.
    network_simulation_event_rid: ReaderId<NetworkSimulationEvent>,
}

#[derive(Derivative, SystemData)]
#[derivative(Debug)]
pub struct SessionJoinRequestSystemData<'s> {
    /// `NetworkJoinEvent` channel.
    #[derivative(Debug = "ignore")]
    pub network_join_ec: Read<'s, EventChannel<NetworkJoinEvent>>,
    /// `SessionServerConfig` resource.
    #[derivative(Debug = "ignore")]
    pub session_server_config: ReadExpect<'s, SessionServerConfig>,
    /// `TransportResource` resource.
    #[derivative(Debug = "ignore")]
    pub transport_resource: Write<'s, TransportResource>,
    /// `NetworkSimulationEvent` channel.
    #[derivative(Debug = "ignore")]
    pub network_simulation_ec: Read<'s, EventChannel<NetworkSimulationEvent>>,
}

impl<'s> System<'s> for SessionJoinRequestSystem {
    type SystemData = SessionJoinRequestSystemData<'s>;

    fn run(
        &mut self,
        SessionJoinRequestSystemData {
            network_join_ec,
            session_server_config,
            mut transport_resource,
            network_simulation_ec,
        }: Self::SystemData,
    ) {
        // Only process one session join event if multiple are received.
        let session_join_request_params = network_join_ec
            .read(&mut self.network_join_event_rid)
            .find_map(|ev| {
                if let NetworkJoinEvent::SessionJoinRequest(session_join_request_params) = ev {
                    Some(session_join_request_params)
                } else {
                    None
                }
            });

        if let Some(session_join_request_params) = session_join_request_params {
            let server_socket_addr =
                SocketAddr::new(session_server_config.address, session_server_config.port);

            let message = format!(
                "Request to join `{}` from `{}`",
                &session_join_request_params.session_code,
                &session_join_request_params.session_device_name
            );

            // Connect to `server_socket_addr` and send request.
            transport_resource.send_with_requirements(
                server_socket_addr,
                message.as_bytes(),
                // None means it uses a default multiplexed stream.
                //
                // Suspect if we give it a value, the value will be a "channel" over the same
                // socket connection.
                DeliveryRequirement::ReliableOrdered(None),
                UrgencyRequirement::OnTick,
            );
        }

        network_simulation_ec
            .read(&mut self.network_simulation_event_rid)
            .for_each(|ev| match ev {
                NetworkSimulationEvent::Message(socket_addr, bytes) => {
                    warn!("Socket: {}, Message: {:?}", socket_addr, bytes);
                }
                NetworkSimulationEvent::SendError(io_error, message) => {
                    error!("Send error: `{}`, message: `{:?}`", io_error, message);
                }
                NetworkSimulationEvent::RecvError(io_error) => {
                    error!("Receive error: `{}`", io_error);
                }
                NetworkSimulationEvent::ConnectionError(io_error, socket_addr) => {
                    error!(
                        "Connection error: `{}`, socket_addr: `{:?}`",
                        io_error, socket_addr
                    );
                }
                _ => {}
            });
    }
}
