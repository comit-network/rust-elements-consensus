#[allow(dead_code, non_camel_case_types, non_upper_case_globals)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use crate::bindings::bitcoinconsensus_SCRIPT_FLAGS_VERIFY_ALL;
pub use crate::bindings::bitcoinconsensus_SCRIPT_FLAGS_VERIFY_CHECKLOCKTIMEVERIFY;
pub use crate::bindings::bitcoinconsensus_SCRIPT_FLAGS_VERIFY_CHECKSEQUENCEVERIFY;
pub use crate::bindings::bitcoinconsensus_SCRIPT_FLAGS_VERIFY_DERSIG;
pub use crate::bindings::bitcoinconsensus_SCRIPT_FLAGS_VERIFY_NONE;
pub use crate::bindings::bitcoinconsensus_SCRIPT_FLAGS_VERIFY_NULLDUMMY;
pub use crate::bindings::bitcoinconsensus_SCRIPT_FLAGS_VERIFY_P2SH;
pub use crate::bindings::bitcoinconsensus_SCRIPT_FLAGS_VERIFY_WITNESS;
pub use crate::bindings::bitcoinconsensus_error_t_bitcoinconsensus_ERR_AMOUNT_REQUIRED;
pub use crate::bindings::bitcoinconsensus_error_t_bitcoinconsensus_ERR_INVALID_FLAGS;
pub use crate::bindings::bitcoinconsensus_error_t_bitcoinconsensus_ERR_OK;
pub use crate::bindings::bitcoinconsensus_error_t_bitcoinconsensus_ERR_TX_DESERIALIZE;
pub use crate::bindings::bitcoinconsensus_error_t_bitcoinconsensus_ERR_TX_INDEX;
pub use crate::bindings::bitcoinconsensus_error_t_bitcoinconsensus_ERR_TX_SIZE_MISMATCH;
pub use crate::bindings::bitcoinconsensus_verify_script;
pub use crate::bindings::bitcoinconsensus_verify_script_with_amount;
