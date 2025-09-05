use anyhow::Result;
use std::{net::{UdpSocket, SocketAddr}, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use renet::{RenetServer, ServerEvent, DefaultChannel, ConnectionConfig};
use renet_netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig};

use hecs::Entity;
use moonhold_protocol::{ClientMsg, ServerMsg};
use moonhold_sim as sim;
use sim::{Scene, SimState, Pos, Vel, Health, Collider};

const HISTORY_TICKS: usize = 64;

#[derive(Default)]
struct Players {
    map: std::collections::HashMap<u64, Entity>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let cfg = load_cfg()?;
    let addr: SocketAddr = cfg.listen.parse()?;

    // Create connection configuration
    let connection_config = ConnectionConfig::default();
    let mut server = RenetServer::new(connection_config);

    // Configure server transport
    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let server_config = ServerConfig {
        current_time,
        max_clients: cfg.max_clients,
        protocol_id: cfg.protocol_id,
        public_addresses: vec![addr],
        authentication: if cfg.insecure { 
            ServerAuthentication::Unsecure 
        } else { 
            ServerAuthentication::Unsecure // Replace with Secure in production
        },
    };
    
    let socket = UdpSocket::bind(addr)?;
    let mut transport = NetcodeServerTransport::new(server_config, socket)?;

    let mut scene = Scene::new();
    let mut state = SimState::new(cfg.tick_hz, HISTORY_TICKS, cfg.max_clients as usize);
    let mut players = Players::default();

    info!("Zone listening on {}", cfg.listen);

    let tick_dt = Duration::from_millis(1000 / cfg.tick_hz as u64);
    let mut last_tick = Instant::now();
    let mut last_update = Instant::now();

    loop {
        // Calculate delta time
        let now = Instant::now();
        let delta = now.duration_since(last_update);
        last_update = now;

        // Update server and transport
        server.update(delta);
        transport.update(delta, &mut server)?;

        // Handle server events
        while let Some(event) = server.get_event() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    let e = scene.world.spawn((
                        Pos(0.0, 0.0, 0.0), 
                        Vel(0.0, 0.0, 0.0), 
                        Health::default(), 
                        Collider { radius: 0.35 }
                    ));
                    players.map.insert(client_id, e);
                    send(&mut server, client_id, &ServerMsg::Welcome { 
                        tick_hz: cfg.tick_hz, 
                        your_id: client_id 
                    });
                    info!("client {} connected (entity {:?})", client_id, e);
                }
                ServerEvent::ClientDisconnected { client_id, reason } => {
                    if let Some(e) = players.map.remove(&client_id) { 
                        let _ = scene.world.despawn(e); 
                    }
                    warn!("client {} disconnected: {:?}", client_id, reason);
                }
            }
        }

        // Receive messages from clients
        for client_id in server.clients_id() {
            // Check reliable channel
            while let Some(pkt) = server.receive_message(client_id, DefaultChannel::ReliableOrdered) {
                if let Ok(msg) = bincode::serde::decode_from_slice::<ClientMsg, _>(&pkt, bincode::config::standard()).map(|(msg, _)| msg) {
                    handle_client_message(&mut scene, &players, &mut server, client_id, msg, &state);
                }
            }
            
            // Check unreliable channel
            while let Some(pkt) = server.receive_message(client_id, DefaultChannel::Unreliable) {
                if let Ok(msg) = bincode::serde::decode_from_slice::<ClientMsg, _>(&pkt, bincode::config::standard()).map(|(msg, _)| msg) {
                    handle_client_message(&mut scene, &players, &mut server, client_id, msg, &state);
                }
            }
        }

        // Tick-based world update
        if now.duration_since(last_tick) >= tick_dt {
            state.tick = state.tick.wrapping_add(1);
            last_tick = now;

            let ents = sim::systems::replication::collect_deltas(&scene.world);
            let delta = ServerMsg::WorldDelta { 
                tick: state.tick, 
                last_processed_input: 0, 
                ents 
            };
            broadcast_unreliable(&mut server, &delta);
        }

        // Send packets
        transport.send_packets(&mut server);

        // Small sleep to prevent busy waiting
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
}

fn handle_client_message(
    scene: &mut Scene,
    players: &Players,
    server: &mut RenetServer,
    client_id: u64,
    msg: ClientMsg,
    state: &SimState,
) {
    match msg {
        ClientMsg::Ping { client_time_ms: _ } => {
            let now_ms = epoch_ms();
            send(server, client_id, &ServerMsg::Pong { 
                server_time_ms: now_ms 
            });
        }
        ClientMsg::Input { seq: _, dt_ms, dx, dy } => {
            if let Some(&e) = players.map.get(&client_id) {
                let dt = (dt_ms as f32).clamp(1.0, 50.0) / 1000.0;
                let mut q = scene.world.query_one::<(&mut Pos, &mut Vel)>(e).unwrap();
                if let Some((pos, vel)) = q.get() {
                    let max_speed = 6.0;
                    let len = (dx*dx + dy*dy).sqrt();
                    let (vx, vy) = if len > 0.0001 { 
                        (dx/len*max_speed, dy/len*max_speed) 
                    } else { 
                        (0.0, 0.0) 
                    };
                    pos.0 += vx * dt; 
                    pos.2 += vy * dt;
                    vel.0 = vx; 
                    vel.2 = vy;
                }
            }
        }
        ClientMsg::Fire { seq: _, client_time_ms: _, origin, dir, weapon: _ } => {
            if let Some(&_e_shooter) = players.map.get(&client_id) {
                let evt = sim::systems::combat::FireEvent {
                    shooter_id: client_id, 
                    origin, 
                    dir, 
                    range: 75.0
                };
                match sim::systems::combat::hitscan(&scene.world, &evt) {
                    sim::systems::combat::Hit::None => {}
                    sim::systems::combat::Hit::Victim(_victim_ent) => {
                        let msg = ServerMsg::HitConfirm { 
                            shooter: client_id, 
                            victim: client_id, 
                            dmg: 10, 
                            at_tick: state.tick 
                        };
                        broadcast(server, &msg);
                    }
                }
            }
        }
    }
}

fn send(srv: &mut RenetServer, client_id: u64, msg: &ServerMsg) {
    let bytes = bincode::serde::encode_to_vec(msg, bincode::config::standard()).unwrap();
    srv.send_message(client_id, DefaultChannel::ReliableOrdered, bytes);
}

fn broadcast(srv: &mut RenetServer, msg: &ServerMsg) {
    let bytes = bincode::serde::encode_to_vec(msg, bincode::config::standard()).unwrap();
    for client_id in srv.clients_id() { 
        srv.send_message(client_id, DefaultChannel::ReliableOrdered, bytes.clone()); 
    }
}

fn broadcast_unreliable(srv: &mut RenetServer, msg: &ServerMsg) {
    let bytes = bincode::serde::encode_to_vec(msg, bincode::config::standard()).unwrap();
    for client_id in srv.clients_id() { 
        srv.send_message(client_id, DefaultChannel::Unreliable, bytes.clone()); 
    }
}

fn epoch_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}

#[derive(serde::Deserialize)]
struct ZoneCfg {
    listen: String,
    protocol_id: u64,
    max_clients: usize,
    tick_hz: u16,
    insecure: bool,
}

fn load_cfg() -> Result<ZoneCfg> {
    let c = config::Config::builder()
        .add_source(config::File::with_name("config/zone.local").required(false))
        .add_source(config::Environment::with_prefix("ZONE").separator("__"))
        .build()?;
    c.try_deserialize().map_err(Into::into)
}