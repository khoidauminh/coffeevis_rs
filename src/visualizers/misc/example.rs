use crate::math::Cplx;
use crate::visualizers::Visualizer;

// The idea is simple.
// Start a ball at the center.
// Compute the average volume of
// the left and right chanel,
// then add those into the acceleration
// of the ball.

// This is an example visualizer,
// where all your parameters and objects
// are stored.

// With the Default trait, Rust will
// write for you the default()
// function for this struct.
#[derive(Default)]
pub struct Example {
    // Because we need the ball to be at
    // the center of the screen, yet we
    // don't know the screen size in advance,
    // declare the field as an Optional type,
    // default() should fill it as None for
    // inialization later.
    position: Option<Cplx>,

    // Acceleration is defined as 0.
    acceleration: Cplx,
}

// Take 256 samples.
const ACCUMULATE_SIZE: usize = 256;

// Here we begin implementing the
// necessary methods for this visualizer.
impl Visualizer for Example {
    // Name of this visualizer.
    // This method is optional,
    // so if you dont implement
    // this your visualizer's
    // name will be "ducky".
    fn name(&self) -> &'static str {
        "Example"
    }

    fn perform(
        &mut self,
        pix: &mut crate::graphics::PixelBuffer,
        stream: &mut crate::audio::AudioBuffer,
    ) {
        let bound = pix.size().to_cplx();

        // Finally initializes the position.
        let pos = self.position.get_or_insert(bound.center());

        // Computes the average
        let mut sum = Cplx::zero();

        for i in 0..ACCUMULATE_SIZE {
            sum += stream.get(i);
        }

        // Updates the position
        self.acceleration += sum / ACCUMULATE_SIZE as f32;
        self.acceleration *= 0.9; // Decay

        *pos += self.acceleration;

        // We have to use rem_euclid here
        // to get the NON-NEGATIVE remainer.
        pos.0 = pos.0.rem_euclid(bound.0);
        pos.1 = pos.1.rem_euclid(bound.1);

        // Draw
        pix.clear();
        pix.color(0xFF_FF_FF_FF); // White
        pix.circle(pos.to_p2(), 3, true); // radius = 3, filled = true
    }
}

// Now that you're done, head to /src/visualizers/mod.rs
// to add it to the list of running visualizers.
