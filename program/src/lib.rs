use bonfida_utils::declare_id_with_central_state;

#[doc(hidden)]
pub mod entrypoint;
#[doc(hidden)]
pub mod error;
/// Program instructions and their CPI-compatible bindings
pub mod instruction;
/// Describes the different data structres that the program uses to encode state
pub mod state;

#[doc(hidden)]
pub mod processor;

#[allow(missing_docs)]
pub mod cpi;

declare_id_with_central_state!("4X9mF1yUx2ez6ifYCmr2aYJnX5DkKAxbu5QD93s7gooG");
