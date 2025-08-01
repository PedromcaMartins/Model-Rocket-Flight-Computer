use defmt_or_log::info;
use postcard_rpc::{header::VarHeader, server::SpawnContext};

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

pub fn ping_handler(_context: &mut Context, _header: VarHeader, rqst: u32) -> u32 {
    info!("ping");
    rqst
}
