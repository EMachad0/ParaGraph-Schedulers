#[derive(Debug, Default, Copy, Clone)]
pub struct Matching {
    pub finish_time: f64,
    pub node: usize,
}

impl std::fmt::Display for Matching {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Node {:2} Finish Time {:8.5}",
            self.node, self.finish_time
        ))
    }
}
