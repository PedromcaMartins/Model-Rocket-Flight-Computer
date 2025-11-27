use defmt_or_log::info;
use postcard_rpc::{header::VarHeader, server::SpawnContext};
use telemetry_messages::{PingRequest, PingResponse};

pub struct Context {
}

pub struct SpawnCtx {
}

impl SpawnContext for Context {
    type SpawnCtxt = SpawnCtx;
    fn spawn_ctxt(&mut self) -> Self::SpawnCtxt {
        SpawnCtx{  }
    }
}

// TODO: implement postcard server with receiving requests from flight computer (instead of current impl)

pub fn ping_handler(_context: &mut Context, _header: VarHeader, rqst: PingRequest) -> PingResponse {
    info!("ping: {}", *rqst);
    (*rqst).into()
}
