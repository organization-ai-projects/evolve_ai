pub mod agent_info;
pub mod cargo;
pub mod copy;
pub mod initiate_project_genome;
pub mod listing;
pub mod log;

// Permet d'accéder à pipelines::* depuis initiate::*
pub use initiate_project_genome::generate_initial_genome;
