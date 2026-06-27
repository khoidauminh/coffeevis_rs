use crate::{
    graphics::{Argb, P2},
    visualizers::Visualizer,
};

pub struct CrtPatch {
    row_idx: usize,
    size: usize,
    sweep_amount: usize,
    color: Argb,
}

impl CrtPatch {
    pub fn new(size: usize, sweep_amount: usize, color: Argb) -> Self {
        Self {
            row_idx: 0,
            size,
            sweep_amount,
            color,
        }
    }

    pub fn default() -> Self {
        Self::new(23, 23, 0x10_FF_FF_FF)
    }
}

impl Visualizer for CrtPatch {
    fn perform(&mut self, args: crate::visualizers::VisualizerArgs) {
        args.pix.mixerm();
        args.pix.color(self.color);
        self.size = args.pix.logical_height() / 3;
        self.sweep_amount = args.pix.logical_height() / 3;

        for row in (0..self.size).step_by(1) {
            let row = (self.row_idx + row) % args.pix.logical_height().max(1);
            args.pix
                .rect(P2(0, row as i32), args.pix.logical_width(), 1);
        }

        self.row_idx = (self.row_idx + self.sweep_amount) % args.pix.logical_height()
    }
}
