/// Amount of simulated pixels in the x direction
pub const SIMULATION_WIDTH: u32 = 400;

/// Amount of simulated pixels in the y direction
pub const SIMULATION_HEIGHT: u32 = 400;

/// Propagation speed of a sound wave in air (m/s) (* sqrt(2) to compensate for TLM-Error)
pub const PROPAGATION_SPEED: f32 = 343.2 * std::f32::consts::SQRT_2;

/// Width of the boundary in pixels
pub const INIT_BOUNDARY_WIDTH: u32 = 50;
