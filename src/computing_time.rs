use para_graph::model::{Device, Task};

pub fn computing_time(d: &Device, t: &Task) -> f64 {
    t.processing_density
        * t.data_size as f64
        * (1. - t.parallel_fraction + t.parallel_fraction / d.number_of_cores as f64)
        / (d.cpu_frequency * 100_000_000.)
}
