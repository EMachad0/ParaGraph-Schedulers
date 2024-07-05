mod buyya_par_cpu;
mod buyya_serial;
mod buyya_par_gpu;

pub use buyya_par_cpu::buyya_par_cpu;
pub use buyya_serial::buyya_serial;
pub use buyya_par_gpu::buyya_par_gpu;
