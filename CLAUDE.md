# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Development Commands

```bash
# Build the entire workspace
cargo build

# Run the zone server
cargo run -p moonhold-zone-server
# or
make run

# Run in release mode
cargo build --release -p moonhold-zone-server
make release

# Code quality checks
cargo fmt --all          # Format code
cargo clippy --all-targets --all-features -D warnings  # Lint
make fmt                 # Format via Makefile
make clippy              # Lint via Makefile

# Environment setup
cp .env.example .env     # Initialize environment variables
```

## Architecture Overview

This is an authoritative MMORPG server built in Rust with a 60Hz tick rate, UDP networking, and ECS-based simulation.

### Workspace Structure

The project uses a Cargo workspace with three crates:

1. **`crates/protocol/`** - Wire protocol definitions shared between client and server
   - `ClientMsg`: Input, Fire, Ping messages from clients
   - `ServerMsg`: WorldDelta, HitConfirm, Welcome, Pong messages to clients
   - Uses bincode 2.0 with serde for binary serialization

2. **`crates/sim_core/`** - Core ECS simulation logic
   - Components: `Pos`, `Vel`, `Health`, `Collider`, `PlayerTag`
   - Systems: movement integration, hitscan combat, state replication
   - Resources: `SimState` (tick tracking), `Scene` (ECS world wrapper)
   - History system for lag compensation (64 tick buffer)
   - Grid-based AOI for spatial partitioning

3. **`services/zone_server/`** - The authoritative game server binary
   - Uses renet 1.1.0 for UDP networking with reliable/unreliable channels
   - Processes client input immediately (no prediction/rollback yet)
   - Broadcasts world state at 60Hz to all connected clients

### Key Architectural Patterns

**Networking Flow:**
1. Server binds to UDP socket (default port 5000)
2. Uses renet's `NetcodeServerTransport` for connection management
3. Two channel types:
   - `ReliableOrdered`: Control messages (connect, ping, hit confirms)
   - `Unreliable`: World state deltas (position/health updates)

**Game Loop Structure:**
```rust
loop {
    // 1. Update networking layer
    server.update(delta);
    transport.update(delta, &mut server)?;
    
    // 2. Handle client events (connect/disconnect)
    while let Some(event) = server.get_event() { ... }
    
    // 3. Process client messages
    for client_id in server.clients_id() {
        // Check both reliable and unreliable channels
        while let Some(pkt) = server.receive_message(...) { ... }
    }
    
    // 4. Tick-based world update (60Hz)
    if tick_elapsed {
        state.tick += 1;
        let deltas = collect_entity_deltas();
        broadcast_unreliable(WorldDelta);
    }
    
    // 5. Send network packets
    transport.send_packets(&mut server);
}
```

**ECS Processing:**
- Uses `hecs` library for sparse-set ECS
- Entity spawned on client connect with Pos, Vel, Health, Collider
- Client ID mapped to Entity via HashMap
- Systems query world for components and apply transformations

**Combat System:**
- Hitscan with ray-sphere intersection
- 75 unit maximum range
- No projectile simulation (instant hit detection)
- History buffer prepared for lag compensation rewind

### Configuration

Server configured via TOML files and environment variables:
- `config/zone.local.toml` - Local development config
- `config/zone.prod.toml` - Production config
- Environment prefix: `ZONE__` (e.g., `ZONE__LISTEN=0.0.0.0:5000`)

Key parameters:
- `tick_hz`: 60 (server tick rate)
- `max_clients`: 128 (concurrent connections)
- `protocol_id`: 42 (netcode protocol identifier)
- `insecure`: true for development (no authentication)

### Important Implementation Details

**Bincode 2.0 Compatibility:**
- Use `bincode::serde::encode_to_vec()` and `bincode::serde::decode_from_slice()`
- Protocol crate requires `bincode = { version = "2.0.1", features = ["serde"] }`

**Entity ID Conversion:**
- hecs uses u32 entity IDs internally
- Cast to u64 for protocol: `entity.id() as u64`

**Movement Processing:**
- Input clamped to max speed of 6.0 units/second
- Direct position updates (no physics simulation)
- Velocity normalized from input direction

**Stubbed Features (Ready for Implementation):**
- Lag compensation using history snapshots
- AOI-based entity filtering for scalability
- Input sequence tracking for client reconciliation
- Persistence layer for saving game state
- Advanced combat with cooldowns/stamina